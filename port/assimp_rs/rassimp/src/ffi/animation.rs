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

//! An animation in the scene.

use crate::ffi::{AiQuaternionFFI, AiVector3DFFI, string::AiStringFFI};

/// ## An animation consists of key-frame data for a number of nodes.
/// For each node affected by the animation a separate series of data is given.
#[repr(C)]
pub struct AiAnimationFFI {
    /// The name of the animation.
    ///
    /// If the modeling package this data was
    /// exported from does support only a single animation channel, this
    /// name is usually empty (length is zero).
    pub name: AiStringFFI,

    /// Duration of the animation in ticks.
    pub duration: f64,

    /// Ticks per second. 0 if not specified in the imported file
    pub ticks_per_second: f64,

    /// The number of bone animation channels. Each channel affects a single node.
    pub num_channels: u32,

    /// The node animation channels. Each channel affects a single node.
    ///
    /// The array is mNumChannels in size.
    pub channels: *mut *mut AiNodeAnimFFI,

    /// The number of mesh animation channels.
    ///
    /// Each channel affects a single mesh.
    /// The array is mNumMeshChannels in size.
    pub num_mesh_channels: u32,

    /// The mesh animation channels.
    ///
    /// Each channel affects a single mesh.
    /// The array is mNumMeshChannels in size.
    pub mesh_channels: *mut *mut AiMeshAnimFFI,

    /// The number of mesh animation channels.
    ///
    /// Each channel affects a single mesh and defines morphing animation.
    pub num_morph_mesh_channels: u32,

    /// The morph mesh animation channels.
    ///
    /// Each channel affects a single mesh.
    /// The array is mNumMorphMeshChannels in size.
    pub morph_mesh_channels: *mut *mut AiMeshMorphAnimFFI,
}

/// ## Defines how an animation channel behaves outside the defined time range.
///
/// This corresponds to
/// [`AiNodeAnimFFI::pre_state`](crate::ffi::animation::AiNodeAnimFFI::pre_state) and
/// [`AiNodeAnimFFI::post_state`](crate::ffi::animation::AiNodeAnimFFI::post_state).
#[cfg_attr(not(feature = "swig"), repr(C))]
#[cfg_attr(feature = "swig", repr(C, u32))]
pub enum AiAnimBehaviourFFI {
    /// The value from the default node transformation is taken
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

/// ## Describes the animation of a single node.
///
/// The name specifies the bone/node which is affected by this animation channel.
/// The keyframes are given in three separate series of values, one each for position, rotation and
/// scaling. The transformation matrix computed from these values replaces the node's original
/// transformation matrix at a specific time.
///
/// This means all keys are absolute and not relative to the bone default pose.
///
/// The order in which the transformations are applied is
/// - as usual - scaling, rotation, translation.
///
/// *`Note:`* All keys are returned in their correct, chronological order.
///
/// Duplicate keys don't pass the validation step. Most likely there
/// will be no negative time values, but they are not forbidden also (so
/// implementations need to cope with them! )
#[repr(C)]
pub struct AiNodeAnimFFI {
    /// The name of the node affected by this animation.
    ///
    /// The node must exist and it must be unique.
    pub node_name: AiStringFFI,

    /// The number of position keys
    pub num_position_keys: u32,

    /// The position keys of this animation channel.
    ///
    /// Positions are specified as 3D vector. The array is mNumPositionKeys in size.
    ///
    /// If there are position keys, there will also be at least one scaling and one rotation key.
    pub position_keys: *mut AiVectorKeyFFI,

    /// The number of rotation keys
    pub num_rotation_keys: u32,

    /// The rotation keys of this animation channel.
    ///
    /// Rotations are given as quaternions, which are 4D vectors. The array is mNumRotationKeys in
    /// size.
    ///
    /// If there are rotation keys, there will also be at least one scaling and one position key.
    pub rotation_keys: *mut AiQuatKeyFFI,

    /// The number of scaling keys
    pub num_scaling_keys: u32,

    /// The scaling keys of this animation channel.
    ///
    /// Scalings are specified as 3D vector. The array is
    /// [`num_scaling_keys`](Self::num_scaling_keys) in size.
    ///
    /// If there are scaling keys, there will also be at least one position and one rotation key.
    /// specified as 3D vector. The array is [`num_scaling_keys`](Self::num_scaling_keys) in size.
    ///
    ///If there are scaling keys, there will also be at least one
    ///position and one rotation key.*/
    pub scaling_keys: *mut AiVectorKeyFFI,

