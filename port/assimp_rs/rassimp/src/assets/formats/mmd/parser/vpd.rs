use alloc::borrow::Cow;

use byteorder::LE;
use glam::{Vec2, Vec3, Vec4};

use crate::{
    assets::formats::mmd::{
        error::MMDImportError,
        parser::{
            error::{MMDParseCommonError, MMDParseError},
            vpd::structs::VPDFile,
        },
    },
    io::reader::text_reader::{CommonTextReader, Utf8TextReader},
};

pub mod error;
pub mod structs;

pub struct VPDParser<'source, 'other> {
    reader: CommonTextReader<'source, 'other>,
}

impl<'source, 'other> VPDParser<'source, 'other> {
    pub fn new(source: &'source [u8]) -> Self {
        Self {
            reader: CommonTextReader::new(source),
        }
    }

    pub fn parse(&mut self) -> Result<VPDFile, MMDImportError> {
        match VPDFile::read(self) {
            Ok(file) => Ok(file),
            Err(e) => Err(MMDImportError::ParseError {
                error: e,
                position: self.position_info(),
            }),
        }
    }

    fn position_info(&self) -> String {
        format!("line {}", self.reader.line_number())
    }

    pub(crate) fn next_token(&mut self) -> &[u8] {
        self.reader.next_token()
    }

    pub(crate) fn check_for_separator(&mut self, separator: u8) -> Result<(), &[u8]> {
        self.reader.check_for_separator(separator)
    }

    pub(crate) fn check_for_comma(&mut self) -> Result<(), &[u8]> {
        self.reader.check_for_comma()
    }

    pub(crate) fn read_f32(&mut self) -> Option<f32> {
        self.reader.read_f32()
    }

    pub(crate) fn read_unsigned_integer(&mut self) -> Option<u64> {
        self.reader.read_unsigned_integer()
    }
}

pub(crate) trait VPDRead {
    fn read<'source, 'other>(
        parser: &mut VPDParser<'source, 'other>,
    ) -> Result<Self, MMDParseError>
    where
        Self: Sized;
}
