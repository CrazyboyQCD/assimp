use byteorder::LE;
use glam::{Vec2, Vec3, Vec4};

use crate::{
    assets::formats::mmd::{
        STRING_ENCODINGS,
        error::MMDImportError,
        parser::{
            error::{MMD_COMMON_ERROR_UNEXPECTED_EOS, MMDParseCommonError, MMDParseError},
            pmx::{
                error::PmxParseError,
                structs::{PMXIndex, PmxModel, PmxSetting},
            },
        },
    },
    io::{
        importer::{error::CommonImportError, traits::EmptyConfig},
        reader::binary_reader::{BinaryBufferReader, BinaryRead},
    },
};

pub mod error;
pub mod structs;

pub struct PMXParser<'source> {
    binary_reader: BinaryBufferReader<'source, LE>,
}

impl<'source> PMXParser<'source> {
    pub const fn new(source: &'source [u8]) -> Self {
        Self {
            binary_reader: BinaryBufferReader::new(source),
        }
    }

    pub fn parse(&mut self) -> Result<PmxModel, MMDImportError> {
        match PmxModel::read(self) {
            Ok(model) => Ok(model),
            Err(e) => Err(MMDImportError::ParseError {
                error: e,
                position: self.position_info(),
            }),
        }
    }

    fn position_info(&self) -> String {
        format!("offset {}", self.binary_reader.offset())
    }

    fn read_index(&mut self, size: PMXIndex) -> Result<i32, MMDParseError> {
        match size {
            PMXIndex::Single => {
                let index = self
                    .binary_reader
                    .read_i8()
                    .ok_or(MMD_COMMON_ERROR_UNEXPECTED_EOS)?;
                Ok(index as i32)
            }
            PMXIndex::Double => {
                let index = self
                    .binary_reader
                    .read_i16()
                    .ok_or(MMD_COMMON_ERROR_UNEXPECTED_EOS)?;
                Ok(index as i32)
            }
            PMXIndex::Quadruple => {
                let index = self
                    .binary_reader
                    .read_i32()
                    .ok_or(MMD_COMMON_ERROR_UNEXPECTED_EOS)?;
                Ok(index)
            }
        }
    }

    fn read_string(&mut self, encoding: u8) -> Result<String, MMDParseError> {
        let size = self
            .binary_reader
            .read_u32()
            .ok_or(MMD_COMMON_ERROR_UNEXPECTED_EOS)?;
        if size == 0 {
            return Ok(String::new());
        }
        let mut buf = vec![0; size as usize];
        self.binary_reader
            .read_into_buffer(&mut buf)
            .ok_or(MMD_COMMON_ERROR_UNEXPECTED_EOS)?;
        if encoding == 0 {
            for encoding in STRING_ENCODINGS {
                let (s, has_error) = encoding.decode_without_bom_handling(&buf);
                if !has_error {
                    return Ok(s.into_owned());
                }
            }
            Err(PmxParseError::UnknownEncodedString(buf))?
        } else {
            match str::from_utf8(&buf) {
                Ok(s) => Ok(s.to_owned()),
                Err(_) => Err(PmxParseError::UnknownEncodedString(buf))?,
            }
        }
    }

    fn read_index_size(&mut self) -> Result<PMXIndex, MMDParseError> {
        match self.binary_reader.read_u8() {
            Some(v) => Ok(PMXIndex::try_from(v)?),
            None => Err(MMD_COMMON_ERROR_UNEXPECTED_EOS)?,
        }
    }

    fn read_u8(&mut self) -> Result<u8, MMDParseError> {
        match self.binary_reader.read_u8() {
            Some(v) => Ok(v),
            None => Err(MMD_COMMON_ERROR_UNEXPECTED_EOS)?,
        }
    }

    fn read_i8(&mut self) -> Result<i8, MMDParseError> {
        match self.binary_reader.read_i8() {
            Some(v) => Ok(v),
            None => Err(MMD_COMMON_ERROR_UNEXPECTED_EOS)?,
        }
    }

    fn read_u16(&mut self) -> Result<u16, MMDParseError> {
        match self.binary_reader.read_u16() {
            Some(v) => Ok(v),
            None => Err(MMD_COMMON_ERROR_UNEXPECTED_EOS)?,
        }
    }

    fn read_u32(&mut self) -> Result<u32, MMDParseError> {
        match self.binary_reader.read_u32() {
            Some(v) => Ok(v),
            None => Err(MMD_COMMON_ERROR_UNEXPECTED_EOS)?,
        }
    }

    fn read_f32(&mut self) -> Result<f32, MMDParseError> {
        match self.binary_reader.read_f32() {
            Some(v) => Ok(v),
            None => Err(MMD_COMMON_ERROR_UNEXPECTED_EOS)?,
        }
    }

    fn read_vec2(&mut self) -> Result<Vec2, MMDParseError> {
        match self.binary_reader.read_vec2() {
            Some(v) => Ok(v),
            None => Err(MMD_COMMON_ERROR_UNEXPECTED_EOS)?,
        }
    }

    fn read_vec3(&mut self) -> Result<Vec3, MMDParseError> {
        match self.binary_reader.read_vec3() {
            Some(v) => Ok(v),
            None => Err(MMD_COMMON_ERROR_UNEXPECTED_EOS)?,
        }
    }

    fn read_vec4(&mut self) -> Result<Vec4, MMDParseError> {
        match self.binary_reader.read_vec4() {
            Some(v) => Ok(v),
            None => Err(MMD_COMMON_ERROR_UNEXPECTED_EOS)?,
        }
    }
}

pub(crate) trait PMXRead {
    fn read(parser: &mut PMXParser<'_>) -> Result<Self, MMDParseError>
    where
        Self: Sized;
}

pub(crate) trait PMXReadWithSetting {
    fn read_with_setting(
        parser: &mut PMXParser<'_>,
        setting: PmxSetting,
    ) -> Result<Self, MMDParseError>
    where
        Self: Sized;
}
