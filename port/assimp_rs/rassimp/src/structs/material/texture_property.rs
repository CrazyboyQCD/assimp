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

//! Defines the data structures in which the imported materials
//! are returned.

/// ## Defines how the Nth texture of a specific type is combined with
/// the result of all previous layers.
///
/// Example (left: key, right: value):
///
/// ```text
/// DiffColor0     - gray
/// DiffTextureOp0 - aiTextureOpMultiply
/// DiffTexture0   - tex1.png
/// DiffTextureOp0 - aiTextureOpAdd
/// DiffTexture1   - tex2.png
/// ```
/// Written as equation, the final diffuse term for a specific pixel would be:
/// ```text
/// diffFinal = DiffColor0 * sampleTex(DiffTexture0,UV0) +
///     sampleTex(DiffTexture1,UV0) * diffContrib;
/// ```
/// where 'diffContrib' is the intensity of the incoming light for that pixel.
#[derive(Clone, Copy, Debug)]
pub enum AiTextureOp {
    /// T = T1 * T2
    Multiply = 0x0,

    /// T = T1 + T2
    Add = 0x1,

    /// T = T1 - T2
    Subtract = 0x2,

    /// T = T1 / T2
    Divide = 0x3,

    /// T = (T1 + T2) - (T1 * T2)
    SmoothAdd = 0x4,

    /// T = T1 + (T2-0.5)
    SignedAdd = 0x5,
}

/// ## Defines how UV coordinates outside the [0...1] range are handled.
///
/// Commonly referred to as 'wrapping mode'.
#[derive(Clone, Copy, Debug)]
pub enum AiTextureMapMode {
    /// A texture coordinate u|v is translated to u%1|v%1
    Wrap = 0x0,

    /// Texture coordinates outside [0...1]
    /// are clamped to the nearest valid value.
    Clamp = 0x1,

    /// If the texture coordinates for a pixel are outside [0...1]
    /// the texture is not applied to that pixel
    Decal = 0x3,

    /// A texture coordinate u|v becomes u%1|v%1 if (u-(u%1))%2 is zero and
    /// 1-(u%1)|1-(v%1) otherwise
    Mirror = 0x2,
}

/// ## Defines how the mapping coords for a texture are generated.
///
/// Real-time applications typically require full UV coordinates, so the use of
/// the aiProcess_GenUVCoords step is highly recommended. It generates proper
/// UV channels for non-UV mapped objects, as long as an accurate description
/// how the mapping should look like (e.g spherical) is given.
/// See the #AI_MATKEY_MAPPING property for more details.
#[derive(Clone, Copy, Debug)]
pub enum AiTextureMapping {
    /// The mapping coordinates are taken from an UV channel.
    ///
    /// #AI_MATKEY_UVWSRC property specifies from which UV channel
    /// the texture coordinates are to be taken from (remember,
    /// meshes can have more than one UV channel).
    UV = 0x0,

    /// Spherical mapping
    Spherical = 0x1,

    /// Cylindrical mapping
    Cylindrical = 0x2,

    /// Cubic mapping
    Cubic = 0x3,

    /// Planar mapping
    Planar = 0x4,

    /// Undefined mapping. Have fun.
    Other = 0x5,
}

