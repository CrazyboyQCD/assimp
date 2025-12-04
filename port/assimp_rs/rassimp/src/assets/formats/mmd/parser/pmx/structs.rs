/*
Open Asset Import Library (assimp)
----------------------------------------------------------------------

Copyright (c) 2006-2025, assimp team

All rights reserved.

Redistribution and use of this software in source and binary forms,
with or without modification, are permitted provided that the
following conditions are met:

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

----------------------------------------------------------------------
*/

use glam::{Vec2, Vec3, Vec4};

use crate::{
    assets::formats::mmd::parser::{
        error::{MMD_COMMON_ERROR_OUT_OF_MEMORY, MMDParseCommonError, MMDParseError},
        pmx::{PMXParser, PMXRead, PMXReadWithSetting, error::PmxParseError},
    },
    io::reader::error::MappingPartEndOfStreamError,
};

#[derive(Clone, Copy, Debug)]
pub enum PMXIndex {
    Single,
    Double,
    Quadruple,
}

impl TryFrom<u8> for PMXIndex {
    type Error = PmxParseError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(PMXIndex::Single),
            2 => Ok(PMXIndex::Double),
            4 => Ok(PMXIndex::Quadruple),
            other => Err(PmxParseError::InvalidIndexSize(other)),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct PmxSetting {
    pub encoding: u8,
    pub uv: u8,
    pub vertex_index_size: PMXIndex,
    pub texture_index_size: PMXIndex,
    pub material_index_size: PMXIndex,
    pub bone_index_size: PMXIndex,
    pub morph_index_size: PMXIndex,
    pub rigidbody_index_size: PMXIndex,
}

impl PMXRead for PmxSetting {
    fn read(parser: &mut PMXParser<'_>) -> Result<Self, MMDParseError> {
        let count = parser
            .read_u8()
            .map_err(|e| PmxParseError::map_end_of_stream_error(e, "setting count"))?;
        if count != 8 {
            Err(PmxParseError::InvalidSettingCount(count))?;
        }
        let encoding = parser
            .read_u8()
            .map_err(|e| PmxParseError::map_end_of_stream_error(e, "setting encoding"))?;
        let uv = parser
            .read_u8()
            .map_err(|e| PmxParseError::map_end_of_stream_error(e, "setting uv"))?;
        let vertex_index_size = parser
            .read_index_size()
            .map_err(|e| PmxParseError::map_end_of_stream_error(e, "setting vertex index size"))?;
        let texture_index_size = parser
            .read_index_size()
            .map_err(|e| PmxParseError::map_end_of_stream_error(e, "setting texture index size"))?;
        let material_index_size = parser.read_index_size().map_err(|e| {
            PmxParseError::map_end_of_stream_error(e, "setting material index size")
        })?;
        let bone_index_size = parser
            .read_index_size()
            .map_err(|e| PmxParseError::map_end_of_stream_error(e, "setting bone index size"))?;
        let morph_index_size = parser
            .read_index_size()
            .map_err(|e| PmxParseError::map_end_of_stream_error(e, "setting morph index size"))?;
        let rigidbody_index_size = parser.read_index_size().map_err(|e| {
            PmxParseError::map_end_of_stream_error(e, "setting rigidbody index size")
        })?;
        Ok(PmxSetting {
            encoding,
            uv,
            vertex_index_size,
            texture_index_size,
            material_index_size,
            bone_index_size,
            morph_index_size,
            rigidbody_index_size,
        })
    }
}

#[derive(Clone, Debug, Default)]
pub struct PmxVertexSkinningBDEF1 {
    pub bone_index: i32,
}

impl PMXReadWithSetting for PmxVertexSkinningBDEF1 {
    fn read_with_setting(
        parser: &mut PMXParser<'_>,
        setting: PmxSetting,
    ) -> Result<Self, MMDParseError> {
        let bone_index = parser.read_index(setting.bone_index_size).map_err(|e| {
            PmxParseError::map_end_of_stream_error(e, "vertex skinning bdef1 bone index")
        })?;
        Ok(PmxVertexSkinningBDEF1 { bone_index })
    }
}

#[derive(Clone, Debug, Default)]
pub struct PmxVertexSkinningBDEF2 {
    pub bone_index_1: i32,
    pub bone_index_2: i32,
    pub bone_weight: f32,
}

impl PMXReadWithSetting for PmxVertexSkinningBDEF2 {
    fn read_with_setting(
        parser: &mut PMXParser<'_>,
        setting: PmxSetting,
    ) -> Result<Self, MMDParseError> {
        let bone_index_1 = parser.read_index(setting.bone_index_size).map_err(|e| {
            PmxParseError::map_end_of_stream_error(e, "vertex skinning bdef2 bone index 1")
        })?;
        let bone_index_2 = parser.read_index(setting.bone_index_size).map_err(|e| {
            PmxParseError::map_end_of_stream_error(e, "vertex skinning bdef2 bone index 2")
        })?;
        let bone_weight = parser.read_f32().map_err(|e| {
            PmxParseError::map_end_of_stream_error(e, "vertex skinning bdef2 bone weight")
        })?;
        Ok(PmxVertexSkinningBDEF2 {
            bone_index_1,
            bone_index_2,
            bone_weight,
        })
    }
}

#[derive(Clone, Debug, Default)]
pub struct PmxVertexSkinningBDEF4 {
    pub bone_index_1: i32,
    pub bone_index_2: i32,
    pub bone_index_3: i32,
    pub bone_index_4: i32,
    pub bone_weight_1: f32,
    pub bone_weight_2: f32,
    pub bone_weight_3: f32,
    pub bone_weight_4: f32,
}

impl PMXReadWithSetting for PmxVertexSkinningBDEF4 {
    fn read_with_setting(
        parser: &mut PMXParser<'_>,
        setting: PmxSetting,
    ) -> Result<Self, MMDParseError> {
        let bone_index_1 = parser.read_index(setting.bone_index_size).map_err(|e| {
            PmxParseError::map_end_of_stream_error(e, "vertex skinning bdef4 bone index 1")
        })?;
        let bone_index_2 = parser.read_index(setting.bone_index_size).map_err(|e| {
            PmxParseError::map_end_of_stream_error(e, "vertex skinning bdef4 bone index 2")
        })?;
        let bone_index_3 = parser.read_index(setting.bone_index_size).map_err(|e| {
            PmxParseError::map_end_of_stream_error(e, "vertex skinning bdef4 bone index 3")
        })?;
        let bone_index_4 = parser.read_index(setting.bone_index_size).map_err(|e| {
            PmxParseError::map_end_of_stream_error(e, "vertex skinning bdef4 bone index 4")
        })?;
        let bone_weight_1 = parser.read_f32().map_err(|e| {
            PmxParseError::map_end_of_stream_error(e, "vertex skinning bdef4 bone weight 1")
        })?;
        let bone_weight_2 = parser.read_f32().map_err(|e| {
            PmxParseError::map_end_of_stream_error(e, "vertex skinning bdef4 bone weight 2")
        })?;
        let bone_weight_3 = parser.read_f32().map_err(|e| {
            PmxParseError::map_end_of_stream_error(e, "vertex skinning bdef4 bone weight 3")
        })?;
        let bone_weight_4 = parser.read_f32().map_err(|e| {
            PmxParseError::map_end_of_stream_error(e, "vertex skinning bdef4 bone weight 4")
        })?;
        Ok(PmxVertexSkinningBDEF4 {
            bone_index_1,
            bone_index_2,
            bone_index_3,
            bone_index_4,
            bone_weight_1,
            bone_weight_2,
            bone_weight_3,
            bone_weight_4,
        })
    }
}

#[derive(Clone, Debug, Default)]
pub struct PmxVertexSkinningSDEF {
    pub bone_index_1: i32,
    pub bone_index_2: i32,
    pub bone_weight: f32,
    pub sdef_c: Vec3,
    pub sdef_r0: Vec3,
    pub sdef_r1: Vec3,
}

impl PMXReadWithSetting for PmxVertexSkinningSDEF {
    fn read_with_setting(
        parser: &mut PMXParser<'_>,
        setting: PmxSetting,
    ) -> Result<Self, MMDParseError> {
        let bone_index_1 = parser.read_index(setting.bone_index_size).map_err(|e| {
            PmxParseError::map_end_of_stream_error(e, "vertex skinning sdef bone index 1")
        })?;
        let bone_index_2 = parser.read_index(setting.bone_index_size).map_err(|e| {
            PmxParseError::map_end_of_stream_error(e, "vertex skinning sdef bone index 2")
        })?;
        let bone_weight = parser.read_f32().map_err(|e| {
            PmxParseError::map_end_of_stream_error(e, "vertex skinning sdef bone weight")
        })?;
        let sdef_c = parser.read_vec3().map_err(|e| {
            PmxParseError::map_end_of_stream_error(e, "vertex skinning sdef sdef_c")
        })?;
        let sdef_r0 = parser.read_vec3().map_err(|e| {
            PmxParseError::map_end_of_stream_error(e, "vertex skinning sdef sdef_r0")
        })?;
        let sdef_r1 = parser.read_vec3().map_err(|e| {
            PmxParseError::map_end_of_stream_error(e, "vertex skinning sdef sdef_r1")
        })?;
        Ok(PmxVertexSkinningSDEF {
            bone_index_1,
            bone_index_2,
            bone_weight,
            sdef_c,
            sdef_r0,
            sdef_r1,
        })
    }
}

#[derive(Clone, Debug, Default)]
pub struct PmxVertexSkinningQDEF {
    pub bone_index_1: i32,
    pub bone_index_2: i32,
    pub bone_index_3: i32,
    pub bone_index_4: i32,
    pub bone_weight_1: f32,
    pub bone_weight_2: f32,
    pub bone_weight_3: f32,
    pub bone_weight_4: f32,
}

impl PMXReadWithSetting for PmxVertexSkinningQDEF {
    fn read_with_setting(
        parser: &mut PMXParser<'_>,
        setting: PmxSetting,
    ) -> Result<Self, MMDParseError> {
        let bone_index_1 = parser.read_index(setting.bone_index_size).map_err(|e| {
            PmxParseError::map_end_of_stream_error(e, "vertex skinning qdef bone index 1")
        })?;
        let bone_index_2 = parser.read_index(setting.bone_index_size).map_err(|e| {
            PmxParseError::map_end_of_stream_error(e, "vertex skinning qdef bone index 2")
        })?;
        let bone_index_3 = parser.read_index(setting.bone_index_size).map_err(|e| {
            PmxParseError::map_end_of_stream_error(e, "vertex skinning qdef bone index 3")
        })?;
        let bone_index_4 = parser.read_index(setting.bone_index_size).map_err(|e| {
            PmxParseError::map_end_of_stream_error(e, "vertex skinning qdef bone index 4")
        })?;
        let bone_weight_1 = parser.read_f32().map_err(|e| {
            PmxParseError::map_end_of_stream_error(e, "vertex skinning qdef bone weight 1")
        })?;
        let bone_weight_2 = parser.read_f32().map_err(|e| {
            PmxParseError::map_end_of_stream_error(e, "vertex skinning qdef bone weight 2")
        })?;
        let bone_weight_3 = parser.read_f32().map_err(|e| {
            PmxParseError::map_end_of_stream_error(e, "vertex skinning qdef bone weight 3")
        })?;
        let bone_weight_4 = parser.read_f32().map_err(|e| {
            PmxParseError::map_end_of_stream_error(e, "vertex skinning qdef bone weight 4")
        })?;
        Ok(PmxVertexSkinningQDEF {
            bone_index_1,
            bone_index_2,
            bone_index_3,
            bone_index_4,
            bone_weight_1,
            bone_weight_2,
            bone_weight_3,
            bone_weight_4,
        })
    }
}

#[derive(Clone, Debug)]
pub enum PmxVertexSkinning {
    BDEF1(PmxVertexSkinningBDEF1),
    BDEF2(PmxVertexSkinningBDEF2),
    BDEF4(PmxVertexSkinningBDEF4),
    SDEF(PmxVertexSkinningSDEF),
    QDEF(PmxVertexSkinningQDEF),
}

impl PMXReadWithSetting for PmxVertexSkinning {
    fn read_with_setting(
        parser: &mut PMXParser<'_>,
        setting: PmxSetting,
    ) -> Result<Self, MMDParseError> {
        let skinning_type = parser
            .read_u8()
            .map_err(|e| PmxParseError::map_end_of_stream_error(e, "vertex skinning type"))?;
        match skinning_type {
            0 => Ok(PmxVertexSkinning::BDEF1(
                PmxVertexSkinningBDEF1::read_with_setting(parser, setting)?,
            )),
            1 => Ok(PmxVertexSkinning::BDEF2(
                PmxVertexSkinningBDEF2::read_with_setting(parser, setting)?,
            )),
            2 => Ok(PmxVertexSkinning::BDEF4(
                PmxVertexSkinningBDEF4::read_with_setting(parser, setting)?,
            )),
            3 => Ok(PmxVertexSkinning::SDEF(
                PmxVertexSkinningSDEF::read_with_setting(parser, setting)?,
            )),
            4 => Ok(PmxVertexSkinning::QDEF(
                PmxVertexSkinningQDEF::read_with_setting(parser, setting)?,
            )),
            other => Err(PmxParseError::InvalidVertexSkinningType(other))?,
        }
    }
}

#[derive(Clone, Debug)]
pub struct PmxVertex {
    pub position: Vec3,
    pub normal: Vec3,
    pub uv: Vec2,
    pub uva: [Vec4; 4],
    pub skinning: PmxVertexSkinning,
    pub edge: f32,
}

impl PMXReadWithSetting for PmxVertex {
    fn read_with_setting(
        parser: &mut PMXParser<'_>,
        setting: PmxSetting,
    ) -> Result<Self, MMDParseError> {
        let position = parser
            .read_vec3()
            .map_err(|e| PmxParseError::map_end_of_stream_error(e, "vertex position"))?;
        let normal = parser
            .read_vec3()
            .map_err(|e| PmxParseError::map_end_of_stream_error(e, "vertex normal"))?;
        let uv = parser
            .read_vec2()
            .map_err(|e| PmxParseError::map_end_of_stream_error(e, "vertex uv"))?;
        let mut uva = [Vec4::ZERO; 4];
        for i in 0..setting.uv {
            uva[i as usize] = parser
                .read_vec4()
                .map_err(|e| PmxParseError::map_end_of_stream_error(e, "vertex uva"))?;
        }
        let skinning = PmxVertexSkinning::read_with_setting(parser, setting)?;
        let edge = parser
            .read_f32()
            .map_err(|e| PmxParseError::map_end_of_stream_error(e, "vertex edge"))?;
        Ok(PmxVertex {
            position,
            normal,
            uv,
            uva,
            skinning,
            edge,
        })
    }
}

#[derive(Clone, Debug)]
pub struct PmxMaterial {
    pub material_name: String,
    pub material_english_name: String,
    pub diffuse: Vec4,
    pub specular: Vec3,
    pub specularlity: f32,
    pub ambient: Vec3,
    pub flag: u8,
    pub edge_color: Vec4,
    pub edge_size: f32,
    pub diffuse_texture_index: i32,
    pub sphere_texture_index: i32,
    pub sphere_op_mode: u8,
    pub common_toon_flag: u8,
    pub toon_texture_index: i32,
    pub memo: String,
    pub index_count: u32,
}

impl PMXReadWithSetting for PmxMaterial {
    fn read_with_setting(
        parser: &mut PMXParser<'_>,
        setting: PmxSetting,
    ) -> Result<Self, MMDParseError> {
        let material_name = parser
            .read_string(setting.encoding)
            .map_err(|e| PmxParseError::map_end_of_stream_error(e, "material name"))?;
        let material_english_name = parser
            .read_string(setting.encoding)
            .map_err(|e| PmxParseError::map_end_of_stream_error(e, "material english name"))?;
        let diffuse = parser
            .read_vec4()
            .map_err(|e| PmxParseError::map_end_of_stream_error(e, "material diffuse"))?;
        let specular = parser
            .read_vec3()
            .map_err(|e| PmxParseError::map_end_of_stream_error(e, "material specular"))?;
        let specularlity = parser
            .read_f32()
            .map_err(|e| PmxParseError::map_end_of_stream_error(e, "material specularlity"))?;
        let ambient = parser
            .read_vec3()
            .map_err(|e| PmxParseError::map_end_of_stream_error(e, "material ambient"))?;
        let flag = parser
            .read_u8()
            .map_err(|e| PmxParseError::map_end_of_stream_error(e, "material flag"))?;
        let edge_color = parser
            .read_vec4()
            .map_err(|e| PmxParseError::map_end_of_stream_error(e, "material edge color"))?;
        let edge_size = parser
            .read_f32()
            .map_err(|e| PmxParseError::map_end_of_stream_error(e, "material edge size"))?;
        let diffuse_texture_index = parser.read_index(setting.texture_index_size).map_err(|e| {
            PmxParseError::map_end_of_stream_error(e, "material diffuse texture index")
        })?;
        let sphere_texture_index = parser.read_index(setting.texture_index_size).map_err(|e| {
            PmxParseError::map_end_of_stream_error(e, "material sphere texture index")
        })?;
        let sphere_op_mode = parser
            .read_u8()
            .map_err(|e| PmxParseError::map_end_of_stream_error(e, "material sphere op mode"))?;
        let common_toon_flag = parser
            .read_u8()
            .map_err(|e| PmxParseError::map_end_of_stream_error(e, "material common toon flag"))?;
        let toon_texture_index = if common_toon_flag == 1 {
            parser.read_i8().map_err(|e| {
                PmxParseError::map_end_of_stream_error(e, "material toon texture index")
            })? as i32
        } else {
            parser.read_index(setting.texture_index_size).map_err(|e| {
                PmxParseError::map_end_of_stream_error(e, "material toon texture index")
            })?
        };
        let memo = parser
            .read_string(setting.encoding)
            .map_err(|e| PmxParseError::map_end_of_stream_error(e, "material memo"))?;
        let index_count = parser
            .read_u32()
            .map_err(|e| PmxParseError::map_end_of_stream_error(e, "material index count"))?;
        Ok(PmxMaterial {
            material_name,
            material_english_name,
            diffuse,
            specular,
            specularlity,
            ambient,
            flag,
            edge_color,
            edge_size,
            diffuse_texture_index,
            sphere_texture_index,
            sphere_op_mode,
            common_toon_flag,
            toon_texture_index,
            memo,
            index_count,
        })
    }
}

#[derive(Clone, Debug, Default)]
pub struct PmxIkLink {
    pub link_target: i32,
    pub angle_lock: u8,
    pub max_radian: Vec3,
    pub min_radian: Vec3,
}

impl PMXReadWithSetting for PmxIkLink {
    fn read_with_setting(
        parser: &mut PMXParser<'_>,
        setting: PmxSetting,
    ) -> Result<Self, MMDParseError> {
        let link_target = parser
            .read_index(setting.bone_index_size)
            .map_err(|e| PmxParseError::map_end_of_stream_error(e, "ik link target"))?;
        let angle_lock = parser
            .read_u8()
            .map_err(|e| PmxParseError::map_end_of_stream_error(e, "ik link angle lock"))?;
        let max_radian;
        let min_radian;
        if angle_lock == 1 {
            max_radian = parser
                .read_vec3()
                .map_err(|e| PmxParseError::map_end_of_stream_error(e, "ik link max radian"))?;
            min_radian = parser
                .read_vec3()
                .map_err(|e| PmxParseError::map_end_of_stream_error(e, "ik link min radian"))?;
        } else {
            max_radian = Vec3::ZERO;
            min_radian = Vec3::ZERO;
        }
        Ok(PmxIkLink {
            link_target,
            angle_lock,
            max_radian,
            min_radian,
        })
    }
}

#[derive(Clone, Debug)]
pub struct PmxBone {
    pub bone_name: String,
    pub bone_english_name: String,
    pub position: Vec3,
    pub parent_index: i32,
    pub level: u32,
    pub bone_flag: u16,
    pub offset: Vec3,
    pub target_index: i32,
    pub grant_parent_index: i32,
    pub grant_weight: f32,
    pub lock_axis_orientation: Vec3,
    pub local_axis_x_orientation: Vec3,
    pub local_axis_y_orientation: Vec3,
    pub key: u32,
    pub ik_target_bone_index: i32,
    pub ik_loop: u32,
    pub ik_loop_angle_limit: f32,
    pub ik_link_count: u32,
    pub ik_links: Vec<PmxIkLink>,
}

impl PMXReadWithSetting for PmxBone {
    fn read_with_setting(
        parser: &mut PMXParser<'_>,
        setting: PmxSetting,
    ) -> Result<Self, MMDParseError> {
        let bone_name = parser
            .read_string(setting.encoding)
            .map_err(|e| PmxParseError::map_end_of_stream_error(e, "bone name"))?;
        let bone_english_name = parser
            .read_string(setting.encoding)
            .map_err(|e| PmxParseError::map_end_of_stream_error(e, "bone english name"))?;
        let position = parser
            .read_vec3()
            .map_err(|e| PmxParseError::map_end_of_stream_error(e, "bone position"))?;
        let parent_index = parser
            .read_index(setting.bone_index_size)
            .map_err(|e| PmxParseError::map_end_of_stream_error(e, "bone parent index"))?;
        let level = parser
            .read_u32()
            .map_err(|e| PmxParseError::map_end_of_stream_error(e, "bone level"))?;
        let bone_flag = parser
            .read_u16()
            .map_err(|e| PmxParseError::map_end_of_stream_error(e, "bone flag"))?;
        let mut target_index = 0;
        let mut offset = Vec3::ZERO;
        let mut grant_parent_index = 0;
        let mut grant_weight = 0.0;
        let mut lock_axis_orientation = Vec3::ZERO;
        let mut local_axis_x_orientation = Vec3::ZERO;
        let mut local_axis_y_orientation = Vec3::ZERO;
        let mut key = 0;
        let mut ik_target_bone_index = 0;
        let mut ik_loop = 0;
        let mut ik_loop_angle_limit = 0.0;
        let mut ik_link_count = 0;
        let mut ik_links = Vec::new();

        if bone_flag & 0x0001 != 0 {
            target_index = parser
                .read_index(setting.bone_index_size)
                .map_err(|e| PmxParseError::map_end_of_stream_error(e, "bone target index"))?;
        } else {
            offset = parser
                .read_vec3()
                .map_err(|e| PmxParseError::map_end_of_stream_error(e, "bone offset"))?;
        }

        if (bone_flag & (0x0100 | 0x0200)) != 0 {
            grant_parent_index = parser.read_index(setting.bone_index_size).map_err(|e| {
                PmxParseError::map_end_of_stream_error(e, "bone grant parent index")
            })?;
            grant_weight = parser
                .read_f32()
                .map_err(|e| PmxParseError::map_end_of_stream_error(e, "bone grant weight"))?;
        }
        if bone_flag & 0x0400 != 0 {
            lock_axis_orientation = parser.read_vec3().map_err(|e| {
                PmxParseError::map_end_of_stream_error(e, "bone lock axis orientation")
            })?;
        }
        if bone_flag & 0x0800 != 0 {
            local_axis_x_orientation = parser.read_vec3().map_err(|e| {
                PmxParseError::map_end_of_stream_error(e, "bone local axis x orientation")
            })?;
            local_axis_y_orientation = parser.read_vec3().map_err(|e| {
                PmxParseError::map_end_of_stream_error(e, "bone local axis y orientation")
            })?;
        }
        if bone_flag & 0x2000 != 0 {
            key = parser
                .read_u32()
                .map_err(|e| PmxParseError::map_end_of_stream_error(e, "bone key"))?;
        }
        if bone_flag & 0x0020 != 0 {
            ik_target_bone_index = parser.read_index(setting.bone_index_size).map_err(|e| {
                PmxParseError::map_end_of_stream_error(e, "bone ik target bone index")
            })?;
            ik_loop = parser
                .read_u32()
                .map_err(|e| PmxParseError::map_end_of_stream_error(e, "bone ik loop"))?;
            ik_loop_angle_limit = parser.read_f32().map_err(|e| {
                PmxParseError::map_end_of_stream_error(e, "bone ik loop angle limit")
            })?;
            ik_link_count = parser
                .read_u32()
                .map_err(|e| PmxParseError::map_end_of_stream_error(e, "bone ik link count"))?;
            ik_links
                .try_reserve(ik_link_count as usize)
                .map_err(|_| MMD_COMMON_ERROR_OUT_OF_MEMORY)?;
            for _ in 0..ik_link_count {
                ik_links.push(PmxIkLink::read_with_setting(parser, setting)?);
            }
        }
        Ok(PmxBone {
            bone_name,
            bone_english_name,
            position,
            parent_index,
            level,
            bone_flag,
            offset,
            target_index,
            grant_parent_index,
            grant_weight,
            lock_axis_orientation,
            local_axis_x_orientation,
            local_axis_y_orientation,
            key,
            ik_target_bone_index,
            ik_loop,
            ik_loop_angle_limit,
            ik_link_count,
            ik_links,
        })
    }
}

#[derive(Clone, Debug, Default)]
pub struct PmxMorphVertexOffset {
    pub vertex_index: i32,
    pub position_offset: Vec3,
}

impl PMXReadWithSetting for PmxMorphVertexOffset {
    fn read_with_setting(
        parser: &mut PMXParser<'_>,
        setting: PmxSetting,
    ) -> Result<Self, MMDParseError> {
        let vertex_index = parser.read_index(setting.vertex_index_size).map_err(|e| {
            PmxParseError::map_end_of_stream_error(e, "morph vertex offset vertex index")
        })?;
        let position_offset = parser.read_vec3().map_err(|e| {
            PmxParseError::map_end_of_stream_error(e, "morph vertex offset position offset")
        })?;
        Ok(PmxMorphVertexOffset {
            vertex_index,
            position_offset,
        })
    }
}

#[derive(Clone, Debug, Default)]
pub struct PmxMorphUVOffset {
    pub vertex_index: i32,
    pub uv_offset: Vec4,
}

impl PMXReadWithSetting for PmxMorphUVOffset {
    fn read_with_setting(
        parser: &mut PMXParser<'_>,
        setting: PmxSetting,
    ) -> Result<Self, MMDParseError> {
        let vertex_index = parser.read_index(setting.vertex_index_size).map_err(|e| {
            PmxParseError::map_end_of_stream_error(e, "morph uv offset vertex index")
        })?;
        let uv_offset = parser
            .read_vec4()
            .map_err(|e| PmxParseError::map_end_of_stream_error(e, "morph uv offset uv offset"))?;
        Ok(PmxMorphUVOffset {
            vertex_index,
            uv_offset,
        })
    }
}

#[derive(Clone, Debug, Default)]
pub struct PmxMorphBoneOffset {
    pub bone_index: i32,
    pub translation: Vec3,
    pub rotation: Vec4,
}

impl PMXReadWithSetting for PmxMorphBoneOffset {
    fn read_with_setting(
        parser: &mut PMXParser<'_>,
        setting: PmxSetting,
    ) -> Result<Self, MMDParseError> {
        let bone_index = parser.read_index(setting.bone_index_size).map_err(|e| {
            PmxParseError::map_end_of_stream_error(e, "morph bone offset bone index")
        })?;
        let translation = parser.read_vec3().map_err(|e| {
            PmxParseError::map_end_of_stream_error(e, "morph bone offset translation")
        })?;
        let rotation = parser
            .read_vec4()
            .map_err(|e| PmxParseError::map_end_of_stream_error(e, "morph bone offset rotation"))?;
        Ok(PmxMorphBoneOffset {
            bone_index,
            translation,
            rotation,
        })
    }
}

#[derive(Clone, Debug, Default)]
pub struct PmxMorphMaterialOffset {
    pub material_index: i32,
    pub offset_operation: u8,
    pub diffuse: Vec4,
    pub specular: Vec3,
    pub specularity: f32,
    pub ambient: Vec3,
    pub edge_color: Vec4,
    pub edge_size: f32,
    pub texture_argb: Vec4,
    pub sphere_texture_argb: Vec4,
    pub toon_texture_argb: Vec4,
}

impl PMXReadWithSetting for PmxMorphMaterialOffset {
    fn read_with_setting(
        parser: &mut PMXParser<'_>,
        setting: PmxSetting,
    ) -> Result<Self, MMDParseError> {
        let material_index = parser
            .read_index(setting.material_index_size)
            .map_err(|e| {
                PmxParseError::map_end_of_stream_error(e, "morph material offset material index")
            })?;
        let offset_operation = parser.read_u8().map_err(|e| {
            PmxParseError::map_end_of_stream_error(e, "morph material offset operation")
        })?;
        let diffuse = parser.read_vec4().map_err(|e| {
            PmxParseError::map_end_of_stream_error(e, "morph material offset diffuse")
        })?;
        let specular = parser.read_vec3().map_err(|e| {
            PmxParseError::map_end_of_stream_error(e, "morph material offset specular")
        })?;
        let specularity = parser.read_f32().map_err(|e| {
            PmxParseError::map_end_of_stream_error(e, "morph material offset specularity")
        })?;
        let ambient = parser.read_vec3().map_err(|e| {
            PmxParseError::map_end_of_stream_error(e, "morph material offset ambient")
        })?;
        let edge_color = parser.read_vec4().map_err(|e| {
            PmxParseError::map_end_of_stream_error(e, "morph material offset edge color")
        })?;
        let edge_size = parser.read_f32().map_err(|e| {
            PmxParseError::map_end_of_stream_error(e, "morph material offset edge size")
        })?;
        let texture_argb = parser.read_vec4().map_err(|e| {
            PmxParseError::map_end_of_stream_error(e, "morph material offset texture argb")
        })?;
        let sphere_texture_argb = parser.read_vec4().map_err(|e| {
            PmxParseError::map_end_of_stream_error(e, "morph material offset sphere texture argb")
        })?;
        let toon_texture_argb = parser.read_vec4().map_err(|e| {
            PmxParseError::map_end_of_stream_error(e, "morph material offset toon texture argb")
        })?;
        Ok(PmxMorphMaterialOffset {
            material_index,
            offset_operation,
            diffuse,
            specular,
            specularity,
            ambient,
            edge_color,
            edge_size,
            texture_argb,
            sphere_texture_argb,
            toon_texture_argb,
        })
    }
}

#[derive(Clone, Debug, Default)]
pub struct PmxMorphGroupOffset {
    pub morph_index: i32,
    pub morph_weight: f32,
}

impl PMXReadWithSetting for PmxMorphGroupOffset {
    fn read_with_setting(
        parser: &mut PMXParser<'_>,
        setting: PmxSetting,
    ) -> Result<Self, MMDParseError> {
        let morph_index = parser.read_index(setting.morph_index_size).map_err(|e| {
            PmxParseError::map_end_of_stream_error(e, "morph group offset morph index")
        })?;
        let morph_weight = parser.read_f32().map_err(|e| {
            PmxParseError::map_end_of_stream_error(e, "morph group offset morph weight")
        })?;
        Ok(PmxMorphGroupOffset {
            morph_index,
            morph_weight,
        })
    }
}

#[derive(Clone, Debug, Default)]
pub struct PmxMorphFlipOffset {
    pub morph_index: i32,
    pub morph_value: f32,
}

impl PMXReadWithSetting for PmxMorphFlipOffset {
    fn read_with_setting(
        parser: &mut PMXParser<'_>,
        setting: PmxSetting,
    ) -> Result<Self, MMDParseError> {
        let morph_index = parser.read_index(setting.morph_index_size).map_err(|e| {
            PmxParseError::map_end_of_stream_error(e, "morph flip offset morph index")
        })?;
        let morph_value = parser.read_f32().map_err(|e| {
            PmxParseError::map_end_of_stream_error(e, "morph flip offset morph value")
        })?;
        Ok(PmxMorphFlipOffset {
            morph_index,
            morph_value,
        })
    }
}

#[derive(Clone, Debug, Default)]
pub struct PmxMorphImplusOffset {
    pub rigid_body_index: i32,
    pub is_local: u8,
    pub velocity: Vec3,
    pub angular_torque: Vec3,
}

impl PMXReadWithSetting for PmxMorphImplusOffset {
    fn read_with_setting(
        parser: &mut PMXParser<'_>,
        setting: PmxSetting,
    ) -> Result<Self, MMDParseError> {
        let rigid_body_index = parser
            .read_index(setting.rigidbody_index_size)
            .map_err(|e| {
                PmxParseError::map_end_of_stream_error(e, "morph implus offset rigid body index")
            })?;
        let is_local = parser.read_u8().map_err(|e| {
            PmxParseError::map_end_of_stream_error(e, "morph implus offset is local")
        })?;
        let velocity = parser.read_vec3().map_err(|e| {
            PmxParseError::map_end_of_stream_error(e, "morph implus offset velocity")
        })?;
        let angular_torque = parser.read_vec3().map_err(|e| {
            PmxParseError::map_end_of_stream_error(e, "morph implus offset angular torque")
        })?;
        Ok(PmxMorphImplusOffset {
            rigid_body_index,
            is_local,
            velocity,
            angular_torque,
        })
    }
}

#[derive(Clone, Debug)]
pub enum PmxMorphOffset {
    Vertex(PmxMorphVertexOffset),
    UV(PmxMorphUVOffset),
    Bone(PmxMorphBoneOffset),
    Material(PmxMorphMaterialOffset),
    Group(PmxMorphGroupOffset),
    Flip(PmxMorphFlipOffset),
    Implus(PmxMorphImplusOffset),
}

#[derive(Clone, Debug)]
pub enum PmxMorphCategory {
    ReservedCategory = 0,
    Eyebrow = 1,
    Eye = 2,
    Mouth = 3,
    Other = 4,
}

impl TryFrom<u8> for PmxMorphCategory {
    type Error = PmxParseError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(PmxMorphCategory::ReservedCategory),
            1 => Ok(PmxMorphCategory::Eyebrow),
            2 => Ok(PmxMorphCategory::Eye),
            3 => Ok(PmxMorphCategory::Mouth),
            4 => Ok(PmxMorphCategory::Other),
            other => Err(PmxParseError::InvalidMorphCategory(other)),
        }
    }
}
#[derive(Clone, Debug)]
pub enum PmxMorphType {
    Group = 0,
    Vertex = 1,
    Bone = 2,
    UV = 3,
    AdditionalUV1 = 4,
    AdditionalUV2 = 5,
    AdditionalUV3 = 6,
    AdditionalUV4 = 7,
    Matrial = 8,
    Flip = 9,
    Implus = 10,
}

