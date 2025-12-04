use core::slice;
use std::io::Read;

use lexical_parse_float::{Options, format::STANDARD, parse::ParseFloat};

pub struct AsciiTextReader<'source> {
    source: &'source [u8],
}

impl<'source> AsciiTextReader<'source> {
    pub fn new(source: &'source [u8]) -> Self {
        Self { source }
    }

    pub(crate) fn forward(&mut self, n: usize) -> Option<&'source [u8]> {
        let (ret, rest) = self.source.split_at_checked(n)?;
        self.source = rest;
        Some(ret)
    }

    /// # Safety:
    ///
    /// Caller must gurantee that the buffer is at least `n` bytes long.
    pub(crate) unsafe fn forward_unchecked(&mut self, n: usize) -> &'source [u8] {
        // SAFETY: Caller gurantees that the buffer is at least `n` bytes long
        let (ret, rest) = unsafe { self.source.split_at_unchecked(n) };
        self.source = rest;
        ret
    }

    pub(crate) fn peek(&self, n: usize) -> Option<&'source [u8]> {
        self.source.get(..n)
    }
}
#[derive(Clone, Copy, Debug)]
pub struct Utf8TextReader<'source> {
    source: &'source str,
}

impl<'source> Utf8TextReader<'source> {
    pub const fn new(source: &'source str) -> Self {
        Self { source }
    }

    pub fn is_eof(&self) -> bool {
        self.source.is_empty()
    }

    pub fn read_line(&mut self) -> Option<&'source str> {
        let pos = self.source.find('\n')?;
        let (ret, rest) = self.source.split_at(pos);
        self.source = rest;
        Some(ret)
    }

    pub fn read_until_no_comments(&mut self) -> Option<&'source str> {
        let pos = self.source.find('#')?;
        let (ret, rest) = self.source.split_at(pos);
        self.source = rest;
        Some(ret)
    }

    pub fn read_lines(&mut self) -> Option<Vec<&'source str>> {
        let mut lines = Vec::new();
        while let Some(line) = self.read_line() {
            lines.push(line);
        }
        Some(lines)
    }

    /// Forward `n` char(s) and return the source string slice.
    pub(crate) fn forward_n_chars(&mut self, n: usize) -> Option<&'source str> {
        if n == 0 {
            Some("")
        } else {
            let mut chars = self.source.chars();
            chars.nth(n - 1)?;
            let pos = self.source.len() - chars.as_str().len();
            // SAFETY: pos should be within the char boundary.
            let (ret, rest) = unsafe { Self::split_at_unchecked(self.source, pos) };
            self.source = rest;
            Some(ret)
        }
    }

    /// Split the source string slice at the given position without checking the validity of the
    /// position.
    ///
    /// # Safety
    ///
    /// The caller must guarantee that `n` is on a char boundary.
    #[inline]
    unsafe fn split_at_unchecked(source: &str, n: usize) -> (&str, &str) {
        let len = source.len();
        let ptr = source.as_ptr();
        // SAFETY: caller guarantees `n` is on a char boundary.
        unsafe {
            (
                str::from_utf8_unchecked(slice::from_raw_parts(ptr, n)),
                str::from_utf8_unchecked(slice::from_raw_parts(ptr.add(n), len - n)),
            )
        }
    }

    /// Forward `n` byte(s) and return the source string slice.
    ///
    /// If position `n` is not on a char boundary, returns `None`.
    #[inline]
    pub(crate) unsafe fn forward_n_bytes(&mut self, n: usize) -> Option<&'source str> {
        let (ret, rest) = self.source.split_at_checked(n)?;
        self.source = rest;
        Some(ret)
    }

    /// Forward `n` byte(s) and return the source string slice without checking the validity of the
    /// position.
    ///
    /// # Safety
    ///
    /// The caller must guarantee that `n` is on a char boundary.
    #[inline]
    pub(crate) unsafe fn forward_n_bytes_unchecked(&mut self, n: usize) -> &'source str {
        // SAFETY: caller guarantees `n` is on a char boundary.
        let (ret, rest) = unsafe { Self::split_at_unchecked(self.source, n) };
        self.source = rest;
        ret
    }

    pub(crate) fn peek(&self, n: usize) -> Option<&'source str> {
        self.source.get(..n)
    }
}

const DEFAULT_TOKEN_SEPERATORS: &[u8] = b";}{,";

#[derive(Clone, Copy, Debug)]
pub struct CommonTextReader<'source, 'other> {
    source: &'source [u8],
    pub line_number: u32,
    pub token_seperators: &'other [u8],
}

