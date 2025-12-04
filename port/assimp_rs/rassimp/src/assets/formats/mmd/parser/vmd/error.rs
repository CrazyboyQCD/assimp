use thiserror::Error;

use crate::{
    assets::formats::mmd::parser::error::MMDParseError,
    io::reader::error::{IntoParsingPartEndOfStreamError, MappingPartEndOfStreamError},
};

#[derive(Debug, Error)]
pub enum VmdParseError {
    #[error(
        "Vmd file: Invalid magic prefix, expected `Vocaloid Motion Data 0002` or `Vocaloid Motion Data file` but got bytes: {0:?}"
    )]
    InvalidMagic(Vec<u8>),

    #[error(
        "Vmd file: Unknown encoded string, expected string encoded in `Shift_JIS` or `GBK` or `GB18030` but got unknown encoded bytes: {0:?}"
    )]
    UnknownEncodedString(Vec<u8>),

    #[error("Vmd file: Unexpected end of stream when parsing on {0}")]
    UnexpectedEnd(&'static str),

    #[error(
        "Vmd file: Unknown shadow frame type, expected 0(Off), 1(mode1) or 2(mode2), but got {0}"
    )]
    UnknownShadowFrameType(u8),
}

impl IntoParsingPartEndOfStreamError for VmdParseError {
    fn unexpected_end_of_stream(part: &'static str) -> VmdParseError {
        VmdParseError::UnexpectedEnd(part)
    }
}
