use core::char::DecodeUtf16Error;

use thiserror::Error;

/// Encoding conversion errors
#[derive(Debug, Error)]
pub enum EncodingError {
    #[error("Not valid UTF-32 length: {0}")]
    NotValidUtf32Length(usize),

    #[error("Not valid UTF-32 BE")]
    NotValidUtf32Be,

    #[error("Not valid UTF-32 LE")]
    NotValidUtf32Le,

    #[error("Not valid UTF-16 length: {0}")]
    NotValidUtf16Length(usize),

    #[error("Not valid UTF-16 BE: {0}")]
    NotValidUtf16Be(DecodeUtf16Error),

    #[error("Not valid UTF-16 LE: {0}")]
    NotValidUtf16Le(DecodeUtf16Error),

    #[error("Not valid UTF-8")]
    NotValidUtf8,

    #[error("Not valid code point: {0}")]
    NotValidCodePoint(u32),

    #[error("Unknown encoding")]
    UnknownEncoding,

    #[error("UTF8 code {0} {1} can not be converted into ISA-8859-1.")]
    NotValidUtf8ToIso8859_1(u8, u8),

    #[error("UTF8 code but only one character remaining")]
    NotValidUtf8OnlyOneCharacterRemaining,
}

/// General import errors
#[derive(Debug, Error)]
pub enum ImportError {
    #[error("File is too small")]
    TooSmall,

    #[error("Encoding error: {0}")]
    EncodingError(#[from] EncodingError),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Not a valid format")]
    InvalidFormat,

    #[error("Parse error")]
    ParseError,
}
