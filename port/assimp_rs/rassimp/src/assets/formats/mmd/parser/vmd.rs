use byteorder::LE;
use glam::{Vec2, Vec3, Vec4};

use crate::{
    assets::formats::mmd::{
        STRING_ENCODINGS,
        error::MMDImportError,
        parser::{
            error::{MMD_COMMON_ERROR_UNEXPECTED_EOS, MMDParseCommonError, MMDParseError},
            vmd::{error::VmdParseError, structs::VmdMotion},
        },
    },
    io::reader::binary_reader::{BinaryBufferReader, BinaryRead},
};

pub mod error;
pub mod structs;

pub struct VMDParser<'source> {
    binary_buffer_reader: BinaryBufferReader<'source, LE>,
}

impl<'source> VMDParser<'source> {
    pub const fn new(source: &'source [u8]) -> Self {
        Self {
            binary_buffer_reader: BinaryBufferReader::new(source),
        }
    }

    pub fn parse(&mut self) -> Result<VmdMotion, MMDImportError> {
        match VmdMotion::read(self) {
            Ok(motion) => Ok(motion),
            Err(e) => Err(MMDImportError::ParseError {
                error: e,
                position: self.position_info(),
            }),
        }
    }

    fn position_info(&self) -> String {
        format!("offset {}", self.binary_buffer_reader.offset())
    }

    pub fn is_eof(&self) -> bool {
        self.binary_buffer_reader.is_empty()
    }

    fn read_string<const N: usize>(&mut self) -> Result<String, MMDParseError> {
        let mut buffer: [u8; N] = [0; N];
        self.read_into_buffer(&mut buffer)
            .map_err(|_| MMD_COMMON_ERROR_UNEXPECTED_EOS)?;
        let bytes = &buffer[..buffer.iter().position(|&b| b == 0).unwrap_or(N)];
        match str::from_utf8(bytes) {
            Ok(s) => Ok(s.to_owned()),
            Err(_) => {
                for encoding in STRING_ENCODINGS {
                    let (s, has_error) = encoding.decode_without_bom_handling(bytes);
                    if !has_error {
                        return Ok(s.into_owned());
                    }
                }
                Err(VmdParseError::UnknownEncodedString(bytes.to_vec()))?
            }
        }
    }

    fn read_into_buffer(&mut self, buf: &mut [u8]) -> Result<(), MMDParseError> {
        match self.binary_buffer_reader.read_into_buffer(buf) {
            Some(_) => Ok(()),
            None => Err(MMD_COMMON_ERROR_UNEXPECTED_EOS)?,
        }
    }

    fn read_u8(&mut self) -> Result<u8, MMDParseError> {
        match self.binary_buffer_reader.read_u8() {
            Some(v) => Ok(v),
            None => Err(MMD_COMMON_ERROR_UNEXPECTED_EOS)?,
        }
    }

    fn read_i8(&mut self) -> Result<i8, MMDParseError> {
        match self.binary_buffer_reader.read_i8() {
            Some(v) => Ok(v),
            None => Err(MMD_COMMON_ERROR_UNEXPECTED_EOS)?,
        }
    }

    fn read_u16(&mut self) -> Result<u16, MMDParseError> {
        match self.binary_buffer_reader.read_u16() {
            Some(v) => Ok(v),
            None => Err(MMD_COMMON_ERROR_UNEXPECTED_EOS)?,
        }
    }

    fn read_u32(&mut self) -> Result<u32, MMDParseError> {
        match self.binary_buffer_reader.read_u32() {
            Some(v) => Ok(v),
            None => Err(MMD_COMMON_ERROR_UNEXPECTED_EOS)?,
        }
    }

    fn read_f32(&mut self) -> Result<f32, MMDParseError> {
        match self.binary_buffer_reader.read_f32() {
            Some(v) => Ok(v),
            None => Err(MMD_COMMON_ERROR_UNEXPECTED_EOS)?,
        }
    }

    fn read_vec2(&mut self) -> Result<Vec2, MMDParseError> {
        match self.binary_buffer_reader.read_vec2() {
            Some(v) => Ok(v),
            None => Err(MMD_COMMON_ERROR_UNEXPECTED_EOS)?,
        }
    }

    fn read_vec3(&mut self) -> Result<Vec3, MMDParseError> {
        match self.binary_buffer_reader.read_vec3() {
            Some(v) => Ok(v),
            None => Err(MMD_COMMON_ERROR_UNEXPECTED_EOS)?,
        }
    }

    fn read_vec4(&mut self) -> Result<Vec4, MMDParseError> {
        match self.binary_buffer_reader.read_vec4() {
            Some(v) => Ok(v),
            None => Err(MMD_COMMON_ERROR_UNEXPECTED_EOS)?,
        }
    }
}

pub(crate) trait VMDRead {
    fn read(parser: &mut VMDParser<'_>) -> Result<Self, MMDParseError>
    where
        Self: Sized;
}
