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

//! Implementation of the material system of the library
//! Example:
//! ```rust
//! use rassimp::structs::material::property::{AiColorDiffuseProperty, AiProperty};
//! let mut material = AiMaterial::default();
//! let property = AiProperty::MaterialColorDiffuse(AiColorDiffuseProperty::Color3D(Vec3::new(1.0, 0.0, 0.0)));
//! material.add_property(property, 0);
//! let diffuse = material.get_property(0, AiProperty::is_material_color_diffuse_property);
//! assert_eq!(diffuse, Some(&AiColorDiffuseProperty::Color3D(Vec3::new(1.0, 0.0, 0.0))));
//! ```

use alloc::{string::String, vec::Vec};

use crate::{
    AiReal, AiVec2, AiVec3, AiVec4,
    structs::{
        color::{Color3D, Color4D},
        material::texture_property::{
            AiTextureFlags, AiTextureMapMode, AiTextureMapping, AiTextureOp, AiTextureType,
        },
    },
};

/// Color Variant
#[derive(Clone, Debug)]
pub enum AiColorDiffuseProperty {
    /// RGB color.
    Color3D(Color3D),
    /// RGBA color.
    Color4D(Color4D),
}

impl From<AiVec3> for AiColorDiffuseProperty {
    fn from(value: AiVec3) -> Self {
        Self::Color3D(value)
    }
}
impl From<AiVec4> for AiColorDiffuseProperty {
    fn from(value: AiVec4) -> Self {
        Self::Color4D(value)
    }
}

bitflags::bitflags! {
    /// ## Defines all shading models supported by the library
    ///
    /// Property: #AI_MATKEY_SHADING_MODEL
    ///
    /// The list of shading modes has been taken from Blender.
    /// See Blender documentation for more information. The API does
    /// not distinguish between "specular" and "diffuse" shaders (thus the
    /// specular term for diffuse shading models like Oren-Nayar remains
    /// undefined).
    ///
    /// Again, this value is just a hint. Assimp tries to select the shader whose
    /// most common implementation matches the original rendering results of the
    /// 3D modeler which wrote a particular model as closely as possible.
    #[derive(Clone, Copy, Debug)]
    pub struct AiShadingMode: u32 {
        /// Flat shading. Shading is done on per-face base,
        /// diffuse only. Also known as 'faceted shading'.
        const Flat = 1 << 0;

        /// Simple Gouraud shading.
        const Gouraud = 1 << 1;

        /// Phong-Shading -
        const Phong = 1 << 2;

        /// Phong-Blinn-Shading
        const Blinn = 1 << 3;

        /// Toon-Shading per pixel
        const Toon = 1 << 4;

        /// OrenNayar-Shading per pixel
        const OrenNayar = 1 << 5;

        /// Minnaert-Shading per pixel
        const Minnaert = 1 << 6;

        /// CookTorrance-Shading per pixel
        const CookTorrance = 1 << 7;

        /// No shading at all. Constant light influence of 1.0.
        const No_Shading = 1 << 8;

        /// Unlit shading
        const Unlit = Self::No_Shading.bits();

        /// Fresnel shading
        const Fresnel = 1 << 9;

        /// Physically-Based Rendering (PBR) shading using
        /// Bidirectional scattering/reflectance distribution function (BSDF/BRDF)
        const Pbr_Brdf = 1 << 10;
    }
}

/// ## Defines alpha-blend flags.
///
/// If you're familiar with OpenGL or D3D, these flags aren't new to you.
/// They define *how* the final color value of a pixel is computed, basing
/// on the previous color at that pixel and the new color value from the
/// material.
/// The blend formula is:
/// ```text
/// SourceColor * SourceBlend + DestColor * DestBlend
/// ```
/// where DestColor is the previous color in the frame-buffer at this
/// position and SourceColor is the material color before the transparency
/// calculation.
///
/// This corresponds to the #AI_MATKEY_BLEND_FUNC property.
#[derive(Clone, Copy, Debug)]
pub enum AiBlendMode {
    /// Formula:
    /// ```text
    /// SourceColor*SourceAlpha + DestColor*(1-SourceAlpha)
    /// ```
    Default = 0x0,

    /// Additive blending
    ///
    /// Formula:
    /// ```text
    /// SourceColor*1 + DestColor*1
    /// ```
    Additive = 0x1,
    // we don't need more for the moment, but we might need them
    // in future versions ...
}

