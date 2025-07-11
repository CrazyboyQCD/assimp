use thiserror::Error;

use crate::{
    traits::importer::error::{EncodingError, ImportError},
    utils::{compression::error::CompressionError, fast_atof::error::FastAtofError},
};

/// X file specific import errors
#[derive(Debug, Error)]
pub enum XFileImportError {
    #[error("File is too small")]
    FileTooSmall,

    #[error("Invalid encoding")]
    InvalidFormat,

    // Memory and resource errors
    #[error("Not enough memory to store materials")]
    InsufficientMemory,

    #[error("Node not found")]
    NodeNotFound,

    #[error("No root node found")]
    NoRootNode,

    #[error("Import error: {0}")]
    ImportError(#[from] ImportError),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("{position}: {error}")]
    XFileParseError {
        position: String,
        error: XFileParseError,
    },
}

impl From<EncodingError> for XFileImportError {
    fn from(error: EncodingError) -> Self {
        Self::ImportError(ImportError::EncodingError(error))
    }
}

#[derive(Debug, Error)]
pub enum XFileExportError {
    #[error("Invalid header, expected 'xof ' but got {0:?}")]
    InvalidHeader([u8; 4]),

    #[error(
        "Invalid format signature, expected 'txt ' or 'bin ' or 'tzip' or 'bzip' but got {0:?}"
    )]
    InvalidFormatSignature([u8; 4]),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Write error: {0}")]
    WriteError(#[from] std::fmt::Error),
}

#[derive(Debug, Error)]
pub enum XFileParseError {
    // Header Parse Errors
    #[error("Need at least 16 bytes to read x file header but only {0} bytes left")]
    NotEnoughDataToReadHeader(usize),

    #[error("Only [txt, bin, tzip, bzip] are supported, but got {0:?}")]
    UnsupportedFileFormat([u8; 4]),

    #[error("Only 32 bits and 64 bits float point number are supported but got {0} bits")]
    UnsupportedFloatSize(u32),

    // Compress Errors
    #[cfg(not(feature = "compression"))]
    #[error("Compression feature is not enabled, cannot decompress compressed x file")]
    CompressionFeatureNotEnabled,

    #[cfg(feature = "compression")]
    #[error(
        "Invalid offset to next MSZIP compressed block, offset should be less than 32786 but got {0}"
    )]
    InvalidOffsetToNextMszipCompressedBlock(usize),

    #[cfg(feature = "compression")]
    #[error("Unsupported compressed format, expected MSZIP header 'CK', but found {0:?}")]
    UnsupportedCompressedFormat([u8; 2]),

    #[cfg(feature = "compression")]
    #[error("Decompression error: {0}")]
    DecompressionError(#[from] CompressionError),

    #[cfg(feature = "compression")]
    #[error(
        "Compressed data is too small, expected at least {offset} bytes, but only {left} bytes left"
    )]
    TooSmallZipFile { left: usize, offset: usize },

    // Text Parse Errors
    #[error("XFileTextParseError: {0}")]
    TextParseError(#[from] XFileTextParseError),

    // Binary Parse Errors
    #[error("XFileBinaryParseError: {0}")]
    BinaryParseError(#[from] XFileBinaryParseError),

    // Common Parse Errors
    #[error("Separator character (';' or ',') expected, got {0}")]
    SeparatorCharacterExpected(String),

    #[error("Semicolon character expected, got {0}")]
    SemicolonExpected(String),

    #[error("Closing brace expected, got {0}")]
    ClosingBraceExpected(String),

    #[error("Unexpected end of file while parsing {context}")]
    UnexpectedEndOfFile { context: &'static str },

    #[error("Expected {expected}, got {found}")]
    UnexpectedToken {
        expected: &'static str,
        found: String,
    },

    #[error("Expected number digit, got {0}")]
    ExpectNumberDigit(u8),

    #[error("Not enough data to read {0} bytes")]
    NotEnoughDataToRead(usize),

    // Mesh validation errors
    #[error("Unknown data object in mesh")]
    UnknownDataObject,

    #[error("Too many sets of texture coordinates")]
    TooManySetsOfTextureCoordinates,

    #[error("Normal face count does not match vertex face count")]
    NormalFaceCountMismatch,

    #[error("Normal face count does not match vertex face count")]
    NormalFaceCountDoesNotMatchVertexFaceCount,

    #[error("Too many color sets")]
    TooManyColorSets,

    #[error("Texture coord count does not match vertex count")]
    TextureCoordCountMismatch,

    #[error("Vertex color count does not match vertex count")]
    VertexColorCountMismatch,

    #[error("Vertex color index out of bounds")]
    VertexColorIndexOutOfBounds,

    #[error("Per-face material index count does not match face count")]
    MaterialIndexCountMismatch,

    #[error("Texture coord count does not match vertex count")]
    TextureCoordCountDoesNotMatchVertexCount,

    #[error("Vertex color count does not match vertex count")]
    VertexColorCountDoesNotMatchVertexCount,

    #[error("Per-face material index count does not match face count")]
    PerFaceMaterialIndexCountDoesNotMatchFaceCount,

    #[error(
        "Invalid number of arguments for {key_type} key in animation, expected {expected} but got {found}"
    )]
    InvalidNumberOfArgumentsForKeyInAnimation {
        key_type: &'static str,
        expected: usize,
        found: usize,
    },

    #[error("Unknown key type {0} in animation")]
    UnknownKeyTypeInAnimation(u32),

    // Animation errors
    #[error("Invalid number of arguments for {key_type} key in animation")]
    InvalidAnimationKeyArgs { key_type: &'static str },

    #[error("Unknown key type {0} in animation")]
    UnknownKeyType(u32),

    // Memory and resource errors
    #[error("Not enough memory to store materials")]
    InsufficientMemory,

    #[error("Node not found")]
    NodeNotFound,

    #[error("No root node found")]
    NoRootNode,

    // Delegate to other error types
    #[error("Numeric parsing error: {0}")]
    FastAtofError(#[from] FastAtofError),

    #[error("Import error: {0}")]
    ImportError(#[from] ImportError),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

impl XFileParseError {
    /// Create an UnexpectedEndOfFile error with context
    #[inline]
    pub const fn unexpected_end_of_file(context: &'static str) -> Self {
        Self::UnexpectedEndOfFile { context }
    }

    /// Create an UnexpectedToken error
    #[inline]
    pub fn unexpected_token(expected: &'static str, found: &[u8]) -> Self {
        Self::UnexpectedToken {
            expected,
            found: match str::from_utf8(found) {
                Ok(s) => s.to_owned(),
                Err(_) => format!("bytes: {:?}", found),
            },
        }
    }

    /// Create an InvalidAnimationKeyArgs error
    #[inline]
    pub fn invalid_animation_key_args(key_type: &'static str) -> Self {
        Self::InvalidAnimationKeyArgs { key_type }
    }
}

#[derive(Debug, Error)]
pub enum XFileTextParseError {
    #[error("Not enough data to read 2 bytes")]
    ReadBinaryWordError,

    #[error("Not enough data to read 4 bytes")]
    ReadBinaryDwordError,
}

#[derive(Debug, Error)]
pub enum XFileBinaryParseError {
    #[error("Not enough data to read 2 bytes")]
    ReadBinaryWordError,

    #[error("Not enough data to read 4 bytes")]
    ReadBinaryDwordError,
}