impl TryFrom<u8> for PmxMorphType {
    type Error = PmxParseError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(PmxMorphType::Group),
            1 => Ok(PmxMorphType::Vertex),
            2 => Ok(PmxMorphType::Bone),
            3 => Ok(PmxMorphType::UV),
            4 => Ok(PmxMorphType::AdditionalUV1),
            5 => Ok(PmxMorphType::AdditionalUV2),
            6 => Ok(PmxMorphType::AdditionalUV3),
            7 => Ok(PmxMorphType::AdditionalUV4),
            8 => Ok(PmxMorphType::Matrial),
            9 => Ok(PmxMorphType::Flip),
            10 => Ok(PmxMorphType::Implus),
            other => Err(PmxParseError::InvalidMorphType(other)),
        }
    }
}

#[derive(Clone, Debug)]
pub struct PmxMorph {
    pub morph_name: String,
    pub morph_english_name: String,
    pub category: PmxMorphCategory,
    pub morph_type: PmxMorphType,
    pub offsets: Vec<PmxMorphOffset>,
}

impl PMXReadWithSetting for PmxMorph {
    fn read_with_setting(
        parser: &mut PMXParser<'_>,
        setting: PmxSetting,
    ) -> Result<Self, MMDParseError> {
        let morph_name = parser
            .read_string(setting.encoding)
            .map_err(|e| PmxParseError::map_end_of_stream_error(e, "morph name"))?;
        let morph_english_name = parser
            .read_string(setting.encoding)
            .map_err(|e| PmxParseError::map_end_of_stream_error(e, "morph english name"))?;
        let category = PmxMorphCategory::try_from(
            parser
                .read_u8()
                .map_err(|e| PmxParseError::map_end_of_stream_error(e, "morph category"))?,
        )?;
        let morph_type = PmxMorphType::try_from(
            parser
                .read_u8()
                .map_err(|e| PmxParseError::map_end_of_stream_error(e, "morph type"))?,
        )?;
        let offset_count = parser
            .read_u32()
            .map_err(|e| PmxParseError::map_end_of_stream_error(e, "morph offset count"))?;
        let mut offsets = Vec::new();
        offsets
            .try_reserve(offset_count as usize)
            .map_err(|_| MMD_COMMON_ERROR_OUT_OF_MEMORY)?;
        match morph_type {
            PmxMorphType::Vertex => {
                for _ in 0..offset_count {
                    offsets.push(PmxMorphOffset::Vertex(
                        PmxMorphVertexOffset::read_with_setting(parser, setting)?,
                    ));
                }
            }
            PmxMorphType::UV
            | PmxMorphType::AdditionalUV1
            | PmxMorphType::AdditionalUV2
            | PmxMorphType::AdditionalUV3
            | PmxMorphType::AdditionalUV4 => {
                for _ in 0..offset_count {
                    offsets.push(PmxMorphOffset::UV(PmxMorphUVOffset::read_with_setting(
                        parser, setting,
                    )?));
                }
            }
            PmxMorphType::Bone => {
                for _ in 0..offset_count {
                    offsets.push(PmxMorphOffset::Bone(PmxMorphBoneOffset::read_with_setting(
                        parser, setting,
                    )?));
                }
            }
            PmxMorphType::Matrial => {
                for _ in 0..offset_count {
                    offsets.push(PmxMorphOffset::Material(
                        PmxMorphMaterialOffset::read_with_setting(parser, setting)?,
                    ));
                }
            }
            PmxMorphType::Group => {
                for _ in 0..offset_count {
                    offsets.push(PmxMorphOffset::Group(
                        PmxMorphGroupOffset::read_with_setting(parser, setting)?,
                    ));
                }
            }
            PmxMorphType::Flip => {
                for _ in 0..offset_count {
                    offsets.push(PmxMorphOffset::Flip(PmxMorphFlipOffset::read_with_setting(
                        parser, setting,
                    )?));
                }
            }
            PmxMorphType::Implus => {
                for _ in 0..offset_count {
                    offsets.push(PmxMorphOffset::Implus(
                        PmxMorphImplusOffset::read_with_setting(parser, setting)?,
                    ));
                }
            }
        }
        Ok(PmxMorph {
            morph_name,
            morph_english_name,
            category,
            morph_type,
            offsets,
        })
    }
}

