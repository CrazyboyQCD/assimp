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

//! A single bone of a mesh.

#[cfg(feature = "armature_populate")]
use crate::ffi::AiNodeFFI;
use crate::ffi::{AiMatrix4x4FFI, skeleton::AiVertexWeightFFI, string::AiStringFFI};

/// ## A single bone of a mesh.
///
/// A bone has a name by which it can be found in the frame hierarchy and by which it can be
/// addressed by animations. In addition it has a number of influences on vertices,
/// and a matrix relating the mesh position to the position of the bone at the time of binding.
#[repr(C)]
pub struct AiBoneFFI {
    /// The name of the bone.
    pub name: AiStringFFI,

    /// The number of vertices affected by this bone.
    ///
    /// The number of vertices affected by this bone.
    /// The maximum value for this member is #AI_MAX_BONE_WEIGHTS.
    pub num_weights: u32,

    #[cfg(feature = "armature_populate")]
    // #ifndef ASSIMP_BUILD_NO_ARMATUREPOPULATE_PROCESS
    ///
    /// The bone armature node - used for skeleton conversion
    /// you must enable aiProcess_PopulateArmatureData to populate this
    pub armature: *mut AiNodeFFI,

    #[cfg(feature = "armature_populate")]
    /// The bone node in the scene - used for skeleton conversion
    /// you must enable aiProcess_PopulateArmatureData to populate this
    pub node: *mut AiNodeFFI,

    // #endif
    /// The influence weights of this bone, by vertex index.
    pub weights: *mut AiVertexWeightFFI,

    /// Matrix that transforms from mesh space to bone space in bind pose.
    ///
    /// This matrix describes the position of the mesh in the local space of this bone when the
    /// skeleton was bound. Thus it can be used directly to determine a desired vertex
    /// position, given the world-space transform of the bone when animated,
    /// and the position of the vertex in mesh space.
    ///
    /// It is sometimes called an inverse-bind matrix, or inverse bind pose matrix.
    pub offset_matrix: AiMatrix4x4FFI,
}
