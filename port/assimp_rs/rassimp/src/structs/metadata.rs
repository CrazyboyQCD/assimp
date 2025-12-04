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

//! Defines the data structures for holding node meta information.

use alloc::string::String;

use indexmap::IndexMap;

use crate::{AiReal, AiVec3};

/// ## Metadata entry
///
/// The type field uniquely identifies the underlying type of the data field
#[derive(Clone, Debug, Default)]
pub enum AiMetadataEntry {
    /// Boolean value
    Bool(bool),
    /// 32-bit integer value
    Int32(i32),
    /// 64-bit unsigned integer value
    UInt64(u64),
    /// Floating-point value
    Float(AiReal),
    /// String value
    String(String),
    /// 3D vector value
    Vector3(AiVec3),
    /// Nested metadata
    Metadata(AiMetadata),
    /// 64-bit integer value
    Int64(i64),
    /// 32-bit unsigned integer value
    UInt32(u32),
    /// Maximum value as placeholder
    #[default]
    MetaMax,
}

impl PartialEq for AiMetadataEntry {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (AiMetadataEntry::Bool(a), AiMetadataEntry::Bool(b)) => a == b,
            (AiMetadataEntry::Int32(a), AiMetadataEntry::Int32(b)) => a == b,
            (AiMetadataEntry::UInt64(a), AiMetadataEntry::UInt64(b)) => a == b,
            (AiMetadataEntry::Float(a), AiMetadataEntry::Float(b)) => a == b,
            (AiMetadataEntry::String(a), AiMetadataEntry::String(b)) => a == b,
            (AiMetadataEntry::Vector3(a), AiMetadataEntry::Vector3(b)) => a == b,
            (AiMetadataEntry::Metadata(a), AiMetadataEntry::Metadata(b)) => a == b,
            (AiMetadataEntry::Int64(a), AiMetadataEntry::Int64(b)) => a == b,
            (AiMetadataEntry::UInt32(a), AiMetadataEntry::UInt32(b)) => a == b,
            (AiMetadataEntry::MetaMax, AiMetadataEntry::MetaMax) => true,
            _ => false,
        }
    }
}

/// ## Container for holding metadata.
///
/// Metadata is a key-value store using string keys and values.
///
/// The metadata is stored in an [`IndexMap`] of [`String`] keys and [`AiMetadataEntry`] values.
#[cfg(feature = "std")]
pub type AiMetadata = IndexMap<String, AiMetadataEntry>;

/// ## Container for holding metadata.
///
/// Metadata is a key-value store using string keys and values.
///
/// The metadata is stored in an [`IndexMap`] of [`String`] keys and [`AiMetadataEntry`] values.
#[cfg(not(feature = "std"))]
pub type AiMetadata = IndexMap<String, AiMetadataEntry, foldhash::fast::RandomState>;
