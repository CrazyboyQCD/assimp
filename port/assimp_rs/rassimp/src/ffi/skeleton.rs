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

//! A skeleton represents the bone hierarchy of an animation.

#[cfg(feature = "armature_populate")]
use crate::ffi::AiNodeFFI;
use crate::ffi::{AiMatrix4x4FFI, mesh::AiMeshFFI, string::AiStringFFI};

/**
 * @brief A skeleton represents the bone hierarchy of an animation.
 *
 * Skeleton animations can be described as a tree of bones:
 *                  root
 *                    |
 *                  node1
 *                  /   \
 *               node3  node4
 * If you want to calculate the transformation of node three you need to compute the
 * transformation hierarchy for the transformation chain of node3:
 * root->node1->node3
 * Each node is represented as a skeleton instance.
 */
#[repr(C)]
pub struct AiSkeletonFFI {
    /**
     *  @brief The name of the skeleton instance.
     */
    pub name: AiStringFFI,

    /**
     *  @brief  The number of bones in the skeleton.
     */
    pub num_bones: u32,

    /**
     *  @brief The bone instance in the skeleton.
     */
    pub bones: *mut *mut AiSkeletonBoneFFI,
}

/**
 * @brief  A skeleton bone represents a single bone is a skeleton structure.
 *
 * Skeleton-Animations can be represented via a skeleton struct, which describes
 * a hierarchical tree assembled from skeleton bones. A bone is linked to a mesh.
 * The bone knows its parent bone. If there is no parent bone the parent id is
 * marked with -1.
 * The skeleton-bone stores a pointer to its used armature. If there is no
 * armature this value if set to nullptr.
 * A skeleton bone stores its offset-matrix, which is the absolute transformation
 * for the bone. The bone stores the locale transformation to its parent as well.
 * You can compute the offset matrix by multiplying the hierarchy like:
 * Tree: s1 -> s2 -> s3
 * Offset-Matrix s3 = locale-s3 * locale-s2 * locale-s1
 */
#[repr(C)]
pub struct AiSkeletonBoneFFI {
    /// The parent bone index, is -1 one if this bone represents the root bone.
    pub parent: i32,

    #[cfg(feature = "armature_populate")]
    // #ifndef ASSIMP_BUILD_NO_ARMATUREPOPULATE_PROCESS
    /// @brief The bone armature node - used for skeleton conversion
    /// you must enable aiProcess_PopulateArmatureData to populate this
    pub armature: *mut AiNodeFFI,

    #[cfg(feature = "armature_populate")]
    /// @brief The bone node in the scene - used for skeleton conversion
    /// you must enable aiProcess_PopulateArmatureData to populate this
    pub node: *mut AiNodeFFI,

    // #endif
    /// @brief The number of weights
    pub num_weights: u32,

    /// The mesh index, which will get influenced by the weight.
    pub mesh_id: *mut AiMeshFFI,

    /// The influence weights of this bone, by vertex index.
    pub weights: *mut AiVertexWeightFFI,

    /** Matrix that transforms from bone space to mesh space in bind pose.
     *
     * This matrix describes the position of the mesh
     * in the local space of this bone when the skeleton was bound.
     * Thus it can be used directly to determine a desired vertex position,
     * given the world-space transform of the bone when animated,
     * and the position of the vertex in mesh space.
     *
     * It is sometimes called an inverse-bind matrix,
     * or inverse bind pose matrix.
     */
    pub offset_matrix: AiMatrix4x4FFI,

    /// Matrix that transforms the locale bone in bind pose.
    pub local_matrix: AiMatrix4x4FFI,
}

/** @brief A single influence of a bone on a vertex.
 */
#[repr(C)]
pub struct AiVertexWeightFFI {
    /// Index of the vertex which is influenced by the bone.
    pub vertex_id: u32,

    /// The strength of the influence in the range (0...1).
    /// The influence from all bones at one vertex amounts to 1.
    pub weight: f32,
}
