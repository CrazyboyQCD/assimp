use byteorder::{LE, ReadBytesExt};
use std::io::Cursor;

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
