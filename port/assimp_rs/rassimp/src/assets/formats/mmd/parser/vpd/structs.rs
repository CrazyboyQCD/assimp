use core::mem;

use glam::{Vec3, Vec4};
use lexical_parse_float::parse::ParseFloat;

use crate::{
    assets::formats::mmd::parser::{
        self,
        error::{MMD_COMMON_ERROR_OUT_OF_MEMORY, MMDParseCommonError, MMDParseError},
        vpd::{VPDParser, VPDRead, error::VpdParseError},
    },
    io::utils::{
        atof::fast_atof32,
        encoding::{convert_to_utf8, decode_shift_jis},
    },
};

#[derive(Clone, Debug, Default)]
pub struct VpdBone {
    pub bone_name: String,
    pub translate: Vec3,
    pub quaternion: Vec4,
}

// Bone0{右腕捩
//   0.000000,0.000000,0.000000;				// trans x,y,z
//   -0.000000,-0.000000,0.000000,1.000000;		// Quaternion x,y,z,w
// }
impl VPDRead for VpdBone {
    fn read<'source, 'other>(
        parser: &mut VPDParser<'source, 'other>,
    ) -> Result<Self, MMDParseError> {
        // read name
        // foarmt: Bone{index}{{name}
        let bone_name_prefix = parser.next_token();
        if bone_name_prefix.is_empty() {
            Err(VpdParseError::UnexpectedEnd("bone header"))?;
        }
        if !bone_name_prefix.starts_with(b"Bone") {
            Err(VpdParseError::InvalidBonePrefix(bone_name_prefix.to_vec()))?;
        }

        if let Err(token) = parser.check_for_separator(b'{') {
            Err(VpdParseError::InvalidBoneBracket(token.to_vec()))?;
        }
        let bone_name = {
            let token = parser.next_token();
            if token.is_empty() {
                Err(VpdParseError::UnexpectedEnd("bone name"))?;
            }
            if let Ok(name) = str::from_utf8(token) {
                name.to_owned()
            } else if let Ok(name) = decode_shift_jis(token) {
                name
            } else {
                return Err(VpdParseError::UnknownEncodedString(token.to_vec()))?;
            }
        };

        // read translate
        // foarmt: {x},{y},{z};
        let x = parser
            .read_f32()
            .ok_or(VpdParseError::UnexpectedEnd("bone translate x"))?;
        if let Err(token) = parser.check_for_comma() {
            Err(VpdParseError::InvalidBoneTranslateSeparator(token.to_vec()))?;
        }
        let y = parser
            .read_f32()
            .ok_or(VpdParseError::UnexpectedEnd("bone translate y"))?;
        if let Err(token) = parser.check_for_comma() {
            Err(VpdParseError::InvalidBoneTranslateSeparator(token.to_vec()))?;
        }
        let z = parser
            .read_f32()
            .ok_or(VpdParseError::UnexpectedEnd("bone translate z"))?;

        if let Err(token) = parser.check_for_separator(b';') {
            Err(VpdParseError::InvalidBoneTranslateEnd(token.to_vec()))?;
        }

        let translate = Vec3::new(x, y, z);

        // read quaternion
        // foarmt: {x},{y},{z},{w};
        let x = parser
            .read_f32()
            .ok_or(VpdParseError::UnexpectedEnd("bone quaternion x"))?;
        if let Err(token) = parser.check_for_comma() {
            Err(VpdParseError::InvalidBoneTranslateSeparator(token.to_vec()))?;
        }
        let y = parser
            .read_f32()
            .ok_or(VpdParseError::UnexpectedEnd("bone quaternion y"))?;
        if let Err(token) = parser.check_for_comma() {
            Err(VpdParseError::InvalidBoneQuaternionSeparator(
                token.to_vec(),
            ))?;
        }
        let z = parser
            .read_f32()
            .ok_or(VpdParseError::UnexpectedEnd("bone quaternion z"))?;
        if let Err(token) = parser.check_for_comma() {
            Err(VpdParseError::InvalidBoneQuaternionSeparator(
                token.to_vec(),
            ))?;
        }
        let w = parser
            .read_f32()
            .ok_or(VpdParseError::UnexpectedEnd("bone quaternion w"))?;
        if let Err(token) = parser.check_for_separator(b';') {
            Err(VpdParseError::InvalidBoneQuaternionEnd(token.to_vec()))?;
        }

        let quaternion = Vec4::new(x, y, z, w);

        if let Err(token) = parser.check_for_separator(b'}') {
            Err(VpdParseError::InvalidBoneEnd(token.to_vec()))?;
        }

        Ok(VpdBone {
            bone_name,
            translate,
            quaternion,
        })
    }
}

#[derive(Clone, Debug, Default)]
pub struct VPDMorph {
    pub morph_name: String,
    pub weight: f32,
}