#[derive(Clone, Debug, Default)]
pub struct PmxFrameElement {
    pub element_target: u8,
    pub index: i32,
}

impl PMXReadWithSetting for PmxFrameElement {
    fn read_with_setting(
        parser: &mut PMXParser<'_>,
        setting: PmxSetting,
    ) -> Result<Self, MMDParseError> {
        let element_target = parser
            .read_u8()
            .map_err(|e| PmxParseError::map_end_of_stream_error(e, "frame element target"))?;
        let index = if element_target == 0 {
            parser
                .read_index(setting.bone_index_size)
                .map_err(|e| PmxParseError::map_end_of_stream_error(e, "frame element index"))?
        } else {
            parser
                .read_index(setting.morph_index_size)
                .map_err(|e| PmxParseError::map_end_of_stream_error(e, "frame element index"))?
        };
        Ok(PmxFrameElement {
            element_target,
            index,
        })
    }
}

#[derive(Clone, Debug, Default)]
pub struct PmxFrame {
    pub frame_name: String,
    pub frame_english_name: String,
    pub frame_flag: u8,
    pub elements: Vec<PmxFrameElement>,
}

impl PMXReadWithSetting for PmxFrame {
    fn read_with_setting(
        parser: &mut PMXParser<'_>,
        setting: PmxSetting,
    ) -> Result<Self, MMDParseError> {
        let frame_name = parser
            .read_string(setting.encoding)
            .map_err(|e| PmxParseError::map_end_of_stream_error(e, "frame name"))?;
        let frame_english_name = parser
            .read_string(setting.encoding)
            .map_err(|e| PmxParseError::map_end_of_stream_error(e, "frame english name"))?;
        let frame_flag = parser
            .read_u8()
            .map_err(|e| PmxParseError::map_end_of_stream_error(e, "frame flag"))?;
        let element_count = parser
            .read_u32()
            .map_err(|e| PmxParseError::map_end_of_stream_error(e, "frame element count"))?;
        let mut elements = Vec::new();
        elements
            .try_reserve(element_count as usize)
            .map_err(|_| MMD_COMMON_ERROR_OUT_OF_MEMORY)?;
        for _ in 0..element_count {
            elements.push(PmxFrameElement::read_with_setting(parser, setting)?);
        }
        Ok(PmxFrame {
            frame_name,
            frame_english_name,
            frame_flag,
            elements,
        })
    }
}

