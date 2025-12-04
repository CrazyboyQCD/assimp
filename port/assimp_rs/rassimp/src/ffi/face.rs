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

//! A single face in a mesh, referring to multiple vertices.

/// ## A single face in a mesh, referring to multiple vertices.
///
/// If mNumIndices is 3, we call the face 'triangle', for mNumIndices > 3
/// it's called 'polygon' (hey, that's just a definition!).
/// <br>
/// aiMesh::mPrimitiveTypes can be queried to quickly examine which types of
/// primitive are actually present in a mesh. The #aiProcess_SortByPType flag
/// executes a special post-processing algorithm which splits meshes with
/// *different* primitive types mixed up (e.g. lines and triangles) in several
/// 'clean' sub-meshes. Furthermore there is a configuration option (
/// #AI_CONFIG_PP_SBP_REMOVE) to force #aiProcess_SortByPType to remove
/// specific kinds of primitives from the imported scene, completely and forever.
/// In many cases you'll probably want to set this setting to
/// @code
/// aiPrimitiveType_LINE|aiPrimitiveType_POINT
/// @endcode
/// Together with the #aiProcess_Triangulate flag you can then be sure that
/// #aiFace::mNumIndices is always 3.
/// @note Take a look at the @link data Data Structures page @endlink for
/// more information on the layout and winding order of a face.
#[repr(C)]
pub struct AiFaceFFI {
    /// Number of indices defining this face.
    /// The maximum value for this member is #AI_MAX_FACE_INDICES.
    pub num_indices: u32,

    /// Pointer to the indices array. Size of the array is given in numIndices.
    pub indices: *mut u32,
}
