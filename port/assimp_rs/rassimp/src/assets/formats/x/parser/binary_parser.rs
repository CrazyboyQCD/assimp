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

//! Implements binary X file parser for the library

use alloc::{borrow::Cow, string::String};
use core::mem::size_of;

use byteorder::LE;

use crate::{
    AiReal,
    assets::formats::x::{
        error::{XFileBinaryParseError, XFileCommonParseError},
        parser::{XFileHeader, XFileParser, constants::binary_tokens::*},
    },
    io::reader::binary_reader::BinaryBufferReader,
};

pub struct BinaryParser<'source, const IS_64_BITS: bool> {
    binary_buffer_reader: BinaryBufferReader<'source, LE>,
    binary_num_count: u32,
}

impl<'source, const IS_64_BITS: bool> BinaryParser<'source, IS_64_BITS> {
    const FLOAT_SIZE: usize = if IS_64_BITS {
        size_of::<f64>()
    } else {
        size_of::<f32>()
    };
    pub fn new(source: &'source [u8]) -> Self {
        Self {
            binary_buffer_reader: BinaryBufferReader::new(source),
            binary_num_count: 0,
        }
    }

    fn read_binary_word(&mut self) -> Result<u16, XFileCommonParseError> {
        let word = self
            .forward(2)
            .map_err(|_| XFileCommonParseError::from(XFileBinaryParseError::ReadBinaryWordError))?;
        Ok(u16::from_le_bytes([word[0], word[1]]))
    }

    fn read_binary_dword(&mut self) -> Result<u32, XFileCommonParseError> {
        let dword = self.forward(4).map_err(|_| {
            XFileCommonParseError::from(XFileBinaryParseError::ReadBinaryDwordError)
        })?;
        Ok(u32::from_le_bytes([dword[0], dword[1], dword[2], dword[3]]))
    }
}