/// ## Defines how an UV channel is transformed.
///
/// This is just a helper structure for the #AI_MATKEY_UVTRANSFORM key.
/// See its documentation for more details.
///
/// Typically you'll want to build a matrix of this information. However,
/// we keep separate scaling/translation/rotation values to make it
/// easier to process and optimize UV transformations internally.
#[derive(Clone, Copy, Debug)]
pub struct AiUVTransform {
    /// Translation on the u and v axes.
    ///
    /// The default value is (0|0).
    pub translation: AiVec2,

    /// Scaling on the u and v axes.
    ///
    /// The default value is (1|1).
    pub scaling: AiVec2,

    /// Rotation - in counter-clockwise direction.
    ///
    /// The rotation angle is specified in radians. The
    /// rotation center is 0.5f|0.5f. The default value
    /// 0.f.
    pub rotation: AiReal,
}

impl Default for AiUVTransform {
    fn default() -> Self {
        Self {
            translation: AiVec2::new(0.0, 0.0),
            scaling: AiVec2::new(1.0, 1.0),
            rotation: 0.0,
        }
    }
}

/// ## Property of a material.
#[derive(Clone, Debug)]
pub enum AiProperty {
    // ---------------------------------------------------------------------------
    // Material Property
    // ---------------------------------------------------------------------------
    /// Material name
    MaterialName(String),

    /// Material is two sided
    MaterialIsTwoSided(bool),

    /// Material shading model
    MaterialShadingMode(AiShadingMode),

    /// Material blend mode
    MaterialBlendMode(AiBlendMode),

    /// Material opacity
    MaterialOpacity(AiReal),

    /// Material transparency factor
    MaterialTransparencyFactor(AiReal),

    /// Material bump scaling
    MaterialBumpScaling(AiReal),

    /// Material shininess
    MaterialShininess(AiReal),

    /// Material reflectivity
    MaterialReflectivity(AiReal),

    /// Material shininess strength
    MaterialShininessStrength(AiReal),

    /// Material refract index
    MaterialRefracti(AiReal),

    /// Material diffuse color
    MaterialColorDiffuse(AiColorDiffuseProperty),

    /// Material ambient color
    MaterialColorAmbient(AiColorDiffuseProperty),

    /// Material specular color
    MaterialColorSpecular(AiVec3),

    /// Material emissive color
    MaterialColorEmissive(AiVec3),

    /// Material transparent color
    MaterialColorTransparent(AiColorDiffuseProperty),

    /// Material reflective color
    MaterialColorReflective(AiColorDiffuseProperty),

    /// Material global background image
    MaterialGlobalBackgroundImage(String),

    /// Material global shader language
    /// To get the used shader language.
    MaterialGlobalShaderLang(String),

    /// Assigned vertex shader code stored as a string.
    MaterialShaderVertex(String),

    /// Assigned fragment shader code stored as a string.
    MaterialShaderFragment(String),

    /// Assigned geometry shader code stored as a string.
    MaterialShaderGeometry(String),

    /// Assigned tessellation shader code stored as a string.
    MaterialShaderTessellation(String),

    /// Assigned primitive shader code stored as a string.
    MaterialShaderPrimitive(String),

    /// Assigned compute shader code stored as a string.
    MaterialShaderCompute(String),

    // PBR material support
    // ---------------------------
    // Properties defining PBR rendering techniques
    /// Whether to use color map.
    MaterialUseColorMap(bool),

    // Metallic/Roughness Workflow
    // ---------------------------
    // Base RGBA color factor. Will be multiplied by final base color texture values if extant
    // Note: Importers may choose to copy this into AI_MATKEY_COLOR_DIFFUSE for compatibility
    // with renderers and formats that do not support Metallic/Roughness PBR
    /// Base color.
    MaterialBaseColor(AiColorDiffuseProperty),

    /// Metallic factor. 0.0 = Full Dielectric, 1.0 = Full Metal
    MaterialMetallicFactor(AiReal),

    /// Whether to use roughness map.
    MaterialUseRoughnessMap(bool),

    /// Roughness factor. 0.0 = Perfectly Smooth, 1.0 = Completely Rough
    MaterialRoughnessFactor(AiReal),

