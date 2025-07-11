use zlib_rs::{
    InflateFlush, ReturnCode,
    c_api::z_stream,
    inflate::{InflateConfig, InflateStream},
};

pub(crate) mod error;
use error::CompressionError;
#[allow(unused)]
const MYBLOCK: usize = 32786;

#[allow(unused)]
pub enum Format {
    InvalidFormat = -1,
    Text = 0,
    Binary = 1,
    Compressed = 2,
}

pub struct Compression {
    is_open: bool,
    stream: z_stream,
    flush_mode: InflateFlush,
}

#[allow(unused)]
impl Compression {
    pub fn new() -> Self {
        Self {
            is_open: false,
            stream: z_stream::default(),
            flush_mode: InflateFlush::NoFlush,
        }
    }

    pub fn open(
        &mut self,
        format: Format,
        flush_mode: InflateFlush,
        window_bits: i32,
    ) -> Result<(), CompressionError> {
        self.stream.data_type = format as i32;
        self.flush_mode = flush_mode;
        let ret = zlib_rs::inflate::init(&mut self.stream, InflateConfig { window_bits });
        if ret != ReturnCode::Ok {
            return Err(ret.into());
        }
        self.is_open = true;
        Ok(())
    }

    pub fn decompress(
        &mut self,
        data: &[u8],
        output: &mut Vec<u8>,
    ) -> Result<usize, CompressionError> {
        self.stream.next_in = data.as_ptr();
        self.stream.avail_in = data.len() as u32;
        let flush_mode = self.flush_mode;
        if flush_mode == InflateFlush::Finish {
            self.stream.avail_out = output.len() as u32;
            self.stream.next_out = output.as_mut_ptr();
            let stream = unsafe { InflateStream::from_stream_mut(&mut self.stream).unwrap() };
            let ret = unsafe { zlib_rs::inflate::inflate(stream, self.flush_mode) };
            if ret != ReturnCode::StreamEnd && ret != ReturnCode::Ok {
                return Err(ret.into());
            }
            return Ok(self.stream.avail_out as usize);
        } else {
            let mut total = 0;
            let mut block: Vec<u8> = {
                let mut s = Vec::with_capacity(MYBLOCK);
                // SAFETY: there is enough space for the block, and zlib will overwrite the uninitialized memory
                unsafe {
                    s.set_len(MYBLOCK);
                }
                s
            };
            self.stream.next_out = block.as_mut_ptr();
            loop {
                self.stream.avail_out = MYBLOCK as u32;
                let stream = unsafe { InflateStream::from_stream_mut(&mut self.stream).unwrap() };
                let ret = unsafe { zlib_rs::inflate::inflate(stream, flush_mode) };
                if ret != ReturnCode::StreamEnd && ret != ReturnCode::Ok {
                    return Err(ret.into());
                }
                let size = MYBLOCK - self.stream.avail_out as usize;
                total += size;
                output.extend_from_slice(&block[..size]);
                if ret == ReturnCode::StreamEnd {
                    return Ok(total);
                }
            }
        }
    }

    pub fn decompress_block(
        &mut self,
        data: &[u8],
        output: &mut [u8],
    ) -> Result<usize, CompressionError> {
        self.stream.next_in = data.as_ptr();
        self.stream.avail_in = data.len() as u32;
        let avail_out = output.len() as u32;
        self.stream.avail_out = avail_out;
        self.stream.next_out = output.as_mut_ptr();
        let stream = unsafe { InflateStream::from_stream_mut(&mut self.stream).unwrap() };
        let ret = unsafe { zlib_rs::inflate::inflate(stream, InflateFlush::SyncFlush) };
        if ret != ReturnCode::StreamEnd && ret != ReturnCode::Ok {
            return Err(ret.into());
        }
        let ret = zlib_rs::inflate::reset(stream);
        if ret != ReturnCode::Ok {
            return Err(ret.into());
        }
        let total = avail_out as usize - self.stream.avail_out as usize;
        let ret = zlib_rs::inflate::set_dictionary(stream, &output[..total]);
        if ret != ReturnCode::Ok {
            return Err(ret.into());
        }
        Ok(total)
    }

    pub fn close(&mut self) -> Result<(), CompressionError> {
        if !self.is_open {
            return Err(CompressionError::TryToCloseClosedStream);
        }
        let stream = unsafe { InflateStream::from_stream_mut(&mut self.stream).unwrap() };
        zlib_rs::inflate::end(stream);
        self.is_open = false;
        return Ok(());
    }
}
