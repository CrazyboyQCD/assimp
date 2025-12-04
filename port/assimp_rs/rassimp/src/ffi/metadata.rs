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

//! Defines metadata c-ffi types for the library

use alloc::{boxed::Box, vec::Vec};
use core::{ffi::c_void, mem, ptr, slice};

use crate::{
    AiReal,
    ffi::string::AiStringFFI,
    structs::metadata::{AiMetadata, AiMetadataEntry},
};

/// ## Enum used to distinguish data types.
#[cfg_attr(not(feature = "swig"), repr(C))]
#[cfg_attr(feature = "swig", repr(C, u32))]
pub enum AiMetadataType {
    /// ### Boolean.
    ///
    /// Boolean.
    BOOL = 0,
    /// ### 32-bit integer.
    ///
    /// 32-bit integer.
    INT32 = 1,
    /// ### 64-bit integer.
    ///
    /// 64-bit integer.
    UINT64 = 2,
    /// ### Float.
    ///
    /// Float.
    FLOAT = 3,
    /// ### Double.
    ///
    /// Double.
    DOUBLE = 4,
    /// ### String.
    ///
    /// String.
    AISTRING = 5,
    /// ### Vector3.
    AIVECTOR3D = 6,
    /// ### Metadata.
    ///
    /// Metadata.
    AIMETADATA = 7,
    /// ### 64-bit integer.
    ///
    /// 64-bit integer.
    INT64 = 8,
    /// ### 32-bit integer.
    ///
    /// 32-bit integer.
    UINT32 = 9,
    /// ### Maximum value as placeholder.
    ///
    /// Maximum value as placeholder.
    METAMAX = 10,
}

/// ## Metadata entry.
///
/// The type field uniquely identifies the underlying type of the data field.
#[repr(C)]
pub struct AiMetadataEntryFFI {
    /// ### The type of the data.
    ///
    /// The type of the data.
    r#type: AiMetadataType,
    /// ### The type-erased data.
    ///
    /// The type-erased data.
    data: *mut c_void,
}

impl Drop for AiMetadataEntryFFI {
    fn drop(&mut self) {
        if self.data.is_null() {
            return;
        }
        unsafe {
            match self.r#type {
                AiMetadataType::BOOL => {
                    let _ = Box::from_raw(self.data as *mut bool);
                }
                AiMetadataType::INT32 => {
                    let _ = Box::from_raw(self.data as *mut i32);
                }
                AiMetadataType::UINT64 => {
                    let _ = Box::from_raw(self.data as *mut u64);
                }
                AiMetadataType::FLOAT => {
                    let _ = Box::from_raw(self.data as *mut f32);
                }
                AiMetadataType::DOUBLE => {
                    let _ = Box::from_raw(self.data as *mut f64);
                }
                AiMetadataType::AISTRING => {
                    let _ = Box::from_raw(self.data as *mut AiStringFFI);
                }
                AiMetadataType::AIVECTOR3D => {
                    let _ = Box::from_raw(self.data as *mut [AiReal; 3]);
                }
                AiMetadataType::AIMETADATA => {
                    let _ = Box::from_raw(self.data as *mut AiMetadataFFI);
                }
                AiMetadataType::INT64 => {
                    let _ = Box::from_raw(self.data as *mut i64);
                }
                AiMetadataType::UINT32 => {
                    let _ = Box::from_raw(self.data as *mut u32);
                }
                _ => {}
            }
            self.data = ptr::null_mut();
            self.r#type = AiMetadataType::METAMAX;
        }
    }
}

/// ## Container for holding metadata.
///
/// Metadata is a key-value store using string keys and values.
#[repr(C)]
pub struct AiMetadataFFI {
    /// Length of the mKeys and mValues arrays, respectively
    num_properties: usize,

    /// Arrays of keys, may not be `NULL`. Entries in this array may not be `NULL` as well.
    keys: *mut AiStringFFI,

    /// Arrays of values, may not be `NULL`. Entries in this array may be `NULL` if the
    /// corresponding property key has no assigned value.
    values: *mut AiMetadataEntryFFI,
}