    /// Anisotropy factor. 0.0 = isotropic, 1.0 = anisotropy along tangent direction,
    MaterialAnisotropyFactor(AiReal),

    // Specular/Glossiness Workflow
    // ---------------------------
    // Diffuse/Albedo Color. Note: Pure Metals have a diffuse of {0,0,0}
    // AI_MATKEY_COLOR_DIFFUSE
    // Specular Color.
    // Note: Metallic/Roughness may also have a Specular Color
    // AI_MATKEY_COLOR_SPECULAR
    /// Specular factor.
    MaterialSpecularFactor(AiReal),

    /// Glossiness factor. 0.0 = Completely Rough, 1.0 = Perfectly Smooth
    MaterialGlossinessFactor(AiReal),

    // Sheen
    // -----
    /// Sheen base RGB color. Default {0,0,0}
    ColorSheenColorFactor(AiColorDiffuseProperty),

    /// Sheen Roughness Factor.
    MaterialSheenRoughnessFactor(AiReal),

    // Clearcoat
    // ---------
    /// Clearcoat layer intensity. 0.0 = none (disabled)
    MaterialClearcoatFactor(AiReal),

    /// Clearcoat Roughness Factor.
    MaterialClearcoatRoughnessFactor(AiReal),

    // Transmission
    // ------------
    // https://github.com/KhronosGroup/glTF/tree/master/extensions/2.0/Khronos/KHR_materials_transmission
    /// Base percentage of light transmitted through the surface. 0.0 = Opaque, 1.0 = Fully
    /// transparent
    MaterialTransmissionFactor(AiReal),

    // Volume
    // ------------
    // https://github.com/KhronosGroup/glTF/tree/main/extensions/2.0/Khronos/KHR_materials_volume
    /// The thickness of the volume beneath the surface. If the value is 0 the material is
    /// thin-walled. Otherwise the material is a volume boundary.
    MaterialVolumeThicknessFactor(AiReal),

    /// Density of the medium given as the average distance that light travels in the medium before
    /// interacting with a particle.
    MaterialVolumeAttenuationDistance(AiReal),

    /// The color that white light turns into due to absorption when reaching the attenuation
    /// distance.
    MaterialVolumeAttenuationColor(AiColorDiffuseProperty),

    // Emissive
    // --------
    /// Whether to use emissive map.
    MaterialUseEmissiveMap(bool),

    /// Emissive intensity. 0.0 = None, 1.0 = Full intensity
    MaterialEmissiveIntensity(AiReal),

    /// Use ambient occlusion map.
    MaterialUseAmbientOcclusionMap(bool),

    // Anisotropy
    // ----------
    /// Anisotropy rotation. 0.0 = 0 degrees, 1.0 = 90 degrees
    MaterialAnisotropyRotation(AiReal),

    /// Anisotropy texture.
    MaterialAnisotropyTexture(AiTextureType),

    // ---------------------------------------------------------------------------
    // Pure key names for all texture-related properties
    // ---------------------------------------------------------------------------
    /// Texture file
    TextureFile(String),

    /// Texture uv index
    TextureUvwsrc(u32),

    /// Texture operation
    TextureOp(AiTextureOp),

    /// Texture mapping uv
    TextureMappingUV(AiTextureMapping),

    /// Texture blend
    TextureBlend(AiBlendMode),

    /// Texture blend factor
    TextureBlendFactor(AiReal),

    /// Texture mapping mode for u axis
    TextureMappingModeU(AiTextureMapMode),

    /// Texture mapping mode for v axis
    TextureMappingModeV(AiTextureMapMode),

    /// Texture mapping mode for w axis
    TextureMappingModeW(AiTextureMapMode),

    /// Texture flags
    TextureFlags(AiTextureFlags),

    /// Texture map axis
    TextureMapAxis(AiVec3),

    /// Texture uv transform
    TextureUvTransform(AiUVTransform),

    // Texture Property
    /// Texture diffuse
    TextureDiffuse(String),

    /// Texture specular
    TextureSpecular(String),

    /// Texture ambient
    TextureAmbient(String),

    /// Texture emissive
    TextureEmissive(String),

    /// Texture normals
    TextureNormals(String),

    /// Texture height
    TextureHeight(String),

    /// Texture shininess
    TextureShininess(String),

    /// Texture opacity
    TextureOpacity(String),

    /// Texture displacement
    TextureDisplacement(String),

