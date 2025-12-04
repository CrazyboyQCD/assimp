/*
---------------------------------------------------------------------------
Open Asset Import Library (assimp)
---------------------------------------------------------------------------

Copyright (c) 2006-2025, assimp team

All rights reserved.

Redistribution and use of this software in source and binary forms,
with or without modification, are permitted provided that the following
conditions are met:

* Redistributions of source code must retain the above
  copyright notice, this list of conditions and the
  following disclaimer.

* Redistributions in binary form must reproduce the above
  copyright notice, this list of conditions and the
  following disclaimer in the documentation and/or other
  materials provided with the distribution.

* Neither the name of the assimp team, nor the names of its
  contributors may be used to endorse or promote products
  derived from this software without specific prior
  written permission of the assimp team.

THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
"AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
OWNER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
(INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
---------------------------------------------------------------------------
*/

//! Defines X format errors for the library

use alloc::{
    borrow::{Cow, ToOwned},
    string::String,
    vec::Vec,
};
use core::str;

use thiserror::Error;

use crate::io::utils::{
    atof::error::FastAtofError, decompression::error::CompressionError,
    encoding::error::EncodingError,
};

/// X file specific import errors
#[allow(missing_docs)]
#[derive(Debug, Error)]
pub enum XFileImportError {
    #[error("File is too small")]
    FileTooSmall,

    #[error(transparent)]
    EncodingError(#[from] EncodingError),

    #[error("Invalid encoding")]
    InvalidFormat,

    // Memory and resource errors
    #[error("Not enough memory to store materials")]
    InsufficientMemory,

    #[error("Node not found")]
    NodeNotFound,

    #[error("No root node found")]
    NoRootNode,

    #[cfg(feature = "std")]
    #[error(transparent)]
    IoError(#[from] std::io::Error),

    #[error("{0}")]
    Other(&'static str),

    #[error("x file parse error: {error} at {position}")]
    XFileParseError {
        error: XFileParseError,
        position: Cow<'static, str>,
    },
}

#[allow(missing_docs)]
#[derive(Debug, Error)]
pub enum XFileExportError {
    #[error("Invalid header, expected 'xof ' but got {0:?}")]
    InvalidHeader([u8; 4]),

    #[error(
        "Invalid format signature, expected 'txt ' or 'bin ' or 'tzip' or 'bzip' but got {0:?}"
    )]
    InvalidFormatSignature([u8; 4]),

    #[cfg(feature = "std")]
    #[error(transparent)]
    IoError(#[from] std::io::Error),

    #[error(transparent)]
    WriteError(#[from] core::fmt::Error),
}

#[allow(missing_docs)]
#[derive(Debug, Error)]
pub enum XFileParseError {
    #[error(transparent)]
    GenericParseError(#[from] XFileCommonParseError),

    #[error("CompressedXFileParseError: {error}")]
    CompressedFileParseError {
        decompressed_source: Vec<u8>,
        error: XFileCommonParseError,
    },
}

#[allow(missing_docs)]
#[derive(Debug, Error)]
pub enum XFileCommonParseError {
    // Header Parse Errors
    #[error("Need at least 16 bytes to read x file header but only {0} bytes left")]
    NotEnoughDataToReadForHeader(usize),

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
    #[error(transparent)]
    TextParseError(#[from] XFileTextParseError),

    // Binary Parse Errors
    #[error(transparent)]
    BinaryParseError(#[from] XFileBinaryParseError),

    // Common Parse Errors
    #[error("Separator character (';' or ',') expected, got {0}")]
    SeparatorCharacterExpected(String),

    #[error("Semicolon character expected, got {0}")]
    SemicolonExpected(String),

    #[error("Closing brace expected, got {0}")]
    ClosingBraceExpected(String),

    #[error("Unexpected end of stream when parsing {context}")]
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
    #[error("Not enough memory")]
    InsufficientMemory,

    #[error("Node not found")]
    NodeNotFound,

    #[error("No root node found")]
    NoRootNode,

    // Delegate to other error types
    #[error("Numeric parsing error: {0}")]
    FastAtofError(#[from] FastAtofError),

    #[error("UTF-8 conversion error: {0}")]
    Utf8ConversionError(#[from] str::Utf8Error),

    #[error(
        "Unknown encoding for bytes(only utf-8, utf-16(le/be), shift-jis, gbk and gb18030 are supported): {0:?}"
    )]
    UnknownEncoding(Vec<u8>),

    #[cfg(feature = "std")]
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

impl XFileCommonParseError {
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

#[allow(missing_docs)]
#[derive(Debug, Error)]
pub enum XFileTextParseError {
    #[error("Not enough data to read 2 bytes")]
    ReadBinaryWordError,

    #[error("Not enough data to read 4 bytes")]
    ReadBinaryDwordError,
}

#[allow(missing_docs)]
#[derive(Debug, Error)]
pub enum XFileBinaryParseError {
    #[error("Not enough data to read 2 bytes")]
    ReadBinaryWordError,

    #[error("Not enough data to read 4 bytes")]
    ReadBinaryDwordError,
}
