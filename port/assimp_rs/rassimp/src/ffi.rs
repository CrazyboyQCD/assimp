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

//! Defines c-ffi types for the library

use alloc::boxed::Box;
use core::{
    ffi::{CStr, c_void},
    mem,
    ptr::{self, NonNull},
    slice,
};

pub mod aabb;
pub mod animation;
pub mod bone;
pub mod camera;
pub mod face;
pub mod light;
pub mod material;
pub mod mesh;
pub mod metadata;
pub mod node;
#[allow(unused)]
mod scene;
pub mod skeleton;
pub mod string;
pub mod texture;

pub use metadata::release_ai_metadata_rs;
pub use node::release_ai_node_from_root_rs;
pub use string::release_ai_string_rs;

use crate::{
    AiReal,
    ffi::string::AiStringFFI,
    io::utils::float_precision::{AiMat4, AiVec2, AiVec3, AiVec4},
    structs::color::{Color3D, Color4D},
};

/// ## Describes an file format which Assimp can export to.
///
/// ~Use [`aiGetExportFormatCount`] to learn how many export-formats are supported by
/// the current Assimp-build and [`aiGetExportFormatDescription`] to retrieve the
/// description of the export format option.~
#[derive(Default)]
pub struct AiExportFormatDescFFI {
    /// a short string ID to uniquely identify the export format. Use this ID string to
    /// specify which file format you want to export to when calling #aiExportScene().
    /// Example: "dae" or "obj"
    pub id: &'static CStr,

    /// A short description of the file format to present to users. Useful if you want
    /// to allow the user to select an export format.
    pub description: &'static CStr,

    /// Recommended file extension for the exported file in lower case.
    pub file_extension: &'static CStr,
}

/// ## Describes an file format which Assimp can export to.
///
/// ~Use [`aiGetExportFormatCount`] to learn how many export-formats are supported by
/// the current Assimp-build and [`aiGetExportFormatDescription`] to retrieve the
/// description of the export format option.~
#[derive(Default)]
pub struct AiExportFormatDesc {
    /// a short string ID to uniquely identify the export format. Use this ID string to
    /// specify which file format you want to export to when calling #aiExportScene().
    /// Example: "dae" or "obj"
    pub id: &'static str,

    /// A short description of the file format to present to users. Useful if you want
    /// to allow the user to select an export format.
    pub description: &'static str,

    /// Recommended file extension for the exported file in lower case.
    pub file_extension: &'static str,
}

const EXPORT_FORMATS: &[AiExportFormatDesc] = &[AiExportFormatDesc {
    id: "x",
    description: "x file",
    file_extension: "x",
}];

const EXPORT_FORMATS_FFI: &[AiExportFormatDescFFI] = &[AiExportFormatDescFFI {
    id: c"x",
    description: c"x file",
    file_extension: c"x",
}];

/// ## Get the export format description.
pub unsafe extern "C" fn ai_get_export_format_description_rs(
    index: usize,
) -> *const AiExportFormatDescFFI {
    if let Some(format) = EXPORT_FORMATS_FFI.get(index) {
        Box::leak(Box::new(AiExportFormatDescFFI {
            id: format.id,
            description: format.description,
            file_extension: format.file_extension,
        }))
    } else {
        ptr::null()
    }
}

/// ## Release the export format description.
pub unsafe extern "C" fn ai_release_export_format_description_rs(
    value: *const AiExportFormatDescFFI,
) {
    if !value.is_null() {
        unsafe {
            let v = Box::from_raw(value as *mut AiExportFormatDescFFI);
            drop(v);
        }
    }
}

/// Describes a blob of exported scene data. Use #aiExportSceneToBlob() to create a blob containing
/// an exported scene. The memory referred by this structure is owned by Assimp.
/// to free its resources. Don't try to free the memory on your side - it will crash for most build
/// configurations due to conflicting heaps.
///
/// Blobs can be nested - each blob may reference another blob, which may in turn reference another
/// blob and so on. This is used when exporters write more than one output file for a given
/// #aiScene. See the remarks for #aiExportDataBlob::name for more information.
#[repr(C)]
#[derive(Default)]
pub struct AiExportDataBlob {
    /// Size of the data in bytes
    pub size: usize,

