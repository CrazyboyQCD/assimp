use core::mem;

use glam::{Vec3, Vec4};

use crate::{
    assets::formats::mmd::{
        STRING_ENCODINGS,
        parser::{
            error::{MMD_COMMON_ERROR_OUT_OF_MEMORY, MMDParseCommonError, MMDParseError},
            vmd::{VMDParser, VMDRead, error::VmdParseError},
        },
    },
    io::reader::error::MappingPartEndOfStreamError,
};

#[derive(Clone, Debug, Default)]
pub struct VmdBoneFrame {
    pub name: String,
    pub frame: u32,
    pub position: Vec3,
    pub orientation: Vec4,
    pub interpolation: [[[u8; 4]; 4]; 4],
}

impl VMDRead for VmdBoneFrame {
    fn read(parser: &mut VMDParser<'_>) -> Result<Self, MMDParseError> {
        let name = parser
            .read_string::<15>()
            .map_err(|_| VmdParseError::UnexpectedEnd("bone name"))?;
        let frame = parser
            .read_u32()
            .map_err(|_| VmdParseError::UnexpectedEnd("bone frame"))?;
        let position = parser
            .read_vec3()
            .map_err(|_| VmdParseError::UnexpectedEnd("bone position"))?;
        let orientation = parser
            .read_vec4()
            .map_err(|_| VmdParseError::UnexpectedEnd("bone orientation"))?;

        let interpolation = {
            let mut interpolation: [u8; 4 * 4 * 4] = [0; 4 * 4 * 4];
            parser
                .read_into_buffer(&mut interpolation)
                .map_err(|_| VmdParseError::UnexpectedEnd("bone interpolation"))?;
            let mut new_interpolation = [[[0; 4]; 4]; 4];
            for i in 0..4 {
                for j in 0..4 {
                    for k in 0..4 {
                        new_interpolation[i][j][k] = interpolation[i * 16 + j * 4 + k];
                    }
                }
            }
            new_interpolation
        };

        Ok(Self {
            name,
            frame,
            position,
            orientation,
            interpolation,
        })
    }
}

#[derive(Clone, Debug, Default)]
pub struct VmdFaceFrame {
    pub face_name: String,
    pub weight: f32,
    pub frame: u32,
}

impl VMDRead for VmdFaceFrame {
    fn read(parser: &mut VMDParser<'_>) -> Result<Self, MMDParseError> {
        let face_name = parser
            .read_string::<15>()
            .map_err(|e| VmdParseError::map_end_of_stream_error(e, "face name"))?;
        let weight = parser
            .read_f32()
            .map_err(|_| VmdParseError::UnexpectedEnd("face weight"))?;
        let frame = parser
            .read_u32()
            .map_err(|_| VmdParseError::UnexpectedEnd("face frame"))?;

        Ok(Self {
            face_name,
            weight,
            frame,
        })
    }
}

#[derive(Clone, Debug, Default)]
pub struct VmdCameraFrame {
    pub frame: u32,
    pub distance: f32,
    pub position: Vec3,
    pub orientation: Vec3,
    pub interpolation: [[u8; 4]; 6],
    pub angle: f32,
    pub is_perspective: bool,
}

impl VMDRead for VmdCameraFrame {
    fn read(parser: &mut VMDParser<'_>) -> Result<Self, MMDParseError> {
        let frame = parser
            .read_u32()
            .map_err(|_| VmdParseError::UnexpectedEnd("camera frame"))?;
        let distance = parser
            .read_f32()
            .map_err(|_| VmdParseError::UnexpectedEnd("camera distance"))?;
        let position = parser
            .read_vec3()
            .map_err(|_| VmdParseError::UnexpectedEnd("camera position"))?;
        let orientation = parser
            .read_vec3()
            .map_err(|_| VmdParseError::UnexpectedEnd("camera orientation"))?;
        let interpolation = {
            let mut interpolation: [u8; 4 * 6] = [0; 4 * 6];
            parser
                .read_into_buffer(&mut interpolation)
                .map_err(|_| VmdParseError::UnexpectedEnd("camera interpolation"))?;
            let mut new_interpolation = [[0; 4]; 6];
            for i in 0..6 {
                for j in 0..4 {
                    new_interpolation[i][j] = interpolation[i * 4 + j];
                }
            }
            new_interpolation
        };
        let angle = parser
            .read_f32()
            .map_err(|_| VmdParseError::UnexpectedEnd("camera angle"))?;
        let is_perspective = parser
            .read_u8()
            .map_err(|_| VmdParseError::UnexpectedEnd("camera perspective"))?
            > 0;

        Ok(Self {
            frame,
            distance,
            position,
            orientation,
            interpolation,
            angle,
            is_perspective,
        })
    }
}

