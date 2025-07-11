use std::io::Cursor;

use byteorder::{LE, ReadBytesExt};

pub struct BinaryReader<'a> {
    source: Cursor<&'a [u8]>,
}

impl<'a> BinaryReader<'a> {
    pub fn new(source: &'a [u8]) -> Self {
        Self {
            source: Cursor::new(source),
        }
    }

    pub fn read_u8(&mut self) -> Option<u8> {
        self.source.read_u8().ok()
    }

    // pub fn read_u8xn<const N: usize>(&mut self) -> Option<[u8; N]> {
    //     let mut buf = [0; N];
    //     self.source.read_u8_into::<LE, [u8; N]>(&mut buf)?;
    //     Some(buf)
    // }

    pub fn read_u16(&mut self) -> Option<u16> {
        self.source.read_u16::<LE>().ok()
    }
}

/// Parse 4 bytes read from bytes into 4 digits.
/// Taken from https://github.com/Alexhuszagh/rust-lexical/blob/988575dad6de2a9e86b34fff242c5f0a6e3dbf2c/lexical-parse-integer/src/algorithm.rs#L259
#[inline]
pub fn parse_4digits_decimal(mut v: u32) -> u32 {
    let radix = 10;
    v -= 0x3030_3030;
    // Scale digits in `0 <= Nn <= 99`.
    v = (v * radix) + (v >> 8);
    // Scale digits in `0 <= Nnnn <= 9999`.
    v = ((v & 0x0000007f) * radix * radix) + ((v >> 16) & 0x0000007f);

    v
}