#[derive(Clone, Debug, Default)]
pub struct PmxRigidBody {
    pub rigid_body_name: String,
    pub rigid_body_english_name: String,
    pub target_bone: i32,
    pub group: u8,
    pub mask: u16,
    pub shape: u8,
    pub size: Vec3,
    pub position: Vec3,
    pub orientation: Vec3,
    pub mass: f32,
    pub move_attenuation: f32,
    pub rotation_attenuation: f32,
    pub repulsion: f32,
    pub friction: f32,
    pub physics_calc_type: u8,
}

impl PMXReadWithSetting for PmxRigidBody {
    fn read_with_setting(
        parser: &mut PMXParser<'_>,
        setting: PmxSetting,
    ) -> Result<Self, MMDParseError> {
        let rigid_body_name = parser
            .read_string(setting.encoding)
            .map_err(|e| PmxParseError::map_end_of_stream_error(e, "rigid body name"))?;
        let rigid_body_english_name = parser
            .read_string(setting.encoding)
            .map_err(|e| PmxParseError::map_end_of_stream_error(e, "rigid body english name"))?;
        let target_bone = parser
            .read_index(setting.bone_index_size)
            .map_err(|e| PmxParseError::map_end_of_stream_error(e, "rigid body target bone"))?;
        let group = parser
            .read_u8()
            .map_err(|e| PmxParseError::map_end_of_stream_error(e, "rigid body group"))?;
        let mask = parser
            .read_u16()
            .map_err(|e| PmxParseError::map_end_of_stream_error(e, "rigid body mask"))?;
        let shape = parser
            .read_u8()
            .map_err(|e| PmxParseError::map_end_of_stream_error(e, "rigid body shape"))?;
        let size = parser
            .read_vec3()
            .map_err(|e| PmxParseError::map_end_of_stream_error(e, "rigid body size"))?;
        let position = parser
            .read_vec3()
            .map_err(|e| PmxParseError::map_end_of_stream_error(e, "rigid body position"))?;
        let orientation = parser
            .read_vec3()
            .map_err(|e| PmxParseError::map_end_of_stream_error(e, "rigid body orientation"))?;
        let mass = parser
            .read_f32()
            .map_err(|e| PmxParseError::map_end_of_stream_error(e, "rigid body mass"))?;
        let move_attenuation = parser.read_f32().map_err(|e| {
            PmxParseError::map_end_of_stream_error(e, "rigid body move attenuation")
        })?;
        let rotation_attenuation = parser.read_f32().map_err(|e| {
            PmxParseError::map_end_of_stream_error(e, "rigid body rotation attenuation")
        })?;
        let repulsion = parser
            .read_f32()
            .map_err(|e| PmxParseError::map_end_of_stream_error(e, "rigid body repulsion"))?;
        let friction = parser
            .read_f32()
            .map_err(|e| PmxParseError::map_end_of_stream_error(e, "rigid body friction"))?;
        let physics_calc_type = parser.read_u8().map_err(|e| {
            PmxParseError::map_end_of_stream_error(e, "rigid body physics calc type")
        })?;
        Ok(PmxRigidBody {
            rigid_body_name,
            rigid_body_english_name,
            target_bone,
            group,
            mask,
            shape,
            size,
            position,
            orientation,
            mass,
            move_attenuation,
            rotation_attenuation,
            repulsion,
            friction,
            physics_calc_type,
        })
    }
}

