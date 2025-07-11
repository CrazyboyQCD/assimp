use thiserror::Error;
use zlib_rs::ReturnCode;

#[derive(Error, Debug)]
pub enum CompressionError {
    #[error("zlib: need dictionary")]
    NeedDict,
    #[error("zlib: file error")]
    ErrNo,
    #[error("zlib: stream error")]
    StreamError,
    #[error("zlib: data error")]
    DataError,
    #[error("zlib: insufficient memory")]
    MemError,
    #[error("zlib: buffer error")]
    BufError,
    #[error("zlib: incompatible version")]
    VersionError,
    #[error("zlib: unknown error code: {0}")]
    Unknown(i32),

    #[error("zlib: try to close a closed stream")]
    TryToCloseClosedStream,
}

impl From<ReturnCode> for CompressionError {
    fn from(value: ReturnCode) -> Self {
        match value {
            ReturnCode::NeedDict => Self::NeedDict,
            ReturnCode::ErrNo => Self::ErrNo,
            ReturnCode::StreamError => Self::StreamError,
            ReturnCode::DataError => Self::DataError,
            ReturnCode::MemError => Self::MemError,
            ReturnCode::BufError => Self::BufError,
            ReturnCode::VersionError => Self::VersionError,
            _ => Self::Unknown(value as i32),
        }
    }
}