impl Drop for AiMetadataFFI {
    fn drop(&mut self) {
        unsafe {
            if !self.keys.is_null() {
                let s = slice::from_raw_parts_mut(self.keys, self.num_properties);
                let _: Box<[AiStringFFI]> = Box::from_raw(s);
                self.keys = ptr::null_mut();
            }
            if !self.values.is_null() {
                let s = slice::from_raw_parts_mut(self.values, self.num_properties);
                let _: Box<[AiMetadataEntryFFI]> = Box::from_raw(s);
                self.values = ptr::null_mut();
            }
            self.num_properties = 0;
        }
    }
}

/// ## Release the AiMetadataFFI.
///
/// Pass mutable reference of the raw pointer and set it to null to avoid double free.
///
/// # Safety
///
/// Caller must make Sure that the pointer is passed from the original rust allocation.
pub unsafe extern "C" fn release_ai_metadata_rs(value: *mut *mut AiMetadataFFI) {
    if let Some(value) = unsafe { value.as_mut() } {
        let ptr = mem::take(value);
        let _ = unsafe { Box::from_raw(ptr) };
    }
}

impl From<&AiMetadata> for AiMetadataFFI {
    fn from(value: &AiMetadata) -> Self {
        let mut keys: Vec<AiStringFFI> = Vec::new();
        let mut values: Vec<AiMetadataEntryFFI> = Vec::new();
        for (key, value) in value.iter() {
            keys.push(key.into());
            values.push(value.into());
        }

        let keys = keys.into_boxed_slice();
        let values = values.into_boxed_slice();
        let num_properties = keys.len();

        AiMetadataFFI {
            num_properties,
            keys: Box::into_raw(keys).cast(),
            values: Box::into_raw(values).cast(),
        }
    }
}

impl From<AiMetadata> for AiMetadataFFI {
    fn from(value: AiMetadata) -> Self {
        let mut keys: Vec<AiStringFFI> = Vec::new();
        let mut values: Vec<AiMetadataEntryFFI> = Vec::new();
        for (key, value) in value.iter() {
            keys.push(key.into());
            values.push(value.into());
        }

        let keys = keys.into_boxed_slice();
        let values = values.into_boxed_slice();
        let num_properties = keys.len();

        AiMetadataFFI {
            num_properties,
            keys: Box::into_raw(keys).cast(),
            values: Box::into_raw(values).cast(),
        }
    }
}

impl From<&AiMetadataEntry> for AiMetadataEntryFFI {
    fn from(value: &AiMetadataEntry) -> Self {
        match value {
            AiMetadataEntry::Bool(value) => AiMetadataEntryFFI {
                r#type: AiMetadataType::BOOL,
                data: Box::into_raw(Box::new(value)).cast(),
            },
            AiMetadataEntry::Int32(value) => AiMetadataEntryFFI {
                r#type: AiMetadataType::INT32,
                data: Box::into_raw(Box::new(value)).cast(),
            },
            AiMetadataEntry::UInt64(value) => AiMetadataEntryFFI {
                r#type: AiMetadataType::UINT64,
                data: Box::into_raw(Box::new(value)).cast(),
            },
            AiMetadataEntry::Float(value) => AiMetadataEntryFFI {
                r#type: AiMetadataType::FLOAT,
                data: Box::into_raw(Box::new(value)).cast(),
            },
            AiMetadataEntry::String(value) => AiMetadataEntryFFI {
                r#type: AiMetadataType::AISTRING,
                data: Box::into_raw(Box::<AiStringFFI>::new(value.into())).cast(),
            },
            AiMetadataEntry::Vector3(value) => AiMetadataEntryFFI {
                r#type: AiMetadataType::AIVECTOR3D,
                data: Box::into_raw(Box::new([
                    value.x as AiReal,
                    value.y as AiReal,
                    value.z as AiReal,
                ]))
                .cast(),
            },
            AiMetadataEntry::Metadata(value) => AiMetadataEntryFFI {
                r#type: AiMetadataType::AIMETADATA,
                data: Box::into_raw(Box::<AiMetadataFFI>::new(value.into())).cast(),
            },
            AiMetadataEntry::Int64(value) => AiMetadataEntryFFI {
                r#type: AiMetadataType::INT64,
                data: Box::into_raw(Box::new(value)).cast(),
            },
            AiMetadataEntry::UInt32(value) => AiMetadataEntryFFI {
                r#type: AiMetadataType::UINT32,
                data: Box::into_raw(Box::new(value)).cast(),
            },
            AiMetadataEntry::MetaMax => AiMetadataEntryFFI {
                r#type: AiMetadataType::METAMAX,
                data: ptr::null_mut(),
            },
        }
    }
}