// Morph0{う
//   0.200000; // weight
// }
//
impl VPDRead for VPDMorph {
    fn read<'source, 'other>(
        parser: &mut VPDParser<'source, 'other>,
    ) -> Result<Self, MMDParseError> {
        // read name
        // foarmt: Morph{index}{{name}
        let morph_name_prefix = parser.next_token();
        if morph_name_prefix.is_empty() {
            Err(VpdParseError::UnexpectedEnd("morph header"))?;
        }
        if !morph_name_prefix.starts_with(b"Morph") {
            Err(VpdParseError::InvalidMorphPrefix(
                morph_name_prefix.to_vec(),
            ))?;
        }
        if let Err(token) = parser.check_for_separator(b'{') {
            Err(VpdParseError::InvalidMorphBracket(token.to_vec()))?;
        }
        let morph_name = {
            let token = parser.next_token();
            if token.is_empty() {
                Err(VpdParseError::UnexpectedEnd("morph name"))?;
            }
            if let Ok(name) = str::from_utf8(token) {
                name.to_owned()
            } else if let Ok(name) = decode_shift_jis(token) {
                name
            } else {
                return Err(VpdParseError::UnknownEncodedString(token.to_vec()))?;
            }
        };

        // read weight
        // foarmt: {weight};
        let weight = parser
            .read_f32()
            .ok_or(VpdParseError::UnexpectedEnd("morph weight"))?;
        if let Err(token) = parser.check_for_separator(b';') {
            Err(VpdParseError::InvalidMorphWeightSeparator(token.to_vec()))?;
        }

        if let Err(token) = parser.check_for_separator(b'}') {
            Err(VpdParseError::InvalidMorphEnd(token.to_vec()))?;
        }
        Ok(VPDMorph { morph_name, weight })
    }
}

/// A VPD file is a plain text file saved with Shift_JIS encoding.
///
/// Below is an example of a VPD file containing Bone and Morph keyframes.
/// ```text
/// Vocaloid Pose Data file
/// YYB式初音ミク_10th_v1.02.osm;        // 親ファイル名
/// 4;                // 総ポーズボーン数
///
/// Bone0{右腕捩
///   0.000000,0.000000,0.000000;                // trans x,y,z
///   -0.000000,-0.000000,0.000000,1.000000;        // Quaternion x,y,z,w
/// }
///
/// Bone1{右ひじ
///   0.000000,0.000000,0.000000;                // trans x,y,z
///   0.176789,-0.061290,0.747712,0.637114;        // Quaternion x,y,z,w
/// }
///
/// Bone2{右手捩
///   0.000000,0.000000,0.000000;                // trans x,y,z
///   -0.000000,-0.000000,-0.000000,1.000000;        // Quaternion x,y,z,w
/// }
///
/// Bone3{右手首
///   0.000000,0.000000,0.000000;                // trans x,y,z
///   -0.574374,-0.615622,0.113957,0.527368;        // Quaternion x,y,z,w
/// }
///
/// Morph0{う
///   0.200000;
/// }
///
/// Morph1{え
///   0;
/// }
/// ```
///
/// The `4`; on the fourth line indicates the number of Bone states. MMD reads exactly that many
/// Bone entries and ignores any subsequent information.
///
/// Therefore, the MMM extension that includes Morph states always places the Morph state
/// information after the Bone state information.
#[derive(Clone, Debug, Default)]
pub struct VPDFile {
    pub model_name: String,
    pub bones: Vec<VpdBone>,
    pub morphs: Vec<VPDMorph>,
}

impl VPDRead for VPDFile {
    fn read<'source, 'other>(
        parser: &mut VPDParser<'source, 'other>,
    ) -> Result<Self, MMDParseError> {
        if parser.next_token() != b"Vocaloid Pose Data file" {
            Err(VpdParseError::InvalidMagic(parser.next_token().to_vec()))?;
        }

        let model_name = {
            let token = parser.next_token();
            if token.is_empty() {
                Err(VpdParseError::UnexpectedEnd("model name"))?;
            }
            if let Ok(name) = str::from_utf8(token) {
                name.to_owned()
            } else if let Ok(name) = decode_shift_jis(token) {
                name
            } else {
                return Err(VpdParseError::UnknownEncodedString(token.to_vec()))?;
            }
        };

        // read bones
        let mut bones = Vec::new();
        if let Some(bone_count) = parser.read_unsigned_integer() {
            bones
                .try_reserve(bone_count as usize)
                .map_err(|_| MMD_COMMON_ERROR_OUT_OF_MEMORY)?;

            for _ in 0..bone_count {
                bones.push(VpdBone::read(parser)?);
            }
        }

        // read morphs
        let mut morphs = Vec::new();
        if let Some(morph_count) = parser.read_unsigned_integer() {
            morphs
                .try_reserve(morph_count as usize)
                .map_err(|_| MMD_COMMON_ERROR_OUT_OF_MEMORY)?;

            for _ in 0..morph_count {
                morphs.push(VPDMorph::read(parser)?);
            }
        }

        Ok(VPDFile {
            model_name,
            bones,
            morphs,
        })
    }
}
