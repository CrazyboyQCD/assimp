/*
---------------------------------------------------------------------------
Open Asset Import Library (assimp)
---------------------------------------------------------------------------

Copyright (c) 2006-2025, assimp team

All rights reserved.

Redistribution and use of this software in source and binary forms,
with or without modification, are permitted provided that the following
conditions are met:

* Redistributions of source code must retain the above
  copyright notice, this list of conditions and the
  following disclaimer.

* Redistributions in binary form must reproduce the above
  copyright notice, this list of conditions and the
  following disclaimer in the documentation and/or other
  materials provided with the distribution.

* Neither the name of the assimp team, nor the names of its
  contributors may be used to endorse or promote products
  derived from this software without specific prior
  written permission of the assimp team.

THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
"AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
OWNER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
(INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
---------------------------------------------------------------------------
*/

//! Defines decompression utils for the library

use alloc::vec::Vec;

#[cfg(feature = "zlib_custom_allocator")]
use zlib_rs::c_api::{alloc_func, free_func};
use zlib_rs::{
    InflateFlush, ReturnCode,
    c_api::z_stream,
    inflate::{self, InflateConfig, InflateStream},
};

pub(crate) mod error;
use error::CompressionError;
#[allow(unused)]
const BLOCK_SIZE: usize = 32786;

#[allow(unused)]
pub enum Format {
    Invalid = -1,
    Text = 0,
    Binary = 1,
    Compressed = 2,
}

pub struct Compression {
    stream: z_stream,
    flush_mode: InflateFlush,
}

#[allow(unused)]
impl Compression {
    pub fn new() -> Self {
        Self {
            stream: z_stream::default(),
            flush_mode: InflateFlush::NoFlush,
        }
    }

    #[cfg(feature = "zlib_custom_allocator")]
    pub fn new_with_custom_allocator(
        alloc_fn: Option<alloc_func>,
        free_fn: Option<free_func>,
    ) -> Self {
        let mut stream = z_stream::default();
        stream.zalloc = alloc_fn;
        stream.zfree = free_fn;
        Self {
            is_open: false,
            stream,
            flush_mode: InflateFlush::NoFlush,
        }
    }

    /// Initialize decompression stream
    ///
    /// This function initializes and opens a decompression stream, setting the decompression
    /// format, flush mode, and window bits. Any previously opened stream will be closed before
    /// initialization.
    pub fn open(
        &mut self,
        format: Format,
        flush_mode: InflateFlush,
        window_bits: i32,
    ) -> Result<(), CompressionError> {
        self.close();
        self.stream.data_type = format as i32;
        self.flush_mode = flush_mode;
        let ret = inflate::init(&mut self.stream, InflateConfig { window_bits });
        if ret != ReturnCode::Ok {
            return Err(ret.into());
        }
        Ok(())
    }

    pub fn decompress(
        &mut self,
        data: &[u8],
        output: &mut Vec<u8>,
    ) -> Result<usize, CompressionError> {
        // SAFETY: stream reference are valid if stream is initialized.
        if let Some(stream) = unsafe { InflateStream::from_stream_mut(&mut self.stream) } {
            // Though we take mutable reference of `self.stream` and modify the original data, we
            // only use the reference after the modification, so it is safe.
            self.stream.next_in = data.as_ptr();
            self.stream.avail_in = data.len() as u32;
            let flush_mode = self.flush_mode;
            if flush_mode == InflateFlush::Finish {
                self.stream.next_out = output.as_mut_ptr();
                self.stream.avail_out = output.len() as u32;

                let ret = unsafe { inflate::inflate(stream, self.flush_mode) };
                if ret != ReturnCode::StreamEnd && ret != ReturnCode::Ok {
                    return Err(ret.into());
                }
                Ok(self.stream.avail_out as usize)
            } else {
                let old_len = output.len();
                let mut capacity = old_len + BLOCK_SIZE;
                output
                    .try_reserve(capacity)
                    .map_err(|_| CompressionError::MemError)?;
                // SAFETY: there are sufficient space if reserve is successful.
                self.stream.next_out = unsafe { output.as_mut_ptr().add(old_len) };
                loop {
                    self.stream.avail_out = BLOCK_SIZE as u32;
                    let ret = unsafe { inflate::inflate(stream, flush_mode) };
                    if ret != ReturnCode::StreamEnd && ret != ReturnCode::Ok {
                        return Err(ret.into());
                    }
                    let size = BLOCK_SIZE - self.stream.avail_out as usize;
                    // Write `size` bytes to output
                    let new_len = output.len() + size;
                    unsafe {
                        output.set_len(new_len);
                    }
                    // Reserve more space for the next block
                    capacity = new_len + BLOCK_SIZE;
                    output
                        .try_reserve(capacity)
                        .map_err(|_| CompressionError::MemError)?;
                    // Move the pointer to the next block
                    self.stream.next_out = unsafe { self.stream.next_out.add(size) };
                    if ret == ReturnCode::StreamEnd {
                        return Ok(output.len() - old_len);
                    }
                }
            }
        } else {
            Err(CompressionError::TryToOperateClosedStream)
        }
    }

    pub fn decompress_block(
        &mut self,
        data: &[u8],
        output: &mut [u8],
    ) -> Result<usize, CompressionError> {
        // SAFETY: stream reference and stream pointer are valid if return `Some`.
        if let Some(stream) = unsafe { InflateStream::from_stream_mut(&mut self.stream) } {
            self.stream.next_in = data.as_ptr();
            self.stream.avail_in = data.len() as u32;
            let avail_out = output.len() as u32;
            self.stream.avail_out = avail_out;
            self.stream.next_out = output.as_mut_ptr();
            let ret = unsafe { inflate::inflate(stream, InflateFlush::SyncFlush) };
            if ret != ReturnCode::StreamEnd && ret != ReturnCode::Ok {
                return Err(ret.into());
            }
            let ret = inflate::reset(stream);
            if ret != ReturnCode::Ok {
                return Err(ret.into());
            }
            let total = avail_out as usize - self.stream.avail_out as usize;
            let ret = inflate::set_dictionary(stream, &output[..total]);
            if ret != ReturnCode::Ok {
                return Err(ret.into());
            }
            Ok(total)
        } else {
            Err(CompressionError::TryToOperateClosedStream)
        }
    }

    fn close(&mut self) {
        if !self.stream.zalloc.is_none() && !self.stream.zfree.is_none() {
            // SAFETY: stream is a non-null reference and is initialized.
            if let Some(stream) = unsafe { InflateStream::from_stream_mut(&mut self.stream) } {
                inflate::end(stream);
            }
        }
    }
}

impl Drop for Compression {
    fn drop(&mut self) {
        self.close();
    }
}