impl<'source, const IS_64_BITS: bool> XFileParser<'source> for BinaryParser<'source, IS_64_BITS> {
    fn get_position_info(&self) -> String {
        format!("Offset {:X}", self.get_position())
    }

    fn get_position(&self) -> usize {
        self.binary_buffer_reader.offset() + XFileHeader::HEADER_BINARY_SIZE
    }

    /// # Safety:
    ///
    /// Caller must gurantee that the buffer is at least `n` bytes long.
    unsafe fn forward_unchecked(&mut self, n: usize) -> &'source [u8] {
        // SAFETY: Caller gurantees that the buffer is at least `n` bytes long
        unsafe { self.binary_buffer_reader.forward_unchecked(n) }
    }

    fn peek<const N: usize>(&self) -> Option<&'source [u8; N]> {
        self.binary_buffer_reader.peek::<N>()
    }

    fn peek_one(&self) -> Option<u8> {
        self.binary_buffer_reader.first()
    }

    fn forward(&mut self, n: usize) -> Result<&'source [u8], XFileCommonParseError> {
        let s = self
            .binary_buffer_reader
            .forward(n)
            .ok_or(XFileCommonParseError::UnexpectedEndOfFile { context: "forward" })?;
        assert!(s.len() == n);
        Ok(s)
    }

    fn read_int(&mut self) -> Result<u32, XFileCommonParseError> {
        if self.binary_num_count == 0 {
            let token = self.read_binary_word()?;
            if token == TOKEN_INTEGER_LIST {
                // array of integers following
                self.binary_num_count = self.read_binary_dword()?;
            } else {
                // single float following
                self.binary_num_count = 1;
            }
        }
        self.binary_num_count -= 1;
        Ok(if let Ok(dword) = self.read_binary_dword() {
            dword
        } else {
            self.binary_buffer_reader.clear();
            0
        })
    }

    fn read_float(&mut self) -> Result<AiReal, XFileCommonParseError> {
        if self.binary_num_count == 0 {
            let token = self.read_binary_word()?;
            if token == TOKEN_FLOAT_LIST {
                // array of floats following
                self.binary_num_count = self.read_binary_dword()?;
            } else {
                // single float following
                self.binary_num_count = 1;
            }
        }
        self.binary_num_count -= 1;
        if let Ok(float) = self.forward(Self::FLOAT_SIZE) {
            assert!(float.len() == Self::FLOAT_SIZE);
            if const { IS_64_BITS } {
                // SAFETY: we know that the next 8 bytes are a double
                Ok(f64::from_le_bytes([
                    float[0], float[1], float[2], float[3], float[4], float[5], float[6], float[7],
                ]) as AiReal)
            } else {
                // SAFETY: we know that the next 4 bytes are a float
                Ok(f32::from_le_bytes([float[0], float[1], float[2], float[3]]) as AiReal)
            }
        } else {
            self.binary_buffer_reader.clear();
            Ok(0.0)
        }
    }

    fn next_token(&mut self) -> &'source [u8] {
        let Ok(token) = self.read_binary_word() else {
            return &[];
        };

        match token {
            TOKEN_NAME => {
                let Ok(len) = self.read_binary_dword() else {
                    return &[];
                };
                let Ok(s) = self.forward(len as usize) else {
                    return &[];
                };
                s
            }
            TOKEN_STRING => {
                let Ok(len) = self.read_binary_dword() else {
                    return &[];
                };
                let Ok(s) = self.forward(len as usize + 2) else {
                    return &[];
                };
                &s[..s.len() - 2]
            }
            TOKEN_INTEGER => {
                let _ = self.forward(4);
                b"<integer>"
            }
            TOKEN_GUID => {
                let _ = self.forward(16);
                b"<guid>"
            }
            TOKEN_INTEGER_LIST => {
                let Ok(count) = self.read_binary_dword() else {
                    return &[];
                };
                let size = count as usize * size_of::<u32>();
                let _ = self.forward(size);
                b"<int_list>"
            }
            TOKEN_FLOAT_LIST => {
                let Ok(count) = self.read_binary_dword() else {
                    return &[];
                };
                let size = count as usize * Self::FLOAT_SIZE;
                let _ = self.forward(size);
                b"<flt_list>"
            }
            TOKEN_OBRACE => b"{",
            TOKEN_CBRACE => b"}",
            TOKEN_OPAREN => b"(",
            TOKEN_CPAREN => b")",
            TOKEN_OBRACKET => b"[",
            TOKEN_CBRACKET => b"]",
            TOKEN_OANGLE => b"<",
            TOKEN_CANGLE => b">",
            TOKEN_DOT => b".",
            TOKEN_COMMA => b",",
            TOKEN_SEMICOLON => b";",
            TOKEN_TEMPLATE => b"template",
            TOKEN_WORD => b"WORD",
            TOKEN_DWORD => b"DWORD",
            TOKEN_FLOAT => b"FLOAT",
            TOKEN_DOUBLE => b"DOUBLE",
            TOKEN_CHAR => b"CHAR",
            TOKEN_UCHAR => b"UCHAR",
            TOKEN_SWORD => b"SWORD",
            TOKEN_SDWORD => b"SDWORD",
            TOKEN_VOID => b"void",
            TOKEN_LPSTR => b"string",
            TOKEN_UNICODE => b"unicode",
            TOKEN_CSTRING => b"cstring",
            TOKEN_ARRAY => b"array",
            _ => &[],
        }
    }

    fn next_token_as_str(&mut self) -> Result<Cow<'source, str>, XFileCommonParseError> {
        let token = self.next_token();
        if let Ok(s) = str::from_utf8(token) {
            Ok(s.into())
        } else {
            if let Some((encoding, rest)) = match token.split_at_checked(2) {
                Some(([0xFF, 0xFE], rest)) => Some((encoding_rs::UTF_16LE, rest)), // UTF-16LE BOM
                Some(([0xFE, 0xFF], rest)) => Some((encoding_rs::UTF_16BE, rest)), // UTF-16BE BOM
                _ => None,
            } {
                let (s, has_errors) = encoding.decode_without_bom_handling(rest);
                if !has_errors {
                    return Ok(s);
                }
            }

            for encoding in [
                encoding_rs::SHIFT_JIS,
                encoding_rs::GBK,
                encoding_rs::GB18030,
            ] {
                let (s, has_error) = encoding.decode_without_bom_handling(token);
                if !has_error {
                    return Ok(s);
                }
            }

            Err(XFileCommonParseError::UnknownEncoding(token.to_owned()))
        }
    }
}
