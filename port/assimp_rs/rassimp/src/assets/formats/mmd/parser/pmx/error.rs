use thiserror::Error;

use crate::{
    assets::formats::mmd::parser::error::MMDParseError,
    io::reader::error::{IntoParsingPartEndOfStreamError, MappingPartEndOfStreamError},
};

#[derive(Debug, Error)]
pub enum PmxParseError {
    #[error("Pmx file: Unexpected end of stream when parsing on {0}")]
    UnexpectedEnd(&'static str),

    #[error("Pmx file: Invalid setting count, expected 8 but got {0}")]
    InvalidSettingCount(u8),

    #[error("Pmx file: Invalid index size, expected 1, 2, 4 but got {0}")]
    InvalidIndexSize(u8),

    #[error("Pmx file: Invalid vertex skinning type, expected 0, 1, 2, 3, 4 but got {0}")]
    InvalidVertexSkinningType(u8),

    #[error("Pmx file: Invalid morph category, expected 0, 1, 2, 3, 4 but got {0}")]
    InvalidMorphCategory(u8),

    #[error("Pmx file: Invalid morph type, expected 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10 but got {0}")]
    InvalidMorphType(u8),

    #[error("Pmx file: Invalid joint type, expected 0, 1, 2, 3, 5, 6 but got {0}")]
    InvalidJointType(u8),

    #[error("Pmx file: Invalid soft body flag, expected 0x01, 0x02, 0x04 but got {0}")]
    InvalidSoftBodyFlag(u8),

    #[error(
        "Pmx file: Unknown encoded string, expected string encoded in `Shift_JIS` or `GBK` or `GB18030` but got unknown encoded bytes: {0:?}"
    )]
    UnknownEncodedString(Vec<u8>),

    #[error("Pmx file: Invalid magic prefix, expected `PMX ` but got bytes: {0:?}")]
    InvalidMagic([u8; 4]),

    #[error("Pmx file: Invalid version, expected 2.0 or 2.1 but got {0}")]
    InvalidVersion(f32),
}

impl IntoParsingPartEndOfStreamError for PmxParseError {
    fn unexpected_end_of_stream(part: &'static str) -> PmxParseError {
        PmxParseError::UnexpectedEnd(part)
    }
}
