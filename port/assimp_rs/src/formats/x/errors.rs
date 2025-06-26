use crate::{
    traits::importer::error::{EncodingError, ImportError},
    utils::fast_atof::error::FastAtofError,
};
use std::borrow::Cow;
use thiserror::Error;

/// X file specific parsing errors
#[derive(Debug, Error)]
pub enum XFileImportError {
    // Format and header errors
    #[error("Invalid binary format size: {0}")]
    InvalidBinaryFormatSize(u32),

    #[error("Not a valid X file format")]
    InvalidFormat,

    #[error("Invalid header, expected 'xof ' but got {0:?}")]
    InvalidHeader([u8; 4]),

    #[error(
        "Invalid format signature, expected 'txt ' or 'bin ' or 'tzip' or 'bzip' but got {0:?}"
    )]
    InvalidFormatSignature([u8; 4]),

    #[error("File is too small, expected at least 16 bytes")]
    FileTooSmall,

    #[error("Unsupported compressed format, expected MSZIP header")]
    UnsupportedCompressedFormat,

    #[error("Decompression error: {0}")]
    DecompressionError(#[from] flate2::DecompressError),

    // Parsing structure errors
    #[error("Unexpected end of file while parsing {context}")]
    UnexpectedEndOfFile { context: &'static str },

    #[error("Separator character (';' or ',') expected, got {0}")]
    SeparatorCharacterExpected(String),

    #[error("Semicolon character expected, got {0}")]
    SemicolonExpected(String),

    #[error("Closing brace expected, got {0}")]
    ClosingBraceExpected(String),

    #[error("Expected {expected:?}, got {found:?}")]
    UnexpectedToken {
        expected: Cow<'static, str>,
        found: Cow<'static, str>,
    },

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

    #[error("Invalid number of arguments for {key_type} key in animation")]
    InvalidNumberOfArgumentsForKeyInAnimation {
        key_type: &'static str,
        expected: usize,
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

impl From<EncodingError> for XFileImportError {
    fn from(error: EncodingError) -> Self {
        Self::ImportError(ImportError::EncodingError(error))
    }
}

impl XFileImportError {
    /// Create an UnexpectedEndOfFile error with context
    #[inline]
    pub fn unexpected_end_of_file(context: &'static str) -> Self {
        Self::UnexpectedEndOfFile { context }
    }

    /// Create an UnexpectedToken error
    #[inline]
    pub fn unexpected_token(
        expected: impl Into<Cow<'static, str>>,
        found: impl Into<Cow<'static, str>>,
    ) -> Self {
        Self::UnexpectedToken {
            expected: expected.into(),
            found: found.into(),
        }
    }

    /// Create an InvalidAnimationKeyArgs error
    #[inline]
    pub fn invalid_animation_key_args(key_type: &'static str) -> Self {
        Self::InvalidAnimationKeyArgs { key_type }
    }
}