#[derive(Clone, Debug, Default)]
pub struct PmxJointParam {
    pub rigid_body1: i32,
    pub rigid_body2: i32,
    pub position: Vec3,
    pub orientaiton: Vec3,
    pub move_limitation_min: Vec3,
    pub move_limitation_max: Vec3,
    pub rotation_limitation_min: Vec3,
    pub rotation_limitation_max: Vec3,
    pub spring_move_coefficient: Vec3,
    pub spring_rotation_coefficient: Vec3,
}

impl PMXReadWithSetting for PmxJointParam {
    fn read_with_setting(
        parser: &mut PMXParser<'_>,
        setting: PmxSetting,
    ) -> Result<Self, MMDParseError> {
        let rigid_body1 = parser
            .read_index(setting.rigidbody_index_size)
            .map_err(|e| PmxParseError::map_end_of_stream_error(e, "joint param rigid body 1"))?;
        let rigid_body2 = parser
            .read_index(setting.rigidbody_index_size)
            .map_err(|e| PmxParseError::map_end_of_stream_error(e, "joint param rigid body 2"))?;
        let position = parser
            .read_vec3()
            .map_err(|e| PmxParseError::map_end_of_stream_error(e, "joint param position"))?;
        let orientaiton = parser
            .read_vec3()
            .map_err(|e| PmxParseError::map_end_of_stream_error(e, "joint param orientaiton"))?;
        let move_limitation_min = parser.read_vec3().map_err(|e| {
            PmxParseError::map_end_of_stream_error(e, "joint param move limitation min")
        })?;
        let move_limitation_max = parser.read_vec3().map_err(|e| {
            PmxParseError::map_end_of_stream_error(e, "joint param move limitation max")
        })?;
        let rotation_limitation_min = parser.read_vec3().map_err(|e| {
            PmxParseError::map_end_of_stream_error(e, "joint param rotation limitation min")
        })?;
        let rotation_limitation_max = parser.read_vec3().map_err(|e| {
            PmxParseError::map_end_of_stream_error(e, "joint param rotation limitation max")
        })?;
        let spring_move_coefficient = parser.read_vec3().map_err(|e| {
            PmxParseError::map_end_of_stream_error(e, "joint param spring move coefficient")
        })?;
        let spring_rotation_coefficient = parser.read_vec3().map_err(|e| {
            PmxParseError::map_end_of_stream_error(e, "joint param spring rotation coefficient")
        })?;
        Ok(PmxJointParam {
            rigid_body1,
            rigid_body2,
            position,
            orientaiton,
            move_limitation_min,
            move_limitation_max,
            rotation_limitation_min,
            rotation_limitation_max,
            spring_move_coefficient,
            spring_rotation_coefficient,
        })
    }
}

