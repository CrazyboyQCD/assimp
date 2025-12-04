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

//! Constants for the X file format

// #[doc(hidden)]
// All these comments are from the Microsoft documentation. Just copy them to save some time for
// developer.
/// See https://learn.microsoft.com/en-us/windows/win32/direct3d9/vertexelement for details.
// pub(super) mod vertex_element {
//     /// ## The fundamental Direct3D color type.
//     /// See https://learn.microsoft.com/en-us/windows/win32/direct3d9/d3dcolor for details.
//     pub type D3DCOLOR = u32;
//     /// D3DDECLTYPE enumeration
//     ///
//     /// See https://learn.microsoft.com/en-us/windows/win32/direct3d9/d3ddecltype for details.
//     pub mod decl_type {
//         /// One-component float expanded to (float, 0, 0, 1).
//         pub const D3DDECLTYPE_FLOAT1: u32 = 0;
//         /// Two-component float expanded to (float, float, 0, 1).
//         pub const D3DDECLTYPE_FLOAT2: u32 = 1;
//         /// Three-component float expanded to (float, float, float, 1).
//         pub const D3DDECLTYPE_FLOAT3: u32 = 2;
//         /// Four-component float expanded to (float, float, float, float).
//         pub const D3DDECLTYPE_FLOAT4: u32 = 3;
//         /// Four-component, packed, unsigned bytes mapped to 0 to 1 range.
//         ///
//         /// Input is a [`D3DCOLOR`](super::D3DCOLOR) and is expanded to RGBA order.
//         pub const D3DDECLTYPE_D3DCOLOR: u32 = 4;
//         /// Four-component, unsigned byte.
//         pub const D3DDECLTYPE_UBYTE4: u32 = 5;
//         /// Two-component, signed short expanded to (value, value, 0, 1).
//         pub const D3DDECLTYPE_SHORT2: u32 = 6;
//         /// Four-component, signed short expanded to (value, value, value, value).
//         pub const D3DDECLTYPE_SHORT4: u32 = 7;
//         /// Four-component byte with each byte normalized by dividing with 255.0f.
//         pub const D3DDECLTYPE_UBYTE4N: u32 = 8;
//         /// Normalized, two-component, signed short,
//         /// expanded to (first short/32767.0, second short/32767.0, 0, 1).
//         pub const D3DDECLTYPE_SHORT2N: u32 = 9;
//         /// Normalized, four-component, signed short,
//         /// expanded to (first short/32767.0, second short/32767.0, third short/32767.0, fourth
// short/32767.0).         pub const D3DDECLTYPE_SHORT4N: u32 = 10;
//         /// Normalized, two-component, unsigned short,
//         /// expanded to (first short/65535.0, short short/65535.0, 0, 1).
//         pub const D3DDECLTYPE_USHORT2N: u32 = 11;
//         /// Normalized, four-component, unsigned short,
//         /// expanded to (first short/65535.0, second short/65535.0, third short/65535.0, fourth
// short/65535.0).         pub const D3DDECLTYPE_USHORT4N: u32 = 12;
//         /// Three-component, unsigned, 10 10 10 format expanded to (value, value, value, 1).
//         pub const D3DDECLTYPE_UDEC3: u32 = 13;
//         /// Three-component, signed, 10 10 10 format normalized and
//         /// expanded to (v[0]/511.0, v[1]/511.0, v[2]/511.0, 1).
//         pub const D3DDECLTYPE_DEC3N: u32 = 14;
//         /// Two-component, 16-bit, floating point expanded to (value, value, 0, 1).
//         pub const D3DDECLTYPE_FLOAT16_2: u32 = 15;
//         /// Four-component, 16-bit, floating point expanded to (value, value, value, value).
//         pub const D3DDECLTYPE_FLOAT16_4: u32 = 16;
//         /// Type field in the declaration is unused.
//         ///
//         /// This is designed for use with [`D3DDECLMETHOD_UV`](super::method::D3DDECLMETHOD_UV)
//         /// and
// [`D3DDECLMETHOD_LOOKUPPRESAMPLED`](super::method::D3DDECLMETHOD_LOOKUPPRESAMPLED).         pub
// const D3DDECLTYPE_UNUSED: u32 = 17;     }

