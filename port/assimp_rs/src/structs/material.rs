use std::borrow::Cow;

use crate::{
    AiReal,
    utils::float_precision::{Vec2, Vec3, Vec4},
};

pub const AI_MATKEY_NAME: &str = "?mat.name";
pub const AI_MATKEY_TWOSIDED: &str = "$mat.twosided";
pub const AI_MATKEY_SHADING_MODEL: &str = "$mat.shadingm";
pub const AI_MATKEY_ENABLE_WIREFRAME: &str = "$mat.wireframe";
pub const AI_MATKEY_BLEND_FUNC: &str = "$mat.blend";
pub const AI_MATKEY_OPACITY: &str = "$mat.opacity";
pub const AI_MATKEY_TRANSPARENCYFACTOR: &str = "$mat.transparencyfactor";
pub const AI_MATKEY_BUMPSCALING: &str = "$mat.bumpscaling";
pub const AI_MATKEY_SHININESS: &str = "$mat.shininess";
pub const AI_MATKEY_REFLECTIVITY: &str = "$mat.reflectivity";
pub const AI_MATKEY_SHININESS_STRENGTH: &str = "$mat.shinpercent";
pub const AI_MATKEY_REFRACTI: &str = "$mat.refracti";
pub const AI_MATKEY_COLOR_DIFFUSE: &str = "$clr.diffuse";
pub const AI_MATKEY_COLOR_AMBIENT: &str = "$clr.ambient";
pub const AI_MATKEY_COLOR_SPECULAR: &str = "$clr.specular";
pub const AI_MATKEY_COLOR_EMISSIVE: &str = "$clr.emissive";
pub const AI_MATKEY_COLOR_TRANSPARENT: &str = "$clr.transparent";
pub const AI_MATKEY_COLOR_REFLECTIVE: &str = "$clr.reflective";
pub const AI_MATKEY_GLOBAL_BACKGROUND_IMAGE: &str = "?bg.global";
pub const AI_MATKEY_GLOBAL_SHADERLANG: &str = "?sh.lang";
pub const AI_MATKEY_SHADER_VERTEX: &str = "?sh.vs";
pub const AI_MATKEY_SHADER_FRAGMENT: &str = "?sh.fs";
pub const AI_MATKEY_SHADER_GEO: &str = "?sh.gs";
pub const AI_MATKEY_SHADER_TESSELATION: &str = "?sh.ts";
pub const AI_MATKEY_SHADER_PRIMITIVE: &str = "?sh.ps";
pub const AI_MATKEY_SHADER_COMPUTE: &str = "?sh.cs";

// ---------------------------------------------------------------------------
// Pure key names for all texture-related properties
pub const AI_MATKEY_TEXTURE: &str = "$tex.file";
pub const AI_MATKEY_UVWSRC: &str = "$tex.uvwsrc";
pub const AI_MATKEY_TEXOP: &str = "$tex.op";
pub const AI_MATKEY_MAPPING: &str = "$tex.mapping";
pub const AI_MATKEY_TEXBLEND: &str = "$tex.blend";
pub const AI_MATKEY_MAPPINGMODE_U: &str = "$tex.mapmodeu";
pub const AI_MATKEY_MAPPINGMODE_V: &str = "$tex.mapmodev";
pub const AI_MATKEY_TEXMAP_AXIS: &str = "$tex.mapaxis";
pub const AI_MATKEY_UVTRANSFORM: &str = "$tex.uvtrafo";
pub const AI_MATKEY_TEXFLAGS: &str = "$tex.flags";

#[derive(Clone, Debug)]
pub enum AiColorDiffuseProperty {
    Color3D(Vec3),
    Color4D(Vec4),
}

impl From<Vec3> for AiColorDiffuseProperty {
    fn from(value: Vec3) -> Self {
        Self::Color3D(value)
    }
}
impl From<Vec4> for AiColorDiffuseProperty {
    fn from(value: Vec4) -> Self {
        Self::Color4D(value)
    }
}

#[derive(Clone, Debug)]
pub enum AiProperty {
    /// Array of single-precision (32 Bit) floats
    ///
    ///  It is possible to use aiGetMaterialInteger[Array]() (or the C++-API
    ///  aiMaterial::Get()) to query properties stored in floating-point format.
    ///  The material system performs the type conversion automatically.
    Floats(Vec<AiReal>),

    Float(AiReal),

    Vec3(Vec3),

    Vec4(Vec4),

