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

//! Defines encoding utils for the library

use alloc::{string::String, vec::Vec};
use core::mem;

pub mod error;

use error::EncodingError;

/// Convert bytes of different encodings to UTF-8 string
///
/// Supported encoding formats:
/// - UTF-8 (with/without BOM)
/// - UTF-16 BE/LE (with BOM)
/// - UTF-32 BE/LE (with BOM)
pub fn convert_to_utf8(buf: &mut Vec<u8>) -> Result<(), EncodingError> {
    if buf.len() < 8 {
        return Err(EncodingError::UnknownEncoding);
    }

    // UTF-8 with BOM
    if buf.len() >= 3 && buf[0] == 0xEF && buf[1] == 0xBB && buf[2] == 0xBF {
        buf.rotate_left(3);
        buf.truncate(buf.len() - 3);
        return Ok(());
    }

    // UTF-32 with BOM
    if let Some(&[b1, b2, b3, b4]) = buf.get(0..4) {
        let b = u32::from_le_bytes([b1, b2, b3, b4]);
        if b == 0xFFFE0000 {
            return convert_utf32_to_string::<true>(buf);
        } else if b == 0x0000FFFE {
            return convert_utf32_to_string::<false>(buf);
        }
    }

    // UTF-16 with BOM
    if let Some(&[b1, b2]) = buf.get(0..2) {
        let b = u16::from_le_bytes([b1, b2]);
        if b == 0xFFFE || b == 0xFEFF {
            return convert_utf16_to_string(buf, b == 0xFFFE);
        }
    }

    // UTF-8
    Ok(())
}

fn convert_utf32_to_string<const IS_BIG_ENDIAN: bool>(
    buf: &mut Vec<u8>,
) -> Result<(), EncodingError> {
    if !buf.len().is_multiple_of(mem::size_of::<u32>()) {
        return Err(EncodingError::NotValidUtf32Length(buf.len()));
    }

    let mut s = String::with_capacity(buf.len() / 4);
    for &bytes in buf.as_chunks::<4>().0 {
        let code_point = if IS_BIG_ENDIAN {
            u32::from_be_bytes(bytes)
        } else {
            u32::from_le_bytes(bytes)
        };

        let c = char::from_u32(code_point).ok_or(EncodingError::NotValidCodePoint(code_point))?;
        s.push(c);
    }
    *buf = s.into_bytes();
    Ok(())
}

fn convert_utf16_to_string(buf: &mut Vec<u8>, is_big_endian: bool) -> Result<(), EncodingError> {
    let len = buf.len();
    if !len.is_multiple_of(mem::size_of::<u16>()) {
        return Err(EncodingError::NotValidUtf16Length(len));
    }

    let result = if is_big_endian {
        char::decode_utf16(
            buf.chunks_exact(2)
                .map(|v| u16::from_be_bytes([v[0], v[1]])),
        )
        .collect::<Result<String, _>>()
        .map_err(EncodingError::NotValidUtf16Be)?
    } else {
        char::decode_utf16(
            buf.chunks_exact(2)
                .map(|v| u16::from_le_bytes([v[0], v[1]])),
        )
        .collect::<Result<String, _>>()
        .map_err(EncodingError::NotValidUtf16Le)?
    };

    *buf = result.into_bytes();
    Ok(())
}

/// Convert UTF-8 to ISO-8859-1(Latin-1)
pub fn convert_utf8_to_iso8859_1(buf: &mut Vec<u8>) -> Result<(), EncodingError> {
    let len = buf.len();
    let mut i = 0;
    let mut j = 0;

    while i < len {
        if buf[i] < 0x80 {
            buf[j] = buf[i];
        } else if i < len - 1 {
            if buf[i] == 0xC2 {
                i += 1;
                buf[j] = buf[i];
            } else if buf[i] == 0xC3 {
                i += 1;
                buf[j] = buf[i] + 0x40;
            } else {
                return Err(EncodingError::NotValidUtf8ToIso8859_1(buf[i], buf[i + 1]));
            }
        } else {
            return Err(EncodingError::NotValidUtf8OnlyOneCharacterRemaining);
        }

        i += 1;
        j += 1;
    }

    buf.truncate(j);
    Ok(())
}

pub fn decode_shift_jis(buf: &[u8]) -> Result<String, EncodingError> {
    let (decoded, has_error) = encoding_rs::SHIFT_JIS.decode_without_bom_handling(buf);
    if has_error {
        Err(EncodingError::UnknownEncoding)
    } else {
        Ok(decoded.into_owned())
    }
}
