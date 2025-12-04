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

//! Declares the data structures in which the imported geometry is
//! returned by ASSIMP: [`AiAnimMesh`] data structures.

use alloc::{string::String, vec::Vec};

use crate::{
    AiVec3,
    structs::{
        color::Color4D,
        mesh::{AI_MAX_NUMBER_OF_COLOR_SETS, AI_MAX_NUMBER_OF_TEXTURECOORDS},
    },
};

/// ## An AnimMesh is an attachment to an [`AiMesh`] stores per-vertex animations for a particular frame.
///
/// You may think of an [`AiAnimMesh`] as a `patch` for the host mesh, which
/// replaces only certain vertex data streams at a particular time.
/// Each mesh stores n attached attached meshes ([`AiMesh::anim_meshes`]).
/// The actual relationship between the time line and anim meshes is
/// established by [`AiMeshAnim`], which references singular mesh attachments
/// by their ID and binds them to a time offset.

#[derive(Clone, Debug, Default)]
pub struct AiAnimMesh {
    /// Anim Mesh name
    pub name: String,

    /// Replacement for aiMesh::mVertices. If this array is non-nullptr,
    ///
    /// it *must* contain mNumVertices entries. The corresponding
    /// array in the host mesh must be non-nullptr as well - animation
    /// meshes may neither add or nor remove vertex components (if
    /// a replacement array is nullptr and the corresponding source
    /// array is not, the source data is taken instead)
    pub vertices: Vec<AiVec3>,

    /// Replacement for aiMesh::mNormals.
    pub normals: Vec<AiVec3>,

    /// Replacement for aiMesh::mTangents.
    pub tangents: Vec<AiVec3>,

    /// Replacement for aiMesh::mBitangents.
    pub bitangents: Vec<AiVec3>,

    /// Replacement for aiMesh::mColors
    pub colors: [Vec<Color4D>; AI_MAX_NUMBER_OF_COLOR_SETS],

    /// Replacement for aiMesh::mTextureCoords
    pub texture_coords: [Vec<AiVec3>; AI_MAX_NUMBER_OF_TEXTURECOORDS],

    /// The number of vertices in the aiAnimMesh, and thus the length of all
    /// the member arrays.
    ///
    /// This has always the same value as the mNumVertices property in the
    /// corresponding aiMesh. It is duplicated here merely to make the length
    /// of the member arrays accessible even if the aiMesh is not known, e.g.
    /// from language bindings.
    pub num_of_vertices: u32,

    /// Weight of the AnimMesh.
    pub weight: f32,
}