    ShadingModel(AiShadingMode),

    ColorEmissive(Vec3),

    ColorSpecular(Vec3),

    ColorDiffuse(AiColorDiffuseProperty),

    Shiness(AiReal),

    /// The material property is an aiString.
    ///
    ///  Arrays of strings aren't possible, aiGetMaterialString() (or the
    ///  C++-API aiMaterial::Get()) *must* be used to query a string property.
    String(String),
    Name(String),
    MaterialName(String),
    TextureDiffuse(String),
    TextureSpecular(String),
    TextureAmbient(String),
    TextureEmissive(String),
    TextureNormals(String),
    TextureHeight(String),
    TextureShininess(String),
    TextureOpacity(String),
    TextureDisplacement(String),
    TextureLightmap(String),
    TextureReflection(String),

    UvTransform(AiUVTransform),

    /// Array of (32 Bit) integers
    ///
    ///  It is possible to use aiGetMaterialFloat[Array]() (or the C++-API
    ///  aiMaterial::Get()) to query properties stored in integer format.
    ///  The material system performs the type conversion automatically.
    Integers(Vec<i32>),

    Integer(i32),

    /// Simple binary buffer, content undefined. Not convertible to anything.
    Buffer(Vec<u8>),

    WildCard(()),
}

macro_rules! match_fn_defines {
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
// All the maching fns
#[allow(unused)]
impl AiProperty {
    match_fn_defines!(
        // float properties
        Floats, Vec<AiReal>, is_floats_property
        Float, AiReal, is_float_property
        // Vector properties
        Vec3, Vec3, is_vec3_property
        Vec4, Vec4, is_vec4_property
        // i32 properties
        Integers, Vec<i32>, is_integers_property
        Integer, i32, is_integer_property
        // Buffer properties
        Buffer, Vec<u8>, is_buffer_property
        // String properties
        String, String, is_string_property
        Name, String, is_name_property
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
        // Wildcard properties
        WildCard, (), is_wildcard_property
    );
}

impl Default for AiProperty {
    fn default() -> Self {
        Self::WildCard(())
    }
}

pub enum AiStringPropertyType {
    Name,
    MaterialName,
    TextureDiffuse,
    TextureSpecular,
    TextureAmbient,
    TextureEmissive,
    TextureNormals,
    TextureHeight,
    TextureShininess,
    TextureOpacity,
    TextureDisplacement,
    TextureLightmap,
    TextureReflection,
}

#[derive(Default, Clone, Debug)]
pub struct AiMaterialProperty {
    pub key: Cow<'static, str>,
    pub index: u32,
    pub property: AiProperty,
}

#[derive(Default, Clone, Debug)]
pub struct AiMaterial {
    pub properties: Vec<AiMaterialProperty>,
}

impl AiMaterial {
    fn inner_get_property<V: ?Sized>(
        &self,
        key: &str,
        index: u32,
        type_match_fn: impl Fn(&AiProperty) -> Option<&V>,
    ) -> Option<&V> {
        for p in self.properties.iter() {
            if p.key == key && (index == u32::MAX || p.index == index) {
                if let Some(v) = type_match_fn(&p.property) {
                    return Some(v);
                }
            }
        }
        None
    }

    #[allow(unused)]
    fn inner_get_property_by_index<V: ?Sized>(
        &self,
        variant_match_fn: impl Fn(&AiProperty) -> Option<&V>,
        index: u32,
    ) -> Option<&V> {
        for p in self.properties.iter() {
            if p.index == index {
                if let Some(v) = variant_match_fn(&p.property) {
                    return Some(v);
                }
            }
        }
        None
    }

    fn inner_add_property<K: Into<Cow<'static, str>>>(
        &mut self,
        key: K,
        property: AiProperty,
        index: u32,
    ) {
        let key = key.into();

        self.properties.push(AiMaterialProperty {
            key,
            index,
            property,
        });
    }

    pub fn add_property_v2(&mut self, property: AiProperty, index: u32) {
        self.properties.push(AiMaterialProperty {
            key: "".into(),
            index,
            property,
        });
    }

    pub fn add_string_property<K: Into<Cow<'static, str>>>(
        &mut self,
        key: K,
        value: String,
        index: u32,
        string_type: AiStringPropertyType,
    ) {
        macro_rules! add_string_property_impl {
            ($($string_type:ident, $variant:ident)*) => {
                match string_type {
                    $(
                        AiStringPropertyType::$string_type => self.inner_add_property(key, AiProperty::$variant(value), index),
                    )*
                }
            };
        }
        add_string_property_impl!(
            Name, Name
            MaterialName, MaterialName
            TextureHeight, TextureHeight
            TextureDiffuse, TextureDiffuse
            TextureSpecular, TextureSpecular
            TextureAmbient, TextureAmbient
            TextureEmissive, TextureEmissive
            TextureNormals, TextureNormals
            TextureShininess, TextureShininess
            TextureOpacity, TextureOpacity
            TextureDisplacement, TextureDisplacement
            TextureLightmap, TextureLightmap
            TextureReflection, TextureReflection
        )
    }

    pub fn get_string_property(
        &self,
        key: &str,
        index: u32,
        string_type: AiStringPropertyType,
    ) -> Option<&str> {
        macro_rules! get_string_property_impl {
            ($($string_type:ident, $variant:ident)*) => {
                match string_type {
                    $(
                        AiStringPropertyType::$string_type => self.inner_get_property(key, index, |v| match v {
                            AiProperty::$variant(v) => Some(v.as_str()),
                            _ => None,
                        }),
                    )*
                }
            };
        }
        get_string_property_impl!(
            Name, Name
            MaterialName, MaterialName
            TextureHeight, TextureHeight
            TextureDiffuse, TextureDiffuse
            TextureSpecular, TextureSpecular
            TextureAmbient, TextureAmbient
            TextureEmissive, TextureEmissive
            TextureNormals, TextureNormals
            TextureShininess, TextureShininess
            TextureOpacity, TextureOpacity
            TextureDisplacement, TextureDisplacement
            TextureLightmap, TextureLightmap
            TextureReflection, TextureReflection
        )
    }
}

