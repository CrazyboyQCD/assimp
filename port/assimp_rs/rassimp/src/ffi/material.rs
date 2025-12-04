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

//! A material in the scene.

use core::ffi::c_char;

use crate::ffi::string::AiStringFFI;

/// A material in the scene.
#[repr(C)]
pub struct AiMaterialFFI {
    /// List of all material properties loaded.
    pub properties: *mut AiMaterialPropertyFFI,

    /// Number of properties in the data base
    pub num_properties: u32,

    /// Storage allocated
    pub num_allocated: u32,
}

/// ## Data structure for a single material property
///
/// As an user, you'll probably never need to deal with this data structure.
/// Just use the provided aiGetMaterialXXX() or aiMaterial::Get() family
/// of functions to query material properties easily. Processing them
/// manually is faster, but it is not the recommended way. It isn't worth
/// the effort. <br>
/// Material property names follow a simple scheme:
/// ```
/// 
/// A public property, there must be corresponding AI_MATKEY_XXX define
///   2nd: Public, but ignored by the [`AiProcessFlags::RemoveRedundantMaterials`](crate::importer_notes::AiProcessFlags::RemoveRedundantMaterials)
///     post-processing step.
///     ~<name>
///     A temporary property for internal use.
/// ```
/// See [`AiMaterialFFI`](crate::ffi::material::AiMaterialFFI)

#[repr(C)]
pub struct AiMaterialPropertyFFI {
    /// Specifies the name of the property (key)
    ///
    /// Keys are generally case insensitive.
    pub key: AiStringFFI,

    /// Textures: Specifies their exact usage semantic.
    ///
    /// For non-texture properties, this member is always 0
    /// (or, better-said, [`AiTextureType::None`](crate::importer_notes::AiTextureType::None)).
    pub semantic: u32,

    /// Textures: Specifies the index of the texture.
    ///
    /// For non-texture properties, this member is always 0.
    pub index: u32,

    /// Size of the buffer mData is pointing to, in bytes.
    ///
    /// This value may not be 0.
    pub data_length: u32,

    /// Type information for the property.
    ///
    /// Defines the data layout inside the data buffer. This is used
    /// by the library internally to perform debug checks and to
    /// utilize proper type conversions.
    /// (It's probably a hacky solution, but it works.)
    pub r#type: AiPropertyTypeInfoFFI,

    /// Binary buffer to hold the property's value.
    ///
    /// The size of the buffer is always mDataLength.
    pub data: *mut c_char,
}

/**
 *  @brief A very primitive RTTI system for the contents of material properties.
 */
#[cfg_attr(not(feature = "swig"), repr(C))]
#[cfg_attr(feature = "swig", repr(C, u32))]
pub enum AiPropertyTypeInfoFFI {
    /// Array of single-precision (32 Bit) floats
    ///
    /// It is possible to use aiGetMaterialInteger[Array]() (or the C++-API
    /// aiMaterial::Get()) to query properties stored in floating-point format.
    /// The material system performs the type conversion automatically.
    Float = 0x1,

    /// Array of double-precision (64 Bit) floats
    ///
    /// It is possible to use aiGetMaterialInteger[Array]() (or the C++-API
    /// aiMaterial::Get()) to query properties stored in floating-point format.
    /// The material system performs the type conversion automatically.
    Double = 0x2,

    /// The material property is an aiString.
    ///
    /// Arrays of strings aren't possible, aiGetMaterialString() (or the
    /// C++-API aiMaterial::Get()) *must* be used to query a string property.
    String = 0x3,

    /// Array of (32 Bit) integers
    ///
    /// It is possible to use aiGetMaterialFloat[Array]() (or the C++-API
    /// aiMaterial::Get()) to query properties stored in integer format.
    /// The material system performs the type conversion automatically.
    Integer = 0x4,

    /// Simple binary buffer, content undefined. Not convertible to anything.
    Buffer = 0x5,
}