    /// Texture lightmap
    TextureLightmap(String),

    /// Texture reflection
    TextureReflection(String),

    /// Texture displacement
    UvTransform(AiUVTransform),

    /// Custom property
    ///
    /// This is a property that is not part of the standard material system.
    /// It is used to store custom properties that are not part of the standard material system.
    Custom((String, AiBasicProperty)),

    /// PlaceHolder for any property
    WildCard(()),
}

/// Basic property
///
/// This is a property that is used to store a single value of a specific type.
/// It is used to store a single value of a specific type that is not part of the standard material
/// system.
#[derive(Clone, Debug)]
pub enum AiBasicProperty {
    /// 32 bit integer
    Int(i32),
    /// 32 bit float
    Float(AiReal),
    /// vector of 3 32/64 bit floats
    Vec3(AiVec3),
    /// vector of 4 32/64 bit floats
    Vec4(AiVec4),
    /// String
    String(String),
    /// Buffer of data
    Buffer(AiBasicBufferProperty),
}

/// Buffer property
///
/// This is a property that is used to store a buffer of data and are all the same type.
/// It is used to store a buffer of data that is not part of the standard material system.
#[derive(Clone, Debug)]
pub enum AiBasicBufferProperty {
    /// Buffer of normal data
    NormalBuffer(Vec<u8>),
    /// Buffer of integer data
    IntBuffer(Vec<i32>),
    /// Buffer of float data
    FloatBuffer(Vec<AiReal>),
    /// Buffer of vector of 3 32/64 bit floats
    Vec3Buffer(Vec<AiVec3>),
    /// Buffer of vector of 4 32/64 bit floats
    Vec4Buffer(Vec<AiVec4>),
    /// Buffer of string data
    StringBuffer(Vec<String>),
}

// Could be a proc macro on enum itself but not sure if it is worth it.
macro_rules! match_fn_defines {
    ($($variant:ident, String, $fn_name:ident)*) => {
        $(
            #[inline(always)]
            pub const fn $fn_name(&self) -> Option<&str> {
                if let AiProperty::$variant(v) = self {
                    Some(v.as_str())
                } else {
                    None
                }
            }
        )*
    };
    ($($variant:ident, $type:ty, $fn_name:ident)*) => {
        $(
            #[inline(always)]
            pub const fn $fn_name(&self) -> Option<&$type> {
                if let AiProperty::$variant(v) = self {
                    Some(v)
                } else {
                    None
                }
            }
        )*
    };
}

