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

//! Implements text X file parser for the library

use alloc::{
    borrow::{Cow, ToOwned},
    string::String,
};

use crate::{
    AiReal,
    assets::formats::x::{error::XFileCommonParseError, parser::XFileParser},
    io::utils::atof::fast_atoreal_move,
};

pub struct TextParser<'source, const CHECK_ILL_FLOAT_FOR_FAULTY_EXPORTERS: bool> {
    source: &'source [u8],
    pub line_number: u32,
}

impl<'source, const CHECK_ILL_FLOAT_FOR_FAULTY_EXPORTERS: bool>
    TextParser<'source, CHECK_ILL_FLOAT_FOR_FAULTY_EXPORTERS>
{
    /// Source should be valid UTF-8 encoded text.
    pub fn new(source: &'source [u8]) -> Self {
        Self {
            source,
            line_number: 1,
        }
    }
}

impl<'source, const CHECK_ILL_FLOAT_FOR_FAULTY_EXPORTERS: bool> XFileParser<'source>
    for TextParser<'source, CHECK_ILL_FLOAT_FOR_FAULTY_EXPORTERS>
{
    fn get_position_info(&self) -> String {
        format!("Line {}", self.get_position())
    }

    fn get_position(&self) -> usize {
        self.line_number as usize
    }

    fn forward(&mut self, n: usize) -> Result<&'source [u8], XFileCommonParseError> {
        let (data, rest) = self
            .source
            .split_at_checked(n)
            .ok_or(XFileCommonParseError::unexpected_end_of_file("forward"))?;
        self.source = rest;
        Ok(data)
    }

    unsafe fn forward_unchecked(&mut self, n: usize) -> &'source [u8] {
        let (data, rest) = unsafe { self.source.split_at_unchecked(n) };
        self.source = rest;
        data
    }

    fn peek<const N: usize>(&self) -> Option<&'source [u8; N]> {
        let (data, _) = self.source.split_at_checked(N)?;
        Some(data.try_into().unwrap())
    }

    fn peek_one(&self) -> Option<u8> {
        self.source.first().copied()
    }

    fn skip_until_next_line(&mut self) {
        while let &[b, ref rest @ ..] = self.source {
            self.source = rest;
            if b == b'\n' || b == b'\r' {
                // process '\r\n' on windows
                self.next_byte_if_eq(b'\n');
                self.line_number += 1;
                break;
            }
        }
    }

    fn skip_whitespace(&mut self) {
        loop {
            while let &[b, ref rest @ ..] = self.source {
                if b.is_ascii_whitespace() {
                    self.line_number += (b == b'\n') as u32;
                    self.source = rest;
                } else {
                    break;
                }
            }
            if let &[a, b, ref rest @ ..] = self.source {
                if a == b'/' && b == b'/' || a == b'#' {
                    self.source = rest;
                    self.skip_until_next_line();
                } else {
                    break;
                }
            } else {
                break;
            }
        }
    }

    fn read_int(&mut self) -> Result<u32, XFileCommonParseError> {
        self.skip_whitespace();
        let Some(b) = self.peek_one() else {
            return Err(XFileCommonParseError::NotEnoughDataToRead(1));
        };
        let is_neg: bool = if b == b'-' {
            // SAFETY: we know that the next byte is a '-'
            unsafe { self.forward_unchecked(1) };
            true
        } else {
            if !b.is_ascii_digit() {
                return Err(XFileCommonParseError::ExpectNumberDigit(b));
            }
            false
        };
        let mut value = 0;
        while let &[b, ref rest @ ..] = self.source {
            if b.is_ascii_digit() {
                value = value * 10 + (b - b'0') as u32;
                self.source = rest;
            } else {
                break;
            }
        }
        self.check_for_separator()?;
        Ok(if is_neg {
            (-(value as i32)) as u32
        } else {
            value
        })
    }

    fn read_float(&mut self) -> Result<AiReal, XFileCommonParseError> {
        self.skip_whitespace();

        // check for various special strings to allow reading files from faulty exporters
        // I mean you, Blender!
        if const { CHECK_ILL_FLOAT_FOR_FAULTY_EXPORTERS } {
            if self.peek::<9>() == Some(b"-1.#IND00") {
                // SAFETY: we know that the next 9 bytes are a special string
                unsafe { self.forward_unchecked(9) };
                self.check_for_separator()?;
                return Ok(0.0);
            } else if match self.peek::<8>() {
                Some(special_string) => {
                    special_string == b"1.#IND00" || special_string == b"1.#QNAN0"
                }
                None => false,
            } {
                // SAFETY: we know that the next 8 bytes are a special string
                unsafe { self.forward_unchecked(8) };
                self.check_for_separator()?;
                return Ok(0.0);
            }
        }
        let (rest, f) =
            fast_atoreal_move(self.source).map_err(XFileCommonParseError::FastAtofError)?;

        self.source = rest;
        self.check_for_separator()?;
        Ok(f)
    }

    fn next_token(&mut self) -> &'source [u8] {
        self.skip_whitespace();
        let mut index = 0;
        let mut next = self.source;
        while let &[b, ref rest @ ..] = next {
            if b.is_ascii_whitespace() {
                break;
            }
            if matches!(b, b';' | b'}' | b'{' | b',') {
                if index == 0 {
                    next = rest;
                    index += 1;
                }
                break;
            }
            next = rest;
            index += 1;
        }
        // SAFETY: index is within the bounds of the source.
        let token = unsafe { self.source.get_unchecked(..index) };
        self.source = next;
        token
    }

    fn next_token_as_str(&mut self) -> Result<Cow<'source, str>, XFileCommonParseError> {
        self.skip_whitespace();
        match self.peek_one() {
            Some(b) => {
                if b != b'"' {
                    return Err(XFileCommonParseError::unexpected_token("\"", &[b]));
                }
                // SAFETY: we know that the next byte is a '"'
                unsafe { self.forward_unchecked(1) };
            }
            None => {
                return Err(XFileCommonParseError::unexpected_end_of_file(
                    "next_token_as_str",
                ));
            }
        }

        let mut cnt = 0;
        for b in self.source {
            if *b == b'"' {
                break;
            }
            cnt += 1;
        }
        // SAFETY: cnt is within the bounds of the source.
        let token = unsafe { self.forward_unchecked(cnt) };
        let deliminator = self
            .forward(2)
            .map_err(|_| XFileCommonParseError::unexpected_end_of_file("next_token_as_str"))?;
        if deliminator != b"\";" {
            return Err(XFileCommonParseError::unexpected_token("\";", deliminator));
        }
        Ok(str::from_utf8(token)
            .map_err(XFileCommonParseError::Utf8ConversionError)?
            .into())
    }

    fn check_for_semicolon(&mut self) -> Result<(), XFileCommonParseError> {
        let next = self.next_token();
        if next != b";" {
            return Err(XFileCommonParseError::SemicolonExpected(
                match str::from_utf8(next) {
                    Ok(s) => s.to_owned(),
                    Err(_) => format!("bytes: {next:?}"),
                },
            ));
        }
        Ok(())
    }

    fn check_for_separator(&mut self) -> Result<(), XFileCommonParseError> {
        let next = self.next_token();
        if !matches!(next, b"," | b";") {
            return Err(XFileCommonParseError::SeparatorCharacterExpected(
                match str::from_utf8(next) {
                    Ok(s) => s.to_owned(),
                    Err(_) => format!("bytes: {next:?}"),
                },
            ));
        }
        Ok(())
    }

    fn test_for_separator(&mut self) {
        self.skip_whitespace();
        if let Some(b) = self.peek_one()
            && matches!(b, b',' | b';')
        {
            // SAFETY: we know that the next byte is a separator
            unsafe { self.forward_unchecked(1) };
        }
    }

    fn consume_version_specific_semicolon(&mut self) {
        self.next_byte_if_eq(b';');
    }
}
