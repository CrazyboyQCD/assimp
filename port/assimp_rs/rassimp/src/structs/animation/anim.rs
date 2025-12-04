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

//! Defines the data structures in which the imported animations
//! are returned.

use alloc::{string::String, vec::Vec};

use crate::structs::key::{AiMeshMorphKey, AiQuatKey, AiVectorKey};

/// ## Binds a anim-mesh to a specific point in time.
#[derive(Clone, Copy, Debug, Default)]
pub struct AiMeshKey {
    /// The time of this key
    pub time: f64,

    /// Index into the [`AiMesh::anim_meshes`] array of the
    /// mesh corresponding to the [`AiMeshAnim`] hosting this
    /// key frame. The referenced anim mesh is evaluated
    /// according to the rules defined in the docs for
    /// [`AiAnimMesh`](crate::structs::mesh::anim_mesh::AiAnimMesh).
    pub value: u32,
}

impl AiMeshKey {
    /// Construction from a given time and key value
    pub const fn new(time: f64, value: u32) -> Self {
        Self { time, value }
    }
}

/// ## Defines how an animation channel behaves outside the defined time range.
///
/// This corresponds to [`AiNodeAnim::pre_state`] and [`AiNodeAnim::post_state`].
#[derive(Clone, Copy, Debug, Default)]
pub enum AiAnimBehaviour {
    /// The value from the default node transformation is taken
    #[default]
    Default = 0x0,

    /// The nearest key value is used without interpolation
    Constant = 0x1,

    /// The value of the nearest two keys is linearly extrapolated for the current time value.
    Linear = 0x2,

    /// The animation is repeated.
    ///
    /// If the animation key go from n to m and the current time is t, use the value at (t-n) %
    /// (|m-n|).
    Repeat = 0x3,
}

/// ## A node animation is a collection of key frames for a single node.
#[derive(Clone, Debug, Default)]
pub struct AiNodeAnim {
    /// The name of the node affected by this animation.
    ///
    ///  The node must exist and it must be unique.
    pub node_name: String,

    /// The position keys of this animation channel.
    ///
    /// Positions are specified as 3D vector.
    ///
    /// If there are position keys, there will also be at least one scaling and one rotation key.
    pub position_keys: Vec<AiVectorKey>,

    /// The rotation keys of this animation channel.
    ///
    /// Rotations are given as quaternions, which are 4D vectors.
    ///
    /// If there are rotation keys, there will also be at least one scaling and one position key.
    pub rotation_keys: Vec<AiQuatKey>,

    /// The scaling keys of this animation channel.
    ///
    /// Scalings are specified as 3D vector.
    ///
    /// If there are scaling keys, there will also be at least one position and one rotation key.
    pub scaling_keys: Vec<AiVectorKey>,

    /// Defines how the animation behaves before the first key is encountered.
    ///
    /// The default value is [`AiAnimBehaviour::Default`] (the original transformation matrix of
    /// the affected node is used).
    pub pre_state: AiAnimBehaviour,

    /// Defines how the animation behaves after the last key was processed.
    ///
    /// The default value is [`AiAnimBehaviour::Default`] (the original transformation matrix of
    /// the affected node is taken).
    pub post_state: AiAnimBehaviour,
}

/// Describes vertex-based animations for a single mesh or a group of
/// meshes. Meshes carry the animation data for each frame in their
/// aiMesh::mAnimMeshes array. The purpose of aiMeshAnim is to
/// define keyframes linking each mesh attachment to a particular
/// point in time.
#[derive(Clone, Debug, Default)]
pub struct AiMeshAnim {
    /// ### Name of the mesh to be animated.
    ///
    /// An empty string is not allowed, animated meshes need to be named
    /// (not necessarily uniquely, the name can basically serve as wild-card
    /// to select a group of meshes with similar animation setup)
    pub name: String,

    /// Key frames of the animation.
    pub key_frames: Vec<AiMeshKey>,
}

/// Describes a morphing animation of a given mesh.
#[derive(Clone, Debug, Default)]
pub struct AiMeshMorphAnim {
    /// ### Name of the mesh to be animated.
    ///
    /// An empty string is not allowed, animated meshes need to be named
    /// (not necessarily uniquely, the name can basically serve as wildcard
    /// to select a group of meshes with similar animation setup)
    pub name: String,

    /// Key frames of the animation.
    pub key_frames: Vec<AiMeshMorphKey>,
}
