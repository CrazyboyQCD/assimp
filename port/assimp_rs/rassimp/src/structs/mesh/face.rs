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
//! returned by ASSIMP: [`AiFace`] data structure.

/// ## A single face in a mesh, referring to multiple vertices.
///
/// If the length of [`Self::indices`] is 3, we call the face 'triangle', for length
/// of[`Self::indices`] > 3 it's called 'polygon' (hey, that's just a definition!).
///
/// [`AiMesh::primitive_types`](crate::structs::mesh::AiMesh::primitive_types) can be queried
/// to quickly examine which types of primitive are actually present in a mesh. The
/// [`AiPostProcessSteps::SortByPType`](crate::postprocess::AiPostProcessSteps::SortByPType)
/// flag executes a special post-processing algorithm which splits meshes with
/// *different* primitive types mixed up (e.g. lines and triangles) in several
/// 'clean' sub-meshes. Furthermore there is a configuration option (
/// [`AiPostProcessSteps::SortByPType`](crate::postprocess::AiPostProcessSteps::SortByPType) to
/// remove specific kinds of primitives from the imported scene, completely and forever.
/// In many cases you'll probably want to set this setting to
/// ```text
/// AiPostProcessSteps::SortByPType = AiPostProcessSteps::PrimitiveTypes | AiPostProcessSteps::PrimitivePoints
/// ```
/// Together with the
/// [`AiPostProcessSteps::Triangulate`](crate::postprocess::AiPostProcessSteps::Triangulate)
/// flag you can then be sure that the length of
/// [`Self::indices`](crate::structs::face::AiFace::indices) is always 3. @note Take a look at
/// the @link data Data Structures page @endlink for more information on the layout and winding
/// order of a face.
use alloc::vec::Vec;

/// ## A single face in a mesh, referring to multiple vertices.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct AiFace {
    /// Indices of the vertices that make up the face
    pub indices: Vec<u32>,
}
