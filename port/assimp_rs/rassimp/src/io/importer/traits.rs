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

//! Defines common importer trait for the library

#[cfg(feature = "std")]
use std::{fs::File, io::Read, path::Path};

use crate::{
    io::importer::error::{CommonImportError, ImportError},
    structs::scene::AiScene,
};

/// Format header (magic signature) trait
///
/// Each format with a fixed header should implement this trait
pub trait FormatHeader<const N: usize> {
    /// Magic byte sequence of the format
    const HEADER: [u8; N];

    /// Check if given byte sequence matches format header
    fn check_header(buf: &[u8]) -> bool {
        match buf.get(..N) {
            Some(b) => b == Self::HEADER,
            None => false,
        }
    }
}

/// Format validator trait
///
/// Unified handling of format validation from different sources (file, Reader, buffer)
pub trait FormatHeaderValidator<const N: usize>: FormatHeader<N> {
    /// Validate format from buffer
    fn check_header_from_buf(buf: &[u8]) -> bool {
        Self::check_header(buf)
    }

    /// Validate format from Reader
    #[cfg(feature = "std")]
    fn check_header_from_reader<R: Read>(reader: &mut R) -> Result<bool, std::io::Error> {
        let mut buffer = [0u8; N];
        reader.read_exact(&mut buffer)?;
        Ok(Self::check_header(&buffer))
    }

    /// Validate format from file
    #[cfg(feature = "std")]
    fn check_header_from_file<P: AsRef<Path>>(file_path: P) -> Result<bool, std::io::Error> {
        match File::open(file_path) {
            Ok(mut file) => Ok(Self::check_header_from_reader(&mut file)?),
            Err(e) => Err(e),
        }
    }
}

// Automatically implement FormatValidator for all types that implement FormatHeader
impl<const N: usize, T: FormatHeader<N>> FormatHeaderValidator<N> for T {}

/// Trait for formats that do not have a fixed header
///
/// These formats need to implement their own validation logic based on content
pub trait FormatValidator {
    /// Validate format from buffer
    fn validate_format_from_buf(_buf: &[u8]) -> bool {
        // By default, assume the format is valid
        // Override this method for actual validation logic
        true
    }

    /// Validate format from Reader
    #[cfg(feature = "std")]
    fn validate_format_from_reader<R: Read>(_reader: &mut R) -> Result<bool, std::io::Error> {
        // By default, assume the format is valid
        // Override this method for actual validation logic
        Ok(true)
    }

    /// Validate format from file
    #[cfg(feature = "std")]
    fn validate_format_from_file<P: AsRef<Path>>(_file_path: P) -> Result<bool, std::io::Error> {
        // By default, assume the format is valid
        // Override this method for actual validation logic
        Ok(true)
    }
}

#[derive(Default)]
pub struct EmptyConfig;

/// Internal importer trait
///
/// Focus on core import logic, excluding format validation and encoding conversion
pub trait InternalImporter<E> {
    /// Extra configuration for the importer
    type ExtraConfig: Default;

    /// Import from file to scene
    #[cfg(feature = "std")]
    fn import_from_file(
        file_path: &Path,
        scene: &mut AiScene,
        config: Self::ExtraConfig,
    ) -> Result<(), E>;

    /// Import from byte buffer to scene
    fn import_from_buf(buf: &[u8], scene: &mut AiScene, config: Self::ExtraConfig)
    -> Result<(), E>;
}

/// Public importer trait
///
/// Provide high-level import API, returning complete scene objects
pub trait Importer<E>: InternalImporter<E> {
    /// Read from file and create scene
    #[cfg(feature = "std")]
    fn read_from_file(file_path: &Path) -> Result<AiScene, E> {
        let mut scene = AiScene::default();
        Self::import_from_file(file_path, &mut scene, Default::default())?;
        Ok(scene)
    }

    /// Read from byte buffer and create scene
    fn read_from_buf(buf: &[u8]) -> Result<AiScene, E> {
        let mut scene = AiScene::default();
        Self::import_from_buf(buf, &mut scene, Default::default())?;
        Ok(scene)
    }
}

// Automatically implement Importer for all types that implement InternalImporter
impl<E, T: InternalImporter<E>> Importer<E> for T {}

/// Complete format importer trait for formats with fixed headers
///
/// Combines format validation and import functionality for header-based formats
pub trait HeaderFormatImporter<const N: usize, E>:
    FormatHeaderValidator<N> + InternalImporter<E> + Importer<E>
where
    E: Into<ImportError>,
{
    /// Try importing from file (including format validation)
    #[cfg(feature = "std")]
    fn try_import_from_file(file_path: &Path) -> Result<AiScene, ImportError> {
        if Self::check_header_from_file(file_path).map_err(CommonImportError::IoError)? {
            Ok(Self::read_from_file(file_path).map_err(Into::into)?)
        } else {
            Err(CommonImportError::InvalidFormat)?
        }
    }

    /// Try importing from buffer (including format validation)
    fn try_import_from_buf(buf: &[u8]) -> Result<AiScene, ImportError> {
        if Self::check_header_from_buf(buf) {
            Ok(Self::read_from_buf(buf).map_err(Into::into)?)
        } else {
            Err(ImportError::CommonError(CommonImportError::InvalidFormat))
        }
    }
}

// Automatically implement HeaderFormatImporter for types that meet the conditions
impl<const N: usize, E, T> HeaderFormatImporter<N, E> for T
where
    T: FormatHeaderValidator<N> + InternalImporter<E> + Importer<E>,
    E: Into<ImportError>,
{
}

/// Complete format importer trait for formats without fixed headers
///
/// Combines format validation and import functionality for content-based formats
pub trait ContentFormatImporter<E>:
    FormatValidator + InternalImporter<E> + Importer<E>
where
    E: Into<ImportError>,
{
    /// Try importing from file (including format validation)
    #[cfg(feature = "std")]
    fn try_import_from_file(file_path: &Path) -> Result<AiScene, ImportError> {
        if Self::validate_format_from_file(file_path).map_err(CommonImportError::IoError)? {
            Ok(Self::read_from_file(file_path).map_err(Into::into)?)
        } else {
            Err(CommonImportError::InvalidFormat)?
        }
    }

    /// Try importing from buffer (including format validation)
    fn try_import_from_buf(buf: &[u8]) -> Result<AiScene, ImportError> {
        if Self::validate_format_from_buf(buf) {
            Ok(Self::read_from_buf(buf).map_err(Into::into)?)
        } else {
            Err(ImportError::CommonError(CommonImportError::InvalidFormat))
        }
    }
}

// Automatically implement ContentFormatImporter for types that meet the conditions
impl<E, T> ContentFormatImporter<E> for T
where
    T: FormatValidator + InternalImporter<E> + Importer<E>,
    E: Into<ImportError>,
{
}