    /// The data.
    pub data: *mut c_void,

    /// Name of the blob. An empty string always
    /// indicates the first (and primary) blob,
    /// which contains the actual file data.
    /// Any other blobs are auxiliary files produced
    /// by exporters (i.e. material files). Existence
    /// of such files depends on the file format. Most
    /// formats don't split assets across multiple files.
    ///
    /// If used, blob names usually contain the file
    /// extension that should be used when writing
    /// the data to disc.
    ///
    /// The blob names generated can be influenced by
    /// setting the #AI_CONFIG_EXPORT_BLOB_NAME export
    /// property to the name that is used for the master
    /// blob. All other names are typically derived from
    /// the base name, by the file format exporter.
    pub name: AiStringFFI,

    /** Pointer to the next blob in the chain or NULL if there is none. */
    pub next: *mut AiExportDataBlob,
}

impl AiExportDataBlob {
    /// Default constructor
    pub fn new() -> Self {
        Self::default()
    }
}

impl Drop for AiExportDataBlob {
    fn drop(&mut self) {
        unsafe {
            if !self.data.is_null() {
                let s = slice::from_raw_parts_mut(self.data as *mut u8, self.size);
                let _: Box<[u8]> = Box::from_raw(s);
                self.data = ptr::null_mut();
                self.size = 0;
            }
            if !self.next.is_null() {
                let _ = Box::from_raw(self.next);
                self.next = ptr::null_mut();
            }
        }
    }
}

/// ## Release the export data blob.
///
/// Pass mutable reference of the raw pointer and set it to null to avoid double free.
///
/// # Safety
///
/// Caller must make Sure that the pointer is passed from the original rust allocation.
pub unsafe extern "C" fn ai_release_export_data_blob_rs(value: *mut *mut AiExportDataBlob) {
    if let Some(value) = unsafe { value.as_mut() } {
        let ptr = mem::take(value);
        if let Some(ptr) = NonNull::new(ptr) {
            let _ = unsafe { Box::from_raw(ptr.as_ptr()) };
        }
    }
}

/// ## Matrix4x4
#[repr(C)]
#[allow(missing_docs)]
#[derive(Clone, Copy, Debug, Default)]
pub struct AiMatrix4x4FFI {
    pub a1: AiReal,
    pub a2: AiReal,
    pub a3: AiReal,
    pub a4: AiReal,
    pub b1: AiReal,
    pub b2: AiReal,
    pub b3: AiReal,
    pub b4: AiReal,
    pub c1: AiReal,
    pub c2: AiReal,
    pub c3: AiReal,
    pub c4: AiReal,
    pub d1: AiReal,
    pub d2: AiReal,
    pub d3: AiReal,
    pub d4: AiReal,
}

impl From<AiMat4> for AiMatrix4x4FFI {
    fn from(
        AiMat4 {
            x_axis,
            y_axis,
            z_axis,
            w_axis,
        }: AiMat4,
    ) -> Self {
        Self {
            a1: x_axis.x,
            a2: x_axis.y,
            a3: x_axis.z,
            a4: x_axis.w,
            b1: y_axis.x,
            b2: y_axis.y,
            b3: y_axis.z,
            b4: y_axis.w,
            c1: z_axis.x,
            c2: z_axis.y,
            c3: z_axis.z,
            c4: z_axis.w,
            d1: w_axis.x,
            d2: w_axis.y,
            d3: w_axis.z,
            d4: w_axis.w,
        }
    }
}

impl From<&AiMat4> for AiMatrix4x4FFI {
    fn from(
        AiMat4 {
            x_axis,
            y_axis,
            z_axis,
            w_axis,
        }: &AiMat4,
    ) -> Self {
        Self {
            a1: x_axis.x,
            a2: x_axis.y,
            a3: x_axis.z,
            a4: x_axis.w,
            b1: y_axis.x,
            b2: y_axis.y,
            b3: y_axis.z,
            b4: y_axis.w,
            c1: z_axis.x,
            c2: z_axis.y,
            c3: z_axis.z,
            c4: z_axis.w,
            d1: w_axis.x,
            d2: w_axis.y,
            d3: w_axis.z,
            d4: w_axis.w,
        }
    }
}

