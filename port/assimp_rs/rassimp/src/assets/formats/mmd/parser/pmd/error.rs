use thiserror::Error;

use crate::{
    assets::formats::mmd::parser::error::MMDParseError,
    io::reader::error::{IntoParsingPartEndOfStreamError, MappingPartEndOfStreamError},
};

#[derive(Debug, Error)]
pub enum PmdParseError {
    #[error("Pmd file: Unexpected end of stream when parsing on {0}")]
    UnexpectedEnd(&'static str),

    #[error(
        "Pmd file: Unknown encoded string, expected string encoded in `Shift_JIS` or `GBK` or `GB18030` but got unknown encoded bytes: {0:?}"
    )]
    UnknownEncodedString(Vec<u8>),

    #[error("Pmd file: Invalid magic prefix, expected `PMD ` but got bytes: {0:?}")]
    InvalidMagic(Vec<u8>),

    #[error("Pmd file: Invalid version, expected 1.0 but got {0}")]
    InvalidVersion(f32),

    #[error("Pmd file: Invalid bone type, expected 0, 1, 2, 3, 4, 5, 6, 7, 8, 9 but got {0}")]
    InvalidBoneType(u8),

    #[error("Pmd file: Invalid face category, expected 0, 1, 2, 3, 4 but got {0}")]
    InvalidFaceCategory(u8),

    #[error("Pmd file: Invalid rigid body shape, expected 0, 1, 2 but got {0}")]
    InvalidRigidBodyShape(u8),

    #[error("Pmd file: Invalid rigid body type, expected 0, 1, 2 but got {0}")]
    InvalidRigidBodyType(u8),
}

impl IntoParsingPartEndOfStreamError for PmdParseError {
    fn unexpected_end_of_stream(part: &'static str) -> PmdParseError {
        PmdParseError::UnexpectedEnd(part)
    }
}