//     /// D3DDECLMETHOD enumeration
//     ///
//     /// See https://learn.microsoft.com/en-us/windows/win32/direct3d9/d3ddeclmethod for details.
//     pub mod method {
//         /// Default value.
//         ///
//         /// The tessellator copies the vertex data (spline data for patches) as is,
//         /// with no additional calculations.
//         /// When the tessellator is used, this element is interpolated.
//         /// Otherwise vertex data is copied into the input register.
//         ///
//         /// The input and output type can be any value, but are always the same type.
//         pub const D3DDECLMETHOD_DEFAULT: u32 = 0;
//         /// Computes the tangent at a point on the rectangle or triangle patch in the U
// direction.         ///
//         /// The input type can be one of the following:
//         ///
//         /// - [`D3DDECLTYPE_D3DCOLOR`](super::decl_type::D3DDECLTYPE_D3DCOLOR)
//         /// - [`D3DDECLTYPE_FLOAT3`](super::decl_type::D3DDECLTYPE_FLOAT3)
//         /// - [`D3DDECLTYPE_FLOAT4`](super::decl_type::D3DDECLTYPE_FLOAT4)
//         /// - [`D3DDECLTYPE_SHORT4`](super::decl_type::D3DDECLTYPE_SHORT4)
//         /// - [`D3DDECLTYPE_UBYTE4`](super::decl_type::D3DDECLTYPE_UBYTE4)
//         ///
//         /// The output type is always
// [`D3DDECLTYPE_FLOAT3`](super::decl_type::D3DDECLTYPE_FLOAT3).         pub const
// D3DDECLMETHOD_PARTIALU: u32 = 1;         /// Computes the tangent at a point on the rectangle or
// triangle patch in the V direction.         ///
//         /// The input type can be one of the following:
//         ///
//         /// - [`D3DDECLTYPE_D3DCOLOR`](super::decl_type::D3DDECLTYPE_D3DCOLOR)
//         /// - [`D3DDECLTYPE_FLOAT3`](super::decl_type::D3DDECLTYPE_FLOAT3)
//         /// - [`D3DDECLTYPE_FLOAT4`](super::decl_type::D3DDECLTYPE_FLOAT4)
//         /// - [`D3DDECLTYPE_SHORT4`](super::decl_type::D3DDECLTYPE_SHORT4)
//         /// - [`D3DDECLTYPE_UBYTE4`](super::decl_type::D3DDECLTYPE_UBYTE4)
//         ///
//         /// The output type is always
// [`D3DDECLTYPE_FLOAT3`](super::decl_type::D3DDECLTYPE_FLOAT3).         pub const
// D3DDECLMETHOD_PARTIALV: u32 = 2;         /// Computes the normal at a point on the rectangle or
// triangle         /// patch by taking the cross product of two tangents.
//         ///
//         /// The input type can be one of the following:
//         ///
//         /// - [`D3DDECLTYPE_D3DCOLOR`](super::decl_type::D3DDECLTYPE_D3DCOLOR)
//         /// - [`D3DDECLTYPE_FLOAT3`](super::decl_type::D3DDECLTYPE_FLOAT3)
//         /// - [`D3DDECLTYPE_FLOAT4`](super::decl_type::D3DDECLTYPE_FLOAT4)
//         /// - [`D3DDECLTYPE_SHORT4`](super::decl_type::D3DDECLTYPE_SHORT4)
//         /// - [`D3DDECLTYPE_UBYTE4`](super::decl_type::D3DDECLTYPE_UBYTE4)
//         ///
//         /// The output type is always
// [`D3DDECLTYPE_FLOAT3`](super::decl_type::D3DDECLTYPE_FLOAT3).         pub const
// D3DDECLMETHOD_CROSSUV: u32 = 3;         /// Copy out the U, V values at a point on the rectangle
// or triangle patch.         ///
//         /// This results in a 2D float. The input type must be
//         /// set to [`D3DDECLTYPE_UNUSED`](super::decl_type::D3DDECLTYPE_UNUSED).
//         ///
//         /// The output data type is always
// [`D3DDECLTYPE_FLOAT2`](super::decl_type::D3DDECLTYPE_FLOAT2).         ///
//         /// The input stream and offset are also unused (but must be set to 0).
//         pub const D3DDECLMETHOD_UV: u32 = 4;
//         /// Look up a displacement map. The input type can be one of the following:
//         ///
//         /// - [`D3DDECLTYPE_FLOAT2`](super::decl_type::D3DDECLTYPE_FLOAT2)
//         /// - [`D3DDECLTYPE_FLOAT3`](super::decl_type::D3DDECLTYPE_FLOAT3)
//         /// - [`D3DDECLTYPE_FLOAT4`](super::decl_type::D3DDECLTYPE_FLOAT4)
//         ///
//         /// Only the .x and .y components are used for the texture map lookup.
//         /// The output type is always
// [`D3DDECLTYPE_FLOAT1`](super::decl_type::D3DDECLTYPE_FLOAT1).         ///
//         /// The device must support displacement mapping.
//         ///
//         /// For more information about displacement mapping,
//         /// see [Displacement Mapping (Direct3D 9)](https://learn.microsoft.com/en-us/windows/win32/direct3d9/displacement-mapping).
//         ///
//         /// This constant is supported only by the programmable pipeline on N-patch data, if
// N-patches are enabled.         pub const D3DDECLMETHOD_LOOKUP: u32 = 5;
//         /// Look up a presampled displacement map.
//         ///
//         /// The input type must be set to D3DDECLTYPE_UNUSED;
//         /// the stream index and the stream offset must be set to 0.
//         ///
//         /// The output type for this operation is always
// [`D3DDECLTYPE_FLOAT1`](super::decl_type::D3DDECLTYPE_FLOAT1).         ///
//         /// The device must support displacement mapping.
//         ///
//         /// For more information about displacement mapping,
//         /// see [Displacement Mapping (Direct3D 9)](https://learn.microsoft.com/en-us/windows/win32/direct3d9/displacement-mapping).
//         ///
//         /// This constant is supported only by the programmable pipeline on N-patch data,
//         /// if N-patches are enabled.
//         ///
//         /// This method can be used only with
// [`D3DDECLUSAGE_SAMPLE`](super::usage::D3DDECLUSAGE_SAMPLE).         pub const
// D3DDECLMETHOD_LOOKUPPRESAMPLED: u32 = 6;     }