/// ## Defines the purpose of a texture
///
/// This is a very difficult topic. Different 3D packages support different
/// kinds of textures. For very common texture types, such as bumpmaps, the
/// rendering results depend on implementation details in the rendering
/// pipelines of these applications. Assimp loads all texture references from
/// the model file and tries to determine which of the predefined texture
/// types below is the best choice to match the original use of the texture
/// as closely as possible.<br>
///
/// In content pipelines you'll usually define how textures have to be handled,
/// and the artists working on models have to conform to this specification,
/// regardless which 3D tool they're using.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum AiTextureType {
    /// Dummy value.
    ///
    /// No texture, but the value to be used as 'texture semantic'
    /// (#aiMaterialProperty::mSemantic) for all material properties
    /// *not* related to textures.
    #[default]
    None = 0,

    // LEGACY API MATERIALS
    //
    // Legacy refers to materials which
    // Were originally implemented in the specifications around 2000.
    // These must never be removed, as most engines support them.
    // Legacy = 1,
    /// The texture is combined with the result of the diffuse
    /// lighting equation.
    /// OR
    /// PBR Specular/Glossiness
    Diffuse = 1,

    /// The texture is combined with the result of the specular
    /// lighting equation.
    /// OR
    /// PBR Specular/Glossiness
    Specular = 2,

    /// The texture is combined with the result of the ambient
    /// lighting equation.
    Ambient = 3,

    /// The texture is added to the result of the lighting
    /// calculation. It isn't influenced by incoming light.
    Emissive = 4,

    /// The texture is a height map.
    ///
    /// By convention, higher gray-scale values stand for
    /// higher elevations from the base height.
    Height = 5,

    /// The texture is a (tangent space) normal-map.
    ///
    /// Again, there are several conventions for tangent-space
    /// normal maps. Assimp does (intentionally) not
    /// distinguish here.
    Normals = 6,

    /// The texture defines the glossiness of the material.
    ///
    /// The glossiness is in fact the exponent of the specular
    /// (phong) lighting equation. Usually there is a conversion
    /// function defined to map the linear color values in the
    /// texture to a suitable exponent. Have fun.
    Shininess = 7,

    /// The texture defines per-pixel opacity.
    ///
    /// Usually 'white' means opaque and 'black' means
    /// 'transparency'. Or quite the opposite. Have fun.
    Opacity = 8,

    /// Displacement texture
    ///
    /// The exact purpose and format is application-dependent.
    /// Higher color values stand for higher vertex displacements.
    Displacement = 9,

    /// Lightmap texture (aka Ambient Occlusion)
    ///
    /// Both 'Lightmaps' and dedicated 'ambient occlusion maps' are
    /// covered by this material property. The texture contains a
    /// scaling value for the final color value of a pixel. Its
    /// intensity is not affected by incoming light.
    Lightmap = 10,

    /// Reflection texture
    ///
    /// Contains the color of a perfect mirror reflection.
    /// Rarely used, almost never for real-time applications.
    Reflection = 11,

    /// PBR Materials
    ///
    /// PBR definitions from maya and other modelling packages now use this standard.
    /// This was originally introduced around 2012.
    /// Support for this is in game engines like Godot, Unreal or Unity3D.
    /// Modelling packages which use this are very common now.
    BaseColor = 12,

    /// Normal camera
    NormalCamera = 13,

    /// Emission color
    EmissionColor = 14,

    /// Metalness
    Metalness = 15,

    /// Diffuse roughness
    DiffuseRoughness = 16,

    /// Ambient occlusion
    AmbientOcclusion = 17,

    /// Unknown texture
    ///
    /// A texture reference that does not match any of the definitions
    /// above is considered to be 'unknown'. It is still imported,
    /// but is excluded from any further post-processing.
    Unknown = 18,

    // PBR Material Modifiers
    //
    // Some modern renderers have further PBR modifiers that may be overlaid
    // on top of the 'base' PBR materials for additional realism.
    // These use multiple texture maps, so only the base type is directly defined
    /// Sheen
    ///
    /// Generally used to simulate textiles that are covered in a layer of microfibers
    /// eg velvet
    /// https://github.com/KhronosGroup/glTF/tree/master/extensions/2.0/Khronos/KHR_materials_sheen
    Sheen = 19,

    /// Clearcoat
    ///
    /// Simulates a layer of 'polish' or 'lacquer' layered on top of a PBR substrate
    /// https://autodesk.github.io/standard-surface/#closures/coating
    /// https://github.com/KhronosGroup/glTF/tree/master/extensions/2.0/Khronos/KHR_materials_clearcoat
    Clearcoat = 20,

    /// Transmission
    ///
    /// Simulates transmission through the surface
    /// May include further information such as wall thickness
    Transmission = 21,

    /// Maya material declarations
    MayaBase = 22,

    /// Maya specular
    MayaSpecular = 23,

    /// Maya specular color
    MayaSpecularColor = 24,

    /// Maya specular roughness
    MayaSpecularRoughness = 25,

    /// Anisotropy
    ///
    /// Simulates a surface with directional properties
    Anisotropy = 26,

    /// gltf material declarations
    ///
    /// Refs:
    /// https://registry.khronos.org/glTF/specs/2.0/glTF-2.0.html#metallic-roughness-material
    ///
    /// "textures for metalness and roughness properties are packed together in a single
    /// texture called metallicRoughnessTexture. Its green channel contains roughness
    /// values and its blue channel contains metalness values..."
    ///
    /// https://registry.khronos.org/glTF/specs/2.0/glTF-2.0.html#_material_pbrmetallicroughness_metallicroughnesstexture
    ///
    /// "The metalness values are sampled from the B channel. The roughness values are
    /// sampled from the G channel..."
    MetallicRoughness = 27,
}