impl<'source, 'other> CommonTextReader<'source, 'other> {
    pub fn new(source: &'source [u8]) -> Self {
        Self {
            source,
            line_number: 0,
            token_seperators: DEFAULT_TOKEN_SEPERATORS,
        }
    }

    pub fn line_number(&self) -> u32 {
        self.line_number
    }

    pub fn new_with_token_seperators(source: &'source [u8], seperators: &'other [u8]) -> Self {
        Self {
            source,
            line_number: 0,
            token_seperators: seperators,
        }
    }

    pub fn set_token_seperators(&mut self, seperators: &'other [u8]) {
        self.token_seperators = seperators;
    }

    pub fn next_token(&mut self) -> &'source [u8] {
        self.skip_whitespace();
        let mut index = 0;
        let mut next = self.source;
        while let &[b, ref rest @ ..] = next {
            if b.is_ascii_whitespace() {
                break;
            }
            if self.token_seperators.contains(&b) {
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

    pub fn peek_token(&self) -> (&'source [u8], usize) {
        let (source, start) = self.skip_whitespace_without_forward();
        let mut len = 0;
        let mut next = source;
        while let &[b, ref rest @ ..] = next {
            if b.is_ascii_whitespace() {
                break;
            }
            if self.token_seperators.contains(&b) {
                if len == 0 {
                    next = rest;
                    len += 1;
                }
                break;
            }
            next = rest;
            len += 1;
        }
        // SAFETY: index is within the bounds of the source.
        let token = unsafe { source.get_unchecked(..len) };
        (token, start)
    }

    pub fn skip_whitespace(&mut self) {
        while let &[b, ref rest @ ..] = self.source {
            if !b.is_ascii_whitespace() {
                break;
            }
            self.line_number += (b == b'\n') as u32;
            self.source = rest;
        }
    }

    fn skip_whitespace_without_forward(&self) -> (&'source [u8], usize) {
        let mut source = self.source;
        let mut index = 0;
        while let &[b, ref rest @ ..] = source {
            if !b.is_ascii_whitespace() {
                break;
            }
            source = rest;
            index += 1;
        }
        (source, index)
    }

    pub fn skip_until_next_line(&mut self) {
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

    pub fn forward(&mut self, n: usize) -> Option<&'source [u8]> {
        let (data, rest) = self.source.split_at_checked(n)?;
        self.source = rest;
        Some(data)
    }

    pub unsafe fn forward_unchecked(&mut self, n: usize) -> &'source [u8] {
        let (data, rest) = unsafe { self.source.split_at_unchecked(n) };
        self.source = rest;
        data
    }

    pub fn peek<const N: usize>(&self) -> Option<&'source [u8; N]> {
        let (data, _) = self.source.split_at_checked(N)?;
        Some(data.try_into().unwrap())
    }

    pub fn peek_one(&self) -> Option<u8> {
        self.source.first().copied()
    }

    pub fn check_for_comma(&mut self) -> Result<(), &[u8]> {
        let token = self.next_token();
        if token != b"," {
            return Err(token);
        }
        Ok(())
    }

    pub fn check_for_separator(&mut self, separator: u8) -> Result<(), &[u8]> {
        let token = self.next_token();
        if token != [separator] {
            return Err(token);
        }
        Ok(())
    }

    pub fn read_f32(&mut self) -> Option<f32> {
        let token = self.next_token();
        match f32::fast_path_partial::<STANDARD>(token, const { &Options::new() }) {
            Ok((f, pos)) => {
                // SAFETY: pos should within the bound.
                unsafe { self.forward_unchecked(pos) };
                Some(f)
            }
            Err(_) => None,
        }
    }

    pub fn read_f64(&mut self) -> Option<f64> {
        let token = self.next_token();
        match f64::fast_path_partial::<STANDARD>(token, const { &Options::new() }) {
            Ok((f, pos)) => {
                // SAFETY: pos should within the bound.
                unsafe { self.forward_unchecked(pos) };
                Some(f)
            }
            Err(_) => None,
        }
    }

    fn next_byte_if_eq(&mut self, test_byte: u8) {
        if self.peek_one() == Some(test_byte) {
            // SAFETY: we know that the next byte is the test byte
            unsafe { self.forward_unchecked(1) };
        }
    }
}

