use core::mem::size_of;
use std::borrow::Cow;

use crate::{
    AiReal,
    formats::x::{
        errors::{XFileBinaryParseError, XFileParseError},
        parser::{XFileHeader, XFileParser},
    },
};

pub struct BinaryParser<'source> {
    start: usize,
    source: &'source [u8],
    pub binary_float_size: u8,
    pub binary_num_count: u32,
}

impl<'source> BinaryParser<'source> {
    pub fn new(source: &'source [u8], binary_float_size: u8) -> Self {
        Self {
            start: source.as_ptr() as usize,
            source,
            binary_float_size,
            binary_num_count: 0,
        }
    }

    fn offset(&self) -> usize {
        self.source.as_ptr() as usize - self.start + XFileHeader::HEADER_BINARY_SIZE
    }

    fn read_binary_word(&mut self) -> Result<u16, XFileParseError> {
        let word = self
            .forward(2)
            .map_err(|_| XFileParseError::from(XFileBinaryParseError::ReadBinaryWordError))?;
        Ok(u16::from_le_bytes([word[0], word[1]]))
    }

    unsafe fn read_binary_word_unchecked(&mut self) -> u16 {
        let word = unsafe { self.forward_unchecked(2) };
        u16::from_le_bytes([word[0], word[1]])
    }

    fn read_binary_dword(&mut self) -> Result<u32, XFileParseError> {
        let dword = self
            .forward(4)
            .map_err(|_| XFileParseError::from(XFileBinaryParseError::ReadBinaryDwordError))?;
        Ok(u32::from_le_bytes([dword[0], dword[1], dword[2], dword[3]]))
    }

    unsafe fn read_binary_dword_unchecked(&mut self) -> u32 {
        let dword = unsafe { self.forward_unchecked(4) };
        u32::from_le_bytes([dword[0], dword[1], dword[2], dword[3]])
    }
}