// All the matching fns for get_property
#[allow(unused, missing_docs)]
impl AiProperty {
    match_fn_defines!(
        // f32 properties
        MaterialOpacity, AiReal, is_material_opacity_property
        MaterialTransparencyFactor, AiReal, is_material_transparency_factor_property
        MaterialBumpScaling, AiReal, is_material_bump_scaling_property
        MaterialShininess, AiReal, is_material_shininess_property
        MaterialReflectivity, AiReal, is_material_reflectivity_property
        MaterialShininessStrength, AiReal, is_material_shininess_strength_property
        MaterialRefracti, AiReal, is_material_refracti_property
        MaterialMetallicFactor, AiReal, is_material_metallic_factor_property
        MaterialRoughnessFactor, AiReal, is_material_roughness_factor_property
        MaterialAnisotropyFactor, AiReal, is_material_anisotropy_factor_property
        MaterialSpecularFactor, AiReal, is_material_specular_factor_property
        MaterialGlossinessFactor, AiReal, is_material_glossiness_factor_property
        MaterialSheenRoughnessFactor, AiReal, is_material_sheen_roughness_factor_property
        MaterialClearcoatFactor, AiReal, is_material_clearcoat_factor_property
        MaterialClearcoatRoughnessFactor, AiReal, is_material_clearcoat_roughness_factor_property
        MaterialTransmissionFactor, AiReal, is_material_transmission_factor_property
        MaterialVolumeThicknessFactor, AiReal, is_material_volume_thickness_factor_property
        MaterialVolumeAttenuationDistance, AiReal, is_material_volume_attenuation_distance_property
        MaterialEmissiveIntensity, AiReal, is_material_emissive_intensity_property
        MaterialAnisotropyRotation, AiReal, is_material_anisotropy_rotation_property
        // Vector properties
        MaterialColorSpecular, AiVec3, is_material_color_specular_property
        MaterialColorEmissive, AiVec3, is_material_color_emissive_property
        // Bool properties
        MaterialIsTwoSided, bool, is_material_is_two_sided_property
        MaterialUseColorMap, bool, is_material_use_color_map_property
        MaterialUseRoughnessMap, bool, is_material_use_roughness_map_property
        MaterialUseEmissiveMap, bool, is_material_use_emissive_map_property
        MaterialUseAmbientOcclusionMap, bool, is_material_use_ambient_occlusion_map_property
        // Enum properties
        MaterialShadingMode, AiShadingMode, is_material_shading_mode_property
        MaterialBlendMode, AiBlendMode, is_material_blend_mode_property
        MaterialAnisotropyTexture, AiTextureType, is_material_anisotropy_texture_property
        // Color properties
        MaterialColorDiffuse, AiColorDiffuseProperty, is_material_color_diffuse_property
        MaterialColorAmbient, AiColorDiffuseProperty, is_material_color_ambient_property
        MaterialColorTransparent, AiColorDiffuseProperty, is_material_color_transparent_property
        MaterialColorReflective, AiColorDiffuseProperty, is_material_color_reflective_property
        MaterialBaseColor, AiColorDiffuseProperty, is_material_base_color_property
        ColorSheenColorFactor, AiColorDiffuseProperty, is_color_sheen_color_factor_property
        MaterialVolumeAttenuationColor, AiColorDiffuseProperty, is_material_volume_attenuation_color_property
        // u32 properties
        TextureOp, AiTextureOp, is_texture_op_property
        TextureMappingUV, AiTextureMapping, is_texture_mapping_uv_property
        TextureBlend, AiBlendMode, is_texture_blend_property
        TextureMappingModeU, AiTextureMapMode, is_texture_mapping_mode_u_property
        TextureMappingModeV, AiTextureMapMode, is_texture_mapping_mode_v_property
        TextureMappingModeW, AiTextureMapMode, is_texture_mapping_mode_w_property
        TextureFlags, AiTextureFlags, is_texture_flags_property
        TextureMapAxis, AiVec3, is_texture_axis_property
        TextureUvwsrc, u32, is_texture_uvwsrc_property
        TextureBlendFactor, AiReal, is_texture_blend_factor_property
        // Transform properties
        TextureUvTransform, AiUVTransform, is_texture_uv_transform_property
        UvTransform, AiUVTransform, is_uv_transform_property
        // Custom properties
        Custom, (String, AiBasicProperty), is_custom_property

        // Wildcard properties
        WildCard, (), is_wildcard_property
    );

    // Make these fields return &str
    match_fn_defines!(
        // String properties
        MaterialName, String, is_material_name_property
        MaterialGlobalBackgroundImage, String, is_material_global_background_image_property
        MaterialGlobalShaderLang, String, is_material_global_shader_lang_property
        MaterialShaderVertex, String, is_material_shader_vertex_property
        MaterialShaderFragment, String, is_material_shader_fragment_property
        MaterialShaderGeometry, String, is_material_shader_geometry_property
        MaterialShaderTessellation, String, is_material_shader_tessellation_property
        MaterialShaderPrimitive, String, is_material_shader_primitive_property
        MaterialShaderCompute, String, is_material_shader_compute_property
        TextureFile, String, is_texture_file_property
        TextureDiffuse, String, is_texture_diffuse_property
        TextureSpecular, String, is_texture_specular_property
        TextureAmbient, String, is_texture_ambient_property
        TextureEmissive, String, is_texture_emissive_property
        TextureNormals, String, is_texture_normals_property
        TextureHeight, String, is_texture_height_property
        TextureShininess, String, is_texture_shininess_property
        TextureOpacity, String, is_texture_opacity_property
        TextureDisplacement, String, is_texture_displacement_property
        TextureLightmap, String, is_texture_lightmap_property
        TextureReflection, String, is_texture_reflection_property
    );
}