pub trait AddProperty<V> {
    fn add_property<K: Into<Cow<'static, str>>>(&mut self, key: K, value: V, index: u32);
}

macro_rules! add_property_impls {
    ($($t:ty, $variant:ident)*) => {
        $(
            impl AddProperty<$t> for AiMaterial {
                fn add_property<K: Into<Cow<'static, str>>>(&mut self, key: K, value: $t, index: u32) {
                    self.inner_add_property(key, AiProperty::$variant(value), index);
                }
            }
        )*
    };
}

add_property_impls!(
    AiReal, Float
    Vec3, Vec3
    Vec4, Vec4
    i32, Integer
    Vec<u8>, Buffer
);

pub trait GetProperty<V> {
    fn get_property(&self, key: &str, index: u32) -> Option<&V>;
}

macro_rules! get_property_impls {
    ($($t:ty, $variant:ident)*) => {
        $(
            impl GetProperty<$t> for AiMaterial {
                fn get_property(&self, key: &str, index: u32) -> Option<&$t> {
                    self.inner_get_property(key, index, |v| match v {
                        AiProperty::$variant(v) => Some(v),
                        _ => None,
                    })
                }
            }
        )*
    };
}

get_property_impls!(
    AiReal, Float
    Vec3, Vec3
    Vec4, Vec4
    i32, Integer
    Vec<u8>, Buffer
);

bitflags::bitflags! {
    /// Defines all shading models supported by the library
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
   #[derive(Clone,Copy, Debug)]
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
       const NoShading = 1 << 8;

       const Unlit = Self::NoShading.bits();

       /// Fresnel shading
       const Fresnel = 1 << 9;

       /// Physically-Based Rendering (PBR) shading using
       /// Bidirectional scattering/reflectance distribution function (BSDF/BRDF)
       const PBR_BRDF = 1 << 10;

   }
}

/// @brief Defines how an UV channel is transformed.
///
/// This is just a helper structure for the #AI_MATKEY_UVTRANSFORM key.
/// See its documentation for more details.
///
/// Typically you'll want to build a matrix of this information. However,
/// we keep separate scaling/translation/rotation values to make it
/// easier to process and optimize UV transformations internally.
///
#[derive(Default, Clone, Debug)]
pub struct AiUVTransform {
    /// Translation on the u and v axes.
    ///
    /// The default value is (0|0).
    pub translation: Vec2,

    /// Scaling on the u and v axes.
    ///
    /// The default value is (1|1).
    pub scaling: Vec2,

    /// Rotation - in counter-clockwise direction.
    ///
    /// The rotation angle is specified in radians. The
    /// rotation center is 0.5f|0.5f. The default value
    /// 0.f.
    pub rotation: AiReal,
}