// Read Number
impl<'source, 'other> CommonTextReader<'source, 'other> {
    /// Read an unsigned integer from the source.
    ///
    /// # Errors
    ///
    /// Returns `None` if the token is empty or contains non-digit characters.
    pub fn read_unsigned_integer_lossy(
        &mut self,
        forward_if_not_valid: bool,
    ) -> Option<(&'source [u8], u64)> {
        let (mut token, start) = self.peek_token();
        match token {
            &[b, ref rest @ ..] if b.is_ascii_digit() => {
                let mut rest = rest;
                let mut num = (b - b'0') as u64;
                let mut len = 0;
                while let &[b, ref _rest @ ..] = rest {
                    if b.is_ascii_digit() {
                        num = num * 10 + (b - b'0') as u64;
                        len += 1;
                        rest = _rest;
                    } else {
                        break;
                    }
                }
                self.forward(start + len);
                Some((rest, num))
            }
            _ => {
                if forward_if_not_valid {
                    self.forward(start + token.len());
                }
                None
            }
        }
    }

    /// Read an signed integer from the source.
    ///
    /// # Errors
    ///
    /// Returns `None` if the token is empty or contains non-digit characters.
    pub fn read_signed_integer_lossy(
        &mut self,
        forward_if_not_valid: bool,
    ) -> Option<(&'source [u8], i64)> {
        let (token, start) = self.peek_token();
        match token {
            &[maybe_sign, ref rest @ ..] if maybe_sign.is_ascii_digit() || maybe_sign == b'-' => {
                let is_neg = maybe_sign == b'-';
                if is_neg {
                    if rest.is_empty() {
                        return None;
                    } else if let Some(b) = rest.first()
                        && b.is_ascii_digit()
                    {
                        return None;
                    }
                } else if !maybe_sign.is_ascii_digit() {
                    return None;
                }
                let mut num = 0;
                let mut len = if is_neg { 1 } else { 0 };

                let mut iter = if is_neg { rest } else { token };
                while let &[b, ref rest @ ..] = iter {
                    if b.is_ascii_digit() {
                        num = num * 10 + (b - b'0') as i64;
                        len += 1;
                        iter = rest;
                    } else {
                        break;
                    }
                }
                self.forward(start + len);
                Some((iter, if is_neg { -num } else { num }))
            }
            _ => {
                if forward_if_not_valid {
                    self.forward(start + token.len());
                }
                None
            }
        }
    }

    /// Read an unsigned integer from the source.
    ///
    /// # Errors
    ///
    /// Returns `None` if the token is not a valid unsigned integer.
    pub fn peek_unsigned_integer(&self) -> (Option<u64>, &'source [u8], usize, usize) {
        let (token, start) = self.peek_token();
        match token {
            &[b, ref rest @ ..] if b.is_ascii_digit() => {
                let mut rest = rest;
                let mut num = (b - b'0') as u64;
                let mut len = 0;
                while let &[b, ref _rest @ ..] = rest {
                    if b.is_ascii_digit() {
                        // TODO: check for overflow
                        num = num * 10 + (b - b'0') as u64;
                        len += 1;
                        rest = _rest;
                    } else {
                        return (None, token, start, len);
                    }
                }
                (Some(num), token, start, len)
            }
            _ => (None, token, start, token.len()),
        }
    }

    /// Read an unsigned integer from the source.
    ///
    /// # Errors
    ///
    /// Returns `None` if the token is not a valid unsigned integer.
    pub fn read_unsigned_integer(&mut self) -> Option<u64> {
        let token = self.next_token();
        match token {
            &[b, ref rest @ ..] if b.is_ascii_digit() => {
                let mut rest = rest;
                let mut num = (b - b'0') as u64;
                while let &[b, ref _rest @ ..] = rest {
                    if b.is_ascii_digit() {
                        // TODO: check for overflow
                        num = num * 10 + (b - b'0') as u64;
                        rest = _rest;
                    } else {
                        return None;
                    }
                }
                Some(num)
            }
            _ => None,
        }
    }

    /// Read a signed integer from the source.
    ///
    /// # Errors
    ///
    /// Returns `None` if the token is not a valid signed integer.
    pub fn read_signed_integer(&mut self) -> Option<i64> {
        let token = self.next_token();
        match token {
            &[maybe_sign, ref rest @ ..] if maybe_sign.is_ascii_digit() || maybe_sign == b'-' => {
                let is_neg = maybe_sign == b'-';
                let mut num = 0;
                if is_neg {
                    if rest.is_empty() {
                        return None;
                    } else if let Some(b) = rest.first()
                        && b.is_ascii_digit()
                    {
                        return None;
                    }
                } else if !maybe_sign.is_ascii_digit() {
                    return None;
                }
                let iter = if is_neg { rest.iter() } else { token.iter() };
                for &b in iter {
                    if b.is_ascii_digit() {
                        // TODO: check for overflow
                        num = num * 10 + (b - b'0') as i64;
                    } else {
                        return None;
                    }
                }
                Some(if is_neg { -num } else { num })
            }
            _ => None,
        }
    }
}
