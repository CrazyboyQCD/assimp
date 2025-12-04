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

//! Defines the [`AiLight`] data structure.

use alloc::string::String;

use crate::{AiVec2, AiVec3, structs::color::Color3D};

/// ## Enumerates all supported types of light sources.
#[derive(Clone, Debug, Default)]
pub enum AiLightType {
    /// Undefined light type.
    #[default]
    Undefined = 0x0,

    /// A directional light source has a well-defined direction
    /// but is infinitely far away. That's quite a good
    /// approximation for sun light.
    Directional = 0x1,

    /// A point light source has a well-defined position
    /// in space but no direction - it emits light in all
    /// directions. A normal bulb is a point light.
    Point = 0x2,

    /// A spot light source emits light in a specific
    /// angle. It has a position and a direction it is pointing to.
    /// A good example for a spot light is a light spot in
    /// sport arenas.
    Spot = 0x3,

    /// The generic light level of the world, including the bounces
    /// of all other light sources.
    /// Typically, there's at most one ambient light in a scene.
    /// This light type doesn't have a valid position, direction, or
    /// other properties, just a color.
    Ambient = 0x4,

    /// An area light is a rectangle with predefined size that uniformly
    /// emits light from one of its sides. The position is center of the
    /// rectangle and direction is its normal vector.
    Area = 0x5,
}

/// ## Helper structure to describe a light source.
///
/// Assimp supports multiple sorts of light sources, including
/// directional, point and spot lights.
///
/// All of them are defined with just
/// a single structure and distinguished by their parameters.
///
/// ### Note:
///
/// Some file formats (such as 3DS, ASE) export a "target point" -
/// the point a spot light is looking at (it can even be animated). Assimp
/// writes the target point as a sub-node of a spot-lights's main node,
/// called "<spotName>.Target".
///
/// However, this is just additional information
/// then, the transformation tracks of the main node make the
/// spot light already point in the right direction.
#[derive(Clone, Debug, Default)]
pub struct AiLight {
    /// ### The name of the light source.
    ///
    /// There must be a node in the scene-graph with the same name.
    /// This node specifies the position of the light in the scene
    /// hierarchy and can be animated.
    pub name: String,

    /// ### The type of the light source.
    ///
    /// [`AiLightType::Undefined`] is not a valid value for this member.
    pub light_type: AiLightType,

    /// ### Position of the light source in space. Relative to the
    /// transformation of the node corresponding to the light.
    ///
    /// The position is undefined for directional lights.
    pub position: AiVec3,

    /// ### Direction of the light source in space. Relative to the
    /// transformation of the node corresponding to the light.
    ///
    /// The direction is undefined for point lights. The vector
    /// may be normalized, but it needn't.
    pub direction: AiVec3,

    /// ### Up direction of the light source in space. Relative to the
    /// transformation of the node corresponding to the light.
    ///
    /// The direction is undefined for point lights. The vector
    /// may be normalized, but it needn't.
    pub up: AiVec3,

    /// ### Constant light attenuation factor.
    ///
    /// The intensity of the light source at a given distance 'd' from
    /// the light's position is
    /// ```text
    /// Atten = 1/( att0 + att1 * d + att2 * d*d)
    /// ```
    /// This member corresponds to the att0 variable in the equation.
    /// Naturally undefined for directional lights.
    pub attenuation_constant: f32,

    /// ### Linear light attenuation factor.
    ///
    /// The intensity of the light source at a given distance 'd' from
    /// the light's position is
    /// ```text
    /// Atten = 1/( att0 + att1 * d + att2 * d*d)
    /// ```
    /// This member corresponds to the att1 variable in the equation.
    /// Naturally undefined for directional lights.
    pub attenuation_linear: f32,

    /// ### Quadratic light attenuation factor.
    ///
    /// The intensity of the light source at a given distance 'd' from
    /// the light's position is
    /// ```text
    /// Atten = 1/( att0 + att1 * d + att2 * d*d)
    /// ```
    /// This member corresponds to the att2 variable in the equation.
    /// Naturally undefined for directional lights.
    pub attenuation_quadratic: f32,

    /// ### Diffuse color of the light source
    ///
    /// The diffuse light color is multiplied with the diffuse
    /// material color to obtain the final color that contributes
    /// to the diffuse shading term.
    pub color_diffuse: Color3D,

    /// ### Specular color of the light source
    ///
    /// The specular light color is multiplied with the specular
    /// material color to obtain the final color that contributes
    /// to the specular shading term.
    pub color_specular: Color3D,

    /// ### Ambient color of the light source
    ///
    /// The ambient light color is multiplied with the ambient
    /// material color to obtain the final color that contributes
    /// to the ambient shading term. Most renderers will ignore
    /// this value it, is just a remaining of the fixed-function pipeline
    /// that is still supported by quite many file formats.
    pub color_ambient: Color3D,

    /// ### Inner angle of a spot light's light cone.
    ///
    /// The spot light has maximum influence on objects inside this
    /// angle. The angle is given in radians. It is 2PI for point
    /// lights and undefined for directional lights.
    pub angle_inner_cone: f32,

    /// ### Outer angle of a spot light's light cone.
    ///
    /// The spot light does not affect objects outside this angle.
    /// The angle is given in radians. It is 2PI for point lights and
    /// undefined for directional lights. The outer angle must be
    /// greater than or equal to the inner angle.
    /// It is assumed that the application uses a smooth
    /// interpolation between the inner and the outer cone of the
    /// spot light.
    pub angle_outer_cone: f32,

    /// ### Size of area light source.
    pub size: AiVec2,
}
