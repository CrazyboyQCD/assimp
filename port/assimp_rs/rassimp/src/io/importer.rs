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

//! Implements importer for the library

#[cfg(feature = "std")]
use std::path::Path;

use crate::io::importer::error::{CommonImportError, ImportError, ImportFormatError};

pub mod error;
pub mod traits;

use crate::{
    assets::{
        formats::{mmd::importer::PmxFormatImporter, x::importer::XFormatImporter},
        postprocess::AiPostProcessSteps,
    },
    ffi::AiExportFormatDesc,
    io::importer::traits::InternalImporter,
    structs::{importer_desc::ImporterDesc, scene::AiScene},
};

/// Importer for the library.
pub struct Importer;

impl Importer {
    /// Constructor for the importer.
    pub fn new() -> Self {
        Self
    }

    #[cfg(feature = "std")]
    /// Import a scene from a path.
    pub fn import_from_path<P: AsRef<Path>>(&self, path: P) -> Result<AiScene, ImportError> {
        let mut scene = AiScene::default();
        let path = path.as_ref();
        self.import_scene_from_file_by_extension(path, &mut scene)?;
        Ok(scene)
    }

    fn import_scene_from_file_by_extension(
        &self,
        path: &Path,
        scene: &mut AiScene,
    ) -> Result<(), ImportError> {
        match path.extension().and_then(|ext| ext.to_str()) {
            #[cfg(feature = "x_format")]
            Some("x" | "X") => {
                XFormatImporter::import_from_file(path, scene, Default::default())
                    .map_err(|e| ImportError::ImportFormatError(ImportFormatError::X(e)))?;
            }
            #[cfg(feature = "mmd_format")]
            Some("pmx") => {
                PmxFormatImporter::import_from_file(path, scene, Default::default())
                    .map_err(|e| ImportError::ImportFormatError(ImportFormatError::Mmd(e)))?;
            }
            _ => Err(CommonImportError::UnsupportedFormat)?,
        }
        Ok(())
    }

    /// Import a scene from a buffer.
    pub fn import_from_buf(&self, buf: &[u8], extension: &str) -> Result<AiScene, ImportError> {
        let mut scene = AiScene::default();
        match extension {
            "x" => XFormatImporter::import_from_buf(buf, &mut scene, Default::default())
                .map_err(|e| ImportError::ImportFormatError(e.into()))?,
            _ => return Err(ImportError::UnknownFormat),
        }
        Ok(scene)
    }
}

impl Default for Importer {
    fn default() -> Self {
        Self::new()
    }
}

// fn get_matching_desc(extension: &str) -> Option<&ImporterDesc> {
//     DESCS
//         .iter()
//         .find(|desc| desc.file_extensions.contains(extension))
// }

const DESCS: &[&ImporterDesc] = &[XFormatImporter::get_info()];

/// Internal description of an Assimp export format option
#[derive(Default)]
pub struct ExportFormatEntry {
    /// Public description structure to be returned by `aiGetExportFormatDescription()`
    pub description: AiExportFormatDesc,

    /// Post-processing steps to be executed PRIOR to invoking mExportFunction
    pub enforce_pp: AiPostProcessSteps,
}
impl ExportFormatEntry {
    /// Constructor to fill all entries
    pub const fn new(
        id: &'static str,
        desc: &'static str,
        extension: &'static str,
        enforce_pp: AiPostProcessSteps,
    ) -> Self {
        Self {
            description: AiExportFormatDesc {
                id,
                description: desc,
                file_extension: extension,
            },
            enforce_pp,
        }
    }
}