impl<'source> XFileParser<'source> for BinaryParser<'source> {
    fn get_position(&self) -> String {
        format!("Offset {:X}", self.offset())
    }

    unsafe fn forward_unchecked(&mut self, n: usize) -> &'source [u8] {
        let (data, rest) = unsafe { self.source.split_at_unchecked(n) };
        self.source = rest;
        data
    }

    fn peek<const N: usize>(&self) -> Option<&'source [u8; N]> {
        self.source.get(..N).map(|slice| slice.try_into().unwrap())
    }

    #[inline(always)]
    fn rest(&self) -> usize {
        self.source.len()
    }

    fn forward(&mut self, n: usize) -> Result<&'source [u8], XFileParseError> {
        let (data, rest) = self
            .source
            .split_at_checked(n)
            .ok_or(XFileParseError::UnexpectedEndOfFile { context: "forward" })?;
        self.source = rest;
        Ok(data)
    }

    fn read_int(&mut self) -> Result<u32, XFileParseError> {
        if self.binary_num_count == 0 && self.rest() >= 2 {
            // SAFETY: we know that the next 2 bytes are a word
            let tmp = unsafe { self.read_binary_word_unchecked() };
            if tmp == 0x06 && self.rest() >= 4 {
                // array of floats following
                // SAFETY: we know that the next 4 bytes are a dword
                self.binary_num_count = unsafe { self.read_binary_dword_unchecked() };
            } else {
                // single float following
                self.binary_num_count = 1;
            }
        }
        self.binary_num_count -= 1;
        if self.rest() >= 4 {
            // SAFETY: we know that the next 4 bytes are a dword
            return Ok(unsafe { self.read_binary_dword_unchecked() });
        } else {
            self.source = &[];
            return Ok(0);
        }
    }

    fn read_float(&mut self) -> Result<AiReal, XFileParseError> {
        if self.binary_num_count == 0 && self.rest() >= 2 {
            // SAFETY: we know that the next 2 bytes are a word
            let tmp = unsafe { self.read_binary_word_unchecked() };
            if tmp == 0x07 && self.rest() >= 4 {
                // array of floats following
                // SAFETY: we know that the next 4 bytes are a dword
                self.binary_num_count = unsafe { self.read_binary_dword_unchecked() };
            } else {
                // single float following
                self.binary_num_count = 1;
            }
        }
        self.binary_num_count -= 1;
        if self.binary_float_size == 8 {
            if self.rest() >= 8 {
                // SAFETY: we know that the next 8 bytes are a double
                return Ok(f64::from_le_bytes(
                    unsafe { self.forward_unchecked(8) }.try_into().unwrap(),
                ) as f32);
            } else {
                self.source = &[];
                return Ok(0.0);
            }
        } else {
            if self.rest() >= 4 {
                return Ok(f32::from_le_bytes(
                    unsafe { self.forward_unchecked(4) }.try_into().unwrap(),
                ));
            } else {
                self.source = &[];
                return Ok(0.0);
            }
        }
    }

    fn next_token(&mut self) -> Result<&'source [u8], XFileParseError> {
        let Ok(token) = self.read_binary_word() else {
            return Ok(&[]);
        };
        // References:
        // https://learn.microsoft.com/en-us/windows/win32/direct3d9/tokens
        // https://learn.microsoft.com/en-us/windows/win32/direct3d9/token-records
        match token {
            1 => {
                let Ok(len) = self.read_binary_dword() else {
                    return Ok(&[]);
                };
                let Ok(s) = self.forward(len as usize) else {
                    return Ok(&[]);
                };
                return Ok(s);
            }
            2 => {
                let Ok(len) = self.read_binary_dword() else {
                    return Ok(&[]);
                };
                let Ok(s) = self.forward(len as usize + 2) else {
                    return Ok(&[]);
                };
                return Ok(&s[..s.len() - 2]);
            }
            3 => {
                let _ = self.forward(4);
                return Ok(b"<integer>");
            }
            5 => {
                let _ = self.forward(16);
                return Ok(b"<guid>");
            }
            6 => {
                let Ok(count) = self.read_binary_dword() else {
                    return Ok(&[]);
                };
                let size = count as usize * size_of::<u32>();
                let _ = self.forward(size);
                return Ok(b"<int_list>");
            }
            7 => {
                let Ok(count) = self.read_binary_dword() else {
                    return Ok(&[]);
                };
                let size = count as usize * self.binary_float_size as usize;
                let _ = self.forward(size);
                return Ok(b"<flt_list>");
            }
            0x0a => {
                return Ok(b"{");
            }
            0x0b => {
                return Ok(b"}");
            }
            0x0c => {
                return Ok(b"(");
            }
            0x0d => {
                return Ok(b")");
            }
            0x0e => {
                return Ok(b"[");
            }
            0x0f => {
                return Ok(b"]");
            }
            0x10 => {
                return Ok(b"<");
            }
            0x11 => {
                return Ok(b">");
            }
            0x12 => {
                return Ok(b".");
            }
            0x13 => {
                return Ok(b",");
            }
            0x14 => {
                return Ok(b";");
            }
            0x1f => {
                return Ok(b"template");
            }
            0x28 => {
                return Ok(b"WORD");
            }
            0x29 => {
                return Ok(b"DWORD");
            }
            0x2a => {
                return Ok(b"FLOAT");
            }
            0x2b => {
                return Ok(b"DOUBLE");
            }
            0x2c => {
                return Ok(b"CHAR");
            }
            0x2d => {
                return Ok(b"UCHAR");
            }
            0x2e => {
                return Ok(b"SWORD");
            }
            0x2f => {
                return Ok(b"SDWORD");
            }
            0x30 => {
                return Ok(b"void");
            }
            0x31 => {
                return Ok(b"string");
            }
            0x32 => {
                return Ok(b"unicode");
            }
            0x33 => {
                return Ok(b"cstring");
            }
            0x34 => {
                return Ok(b"array");
            }
            _ => {
                return Ok(&[]);
            }
        }
    }

    fn next_token_as_str(&mut self) -> Result<Cow<'source, str>, XFileParseError> {
        let token = self.next_token()?;
        Ok(String::from_utf8_lossy(token))
    }
}