#[repr(C)]
#[allow(missing_docs)]
#[derive(Clone, Copy, Debug, Default)]
pub struct AiVector2DFFI {
    pub x: AiReal,
    pub y: AiReal,
}

impl From<AiVec2> for AiVector2DFFI {
    fn from(vec2: AiVec2) -> Self {
        Self {
            x: vec2.x,
            y: vec2.y,
        }
    }
}

impl From<&AiVec2> for AiVector2DFFI {
    fn from(vec2: &AiVec2) -> Self {
        Self {
            x: vec2.x,
            y: vec2.y,
        }
    }
}

#[repr(C)]
#[allow(missing_docs)]
#[derive(Clone, Copy, Debug, Default)]
pub struct AiVector3DFFI {
    pub x: AiReal,
    pub y: AiReal,
    pub z: AiReal,
}

impl From<AiVec3> for AiVector3DFFI {
    fn from(vec3: AiVec3) -> Self {
        Self {
            x: vec3.x,
            y: vec3.y,
            z: vec3.z,
        }
    }
}

impl From<&AiVec3> for AiVector3DFFI {
    fn from(vec3: &AiVec3) -> Self {
        Self {
            x: vec3.x,
            y: vec3.y,
            z: vec3.z,
        }
    }
}

#[repr(C)]
#[allow(missing_docs)]
#[derive(Clone, Copy, Debug, Default)]
pub struct AiVector4DFFI {
    pub x: AiReal,
    pub y: AiReal,
    pub z: AiReal,
    pub w: AiReal,
}

impl From<AiVec4> for AiVector4DFFI {
    fn from(vec4: AiVec4) -> Self {
        Self {
            x: vec4.x,
            y: vec4.y,
            z: vec4.z,
            w: vec4.w,
        }
    }
}

impl From<&AiVec4> for AiVector4DFFI {
    fn from(vec4: &AiVec4) -> Self {
        Self {
            x: vec4.x,
            y: vec4.y,
            z: vec4.z,
            w: vec4.w,
        }
    }
}

#[repr(C)]
#[allow(missing_docs)]
#[derive(Clone, Copy, Debug, Default)]
pub struct AiColor3DFFI {
    pub r: AiReal,
    pub g: AiReal,
    pub b: AiReal,
}

impl From<Color3D> for AiColor3DFFI {
    fn from(color3: Color3D) -> Self {
        Self {
            r: color3.x,
            g: color3.y,
            b: color3.z,
        }
    }
}

impl From<&Color3D> for AiColor3DFFI {
    fn from(color3: &Color3D) -> Self {
        Self {
            r: color3.x,
            g: color3.y,
            b: color3.z,
        }
    }
}

#[repr(C)]
#[allow(missing_docs)]
#[derive(Clone, Copy, Debug, Default)]
pub struct AiColor4DFFI {
    pub r: AiReal,
    pub g: AiReal,
    pub b: AiReal,
    pub a: AiReal,
}

impl From<Color4D> for AiColor4DFFI {
    fn from(color4: Color4D) -> Self {
        Self {
            r: color4.x,
            g: color4.y,
            b: color4.z,
            a: color4.w,
        }
    }
}

impl From<&Color4D> for AiColor4DFFI {
    fn from(color4: &Color4D) -> Self {
        Self {
            r: color4.x,
            g: color4.y,
            b: color4.z,
            a: color4.w,
        }
    }
}

#[repr(C)]
#[allow(missing_docs)]
#[derive(Clone, Copy, Debug, Default)]
pub struct AiQuaternionFFI {
    pub w: AiReal,
    pub x: AiReal,
    pub y: AiReal,
    pub z: AiReal,
}
