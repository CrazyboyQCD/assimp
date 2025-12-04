use thiserror::Error;

use crate::{
    assets::formats::mmd::parser::error::MMDParseError,
    io::reader::error::{IntoParsingPartEndOfStreamError, MappingPartEndOfStreamError},
};

#[derive(Debug, Error)]
pub enum VpdParseError {
    #[error(
        "Vpd file: Invalid magic prefix, expected `Vocaloid Pose Data file` but got bytes: {0:?}"
    )]
    InvalidMagic(Vec<u8>),

    #[error(
        "Vpd file: Unknown encoded string, expected string encoded in `Shift_JIS` or `UTF-8` but got unknown encoded bytes: {0:?}"
    )]
    UnknownEncodedString(Vec<u8>),

    #[error("Vpd file: Unexpected end of stream when parsing on {0}")]
    UnexpectedEnd(&'static str),

    // bone block errors
    #[error("Vpd file: Invalid bone block header prefix, expected `Bone` but got bytes: {0:?}")]
    InvalidBonePrefix(Vec<u8>),

    #[error("Vpd file: Invalid bone block header suffix, expected `{{` but got bytes: {0:?}")]
    InvalidBoneBracket(Vec<u8>),

    #[error("Vpd file: Invalid bone block translate separator, expected `,` but got bytes: {0:?}")]
    InvalidBoneTranslateSeparator(Vec<u8>),

    #[error("Vpd file: Invalid bone block translate separator, expected `;` but got bytes: {0:?}")]
    InvalidBoneTranslateEnd(Vec<u8>),

    #[error("Vpd file: Invalid bone block quaternion separator, expected `,` but got bytes: {0:?}")]
    InvalidBoneQuaternionSeparator(Vec<u8>),

    #[error("Vpd file: Invalid bone block quaternion separator, expected `;` but got bytes: {0:?}")]
    InvalidBoneQuaternionEnd(Vec<u8>),

    #[error("Vpd file: Invalid bone block end, expected `}}` but got bytes: {0:?}")]
    InvalidBoneEnd(Vec<u8>),

    // morph block errors
    #[error("Vpd file: Invalid morph block header prefix, expected `Morph` but got bytes: {0:?}")]
    InvalidMorphPrefix(Vec<u8>),

    #[error("Vpd file: Invalid morph block header suffix, expected `{{` but got bytes: {0:?}")]
    InvalidMorphBracket(Vec<u8>),

    #[error("Vpd file: Invalid morph block weight separator, expected `;` but got bytes: {0:?}")]
    InvalidMorphWeightSeparator(Vec<u8>),

    #[error("Vpd file: Invalid morph block end, expected `}}` but got bytes: {0:?}")]
    InvalidMorphEnd(Vec<u8>),
}

impl IntoParsingPartEndOfStreamError for VpdParseError {
    fn unexpected_end_of_stream(part: &'static str) -> VpdParseError {
        VpdParseError::UnexpectedEnd(part)
    }
}