impl AiTextureType {
    /// Returns the string representation of the texture type
    pub fn as_str(&self) -> &str {
        match self {
            AiTextureType::None => "n/a",
            AiTextureType::Diffuse => "Diffuse",
            AiTextureType::Specular => "Specular",
            AiTextureType::Ambient => "Ambient",
            AiTextureType::Emissive => "Emissive",
            AiTextureType::Opacity => "Opacity",
            AiTextureType::Normals => "Normals",
            AiTextureType::Height => "Height",
            AiTextureType::Shininess => "Shininess",
            AiTextureType::Displacement => "Displacement",
            AiTextureType::Lightmap => "Lightmap",
            AiTextureType::Reflection => "Reflection",
            AiTextureType::BaseColor => "BaseColor",
            AiTextureType::NormalCamera => "NormalCamera",
            AiTextureType::EmissionColor => "EmissionColor",
            AiTextureType::Metalness => "Metalness",
            AiTextureType::DiffuseRoughness => "DiffuseRoughness",
            AiTextureType::AmbientOcclusion => "AmbientOcclusion",
            AiTextureType::Sheen => "Sheen",
            AiTextureType::Clearcoat => "Clearcoat",
            AiTextureType::Transmission => "Transmission",
            AiTextureType::MayaBase => "MayaBase",
            AiTextureType::MayaSpecular => "MayaSpecular",
            AiTextureType::MayaSpecularColor => "MayaSpecularColor",
            AiTextureType::MayaSpecularRoughness => "MayaSpecularRoughness",
            AiTextureType::Anisotropy => "Anisotropy",
            AiTextureType::MetallicRoughness => "MetallicRoughness",
            AiTextureType::Unknown => "Unknown",
        }
    }
}

/// ## Defines some mixed flags for a particular texture.
///
/// Usually you'll instruct your cg artists how textures have to look like ...
/// and how they will be processed in your application. However, if you use
/// Assimp for completely generic loading purposes you might also need to
/// process these flags in order to display as many 'unknown' 3D models as
/// possible correctly.
///
/// This corresponds to the #AI_MATKEY_TEXFLAGS property.
#[derive(Clone, Copy, Debug)]
pub enum AiTextureFlags {
    /// The texture's color values have to be inverted (component-wise 1-n)
    Invert = 0x1,

    /// Explicit request to the application to process the alpha channel
    /// of the texture.
    ///
    /// Mutually exclusive with #aiTextureFlags_IgnoreAlpha. These
    /// flags are set if the library can say for sure that the alpha
    /// channel is used/is not used. If the model format does not
    /// define this, it is left to the application to decide whether
    /// the texture alpha channel - if any - is evaluated or not.
    UseAlpha = 0x2,

    /// Explicit request to the application to ignore the alpha channel
    /// of the texture.
    ///
    /// Mutually exclusive with #aiTextureFlags_UseAlpha.
    IgnoreAlpha = 0x4,
}
