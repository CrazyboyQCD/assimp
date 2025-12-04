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

//! Defines import error for the library

use thiserror::Error;

#[cfg(feature = "mmd_format")]
use crate::assets::formats::mmd::error::MMDImportError;
#[cfg(feature = "x_format")]
use crate::assets::formats::x::error::XFileImportError;
use crate::io::utils::encoding::error::EncodingError;

#[allow(missing_docs)]
/// General import errors
#[derive(Debug, Error)]
pub enum ImportError {
    #[error(transparent)]
    CommonError(#[from] CommonImportError),

    #[error(transparent)]
    ImportFormatError(#[from] ImportFormatError),

    #[error("Unknown format")]
    UnknownFormat,
}

#[allow(missing_docs)]
#[derive(Debug, Error)]
pub enum CommonImportError {
    #[cfg(feature = "std")]
    #[error(transparent)]
    IoError(#[from] std::io::Error),

    #[error(transparent)]
    InvalidEncoding(#[from] EncodingError),

    #[error("Not a valid format")]
    InvalidFormat,

    #[error("Not a supported format")]
    UnsupportedFormat,

    #[error("out of memory")]
    OutOfMemory,

    #[error("file too small")]
    FileTooSmall,

    #[error("Unexpected end of stream")]
    UnexpectedEOS,
}

/// Specific format import errors
#[allow(missing_docs)]
#[derive(Debug, Error)]
pub enum ImportFormatError {
    #[cfg(feature = "x_format")]
    #[error(transparent)]
    X(#[from] XFileImportError),

    #[cfg(feature = "mmd_format")]
    #[error(transparent)]
    Mmd(#[from] MMDImportError),

    #[error("OBJ error: ")]
    Obj(()),

    #[error("FBX error: ")]
    Fbx(()),

    #[error("STL error: ")]
    Stl(()),
}