#[derive(Clone, Debug, Default)]
pub struct VmdLightFrame {
    pub frame: u32,
    pub color: Vec3,
    pub position: Vec3,
}

impl VMDRead for VmdLightFrame {
    fn read(parser: &mut VMDParser<'_>) -> Result<Self, MMDParseError> {
        let frame = parser
            .read_u32()
            .map_err(|_| VmdParseError::UnexpectedEnd("light frame"))?;
        let color = parser
            .read_vec3()
            .map_err(|_| VmdParseError::UnexpectedEnd("light color"))?;
        let position = parser
            .read_vec3()
            .map_err(|_| VmdParseError::UnexpectedEnd("light position"))?;
        Ok(Self {
            frame,
            color,
            position,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum VmdShadowType {
    Off = 0,
    Mode1 = 1,
    Mode2 = 2,
}

impl TryFrom<u8> for VmdShadowType {
    type Error = VmdParseError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Off),
            1 => Ok(Self::Mode1),
            2 => Ok(Self::Mode2),
            other => Err(VmdParseError::UnknownShadowFrameType(other)),
        }
    }
}

#[derive(Clone, Debug)]
pub struct VmdShadowFrame {
    pub frame: u32,
    pub shadow_type: VmdShadowType,
    pub dist: u32,
}

impl VMDRead for VmdShadowFrame {
    fn read(parser: &mut VMDParser<'_>) -> Result<Self, MMDParseError> {
        let frame = parser
            .read_u32()
            .map_err(|_| VmdParseError::UnexpectedEnd("shadow frame"))?;
        let shadow_type = VmdShadowType::try_from(
            parser
                .read_u8()
                .map_err(|_| VmdParseError::UnexpectedEnd("shadow type"))?,
        )?;
        let dist = parser
            .read_u32()
            .map_err(|_| VmdParseError::UnexpectedEnd("shadow dist"))?;
        Ok(Self {
            frame,
            shadow_type,
            dist,
        })
    }
}

#[derive(Clone, Debug, Default)]
pub struct VmdIkEnable {
    pub ik_name: String,
    pub enable: bool,
}

impl VMDRead for VmdIkEnable {
    fn read(parser: &mut VMDParser<'_>) -> Result<Self, MMDParseError> {
        let ik_name = parser
            .read_string::<20>()
            .map_err(|e| VmdParseError::map_end_of_stream_error(e, "ik name"))?;
        let enable = parser
            .read_u8()
            .map_err(|_| VmdParseError::UnexpectedEnd("ik enable"))?
            > 0;
        Ok(Self { ik_name, enable })
    }
}

#[derive(Clone, Debug, Default)]
pub struct VmdIkFrame {
    pub frame: u32,
    pub display: bool,
    pub ik_enable: Vec<VmdIkEnable>,
}

impl VMDRead for VmdIkFrame {
    fn read(parser: &mut VMDParser<'_>) -> Result<Self, MMDParseError> {
        let frame = parser
            .read_u32()
            .map_err(|_| VmdParseError::UnexpectedEnd("ik frame"))?;
        let display = parser
            .read_u8()
            .map_err(|_| VmdParseError::UnexpectedEnd("ik display"))?
            > 0;
        let ik_count = parser
            .read_u32()
            .map_err(|_| VmdParseError::UnexpectedEnd("ik count"))?;
        let mut ik_enable = Vec::new();
        ik_enable
            .try_reserve(ik_count as usize)
            .map_err(|_| MMD_COMMON_ERROR_OUT_OF_MEMORY)?;
        for _ in 0..ik_count {
            ik_enable.push(VmdIkEnable::read(parser)?);
        }
        Ok(Self {
            frame,
            display,
            ik_enable,
        })
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum VmdVersion {
    V1,
    #[default]
    V2,
}

#[derive(Clone, Debug, Default)]
pub struct VmdMotion {
    pub model_name: String,
    pub version: VmdVersion,
    pub bone_frames: Vec<VmdBoneFrame>,
    pub face_frames: Vec<VmdFaceFrame>,
    pub camera_frames: Vec<VmdCameraFrame>,
    pub light_frames: Vec<VmdLightFrame>,
    pub shadow_frames: Vec<VmdShadowFrame>,
    pub ik_frames: Vec<VmdIkFrame>,
}

impl VMDRead for VmdMotion {
    fn read(parser: &mut VMDParser<'_>) -> Result<Self, MMDParseError> {
        let mut buf = [0; 30];
        parser
            .read_into_buffer(&mut buf)
            .map_err(|_| VmdParseError::UnexpectedEnd("magic"))?;
        // compiler should be clever enough to remove this panic.
        let magic = &buf[..25];
        let version = if magic == b"Vocaloid Motion Data 0002" {
            VmdVersion::V2
        } else if magic == b"Vocaloid Motion Data file" {
            VmdVersion::V1
        } else {
            Err(VmdParseError::InvalidMagic(magic.to_vec()))?
        };

        let name_bytes = if version == VmdVersion::V1 {
            &mut buf[..10]
        } else {
            &mut buf[..20]
        };

        parser
            .read_into_buffer(name_bytes)
            .map_err(|_| VmdParseError::UnexpectedEnd("model name"))?;

        // Try decode with Multiple Encodings
        let model_name = 'blk: {
            for encoding in STRING_ENCODINGS {
                let (decoded, has_error) = encoding.decode_without_bom_handling(name_bytes);
                if !has_error {
                    break 'blk decoded.into_owned();
                }
            }
            Err(VmdParseError::UnknownEncodedString(name_bytes.to_vec()))?
        };

        // bone frames
        let bone_count = parser
            .read_u32()
            .map_err(|_| VmdParseError::UnexpectedEnd("bone count"))?;
        let mut bone_frames = Vec::new();
        bone_frames
            .try_reserve(bone_count as usize)
            .map_err(|_| MMD_COMMON_ERROR_OUT_OF_MEMORY)?;
        for _ in 0..bone_count {
            bone_frames.push(VmdBoneFrame::read(parser)?);
        }

        // face frames
        let face_count = parser
            .read_u32()
            .map_err(|_| VmdParseError::UnexpectedEnd("face count"))?;
        let mut face_frames = Vec::new();
        face_frames
            .try_reserve(face_count as usize)
            .map_err(|_| MMD_COMMON_ERROR_OUT_OF_MEMORY)?;
        for _ in 0..face_count {
            face_frames.push(VmdFaceFrame::read(parser)?);
        }

        // camera frames
        let mut camera_frames = Vec::new();
        if !parser.is_eof() {
            let camera_frame_count = parser
                .read_u32()
                .map_err(|_| VmdParseError::UnexpectedEnd("camera frame count"))?;
            camera_frames
                .try_reserve(camera_frame_count as usize)
                .map_err(|_| MMD_COMMON_ERROR_OUT_OF_MEMORY)?;
            for _ in 0..camera_frame_count {
                camera_frames.push(VmdCameraFrame::read(parser)?);
            }
        }

        // light frames
        let mut light_frames = Vec::new();
        if !parser.is_eof() {
            let light_frame_count = parser
                .read_u32()
                .map_err(|_| VmdParseError::UnexpectedEnd("light frame count"))?;
            light_frames
                .try_reserve(light_frame_count as usize)
                .map_err(|_| MMD_COMMON_ERROR_OUT_OF_MEMORY)?;
            for _ in 0..light_frame_count {
                light_frames.push(VmdLightFrame::read(parser)?);
            }
        }

        // shadow frames
        let mut shadow_frames = Vec::new();
        if !parser.is_eof() {
            let shadow_frame_count = parser
                .read_u32()
                .map_err(|_| VmdParseError::UnexpectedEnd("shadow frame count"))?;
            shadow_frames
                .try_reserve(shadow_frame_count as usize)
                .map_err(|_| MMD_COMMON_ERROR_OUT_OF_MEMORY)?;
            for _ in 0..shadow_frame_count {
                shadow_frames.push(VmdShadowFrame::read(parser)?);
            }
        }

        // ik frames
        let mut ik_frames = Vec::new();
        if !parser.is_eof() {
            let ik_frame_count = parser
                .read_u32()
                .map_err(|_| VmdParseError::UnexpectedEnd("ik frame count"))?;
            ik_frames
                .try_reserve(ik_frame_count as usize)
                .map_err(|_| MMD_COMMON_ERROR_OUT_OF_MEMORY)?;
            for _ in 0..ik_frame_count {
                ik_frames.push(VmdIkFrame::read(parser)?);
            }
        }

        Ok(Self {
            model_name,
            version,
            bone_frames,
            face_frames,
            camera_frames,
            light_frames,
            shadow_frames,
            ik_frames,
        })
    }
}