#[derive(Clone, Debug)]
pub enum PmxJointType {
    Generic6DofSpring = 0,
    Generic6Dof = 1,
    Point2Point = 2,
    ConeTwist = 3,
    Slider = 5,
    Hinge = 6,
}

impl TryFrom<u8> for PmxJointType {
    type Error = PmxParseError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(PmxJointType::Generic6DofSpring),
            1 => Ok(PmxJointType::Generic6Dof),
            2 => Ok(PmxJointType::Point2Point),
            3 => Ok(PmxJointType::ConeTwist),
            5 => Ok(PmxJointType::Slider),
            6 => Ok(PmxJointType::Hinge),
            other => Err(PmxParseError::InvalidJointType(other)),
        }
    }
}

#[derive(Clone, Debug)]
pub struct PmxJoint {
    pub joint_name: String,
    pub joint_english_name: String,
    pub joint_type: PmxJointType,
    pub param: PmxJointParam,
}

impl PMXReadWithSetting for PmxJoint {
    fn read_with_setting(
        parser: &mut PMXParser<'_>,
        setting: PmxSetting,
    ) -> Result<Self, MMDParseError> {
        let joint_name = parser
            .read_string(setting.encoding)
            .map_err(|e| PmxParseError::map_end_of_stream_error(e, "joint name"))?;
        let joint_english_name = parser
            .read_string(setting.encoding)
            .map_err(|e| PmxParseError::map_end_of_stream_error(e, "joint english name"))?;
        let joint_type = PmxJointType::try_from(
            parser
                .read_u8()
                .map_err(|e| PmxParseError::map_end_of_stream_error(e, "joint type"))?,
        )?;
        let param = PmxJointParam::read_with_setting(parser, setting)
            .map_err(|e| PmxParseError::map_end_of_stream_error(e, "joint param"))?;
        Ok(PmxJoint {
            joint_name,
            joint_english_name,
            joint_type,
            param,
        })
    }
}

#[derive(Clone, Debug)]
pub enum PmxSoftBodyFlag {
    BLink = 0x01,
    Cluster = 0x02,
    Link = 0x04,
}

impl TryFrom<u8> for PmxSoftBodyFlag {
    type Error = PmxParseError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x01 => Ok(PmxSoftBodyFlag::BLink),
            0x02 => Ok(PmxSoftBodyFlag::Cluster),
            0x04 => Ok(PmxSoftBodyFlag::Link),
            other => Err(PmxParseError::InvalidSoftBodyFlag(other)),
        }
    }
}

#[derive(Clone, Debug)]
pub struct PmxAncherRigidBody {
    pub related_rigid_body: i32,
    pub related_vertex: i32,
    pub is_near: bool,
}

