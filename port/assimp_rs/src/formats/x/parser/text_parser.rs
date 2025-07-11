use std::borrow::Cow;

use crate::{
    AiReal,
    formats::x::{errors::XFileParseError, parser::XFileParser},
    utils::fast_atof::fast_atoreal_move,
};

pub struct TextParser<'source> {
    source: &'source [u8],
    pub line_number: u32,
}

impl<'source> TextParser<'source> {
    pub fn new(source: &'source [u8]) -> Self {
        Self {
            source,
            line_number: 1,
        }
    }
}

impl<'source> XFileParser<'source> for TextParser<'source> {
    fn get_position(&self) -> String {
        format!("Line {}", self.line_number)
    }

    #[inline(always)]
    fn rest(&self) -> usize {
        self.source.len()
    }

    fn forward(&mut self, n: usize) -> Result<&'source [u8], XFileParseError> {
        let (data, rest) = self
            .source
            .split_at_checked(n)
            .ok_or(XFileParseError::unexpected_end_of_file("forward"))?;
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
        self.source.split_first().map(|(b, _)| *b)
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
            if self.rest() == 0 {
                return;
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

    fn read_int(&mut self) -> Result<u32, XFileParseError> {
        self.skip_whitespace();
        let Some(b) = self.peek_one() else {
            return Err(XFileParseError::NotEnoughDataToRead(1));
        };
        let is_neg: bool = if b == b'-' {
            // SAFETY: we know that the next byte is a '-'
            unsafe { self.forward_unchecked(1) };
            true
        } else {
            if !b.is_ascii_digit() {
                return Err(XFileParseError::ExpectNumberDigit(b));
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
        return Ok(if is_neg {
            (-(value as i32)) as u32
        } else {
            value
        });
    }

    fn read_float(&mut self) -> Result<AiReal, XFileParseError> {
        self.skip_whitespace();

        // check for various special strings to allow reading files from faulty exporters
        // I mean you, Blender!
        let special_string = self.peek::<9>();

        if special_string == Some(b"-1.#IND00") {
            // SAFETY: we know that the next 8 bytes are a special string
            unsafe { self.forward_unchecked(8) };
            self.check_for_separator()?;
            return Ok(0.0);
        } else if matches!(self.peek::<8>(), Some(b"1.#IND00") | Some(b"1.#QNAN0")) {
            // SAFETY: we know that the next 8 bytes are a special string
            unsafe { self.forward_unchecked(8) };
            self.check_for_separator()?;
            return Ok(0.0);
        }
        let (rest, f) =
            fast_atoreal_move(self.source, true).map_err(|e| XFileParseError::FastAtofError(e))?;

        self.source = rest;
        self.check_for_separator()?;
        Ok(f)
    }

    fn next_token(&mut self) -> Result<&'source [u8], XFileParseError> {
        self.skip_whitespace();
        if self.rest() == 0 {
            return Ok(&[]);
        }
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
        let token = &self.source[..index];
        self.source = next;
        return Ok(token);
    }

    fn next_token_as_str(&mut self) -> Result<Cow<'source, str>, XFileParseError> {
        self.skip_whitespace();
        let Some(b) = self.peek_one() else {
            return Err(XFileParseError::unexpected_end_of_file("next_token_as_str"));
        };
        if b != b'"' {
            return Err(XFileParseError::unexpected_token("\"", &[b]));
        }
        // SAFETY: we know that the next byte is '"'
        unsafe { self.forward_unchecked(1) };
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
            .map_err(|_| XFileParseError::unexpected_end_of_file("next_token_as_str"))?;
        if deliminator != b"\";" {
            return Err(XFileParseError::unexpected_token("\";", deliminator));
        }
        Ok(String::from_utf8_lossy(token))
    }

    fn check_for_semicolon(&mut self) -> Result<(), XFileParseError> {
        let next = self.next_token()?;
        if next != b";" {
            return Err(XFileParseError::SemicolonExpected(
                String::from_utf8_lossy(next).into_owned(),
            ));
        }
        Ok(())
    }

    fn check_for_separator(&mut self) -> Result<(), XFileParseError> {
        let next = self.next_token()?;
        if !matches!(next, b"," | b";") {
            return Err(XFileParseError::SeparatorCharacterExpected(
                String::from_utf8_lossy(next).into_owned(),
            ));
        }
        Ok(())
    }

    fn test_for_separator(&mut self) {
        self.skip_whitespace();
        if let Some(b) = self.peek_one() {
            if matches!(b, b',' | b';') {
                // SAFETY: we know that the next byte is a separator
                unsafe { self.forward_unchecked(1) };
            }
        }
    }
}