//     /// D3DDECLUSAGE enumeration
//     pub mod usage {
//         /// Position data ranging from (-1,-1) to (1,1).
//         ///
//         /// Use [`D3DDECLUSAGE_POSITION`] with a usage index of 0 to specify untransformed
// position         /// for fixed function vertex processing and the n-patch tessellator.
//         ///
//         /// Use [`D3DDECLUSAGE_POSITION`] with a usage index of 1 to specify untransformed
// position         /// in the fixed function vertex shader for vertex tweening.
//         pub const D3DDECLUSAGE_POSITION: u32 = 0;
//         /// Blending weight data.
//         ///
//         /// Use [`D3DDECLUSAGE_BLENDWEIGHT`] with a usage index of 0 to specify the blend weights
// used         /// in indexed and nonindexed vertex blending.
//         pub const D3DDECLUSAGE_BLENDWEIGHT: u32 = 1;
//         /// Blending indices data.
//         ///
//         /// Use [`D3DDECLUSAGE_BLENDINDICES`] with a usage index of 0 to specify matrix indices
// for         /// indexed paletted skinning.
//         pub const D3DDECLUSAGE_BLENDINDICES: u32 = 2;
//         /// Vertex normal data.
//         ///
//         /// Use [`D3DDECLUSAGE_NORMAL`] with a usage index of 0 to specify vertex normals
//         /// for fixed function vertex processing and the n-patch tessellator.
//         ///
//         /// Use [`D3DDECLUSAGE_NORMAL`] with a usage index of 1 to specify vertex normals for
//         /// fixed function vertex processing for vertex tweening.
//         pub const D3DDECLUSAGE_NORMAL: u32 = 3;
//         /// Point size data.
//         ///
//         /// Use [`D3DDECLUSAGE_PSIZE`] with a usage index of 0 to specify the point-size
// attribute         /// used by the setup engine of the rasterizer to expand a point into a quad
// for         /// the point-sprite functionality.
//         pub const D3DDECLUSAGE_PSIZE: u32 = 4;
//         /// Texture coordinate data.
//         ///
//         /// Use [`D3DDECLUSAGE_TEXCOORD`], n to specify texture coordinates in
//         /// fixed function vertex processing and in pixel shaders prior to ps_3_0.
//         ///
//         /// These can be used to pass user defined data.
//         pub const D3DDECLUSAGE_TEXCOORD: u32 = 5;
//         /// Vertex tangent data.
//         pub const D3DDECLUSAGE_TANGENT: u32 = 6;
//         /// Vertex binormal data.
//         pub const D3DDECLUSAGE_BINORMAL: u32 = 7;
//         /// Single positive floating point value.
//         ///
//         /// Use [`D3DDECLUSAGE_TESSFACTOR`] with a usage index of 0 to specify a tessellation
// factor used         /// in the tessellation unit to control the rate of tessellation.
//         ///
//         /// For more information about the data type, see
// [`D3DDECLTYPE_FLOAT1`](super::decl_type::D3DDECLTYPE_FLOAT1).         pub const
// D3DDECLUSAGE_TESSFACTOR: u32 = 8;         /// Vertex data contains transformed position data
// ranging from (0,0) to (viewport width, viewport height).         ///
//         /// Use [`D3DDECLUSAGE_POSITIONT`] with a usage index of 0 to specify transformed
// position.         ///
//         /// When a declaration containing this is set, the pipeline does not perform vertex
// processing.         pub const D3DDECLUSAGE_POSITIONT: u32 = 9;
//         /// Vertex data contains diffuse or specular color.
//         ///
//         /// Use [`D3DDECLUSAGE_COLOR`] with a usage index of 0 to specify the diffuse color
//         /// in the fixed function vertex shader and pixel shaders prior to ps_3_0.
//         ///
//         /// Use [`D3DDECLUSAGE_COLOR`] with a usage index of 1 to specify the specular color
//         /// in the fixed function vertex shader and pixel shaders prior to ps_3_0.
//         pub const D3DDECLUSAGE_COLOR: u32 = 10;
//         /// Vertex data contains fog data.
//         ///
//         /// Use [`D3DDECLUSAGE_FOG`] with a usage index of 0 to
//         /// specify a fog blend value used after pixel shading finishes.
//         ///
//         /// This applies to pixel shaders prior to version ps_3_0.
//         pub const D3DDECLUSAGE_FOG: u32 = 11;
//         /// Vertex data contains depth data.
//         pub const D3DDECLUSAGE_DEPTH: u32 = 12;
//         /// Vertex data contains sampler data.
//         ///
//         /// Use [`D3DDECLUSAGE_SAMPLE`] with a usage index of 0 to
//         /// specify the displacement value to look up.
//         ///
//         /// It can be used only with
// [`D3DDECLMETHOD_LOOKUPPRESAMPLED`](super::method::D3DDECLMETHOD_LOOKUPPRESAMPLED)         /// or
// [`D3DDECLMETHOD_LOOKUP`](super::method::D3DDECLMETHOD_LOOKUP).         pub const
// D3DDECLUSAGE_SAMPLE: u32 = 13;     }
// }