impl PMXReadWithSetting for PmxAncherRigidBody {
    fn read_with_setting(
        parser: &mut PMXParser<'_>,
        setting: PmxSetting,
    ) -> Result<Self, MMDParseError> {
        let related_rigid_body = parser
            .read_index(setting.rigidbody_index_size)
            .map_err(|e| {
                PmxParseError::map_end_of_stream_error(e, "ancher rigid body related rigid body")
            })?;
        let related_vertex = parser.read_index(setting.vertex_index_size).map_err(|e| {
            PmxParseError::map_end_of_stream_error(e, "ancher rigid body related vertex")
        })?;
        let is_near = parser
            .read_u8()
            .map_err(|e| PmxParseError::map_end_of_stream_error(e, "ancher rigid body is near"))?
            > 0;
        Ok(PmxAncherRigidBody {
            related_rigid_body,
            related_vertex,
            is_near,
        })
    }
}
#[allow(non_snake_case)]
#[derive(Clone, Debug)]
pub struct PmxSoftBody {
    pub soft_body_name: String,
    pub soft_body_english_name: String,
    pub shape: u8,
    pub target_material: i32,
    pub group: u8,
    pub mask: u16,
    pub flag: PmxSoftBodyFlag,
    pub blink_distance: u32,
    pub cluster_count: u32,
    pub mass: f32,
    pub collisioni_margin: f32,
    pub aero_model: u32,
    pub VCF: f32,
    pub DP: f32,
    pub DG: f32,
    pub LF: f32,
    pub PR: f32,
    pub VC: f32,
    pub DF: f32,
    pub MT: f32,
    pub CHR: f32,
    pub KHR: f32,
    pub SHR: f32,
    pub AHR: f32,
    pub SRHR_CL: f32,
    pub SKHR_CL: f32,
    pub SSHR_CL: f32,
    pub SR_SPLT_CL: f32,
    pub SK_SPLT_CL: f32,
    pub SS_SPLT_CL: f32,
    pub V_IT: u32,
    pub P_IT: u32,
    pub D_IT: u32,
    pub C_IT: u32,
    pub LST: f32,
    pub AST: f32,
    pub VST: f32,
    pub anchor_count: u32,
    pub anchers: Vec<PmxAncherRigidBody>,
    pub pin_vertex_count: u32,
    pub pin_vertices: Vec<i32>,
}

impl PMXReadWithSetting for PmxSoftBody {
    #[allow(non_snake_case)]
    fn read_with_setting(
        parser: &mut PMXParser<'_>,
        setting: PmxSetting,
    ) -> Result<Self, MMDParseError> {
        let soft_body_name = parser
            .read_string(setting.encoding)
            .map_err(|e| PmxParseError::map_end_of_stream_error(e, "soft body name"))?;
        let soft_body_english_name = parser
            .read_string(setting.encoding)
            .map_err(|e| PmxParseError::map_end_of_stream_error(e, "soft body english name"))?;
        let shape = parser
            .read_u8()
            .map_err(|e| PmxParseError::map_end_of_stream_error(e, "soft body shape"))?;
        let target_material = parser
            .read_index(setting.material_index_size)
            .map_err(|e| PmxParseError::map_end_of_stream_error(e, "soft body target material"))?;
        let group = parser
            .read_u8()
            .map_err(|e| PmxParseError::map_end_of_stream_error(e, "soft body group"))?;
        let mask = parser
            .read_u16()
            .map_err(|e| PmxParseError::map_end_of_stream_error(e, "soft body mask"))?;
        let flag = PmxSoftBodyFlag::try_from(
            parser
                .read_u8()
                .map_err(|e| PmxParseError::map_end_of_stream_error(e, "soft body flag"))?,
        )?;
        let blink_distance = parser
            .read_u32()
            .map_err(|e| PmxParseError::map_end_of_stream_error(e, "soft body blink distance"))?;
        let cluster_count = parser
            .read_u32()
            .map_err(|e| PmxParseError::map_end_of_stream_error(e, "soft body cluster count"))?;
        let mass = parser
            .read_f32()
            .map_err(|e| PmxParseError::map_end_of_stream_error(e, "soft body mass"))?;
        let collisioni_margin = parser.read_f32().map_err(|e| {
            PmxParseError::map_end_of_stream_error(e, "soft body collisioni margin")
        })?;
        let aero_model = parser
            .read_u32()
            .map_err(|e| PmxParseError::map_end_of_stream_error(e, "soft body aero model"))?;
        let VCF = parser
            .read_f32()
            .map_err(|e| PmxParseError::map_end_of_stream_error(e, "soft body VCF"))?;
        let DP = parser
            .read_f32()
            .map_err(|e| PmxParseError::map_end_of_stream_error(e, "soft body DP"))?;
        let DG = parser
            .read_f32()
            .map_err(|e| PmxParseError::map_end_of_stream_error(e, "soft body DG"))?;
        let LF = parser
            .read_f32()
            .map_err(|e| PmxParseError::map_end_of_stream_error(e, "soft body LF"))?;
        let PR = parser
            .read_f32()
            .map_err(|e| PmxParseError::map_end_of_stream_error(e, "soft body PR"))?;
        let VC = parser
            .read_f32()
            .map_err(|e| PmxParseError::map_end_of_stream_error(e, "soft body VC"))?;
        let DF = parser
            .read_f32()
            .map_err(|e| PmxParseError::map_end_of_stream_error(e, "soft body DF"))?;
        let MT = parser
            .read_f32()
            .map_err(|e| PmxParseError::map_end_of_stream_error(e, "soft body MT"))?;
        let CHR = parser
            .read_f32()
            .map_err(|e| PmxParseError::map_end_of_stream_error(e, "soft body CHR"))?;
        let KHR = parser
            .read_f32()
            .map_err(|e| PmxParseError::map_end_of_stream_error(e, "soft body KHR"))?;
        let SHR = parser
            .read_f32()
            .map_err(|e| PmxParseError::map_end_of_stream_error(e, "soft body SHR"))?;
        let AHR = parser
            .read_f32()
            .map_err(|e| PmxParseError::map_end_of_stream_error(e, "soft body AHR"))?;
        let SRHR_CL = parser
            .read_f32()
            .map_err(|e| PmxParseError::map_end_of_stream_error(e, "soft body SRHR_CL"))?;
        let SKHR_CL = parser
            .read_f32()
            .map_err(|e| PmxParseError::map_end_of_stream_error(e, "soft body SKHR_CL"))?;
        let SSHR_CL = parser
            .read_f32()
            .map_err(|e| PmxParseError::map_end_of_stream_error(e, "soft body SSHR_CL"))?;
        let SR_SPLT_CL = parser
            .read_f32()
            .map_err(|e| PmxParseError::map_end_of_stream_error(e, "soft body SR_SPLT_CL"))?;
        let SK_SPLT_CL = parser
            .read_f32()
            .map_err(|e| PmxParseError::map_end_of_stream_error(e, "soft body SK_SPLT_CL"))?;
        let SS_SPLT_CL = parser
            .read_f32()
            .map_err(|e| PmxParseError::map_end_of_stream_error(e, "soft body SS_SPLT_CL"))?;
        let V_IT = parser
            .read_u32()
            .map_err(|e| PmxParseError::map_end_of_stream_error(e, "soft body V_IT"))?;
        let P_IT = parser
            .read_u32()
            .map_err(|e| PmxParseError::map_end_of_stream_error(e, "soft body P_IT"))?;
        let D_IT = parser
            .read_u32()
            .map_err(|e| PmxParseError::map_end_of_stream_error(e, "soft body D_IT"))?;
        let C_IT = parser
            .read_u32()
            .map_err(|e| PmxParseError::map_end_of_stream_error(e, "soft body C_IT"))?;
        let LST = parser
            .read_f32()
            .map_err(|e| PmxParseError::map_end_of_stream_error(e, "soft body LST"))?;
        let AST = parser
            .read_f32()
            .map_err(|e| PmxParseError::map_end_of_stream_error(e, "soft body AST"))?;
        let VST = parser
            .read_f32()
            .map_err(|e| PmxParseError::map_end_of_stream_error(e, "soft body VST"))?;
        let anchor_count = parser
            .read_u32()
            .map_err(|e| PmxParseError::map_end_of_stream_error(e, "soft body anchor count"))?;
        let mut anchers = Vec::new();
        anchers
            .try_reserve(anchor_count as usize)
            .map_err(|_| MMD_COMMON_ERROR_OUT_OF_MEMORY)?;
        for _ in 0..anchor_count {
            anchers.push(
                PmxAncherRigidBody::read_with_setting(parser, setting)
                    .map_err(|e| PmxParseError::map_end_of_stream_error(e, "soft body anchers"))?,
            );
        }
        let pin_vertex_count = parser
            .read_u32()
            .map_err(|e| PmxParseError::map_end_of_stream_error(e, "soft body pin vertex count"))?;
        let mut pin_vertices = Vec::new();
        pin_vertices
            .try_reserve(pin_vertex_count as usize)
            .map_err(|_| MMD_COMMON_ERROR_OUT_OF_MEMORY)?;
        for _ in 0..pin_vertex_count {
            pin_vertices.push(parser.read_index(setting.vertex_index_size).map_err(|e| {
                PmxParseError::map_end_of_stream_error(e, "soft body pin vertices")
            })?);
        }
        Ok(PmxSoftBody {
            soft_body_name,
            soft_body_english_name,
            shape,
            target_material,
            group,
            mask,
            flag,
            blink_distance,
            cluster_count,
            mass,
            collisioni_margin,
            aero_model,
            VCF,
            DP,
            DG,
            LF,
            PR,
            VC,
            DF,
            MT,
            CHR,
            KHR,
            SHR,
            AHR,
            SRHR_CL,
            SKHR_CL,
            SSHR_CL,
            SR_SPLT_CL,
            SK_SPLT_CL,
            SS_SPLT_CL,
            V_IT,
            P_IT,
            D_IT,
            C_IT,
            LST,
            AST,
            VST,
            anchor_count,
            anchers,
            pin_vertex_count,
            pin_vertices,
        })
    }
}