impl AiProperty {
    pub const fn get_field_name(&self) -> &'static str {
        match self {
            AiProperty::MaterialName(_) => "AiProperty::MaterialName",
            AiProperty::MaterialIsTwoSided(_) => "AiProperty::MaterialIsTwoSided",
            AiProperty::MaterialShadingMode(_) => "AiProperty::MaterialShadingMode",
            AiProperty::MaterialBlendMode(_) => "AiProperty::MaterialBlendMode",
            AiProperty::MaterialOpacity(_) => "AiProperty::MaterialOpacity",
            AiProperty::MaterialTransparencyFactor(_) => "AiProperty::MaterialTransparencyFactor",
            AiProperty::MaterialBumpScaling(_) => "AiProperty::MaterialBumpScaling",
            AiProperty::MaterialShininess(_) => "AiProperty::MaterialShininess",
            AiProperty::MaterialReflectivity(_) => "AiProperty::MaterialReflectivity",
            AiProperty::MaterialShininessStrength(_) => "AiProperty::MaterialShininessStrength",
            AiProperty::MaterialRefracti(_) => "AiProperty::MaterialRefracti",
            AiProperty::MaterialColorDiffuse(_) => "AiProperty::MaterialColorDiffuse",
            AiProperty::MaterialColorAmbient(_) => "AiProperty::MaterialColorAmbient",
            AiProperty::MaterialColorSpecular(_) => "AiProperty::MaterialColorSpecular",
            AiProperty::MaterialColorEmissive(_) => "AiProperty::MaterialColorEmissive",
            AiProperty::MaterialColorTransparent(_) => "AiProperty::MaterialColorTransparent",
            AiProperty::MaterialColorReflective(_) => "AiProperty::MaterialColorReflective",
            AiProperty::MaterialGlobalBackgroundImage(_) => {
                "AiProperty::MaterialGlobalBackgroundImage"
            }
            AiProperty::MaterialGlobalShaderLang(_) => "AiProperty::MaterialGlobalShaderLang",
            AiProperty::MaterialShaderVertex(_) => "AiProperty::MaterialShaderVertex",
            AiProperty::MaterialShaderFragment(_) => "AiProperty::MaterialShaderFragment",
            AiProperty::MaterialShaderGeometry(_) => "AiProperty::MaterialShaderGeometry",
            AiProperty::MaterialShaderTessellation(_) => "AiProperty::MaterialShaderTessellation",
            AiProperty::MaterialShaderPrimitive(_) => "AiProperty::MaterialShaderPrimitive",
            AiProperty::MaterialShaderCompute(_) => "AiProperty::MaterialShaderCompute",
            AiProperty::MaterialUseColorMap(_) => "AiProperty::MaterialUseColorMap",
            AiProperty::MaterialBaseColor(_) => "AiProperty::MaterialBaseColor",
            AiProperty::MaterialMetallicFactor(_) => "AiProperty::MaterialMetallicFactor",
            AiProperty::MaterialUseRoughnessMap(_) => "AiProperty::MaterialUseRoughnessMap",
            AiProperty::MaterialRoughnessFactor(_) => "AiProperty::MaterialRoughnessFactor",
            AiProperty::MaterialAnisotropyFactor(_) => "AiProperty::MaterialAnisotropyFactor",
            AiProperty::MaterialSpecularFactor(_) => "AiProperty::MaterialSpecularFactor",
            AiProperty::MaterialGlossinessFactor(_) => "AiProperty::MaterialGlossinessFactor",
            AiProperty::ColorSheenColorFactor(_) => "AiProperty::ColorSheenColorFactor",
            AiProperty::MaterialSheenRoughnessFactor(_) => {
                "AiProperty::MaterialSheenRoughnessFactor"
            }
            AiProperty::MaterialClearcoatFactor(_) => "AiProperty::MaterialClearcoatFactor",
            AiProperty::MaterialClearcoatRoughnessFactor(_) => {
                "AiProperty::MaterialClearcoatRoughnessFactor"
            }
            AiProperty::MaterialTransmissionFactor(_) => "AiProperty::MaterialTransmissionFactor",
            AiProperty::MaterialVolumeThicknessFactor(_) => {
                "AiProperty::MaterialVolumeThicknessFactor"
            }
            AiProperty::MaterialVolumeAttenuationDistance(_) => {
                "AiProperty::MaterialVolumeAttenuationDistance"
            }
            AiProperty::MaterialVolumeAttenuationColor(_) => {
                "AiProperty::MaterialVolumeAttenuationColor"
            }
            AiProperty::MaterialUseEmissiveMap(_) => "AiProperty::MaterialUseEmissiveMap",
            AiProperty::MaterialEmissiveIntensity(_) => "AiProperty::MaterialEmissiveIntensity",
            AiProperty::MaterialUseAmbientOcclusionMap(_) => {
                "AiProperty::MaterialUseAmbientOcclusionMap"
            }
            AiProperty::MaterialAnisotropyRotation(_) => "AiProperty::MaterialAnisotropyRotation",
            AiProperty::MaterialAnisotropyTexture(_) => "AiProperty::MaterialAnisotropyTexture",
            AiProperty::TextureFile(_) => "AiProperty::TextureFile",
            AiProperty::TextureUvwsrc(_) => "AiProperty::TextureUvwsrc",
            AiProperty::TextureOp(_) => "AiProperty::TextureOp",
            AiProperty::TextureMappingUV(_) => "AiProperty::TextureMappingUV",
            AiProperty::TextureBlend(_) => "AiProperty::TextureBlend",
            AiProperty::TextureBlendFactor(_) => "AiProperty::TextureBlendFactor",
            AiProperty::TextureMappingModeU(_) => "AiProperty::TextureMappingModeU",
            AiProperty::TextureMappingModeV(_) => "AiProperty::TextureMappingModeV",
            AiProperty::TextureMappingModeW(_) => "AiProperty::TextureMappingModeW",
            AiProperty::TextureFlags(_) => "AiProperty::TextureFlags",
            AiProperty::TextureMapAxis(_) => "AiProperty::TextureMapAxis",
            AiProperty::TextureUvTransform(_) => "AiProperty::TextureUvTransform",
            AiProperty::TextureDiffuse(_) => "AiProperty::TextureDiffuse",
            AiProperty::TextureSpecular(_) => "AiProperty::TextureSpecular",
            AiProperty::TextureAmbient(_) => "AiProperty::TextureAmbient",
            AiProperty::TextureEmissive(_) => "AiProperty::TextureEmissive",
            AiProperty::TextureNormals(_) => "AiProperty::TextureNormals",
            AiProperty::TextureHeight(_) => "AiProperty::TextureHeight",
            AiProperty::TextureShininess(_) => "AiProperty::TextureShininess",
            AiProperty::TextureOpacity(_) => "AiProperty::TextureOpacity",
            AiProperty::TextureDisplacement(_) => "AiProperty::TextureDisplacement",
            AiProperty::TextureLightmap(_) => "AiProperty::TextureLightmap",
            AiProperty::TextureReflection(_) => "AiProperty::TextureReflection",
            AiProperty::UvTransform(_) => "AiProperty::UvTransform",
            AiProperty::Custom(_) => "AiProperty::Custom",
            AiProperty::WildCard(_) => "AiProperty::WildCard",
        }
    }
    pub const fn get_inner_string(&self) -> Option<&String> {
        match self {
            AiProperty::MaterialName(s)
            | AiProperty::MaterialGlobalBackgroundImage(s)
            | AiProperty::MaterialGlobalShaderLang(s)
            | AiProperty::MaterialShaderVertex(s)
            | AiProperty::MaterialShaderFragment(s)
            | AiProperty::MaterialShaderGeometry(s)
            | AiProperty::MaterialShaderTessellation(s)
            | AiProperty::MaterialShaderPrimitive(s)
            | AiProperty::MaterialShaderCompute(s)
            | AiProperty::TextureFile(s)
            | AiProperty::TextureDiffuse(s)
            | AiProperty::TextureSpecular(s)
            | AiProperty::TextureAmbient(s)
            | AiProperty::TextureEmissive(s)
            | AiProperty::TextureNormals(s)
            | AiProperty::TextureHeight(s)
            | AiProperty::TextureShininess(s)
            | AiProperty::TextureOpacity(s)
            | AiProperty::TextureDisplacement(s)
            | AiProperty::TextureLightmap(s)
            | AiProperty::TextureReflection(s)
            | AiProperty::Custom((_, AiBasicProperty::String(s))) => Some(s),
            _ => None,
        }
    }
}

impl Default for AiProperty {
    fn default() -> Self {
        Self::WildCard(())
    }
}

#[derive(Clone, Debug, Default)]
/// A property of a material
pub struct AiMaterialProperty {
    /// The index of the property
    pub index: u32,
    /// The type of the property
    pub r#type: AiTextureType,
    /// The property
    pub property: AiProperty,
}