pub(super) mod ms_compression {
    /// MSZIP magic number
    pub const MSZIP_MAGIC: u16 = u16::from_le_bytes([b'C', b'K']);
    /// MSZIP block size
    pub const MSZIP_BLOCK: usize = 32786;
}

pub(super) mod binary_tokens {

    // References:
    // https://learn.microsoft.com/en-us/windows/win32/direct3d9/tokens
    // https://learn.microsoft.com/en-us/windows/win32/direct3d9/token-records
    // The record-bearing tokens
    /// name
    pub const TOKEN_NAME: u16 = 1;
    /// string
    pub const TOKEN_STRING: u16 = 2;
    /// integer
    pub const TOKEN_INTEGER: u16 = 3;
    /// guid
    pub const TOKEN_GUID: u16 = 5;
    /// integer list
    pub const TOKEN_INTEGER_LIST: u16 = 6;
    /// float list
    pub const TOKEN_FLOAT_LIST: u16 = 7;
    // The stand-alone tokens
    /// open brace
    pub const TOKEN_OBRACE: u16 = 10;
    /// close brace
    pub const TOKEN_CBRACE: u16 = 11;
    /// open parenthesis
    pub const TOKEN_OPAREN: u16 = 12;
    /// close parenthesis
    pub const TOKEN_CPAREN: u16 = 13;
    /// open bracket
    pub const TOKEN_OBRACKET: u16 = 14;
    /// close bracket
    pub const TOKEN_CBRACKET: u16 = 15;
    /// open angle
    pub const TOKEN_OANGLE: u16 = 16;
    /// close angle
    pub const TOKEN_CANGLE: u16 = 17;
    /// dot
    pub const TOKEN_DOT: u16 = 18;
    /// comma
    pub const TOKEN_COMMA: u16 = 19;
    /// semicolon
    pub const TOKEN_SEMICOLON: u16 = 20;
    /// template
    pub const TOKEN_TEMPLATE: u16 = 31;
    /// word
    pub const TOKEN_WORD: u16 = 40;
    /// double word
    pub const TOKEN_DWORD: u16 = 41;
    /// float
    pub const TOKEN_FLOAT: u16 = 42;
    /// double
    pub const TOKEN_DOUBLE: u16 = 43;
    /// char
    pub const TOKEN_CHAR: u16 = 44;
    /// unsigned char
    pub const TOKEN_UCHAR: u16 = 45;
    /// short word
    pub const TOKEN_SWORD: u16 = 46;
    /// signed double word
    pub const TOKEN_SDWORD: u16 = 47;
    /// void
    pub const TOKEN_VOID: u16 = 48;
    /// long pointer string
    pub const TOKEN_LPSTR: u16 = 49;
    /// unicode
    pub const TOKEN_UNICODE: u16 = 50;
    /// c string
    pub const TOKEN_CSTRING: u16 = 51;
    /// array
    pub const TOKEN_ARRAY: u16 = 52;
}