#[derive(Clone, Debug)]
pub struct PmxModel {
    pub version: f32,
    pub setting: PmxSetting,
    pub model_name: String,
    pub model_english_name: String,
    pub model_comment: String,
    pub model_english_comment: String,
    pub vertices: Vec<PmxVertex>,
    pub indices: Vec<i32>,
    pub textures: Vec<String>,
    pub materials: Vec<PmxMaterial>,
    pub bones: Vec<PmxBone>,
    pub morphs: Vec<PmxMorph>,
    pub frames: Vec<PmxFrame>,
    pub rigid_bodies: Vec<PmxRigidBody>,
    pub joints: Vec<PmxJoint>,
    pub soft_bodies: Vec<PmxSoftBody>,
}

impl PMXRead for PmxModel {
    fn read(parser: &mut PMXParser<'_>) -> Result<Self, MMDParseError> {
        let magic = parser
            .read_u32()
            .map_err(|e| PmxParseError::map_end_of_stream_error(e, "PMX magic"))?
            .to_le_bytes();
        if magic != *b"PMX " {
            Err(PmxParseError::InvalidMagic(magic))?;
        }

        let version = parser
            .read_f32()
            .map_err(|e| PmxParseError::map_end_of_stream_error(e, "PMX version"))?;
        if version != 2.0 && version != 2.1 {
            Err(PmxParseError::InvalidVersion(version))?;
        }
        let setting = PmxSetting::read(parser)?;
        let model_name = parser
            .read_string(setting.encoding)
            .map_err(|e| PmxParseError::map_end_of_stream_error(e, "PMX model name"))?;
        let model_english_name = parser
            .read_string(setting.encoding)
            .map_err(|e| PmxParseError::map_end_of_stream_error(e, "PMX model english name"))?;
        let model_comment = parser
            .read_string(setting.encoding)
            .map_err(|e| PmxParseError::map_end_of_stream_error(e, "PMX model comment"))?;
        let model_english_comment = parser
            .read_string(setting.encoding)
            .map_err(|e| PmxParseError::map_end_of_stream_error(e, "PMX model english comment"))?;

        // read vertices
        let vertex_count = parser
            .read_u32()
            .map_err(|e| PmxParseError::map_end_of_stream_error(e, "PMX vertex count"))?;
        let mut vertices = Vec::new();
        vertices
            .try_reserve(vertex_count as usize)
            .map_err(|_| MMD_COMMON_ERROR_OUT_OF_MEMORY)?;
        for _ in 0..vertex_count {
            vertices.push(PmxVertex::read_with_setting(parser, setting)?);
        }

        // read indices
        let index_count = parser
            .read_u32()
            .map_err(|e| PmxParseError::map_end_of_stream_error(e, "PMX index count"))?;
        let mut indices = Vec::new();
        indices
            .try_reserve(index_count as usize)
            .map_err(|_| MMD_COMMON_ERROR_OUT_OF_MEMORY)?;
        for _ in 0..index_count {
            indices.push(parser.read_index(setting.vertex_index_size)?);
        }

        // read textures names
        let texture_count = parser
            .read_u32()
            .map_err(|e| PmxParseError::map_end_of_stream_error(e, "PMX texture count"))?;
        let mut textures = Vec::new();
        textures
            .try_reserve(texture_count as usize)
            .map_err(|_| MMD_COMMON_ERROR_OUT_OF_MEMORY)?;
        for _ in 0..texture_count {
            textures.push(
                parser
                    .read_string(setting.encoding)
                    .map_err(|e| PmxParseError::map_end_of_stream_error(e, "PMX texture name"))?,
            );
        }

        // read materials
        let material_count = parser
            .read_u32()
            .map_err(|e| PmxParseError::map_end_of_stream_error(e, "PMX material count"))?;
        let mut materials = Vec::new();
        materials
            .try_reserve(material_count as usize)
            .map_err(|_| MMD_COMMON_ERROR_OUT_OF_MEMORY)?;
        for _ in 0..material_count {
            materials.push(PmxMaterial::read_with_setting(parser, setting)?);
        }

        // read bones
        let bone_count = parser
            .read_u32()
            .map_err(|e| PmxParseError::map_end_of_stream_error(e, "PMX bone count"))?;
        let mut bones = Vec::new();
        bones
            .try_reserve(bone_count as usize)
            .map_err(|_| MMD_COMMON_ERROR_OUT_OF_MEMORY)?;
        for _ in 0..bone_count {
            bones.push(PmxBone::read_with_setting(parser, setting)?);
        }

        // read morphs
        let morph_count = parser
            .read_u32()
            .map_err(|e| PmxParseError::map_end_of_stream_error(e, "PMX morph count"))?;
        let mut morphs = Vec::new();
        morphs
            .try_reserve(morph_count as usize)
            .map_err(|_| MMD_COMMON_ERROR_OUT_OF_MEMORY)?;
        for _ in 0..morph_count {
            morphs.push(PmxMorph::read_with_setting(parser, setting)?);
        }

        // read display frames
        let frame_count = parser
            .read_u32()
            .map_err(|e| PmxParseError::map_end_of_stream_error(e, "PMX frame count"))?;
        let mut frames = Vec::new();
        frames
            .try_reserve(frame_count as usize)
            .map_err(|_| MMD_COMMON_ERROR_OUT_OF_MEMORY)?;
        for _ in 0..frame_count {
            frames.push(PmxFrame::read_with_setting(parser, setting)?);
        }

        // read rigid bodies
        let rigid_body_count = parser
            .read_u32()
            .map_err(|e| PmxParseError::map_end_of_stream_error(e, "PMX rigid body count"))?;
        let mut rigid_bodies = Vec::new();
        rigid_bodies
            .try_reserve(rigid_body_count as usize)
            .map_err(|_| MMD_COMMON_ERROR_OUT_OF_MEMORY)?;
        for _ in 0..rigid_body_count {
            rigid_bodies.push(PmxRigidBody::read_with_setting(parser, setting)?);
        }

        // read joints
        let joint_count = parser
            .read_u32()
            .map_err(|e| PmxParseError::map_end_of_stream_error(e, "PMX joint count"))?;
        let mut joints = Vec::new();
        joints
            .try_reserve(joint_count as usize)
            .map_err(|_| MMD_COMMON_ERROR_OUT_OF_MEMORY)?;
        for _ in 0..joint_count {
            joints.push(PmxJoint::read_with_setting(parser, setting)?);
        }

        // read soft bodies
        let mut soft_bodies = Vec::new();
        if version == 2.1 {
            let soft_body_count = parser
                .read_u32()
                .map_err(|e| PmxParseError::map_end_of_stream_error(e, "PMX soft body count"))?;
            soft_bodies
                .try_reserve(soft_body_count as usize)
                .map_err(|_| MMD_COMMON_ERROR_OUT_OF_MEMORY)?;
            for _ in 0..soft_body_count {
                soft_bodies.push(PmxSoftBody::read_with_setting(parser, setting)?);
            }
        }
        Ok(PmxModel {
            version,
            setting,
            model_name,
            model_english_name,
            model_comment,
            model_english_comment,
            vertices,
            indices,
            textures,
            materials,
            bones,
            morphs,
            frames,
            rigid_bodies,
            joints,
            soft_bodies,
        })
    }
}
