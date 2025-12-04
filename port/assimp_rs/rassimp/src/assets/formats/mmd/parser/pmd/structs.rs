use glam::{Vec2, Vec3, Vec4};

use crate::{
    assets::formats::mmd::parser::{
        error::{MMD_COMMON_ERROR_OUT_OF_MEMORY, MMDParseCommonError, MMDParseError},
        pmd::{PMDParser, PMDRead, error::PmdParseError},
    },
    io::{importer::error::CommonImportError, reader::error::MappingPartEndOfStreamError},
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PmdHeader {
    pub name: String,
    pub name_english: String,
    pub comment: String,
    pub comment_english: String,
}

impl PMDRead for PmdHeader {
    fn read(parser: &mut PMDParser<'_>) -> Result<Self, MMDParseError> {
        let name = parser
            .read_string::<20>()
            .map_err(|e| PmdParseError::map_end_of_stream_error(e, "header name"))?;
        let name_english = String::new();
        let comment = parser
            .read_string::<256>()
            .map_err(|e| PmdParseError::map_end_of_stream_error(e, "header comment"))?;
        let comment_english = String::new();
        Ok(Self {
            name,
            name_english,
            comment,
            comment_english,
        })
    }

    fn read_extension(&mut self, parser: &mut PMDParser<'_>) -> Result<(), MMDParseError> {
        self.name_english = parser
            .read_string::<20>()
            .map_err(|e| PmdParseError::map_end_of_stream_error(e, "header name_english"))?;
        self.comment_english = parser
            .read_string::<256>()
            .map_err(|e| PmdParseError::map_end_of_stream_error(e, "header comment_english"))?;
        Ok(())
    }
}

pub struct PmdVertex {
    pub position: Vec3,
    pub normal: Vec3,
    pub uv: Vec2,
    pub bone_index: [u16; 2],
    pub bone_weight: u8,
    pub edge_invisible: bool,
}

impl PMDRead for PmdVertex {
    fn read(parser: &mut PMDParser<'_>) -> Result<Self, MMDParseError> {
        let position = parser
            .read_vec3()
            .map_err(|e| PmdParseError::map_end_of_stream_error(e, "vertex position"))?;
        let normal = parser
            .read_vec3()
            .map_err(|e| PmdParseError::map_end_of_stream_error(e, "vertex normal"))?;
        let uv = parser
            .read_vec2()
            .map_err(|e| PmdParseError::map_end_of_stream_error(e, "vertex uv"))?;
        let bone_index = [
            parser
                .read_u16()
                .map_err(|e| PmdParseError::map_end_of_stream_error(e, "vertex bone index 1"))?,
            parser
                .read_u16()
                .map_err(|e| PmdParseError::map_end_of_stream_error(e, "vertex bone index 2"))?,
        ];
        let bone_weight = parser
            .read_u8()
            .map_err(|e| PmdParseError::map_end_of_stream_error(e, "vertex bone weight"))?;
        let edge_invisible = parser
            .read_u8()
            .map_err(|e| PmdParseError::map_end_of_stream_error(e, "vertex edge invisible"))?
            > 0;
        Ok(Self {
            position,
            normal,
            uv,
            bone_index,
            bone_weight,
            edge_invisible,
        })
    }
}

pub struct PmdMaterial {
    pub diffuse: Vec4,
    pub power: f32,
    pub specular: Vec3,
    pub ambient: Vec3,
    pub toon_index: u8,
    pub edge_flag: u8,
    pub index_count: u32,
    pub texture_filename: String,
    pub sphere_filename: String,
}

impl PMDRead for PmdMaterial {
    fn read(parser: &mut PMDParser<'_>) -> Result<Self, MMDParseError> {
        let diffuse = parser
            .read_vec4()
            .map_err(|e| PmdParseError::map_end_of_stream_error(e, "material diffuse"))?;
        let power = parser
            .read_f32()
            .map_err(|e| PmdParseError::map_end_of_stream_error(e, "material power"))?;
        let specular = parser
            .read_vec3()
            .map_err(|e| PmdParseError::map_end_of_stream_error(e, "material specular"))?;
        let ambient = parser
            .read_vec3()
            .map_err(|e| PmdParseError::map_end_of_stream_error(e, "material ambient"))?;
        let toon_index = parser
            .read_u8()
            .map_err(|e| PmdParseError::map_end_of_stream_error(e, "material toon index"))?;
        let edge_flag = parser
            .read_u8()
            .map_err(|e| PmdParseError::map_end_of_stream_error(e, "material edge flag"))?;
        let index_count = parser
            .read_u32()
            .map_err(|e| PmdParseError::map_end_of_stream_error(e, "material index count"))?;
        let buf = parser
            .read_string::<20>()
            .map_err(|e| PmdParseError::map_end_of_stream_error(e, "material texture filename"))?;
        let mut iter = buf.split('*');
        let (texture_filename, sphere_filename) = match iter.next() {
            Some(texture_filename)
                if !texture_filename.is_empty() && texture_filename.len() < 20 =>
            {
                let sphere_filename = iter.next().unwrap_or_default();
                (texture_filename.to_owned(), sphere_filename.to_owned())
            }
            _ => {
                if let Some((texture_filename, sphere_filename)) = buf.split_at_checked(1) {
                    (texture_filename.to_owned(), sphere_filename.to_owned())
                } else {
                    (String::new(), String::new())
                }
            }
        };
        Ok(Self {
            diffuse,
            power,
            specular,
            ambient,
            toon_index,
            edge_flag,
            index_count,
            texture_filename,
            sphere_filename,
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BoneType {
    Rotation = 0,
    RotationAndMove = 1,
    IkEffector = 2,
    Unknown = 3,
    IkEffectable = 4,
    RotationEffectable = 5,
    IkTarget = 6,
    Invisible = 7,
    Twist = 8,
    RotationMovement = 9,
}

impl TryFrom<u8> for BoneType {
    type Error = PmdParseError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Rotation),
            1 => Ok(Self::RotationAndMove),
            2 => Ok(Self::IkEffector),
            3 => Ok(Self::Unknown),
            4 => Ok(Self::IkEffectable),
            5 => Ok(Self::RotationEffectable),
            6 => Ok(Self::IkTarget),
            7 => Ok(Self::Invisible),
            8 => Ok(Self::Twist),
            9 => Ok(Self::RotationMovement),
            other => Err(PmdParseError::InvalidBoneType(other)),
        }
    }
}
pub struct PmdBone {
    pub name: String,
    pub name_english: String,
    pub parent_bone_index: u16,
    pub tail_pos_bone_index: u16,
    pub bone_type: BoneType,
    pub ik_parent_bone_index: u16,
    pub bone_head_pos: Vec3,
}

impl PMDRead for PmdBone {
    fn read(parser: &mut PMDParser<'_>) -> Result<Self, MMDParseError> {
        let name = parser
            .read_string::<20>()
            .map_err(|e| PmdParseError::map_end_of_stream_error(e, "bone name"))?;
        let name_english = String::new();
        let parent_bone_index = parser
            .read_u16()
            .map_err(|e| PmdParseError::map_end_of_stream_error(e, "bone parent bone index"))?;
        let tail_pos_bone_index = parser
            .read_u16()
            .map_err(|e| PmdParseError::map_end_of_stream_error(e, "bone tail pos bone index"))?;
        let bone_type = BoneType::try_from(
            parser
                .read_u8()
                .map_err(|e| PmdParseError::map_end_of_stream_error(e, "bone type"))?,
        )?;
        let ik_parent_bone_index = parser
            .read_u16()
            .map_err(|e| PmdParseError::map_end_of_stream_error(e, "bone ik parent bone index"))?;
        let bone_head_pos = parser
            .read_vec3()
            .map_err(|e| PmdParseError::map_end_of_stream_error(e, "bone head pos"))?;
        Ok(Self {
            name,
            name_english,
            parent_bone_index,
            tail_pos_bone_index,
            bone_type,
            ik_parent_bone_index,
            bone_head_pos,
        })
    }

    fn read_expantion(&mut self, parser: &mut PMDParser<'_>) -> Result<(), MMDParseError> {
        self.name_english = parser
            .read_string::<20>()
            .map_err(|e| PmdParseError::map_end_of_stream_error(e, "bone name_english"))?;
        Ok(())
    }
}

pub struct PmdIk {
    pub ik_bone_index: u16,
    pub target_bone_index: u16,
    pub iterations: u16,
    pub angle_limit: f32,
    pub ik_child_bone_index: Vec<u16>,
}

impl PMDRead for PmdIk {
    fn read(parser: &mut PMDParser<'_>) -> Result<Self, MMDParseError> {
        let ik_bone_index = parser
            .read_u16()
            .map_err(|e| PmdParseError::map_end_of_stream_error(e, "ik bone index"))?;
        let target_bone_index = parser
            .read_u16()
            .map_err(|e| PmdParseError::map_end_of_stream_error(e, "ik target bone index"))?;
        let ik_chain_length = parser
            .read_u8()
            .map_err(|e| PmdParseError::map_end_of_stream_error(e, "ik chain length"))?;
        let iterations = parser
            .read_u16()
            .map_err(|e| PmdParseError::map_end_of_stream_error(e, "ik iterations"))?;
        let angle_limit = parser
            .read_f32()
            .map_err(|e| PmdParseError::map_end_of_stream_error(e, "ik angle limit"))?;
        let mut ik_child_bone_index = Vec::new();
        ik_child_bone_index
            .try_reserve(ik_chain_length as usize)
            .map_err(|_| MMD_COMMON_ERROR_OUT_OF_MEMORY)?;
        for _ in 0..ik_chain_length {
            ik_child_bone_index.push(
                parser.read_u16().map_err(|e| {
                    PmdParseError::map_end_of_stream_error(e, "ik child bone index")
                })?,
            );
        }
        Ok(Self {
            ik_bone_index,
            target_bone_index,
            iterations,
            angle_limit,
            ik_child_bone_index,
        })
    }
}

pub struct PmdFaceVertex {
    pub vertex_index: u16,
    pub position: Vec3,
}

impl PMDRead for PmdFaceVertex {
    fn read(parser: &mut PMDParser<'_>) -> Result<Self, MMDParseError> {
        let vertex_index = parser
            .read_u16()
            .map_err(|e| PmdParseError::map_end_of_stream_error(e, "face vertex index"))?;
        let position = parser
            .read_vec3()
            .map_err(|e| PmdParseError::map_end_of_stream_error(e, "face vertex position"))?;
        Ok(Self {
            vertex_index,
            position,
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FaceCategory {
    Base = 0,
    Eyebrow = 1,
    Eye = 2,
    Mouth = 3,
    Other = 4,
}

impl TryFrom<u8> for FaceCategory {
    type Error = PmdParseError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Base),
            1 => Ok(Self::Eyebrow),
            2 => Ok(Self::Eye),
            3 => Ok(Self::Mouth),
            4 => Ok(Self::Other),
            other => Err(PmdParseError::InvalidFaceCategory(other)),
        }
    }
}

pub struct PmdFace {
    pub name: String,
    pub face_category: FaceCategory,
    pub vertices: Vec<PmdFaceVertex>,
    pub name_english: String,
}

impl PMDRead for PmdFace {
    fn read(parser: &mut PMDParser<'_>) -> Result<Self, MMDParseError> {
        let name = parser
            .read_string::<20>()
            .map_err(|e| PmdParseError::map_end_of_stream_error(e, "face name"))?;
        let vertex_count = parser
            .read_u32()
            .map_err(|e| PmdParseError::map_end_of_stream_error(e, "face vertex count"))?;
        let face_category = FaceCategory::try_from(
            parser
                .read_u8()
                .map_err(|e| PmdParseError::map_end_of_stream_error(e, "face category"))?,
        )?;
        let mut vertices = Vec::new();
        vertices
            .try_reserve(vertex_count as usize)
            .map_err(|_| MMD_COMMON_ERROR_OUT_OF_MEMORY)?;
        for _ in 0..vertex_count {
            vertices.push(PmdFaceVertex::read(parser)?);
        }
        let name_english = String::new();
        Ok(Self {
            name,
            face_category,
            vertices,
            name_english,
        })
    }

    fn read_expantion(&mut self, parser: &mut PMDParser<'_>) -> Result<(), MMDParseError> {
        self.name_english = parser
            .read_string::<20>()
            .map_err(|e| PmdParseError::map_end_of_stream_error(e, "face name_english"))?;
        Ok(())
    }
}

pub struct PmdBoneDispName {
    pub bone_disp_name: String,
    pub bone_disp_name_english: String,
}

impl PMDRead for PmdBoneDispName {
    fn read(parser: &mut PMDParser<'_>) -> Result<Self, MMDParseError> {
        let bone_disp_name = parser
            .read_string::<20>()
            .map_err(|e| PmdParseError::map_end_of_stream_error(e, "bone disp name"))?;
        let bone_disp_name_english = String::new();
        Ok(Self {
            bone_disp_name,
            bone_disp_name_english,
        })
    }

    fn read_expantion(&mut self, parser: &mut PMDParser<'_>) -> Result<(), MMDParseError> {
        self.bone_disp_name_english = parser
            .read_string::<20>()
            .map_err(|e| PmdParseError::map_end_of_stream_error(e, "bone disp name english"))?;
        Ok(())
    }
}

pub struct PmdBoneDisp {
    pub bone_index: u16,
    pub bone_disp_index: u8,
}

impl PMDRead for PmdBoneDisp {
    fn read(parser: &mut PMDParser<'_>) -> Result<Self, MMDParseError> {
        let bone_index = parser
            .read_u16()
            .map_err(|e| PmdParseError::map_end_of_stream_error(e, "bone disp bone index"))?;
        let bone_disp_index = parser
            .read_u8()
            .map_err(|e| PmdParseError::map_end_of_stream_error(e, "bone disp index"))?;
        Ok(Self {
            bone_index,
            bone_disp_index,
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RigidBodyShape {
    Sphere = 0,
    Box = 1,
    Cpusel = 2,
}

impl TryFrom<u8> for RigidBodyShape {
    type Error = PmdParseError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Sphere),
            1 => Ok(Self::Box),
            2 => Ok(Self::Cpusel),
            other => Err(PmdParseError::InvalidRigidBodyShape(other)),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RigidBodyType {
    BoneConnected = 0,
    Physics = 1,
    ConnectedPhysics = 2,
}

impl TryFrom<u8> for RigidBodyType {
    type Error = PmdParseError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::BoneConnected),
            1 => Ok(Self::Physics),
            2 => Ok(Self::ConnectedPhysics),
            other => Err(PmdParseError::InvalidRigidBodyType(other)),
        }
    }
}

pub struct PmdRigidBody {
    pub name: String,
    pub related_bone_index: u16,
    pub group_index: u8,
    pub mask: u16,
    pub shape: RigidBodyShape,
    pub size: Vec3,
    pub position: Vec3,
    pub orientation: Vec3,
    pub weight: f32,
    pub linear_damping: f32,
    pub anglar_damping: f32,
    pub restitution: f32,
    pub friction: f32,
    pub rigid_type: RigidBodyType,
}

impl PMDRead for PmdRigidBody {
    fn read(parser: &mut PMDParser<'_>) -> Result<Self, MMDParseError> {
        let name = parser
            .read_string::<20>()
            .map_err(|e| PmdParseError::map_end_of_stream_error(e, "rigid body name"))?;
        let related_bone_index = parser.read_u16().map_err(|e| {
            PmdParseError::map_end_of_stream_error(e, "rigid body related bone index")
        })?;
        let group_index = parser
            .read_u8()
            .map_err(|e| PmdParseError::map_end_of_stream_error(e, "rigid body group index"))?;
        let mask = parser
            .read_u16()
            .map_err(|e| PmdParseError::map_end_of_stream_error(e, "rigid body mask"))?;
        let shape = RigidBodyShape::try_from(
            parser
                .read_u8()
                .map_err(|e| PmdParseError::map_end_of_stream_error(e, "rigid body shape"))?,
        )?;
        let size = parser
            .read_vec3()
            .map_err(|e| PmdParseError::map_end_of_stream_error(e, "rigid body size"))?;
        let position = parser
            .read_vec3()
            .map_err(|e| PmdParseError::map_end_of_stream_error(e, "rigid body position"))?;
        let orientation = parser
            .read_vec3()
            .map_err(|e| PmdParseError::map_end_of_stream_error(e, "rigid body orientation"))?;
        let weight = parser
            .read_f32()
            .map_err(|e| PmdParseError::map_end_of_stream_error(e, "rigid body weight"))?;
        let linear_damping = parser
            .read_f32()
            .map_err(|e| PmdParseError::map_end_of_stream_error(e, "rigid body linear damping"))?;
        let anglar_damping = parser
            .read_f32()
            .map_err(|e| PmdParseError::map_end_of_stream_error(e, "rigid body anglar damping"))?;
        let restitution = parser
            .read_f32()
            .map_err(|e| PmdParseError::map_end_of_stream_error(e, "rigid body restitution"))?;
        let friction = parser
            .read_f32()
            .map_err(|e| PmdParseError::map_end_of_stream_error(e, "rigid body friction"))?;
        let rigid_type = RigidBodyType::try_from(
            parser
                .read_u8()
                .map_err(|e| PmdParseError::map_end_of_stream_error(e, "rigid body type"))?,
        )?;
        Ok(Self {
            name,
            related_bone_index,
            group_index,
            mask,
            shape,
            size,
            position,
            orientation,
            weight,
            linear_damping,
            anglar_damping,
            restitution,
            friction,
            rigid_type,
        })
    }
}

pub struct PmdConstraint {
    pub name: String,
    pub rigid_body_index_a: u32,
    pub rigid_body_index_b: u32,
    pub position: Vec3,
    pub orientation: Vec3,
    pub linear_lower_limit: Vec3,
    pub linear_upper_limit: Vec3,
    pub angular_lower_limit: Vec3,
    pub angular_upper_limit: Vec3,
    pub linear_stiffness: Vec3,
}

impl PMDRead for PmdConstraint {
    fn read(parser: &mut PMDParser<'_>) -> Result<Self, MMDParseError> {
        let name = parser
            .read_string::<20>()
            .map_err(|e| PmdParseError::map_end_of_stream_error(e, "constraint name"))?;
        let rigid_body_index_a = parser.read_u32().map_err(|e| {
            PmdParseError::map_end_of_stream_error(e, "constraint rigid body index a")
        })?;
        let rigid_body_index_b = parser.read_u32().map_err(|e| {
            PmdParseError::map_end_of_stream_error(e, "constraint rigid body index b")
        })?;
        let position = parser
            .read_vec3()
            .map_err(|e| PmdParseError::map_end_of_stream_error(e, "constraint position"))?;
        let orientation = parser
            .read_vec3()
            .map_err(|e| PmdParseError::map_end_of_stream_error(e, "constraint orientation"))?;
        let linear_lower_limit = parser.read_vec3().map_err(|e| {
            PmdParseError::map_end_of_stream_error(e, "constraint linear lower limit")
        })?;
        let linear_upper_limit = parser.read_vec3().map_err(|e| {
            PmdParseError::map_end_of_stream_error(e, "constraint linear upper limit")
        })?;
        let angular_lower_limit = parser.read_vec3().map_err(|e| {
            PmdParseError::map_end_of_stream_error(e, "constraint angular lower limit")
        })?;
        let angular_upper_limit = parser.read_vec3().map_err(|e| {
            PmdParseError::map_end_of_stream_error(e, "constraint angular upper limit")
        })?;
        let linear_stiffness = parser.read_vec3().map_err(|e| {
            PmdParseError::map_end_of_stream_error(e, "constraint linear stiffness")
        })?;
        Ok(Self {
            name,
            rigid_body_index_a,
            rigid_body_index_b,
            position,
            orientation,
            linear_lower_limit,
            linear_upper_limit,
            angular_lower_limit,
            angular_upper_limit,
            linear_stiffness,
        })
    }
}

pub struct PmdModel {
    pub version: f32,
    pub header: PmdHeader,
    pub vertices: Vec<PmdVertex>,
    pub indices: Vec<u16>,
    pub materials: Vec<PmdMaterial>,
    pub bones: Vec<PmdBone>,
    pub iks: Vec<PmdIk>,
    pub faces: Vec<PmdFace>,
    pub faces_indices: Vec<u16>,
    pub bone_disp_name: Vec<PmdBoneDispName>,
    pub bone_disp: Vec<PmdBoneDisp>,
    pub toon_filenames: Vec<String>,
    pub rigid_bodies: Vec<PmdRigidBody>,
    pub constraints: Vec<PmdConstraint>,
}

impl PMDRead for PmdModel {
    fn read(parser: &mut PMDParser<'_>) -> Result<Self, MMDParseError> {
        // magic
        let mut magic = [0u8; 3];
        parser.read_into_buffer(&mut magic)?;
        if magic != *b"Pmd" {
            Err(PmdParseError::InvalidMagic(magic.to_vec()))?;
        }

        // version
        let version = parser
            .read_f32()
            .map_err(|e| PmdParseError::map_end_of_stream_error(e, "model version"))?;
        if version != 1.0 {
            Err(PmdParseError::InvalidVersion(version))?;
        }

        // header
        let mut header = PmdHeader::read(parser)?;

        let vertex_count = parser
            .read_u32()
            .map_err(|e| PmdParseError::map_end_of_stream_error(e, "model vertex count"))?;
        let mut vertices = Vec::new();
        vertices
            .try_reserve(vertex_count as usize)
            .map_err(|_| MMD_COMMON_ERROR_OUT_OF_MEMORY)?;
        for _ in 0..vertex_count {
            vertices.push(PmdVertex::read(parser)?);
        }

        // indices
        let index_count = parser
            .read_u32()
            .map_err(|e| PmdParseError::map_end_of_stream_error(e, "model index count"))?;
        let mut indices = Vec::new();
        indices
            .try_reserve(index_count as usize)
            .map_err(|_| MMD_COMMON_ERROR_OUT_OF_MEMORY)?;
        for _ in 0..index_count {
            indices.push(
                parser
                    .read_u16()
                    .map_err(|e| PmdParseError::map_end_of_stream_error(e, "model index"))?,
            );
        }

        // materials
        let material_count = parser
            .read_u32()
            .map_err(|e| PmdParseError::map_end_of_stream_error(e, "model material count"))?;
        let mut materials = Vec::new();
        materials
            .try_reserve(material_count as usize)
            .map_err(|_| MMD_COMMON_ERROR_OUT_OF_MEMORY)?;
        for _ in 0..material_count {
            materials.push(PmdMaterial::read(parser)?);
        }

        // bones
        let bone_count = parser
            .read_u16()
            .map_err(|e| PmdParseError::map_end_of_stream_error(e, "model bone count"))?;
        let mut bones = Vec::new();
        bones
            .try_reserve(bone_count as usize)
            .map_err(|_| MMD_COMMON_ERROR_OUT_OF_MEMORY)?;
        for _ in 0..bone_count {
            bones.push(PmdBone::read(parser)?);
        }

        // iks
        let ik_count = parser
            .read_u16()
            .map_err(|e| PmdParseError::map_end_of_stream_error(e, "model ik count"))?;
        let mut iks = Vec::new();
        iks.try_reserve(ik_count as usize)
            .map_err(|_| MMD_COMMON_ERROR_OUT_OF_MEMORY)?;
        for _ in 0..ik_count {
            iks.push(PmdIk::read(parser)?);
        }

        // faces
        let face_count = parser
            .read_u16()
            .map_err(|e| PmdParseError::map_end_of_stream_error(e, "model face count"))?;
        let mut faces = Vec::new();
        faces
            .try_reserve(face_count as usize)
            .map_err(|_| MMD_COMMON_ERROR_OUT_OF_MEMORY)?;
        for _ in 0..face_count {
            faces.push(PmdFace::read(parser)?);
        }

        // face frames
        let face_frame_count = parser
            .read_u8()
            .map_err(|e| PmdParseError::map_end_of_stream_error(e, "model face frame count"))?;
        let mut faces_indices = Vec::new();
        faces_indices
            .try_reserve(face_frame_count as usize)
            .map_err(|_| MMD_COMMON_ERROR_OUT_OF_MEMORY)?;
        for _ in 0..face_frame_count {
            faces_indices.push(
                parser
                    .read_u16()
                    .map_err(|e| PmdParseError::map_end_of_stream_error(e, "model face index"))?,
            );
        }

        // bone names
        let bone_disp_num = parser
            .read_u8()
            .map_err(|e| PmdParseError::map_end_of_stream_error(e, "model bone disp num"))?;
        let mut bone_disp_name = Vec::new();
        bone_disp_name
            .try_reserve(bone_disp_num as usize)
            .map_err(|_| MMD_COMMON_ERROR_OUT_OF_MEMORY)?;
        for _ in 0..bone_disp_num {
            bone_disp_name.push(PmdBoneDispName::read(parser)?);
        }

        // bone frame
        let bone_frame_num = parser
            .read_u32()
            .map_err(|e| PmdParseError::map_end_of_stream_error(e, "model bone frame num"))?;
        let mut bone_disp = Vec::new();
        bone_disp
            .try_reserve(bone_frame_num as usize)
            .map_err(|_| MMD_COMMON_ERROR_OUT_OF_MEMORY)?;
        for _ in 0..bone_frame_num {
            bone_disp.push(PmdBoneDisp::read(parser)?);
        }

        // english name
        let english = parser
            .read_u8()
            .map_err(|e| PmdParseError::map_end_of_stream_error(e, "model english name"))?
            > 0;
        if english {
            header.read_extension(parser)?;
            for bone in &mut bones {
                bone.read_expantion(parser)?;
            }
            for face in &mut faces {
                if face.face_category == FaceCategory::Base {
                    continue;
                }
                face.read_expantion(parser)?;
            }
            for bone_disp_name in &mut bone_disp_name {
                bone_disp_name.read_expantion(parser)?;
            }
        }

        // toon textures
        let mut toon_filenames = Vec::new();
        if !parser.is_eof() {
            toon_filenames
                .try_reserve(10)
                .map_err(|_| MMD_COMMON_ERROR_OUT_OF_MEMORY)?;
            for _ in 0..10 {
                toon_filenames.push(parser.read_string::<100>().map_err(|e| {
                    PmdParseError::map_end_of_stream_error(e, "model toon filename")
                })?);
            }
        }

        // physics
        let mut rigid_bodies = Vec::new();
        let mut constraints = Vec::new();
        if !parser.is_eof() {
            let rigid_body_num = parser
                .read_u32()
                .map_err(|e| PmdParseError::map_end_of_stream_error(e, "model rigid body num"))?;
            rigid_bodies
                .try_reserve(rigid_body_num as usize)
                .map_err(|_| MMD_COMMON_ERROR_OUT_OF_MEMORY)?;
            for _ in 0..rigid_body_num {
                rigid_bodies.push(PmdRigidBody::read(parser)?);
            }
            let constraint_num = parser
                .read_u32()
                .map_err(|e| PmdParseError::map_end_of_stream_error(e, "model constraint num"))?;
            constraints
                .try_reserve(constraint_num as usize)
                .map_err(|_| MMD_COMMON_ERROR_OUT_OF_MEMORY)?;
            for _ in 0..constraint_num {
                constraints.push(PmdConstraint::read(parser)?);
            }
        }

        Ok(Self {
            version,
            header,
            vertices,
            indices,
            materials,
            bones,
            iks,
            faces,
            faces_indices,
            bone_disp_name,
            bone_disp,
            toon_filenames,
            rigid_bodies,
            constraints,
        })
    }
}