    /// Defines how the animation behaves before the first key is encountered.
    ///
    ///  The default value is aiAnimBehaviour_DEFAULT (the original
    ///  transformation matrix of the affected node is used).
    pub pre_state: AiAnimBehaviourFFI,

    /// Defines how the animation behaves after the last key was processed.
    ///
    /// The default value is aiAnimBehaviour_DEFAULT (the original
    /// transformation matrix of the affected node is taken).*/
    pub post_state: AiAnimBehaviourFFI,
}

/// ## A time-value pair specifying a certain 3D vector for the given time.
#[repr(C)]
pub struct AiVectorKeyFFI {
    /// The time of this key
    pub time: f64,

    /// The value of this key
    pub value: AiVector3DFFI,

    /// The interpolation setting of this key
    pub interpolation: AiAnimInterpolationFFI,
}

/// The interpolation setting of a key.
#[cfg_attr(not(feature = "swig"), repr(C))]
#[cfg_attr(feature = "swig", repr(C, u32))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AiAnimInterpolationFFI {
    /// The value from the default node transformation is taken
    Step,

    /// The nearest key value is used without interpolation
    Linear,

    /// The value of the nearest two keys is linearly extrapolated for the current time value.
    SphericalLinear,

    /// The animation is repeated.
    ///
    /// If the animation key go from n to m and the current time is t, use the value at (t-n) %
    /// (|m-n|).
    CubicSpline,
}

/// ## A time-value pair specifying a rotation for the given time.
///
/// Rotations are expressed with quaternions.
#[repr(C)]
pub struct AiQuatKeyFFI {
    /// The time of this key
    pub time: f64,

    /// The value of this key
    pub value: AiQuaternionFFI,

    /// The interpolation setting of this key
    pub interpolation: AiAnimInterpolationFFI,
}

/// ## Describes vertex-based animations for a single mesh or a group of meshes.
///
/// Meshes carry the animation data for each frame in their aiMesh::mAnimMeshes array.
/// The purpose of aiMeshAnim is to define keyframes linking each mesh attachment to a particular
/// point in time.
#[repr(C)]
pub struct AiMeshAnimFFI {
    /// Name of the mesh to be animated.
    ///
    /// An empty string is not allowed,
    /// animated meshes need to be named (not necessarily uniquely,
    /// the name can basically serve as wild-card to select a group
    /// of meshes with similar animation setup)
    pub name: AiStringFFI,

    /// Size of the #mKeys array. Must be 1, at least.
    pub num_keys: u32,

    /// Key frames of the animation. May not be nullptr.
    pub keys: *mut AiMeshKeyFFI,
}

/// ## Describes a morphing animation of a given mesh.
#[repr(C)]
pub struct AiMeshMorphAnimFFI {
    /// Name of the mesh to be animated.
    ///
    /// An empty string is not allowed,
    /// animated meshes need to be named (not necessarily uniquely,
    /// the name can basically serve as wildcard to select a group
    /// of meshes with similar animation setup)
    pub name: AiStringFFI,

    /// Size of the #mKeys array. Must be 1, at least.
    pub num_keys: u32,

    /// Key frames of the animation. May not be nullptr.
    pub keys: *mut AiMeshMorphKeyFFI,
}

/// ## Binds a anim-mesh to a specific point in time.
#[repr(C)]
pub struct AiMeshKeyFFI {
    /// The time of this key
    pub time: f64,

    /// Index into the aiMesh::mAnimMeshes array of the mesh corresponding to the #aiMeshAnim
    /// hosting this key frame.
    ///
    /// The referenced anim mesh is evaluated according to the rules defined in the docs for
    /// #aiAnimMesh.
    pub value: u32,
}

/// ## Binds a morph anim mesh to a specific point in time.
#[repr(C)]
pub struct AiMeshMorphKeyFFI {
    /// The time of this key
    pub time: f64,

    /// The values and weights at the time of this key
    ///
    /// - mValues: index of attachment mesh to apply weight at the same position in mWeights
    /// - mWeights: weight to apply to the blend shape index at the same position in mValues
    pub values: *mut u32,

    /// The weights of the values.
    pub weights: *mut f64,

    /// The number of values and weights
    pub num_values_and_weights: u32,
}
