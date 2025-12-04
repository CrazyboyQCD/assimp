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

//! Defines the helper data structures for importing XFiles

use alloc::{string::String, vec::Vec};
use core::array;

use crate::{
    AiMat4, AiQuat, AiReal, AiVec2, AiVec3, AiVec4,
    assets::formats::x::{
        error::XFileCommonParseError,
        parser::{ParserCtx, XFileParse, XFileParser, text_parser::TextParser},
    },
    structs::{
        animation::AiAnimInterpolation,
        color::{Color3D, Color4D},
        index::Index,
        key::{AiQuatKey, AiVectorKey},
        mesh::{AI_MAX_NUMBER_OF_COLOR_SETS, AI_MAX_NUMBER_OF_TEXTURECOORDS},
    },
};

struct ObjectHeader;

impl<'source> XFileParse<'source> for ObjectHeader {
    type Output = &'source [u8];
    fn parse<P: XFileParser<'source>>(
        parser: &mut ParserCtx<'source, P>,
    ) -> Result<Self::Output, XFileCommonParseError> {
        let name_or_brace = parser.next_token();
        if name_or_brace != b"{" {
            let next = parser.next_token();
            if next != b"{" {
                Err(XFileCommonParseError::unexpected_token("{", next))
            } else {
                Ok(name_or_brace)
            }
        } else {
            Ok(&[])
        }
    }
}

pub(super) struct UnknownDataObject;

impl UnknownDataObject {
    pub(super) fn parse<'source, P: XFileParser<'source>>(
        parser: &mut ParserCtx<'source, P>,
    ) -> Result<(), XFileCommonParseError> {
        // find opening delimiter
        loop {
            let token = parser.next_token();
            if token.is_empty() {
                return Err(XFileCommonParseError::unexpected_end_of_file(
                    "UnknownDataObject",
                ));
            }
            if token == b"{" {
                break;
            }
        }

        let mut left_braces = 1;

        // parse until closing delimiter
        while left_braces > 0 {
            let token = parser.next_token();
            if token.is_empty() {
                return Err(XFileCommonParseError::unexpected_end_of_file(
                    "UnknownDataObject",
                ));
            }
            left_braces = left_braces + usize::from(token == b"{") - usize::from(token == b"}");
        }
        Ok(())
    }
}

pub(super) struct Template;

impl Template {
    pub(super) fn parse<'source, P: XFileParser<'source>>(
        parser: &mut ParserCtx<'source, P>,
    ) -> Result<(), XFileCommonParseError> {
        let _name = ObjectHeader::parse(parser)?;
        let _guid = parser.next_token();

        loop {
            let token = parser.next_token();
            if token.is_empty() {
                return Err(XFileCommonParseError::unexpected_end_of_file("Template"));
            }

            if token == b"}" {
                return Ok(());
            }
        }
    }
}

pub(super) struct Frame;

impl<'source> XFileParse<'source> for Frame {
    type Output = ();
    fn parse<P: XFileParser<'source>>(
        parser: &mut ParserCtx<'source, P>,
    ) -> Result<Self::Output, XFileCommonParseError> {
        fn parse_inner<'source, P: XFileParser<'source>>(
            parser: &mut ParserCtx<'source, P>,
            parent: Option<Index<Node>>,
        ) -> Result<(), XFileCommonParseError> {
            let name = if let Ok(s) = ObjectHeader::parse(parser) {
                str::from_utf8(s).unwrap_or_default()
            } else {
                ""
            };
            let parent = parent.unwrap_or(Index::GUARD_INDEX);
            let mut node = Node::new(parent);
            node.name = name.to_owned();

            let node_index = parser.scene.push_node(parent, node);
            loop {
                let token = parser.next_token();
                if token.is_empty() {
                    return Err(XFileCommonParseError::unexpected_end_of_file("Frame"));
                }
                if token == b"}" {
                    break; // frame finished
                } else if token == b"Frame" {
                    parse_inner(parser, Some(node_index))?; // child frame
                } else if token == b"FrameTransformMatrix" {
                    let matrix = FrameTransformMatrix::parse(parser)?;
                    // SAFETY: node_index is guaranteed to be valid
                    let node = unsafe { node_index.get_mut_unchecked(&mut parser.scene.nodes) };
                    node.transformation_matrix = matrix;
                } else if token == b"Mesh" {
                    let mut mesh = Mesh::parse(parser)?;
                    mesh.name = name.to_owned();
                    // SAFETY: node_index is guaranteed to be valid
                    let node = unsafe { node_index.get_mut_unchecked(&mut parser.scene.nodes) };
                    node.meshes.push(mesh);
                } else {
                    UnknownDataObject::parse(parser)?;
                }
            }
            Ok(())
        }
        parse_inner(parser, None)
    }
}

struct FrameTransformMatrix;

impl<'source> XFileParse<'source> for FrameTransformMatrix {
    type Output = AiMat4;
    fn parse<P: XFileParser<'source>>(
        parser: &mut ParserCtx<'source, P>,
    ) -> Result<Self::Output, XFileCommonParseError> {
        ObjectHeader::parse(parser)?;
        let x1 = parser.read_float()?;
        let y1 = parser.read_float()?;
        let z1 = parser.read_float()?;
        let w1 = parser.read_float()?;
        let x2 = parser.read_float()?;
        let y2 = parser.read_float()?;
        let z2 = parser.read_float()?;
        let w2 = parser.read_float()?;
        let x3 = parser.read_float()?;
        let y3 = parser.read_float()?;
        let z3 = parser.read_float()?;
        let w3 = parser.read_float()?;
        let x4 = parser.read_float()?;
        let y4 = parser.read_float()?;
        let z4 = parser.read_float()?;
        let w4 = parser.read_float()?;
        parser.check_for_semicolon()?;
        parser.check_for_closing_brace()?;
        Ok(AiMat4::from_cols(
            AiVec4::new(x1, x2, x3, x4),
            AiVec4::new(y1, y2, y3, y4),
            AiVec4::new(z1, z2, z3, z4),
            AiVec4::new(w1, w2, w3, w4),
        ))
    }
}

/// Helper structure representing a XFile mesh face
#[derive(Clone, Debug, Default)]
pub struct Face {
    /// The indices of the face
    pub indices: Vec<u32>,
}

/// Helper structure representing a texture filename inside a material and its potential source
#[derive(Clone, Debug, Default)]
pub struct TexEntry {
    /// The name of the texture
    pub name: String,
    /// `true` if the [`texname`](TexEntry::name) was specified in a `NormalmapFilename` tag
    pub is_normal_map: bool,
}

impl TexEntry {
    /// Constructor for the texture entry
    pub fn new(name: String, is_normal_map: bool) -> Self {
        Self {
            name,
            is_normal_map,
        }
    }
}

/// Helper structure to represent a material in a XFile
#[derive(Clone, Debug)]
pub struct Material {
    /// The name of the material
    pub name: String,
    /// `true` if the [`name`](Material::name) holds a name by which the actual material can be
    /// found in the material list
    pub is_reference: bool,
    /// The diffuse color of the material
    pub diffuse: Color4D,
    /// The specular exponent of the material
    pub specular_exponent: AiReal,
    /// The specular color of the material
    pub specular: Color3D,
    /// The emissive color of the material
    pub emissive: Color3D,
    /// The textures of the material
    pub textures: Vec<TexEntry>,
    /// The index under which it was stored in the scene's material list
    pub scene_index: u32,
}

impl Default for Material {
    fn default() -> Self {
        Self {
            name: String::new(),
            is_reference: false,
            diffuse: Color4D::default(),
            specular_exponent: 0.0,
            specular: Color3D::default(),
            emissive: Color3D::default(),
            textures: Vec::new(),
            scene_index: u32::MAX,
        }
    }
}

impl Material {
    fn parse_material_texture_filename<'source, P: XFileParser<'source>>(
        parser: &mut ParserCtx<'source, P>,
    ) -> Result<String, XFileCommonParseError> {
        ObjectHeader::parse(parser)?;
        let name = parser.next_token_as_str()?.replace("\\\\", "\\");
        parser.check_for_closing_brace()?;
        Ok(name)
    }
}

impl<'source> XFileParse<'source> for Material {
    type Output = Self;
    fn parse<P: XFileParser<'source>>(
        parser: &mut ParserCtx<'source, P>,
    ) -> Result<Self::Output, XFileCommonParseError> {
        let mat_name = ObjectHeader::parse(parser)?;
        let name = if mat_name.is_empty() {
            format!("material_{}", parser.get_position())
        } else {
            str::from_utf8(mat_name)?.to_owned()
        };
        let diffuse = parser.read_rgba()?;
        let specular_exponent = parser.read_float()?;
        let specular = parser.read_rgb()?;
        let emissive = parser.read_rgb()?;
        let mut textures = Vec::new();
        // read other data objects
        loop {
            let token = parser.next_token();
            if token.is_empty() {
                return Err(XFileCommonParseError::unexpected_end_of_file("Material"));
            }
            if token == b"}" {
                break; // material finished
            }

            if token == b"TextureFilename" || token == b"TextureFileName" {
                // some exporters write "TextureFileName" instead.
                let tex_name = Self::parse_material_texture_filename(parser)?;
                textures.push(TexEntry::new(tex_name, false));
            } else if token == b"NormalmapFilename" || token == b"NormalmapFileName" {
                // one exporter writes out the normal map in a separate filename tag
                let tex_name = Self::parse_material_texture_filename(parser)?;
                textures.push(TexEntry::new(tex_name, true));
            } else {
                UnknownDataObject::parse(parser)?;
            }
        }
        Ok(Material {
            name,
            is_reference: false,
            diffuse,
            specular_exponent,
            specular,
            emissive,
            textures,
            scene_index: 0,
        })
    }
}

/// Helper structure to represent a bone weight in a XFile
#[derive(Clone, Copy, Debug, Default)]
pub struct BoneWeight {
    /// The index of the vertex.
    pub vertex: u32,
    /// The weight of the bone.
    pub weight: AiReal,
}

/// Helper structure to represent a bone in a XFile
#[derive(Clone, Debug, Default)]
pub struct Bone {
    /// The name of the bone.
    pub name: String,
    /// The weights of the bone.
    pub weights: Vec<BoneWeight>,
    /// The offset matrix of the bone.
    pub offset_matrix: AiMat4,
}

impl Bone {
    /// Constructor for the bone.
    pub fn new(name: String) -> Self {
        Self {
            name,
            weights: Vec::new(),
            offset_matrix: AiMat4::ZERO,
        }
    }
}

impl<'source> XFileParse<'source> for Bone {
    type Output = Self;
    fn parse<P: XFileParser<'source>>(
        parser: &mut ParserCtx<'source, P>,
    ) -> Result<Self, XFileCommonParseError> {
        ObjectHeader::parse(parser)?;

        let transform_node_name = parser.next_token_as_str()?;
        let mut bone = Self::new(transform_node_name.into_owned());

        // read vertex weights
        let num_weights = parser.read_int()?;
        bone.weights
            .try_reserve(num_weights as usize)
            .map_err(|_| XFileCommonParseError::InsufficientMemory)?;

        for _ in 0..num_weights {
            bone.weights.push(BoneWeight {
                vertex: parser.read_int()?,
                ..Default::default()
            });
        }

        // read vertex weights
        for weight in bone.weights.iter_mut() {
            weight.weight = parser.read_float()?;
        }

        // read matrix offset
        bone.offset_matrix.x_axis.x = parser.read_float()?;
        bone.offset_matrix.y_axis.x = parser.read_float()?;
        bone.offset_matrix.z_axis.x = parser.read_float()?;
        bone.offset_matrix.w_axis.x = parser.read_float()?;
        bone.offset_matrix.x_axis.y = parser.read_float()?;
        bone.offset_matrix.y_axis.y = parser.read_float()?;
        bone.offset_matrix.z_axis.y = parser.read_float()?;
        bone.offset_matrix.w_axis.y = parser.read_float()?;
        bone.offset_matrix.x_axis.z = parser.read_float()?;
        bone.offset_matrix.y_axis.z = parser.read_float()?;
        bone.offset_matrix.z_axis.z = parser.read_float()?;
        bone.offset_matrix.w_axis.z = parser.read_float()?;
        bone.offset_matrix.x_axis.w = parser.read_float()?;
        bone.offset_matrix.y_axis.w = parser.read_float()?;
        bone.offset_matrix.z_axis.w = parser.read_float()?;
        bone.offset_matrix.w_axis.w = parser.read_float()?;

        parser.check_for_semicolon()?;
        parser.check_for_closing_brace()?;

        Ok(bone)
    }
}

/// Helper structure to represent an XFile mesh
#[derive(Clone, Debug)]
pub struct Mesh {
    /// The name of the mesh.
    pub name: String,
    /// The positions of the mesh.
    pub positions: Vec<AiVec3>,
    /// The faces of the mesh.
    pub pos_faces: Vec<Face>,
    /// The normals of the mesh.
    pub normals: Vec<AiVec3>,
    /// The faces of the mesh.
    pub norm_faces: Vec<Face>,
    /// The number of textures of the mesh.
    pub num_textures: u32,
    /// The texture coordinates of the mesh.
    pub tex_coords: [Vec<AiVec2>; AI_MAX_NUMBER_OF_TEXTURECOORDS],
    /// The number of color sets of the mesh.
    pub num_color_sets: u32,
    /// The colors of the mesh.
    pub colors: [Vec<Color4D>; AI_MAX_NUMBER_OF_COLOR_SETS],

    /// The face materials of the mesh.
    pub face_materials: Vec<u32>,
    /// The materials of the mesh.
    pub materials: Vec<Material>,
    /// The bones of the mesh.
    pub bones: Vec<Bone>,
}

impl Default for Mesh {
    fn default() -> Self {
        Self {
            name: String::new(),
            positions: Vec::new(),
            pos_faces: Vec::new(),
            normals: Vec::new(),
            norm_faces: Vec::new(),
            num_textures: 0,
            tex_coords: array::from_fn(|_| Vec::new()),
            num_color_sets: 0,
            colors: array::from_fn(|_| Vec::new()),
            face_materials: Vec::new(),
            materials: Vec::new(),
            bones: Vec::new(),
        }
    }
}

impl Mesh {
    /// Constructor for the mesh.
    pub fn new(name: String) -> Self {
        Self {
            name,
            ..Default::default()
        }
    }

    fn parse_mesh_normals<'source, P: XFileParser<'source>>(
        &mut self,
        parser: &mut ParserCtx<'source, P>,
    ) -> Result<(), XFileCommonParseError> {
        ObjectHeader::parse(parser)?;

        // read count
        let num_of_normals = parser.read_int()?;
        if num_of_normals == 0 {
            return Ok(());
        }

        self.normals.resize(num_of_normals as usize, AiVec3::ZERO);

        // read normal vectors
        for normal in self.normals.iter_mut() {
            *normal = parser.read_vec3()?;
        }

        // read normal indices
        let num_of_indices = parser.read_int()?;
        if num_of_indices != self.pos_faces.len() as u32 {
            return Err(XFileCommonParseError::NormalFaceCountDoesNotMatchVertexFaceCount);
        }

        if num_of_indices > 0 {
            self.norm_faces
                .resize(num_of_indices as usize, Face::default());
            for face in self.norm_faces.iter_mut() {
                let num_indices = parser.read_int()?;
                *face = Face::default();
                face.indices
                    .try_reserve(num_indices as usize)
                    .map_err(|_| XFileCommonParseError::InsufficientMemory)?;
                for _ in 0..num_indices {
                    let idx = parser.read_int()?;
                    // if idx <= num_indices {
                    face.indices.push(idx);
                    // }
                }
                parser.test_for_separator();
            }
        }
        parser.check_for_closing_brace()?;
        Ok(())
    }

    fn parse_mesh_texture_coords<'source, P: XFileParser<'source>>(
        &mut self,
        parser: &mut ParserCtx<'source, P>,
    ) -> Result<(), XFileCommonParseError> {
        ObjectHeader::parse(parser)?;
        if self.num_textures + 1 > AI_MAX_NUMBER_OF_TEXTURECOORDS as u32 {
            return Err(XFileCommonParseError::TooManySetsOfTextureCoordinates);
        }

        let tex_coords = &mut self.tex_coords[self.num_textures as usize];
        self.num_textures += 1;
        let num_coords = parser.read_int()?;
        if num_coords != self.positions.len() as u32 {
            return Err(XFileCommonParseError::TextureCoordCountDoesNotMatchVertexCount);
        }

        tex_coords.resize(num_coords as usize, AiVec2::ZERO);
        for coord in tex_coords.iter_mut() {
            *coord = parser.read_vec2()?;
        }
        parser.check_for_closing_brace()?;
        Ok(())
    }

    fn parse_mesh_vertex_colors<'source, P: XFileParser<'source>>(
        &mut self,
        parser: &mut ParserCtx<'source, P>,
    ) -> Result<(), XFileCommonParseError> {
        ObjectHeader::parse(parser)?;
        let Some(colors) = self.colors.get_mut(self.num_color_sets as usize) else {
            return Err(XFileCommonParseError::TooManyColorSets);
        };
        self.num_color_sets += 1;
        let num_colors = parser.read_int()? as usize;
        if num_colors != self.positions.len() {
            return Err(XFileCommonParseError::VertexColorCountDoesNotMatchVertexCount);
        }

        *colors = vec![Color4D::default(); num_colors];
        for _ in 0..num_colors {
            let index = parser.read_int()? as usize;

            match colors.get_mut(index) {
                Some(color) => *color = parser.read_rgba()?,
                None => return Err(XFileCommonParseError::VertexColorIndexOutOfBounds),
            }
            // HACK: (thom) Maxon Cinema XPort plugin puts a third separator here, kwxPort puts a
            // comma. Ignore gracefully.
            parser.test_for_separator();
        }

        parser.check_for_closing_brace()?;
        Ok(())
    }

    fn parse_mesh_material_list<'source, P: XFileParser<'source>>(
        &mut self,
        parser: &mut ParserCtx<'source, P>,
    ) -> Result<(), XFileCommonParseError> {
        ObjectHeader::parse(parser)?;
        // read material count
        let _num_materials = parser.read_int()?;
        // read non triangulated face material index count
        let num_mat_indices = parser.read_int()? as usize;

        // some models have a material index count of 1... to be able to read them we
        // replicate this single material index on every face
        if num_mat_indices != self.pos_faces.len() && num_mat_indices != 1 {
            return Err(XFileCommonParseError::PerFaceMaterialIndexCountDoesNotMatchFaceCount);
        }

        // read per-face material indices
        for _ in 0..num_mat_indices {
            self.face_materials.push(parser.read_int()?);
        }

        parser.consume_version_specific_semicolon();

        // if there was only a single material index, replicate it on all faces
        if self.face_materials.len() < self.pos_faces.len() {
            self.face_materials.extend(core::iter::repeat_n(
                self.face_materials.first().copied().unwrap_or_default(),
                self.pos_faces.len() - self.face_materials.len(),
            ));
        }

        // read following data objects
        loop {
            let token = parser.next_token();
            if token.is_empty() {
                return Err(XFileCommonParseError::unexpected_end_of_file(
                    "MaterialList",
                ));
            }
            if token == b"}" {
                break; // material list finished
            } else if token == b"{" {
                // template materials
                let mat_name = parser.next_token();

                self.materials.push(Material {
                    is_reference: true,
                    name: str::from_utf8(mat_name)?.to_owned(),
                    ..Default::default()
                });

                parser.check_for_closing_brace()?; // skip }
            } else if token == b"Material" {
                self.materials.push(Material::parse(parser)?);
            } else if token == b";" {
                // ignore
            } else {
                UnknownDataObject::parse(parser)?;
            }
        }
        Ok(())
    }
}

impl<'source> XFileParse<'source> for Mesh {
    type Output = Self;
    fn parse<P: XFileParser<'source>>(
        parser: &mut ParserCtx<'source, P>,
    ) -> Result<Self::Output, XFileCommonParseError> {
        let mut mesh = Self::default();
        ObjectHeader::parse(parser)?;

        // read vertex count
        let num_of_vertices = parser.read_int()?;
        mesh.positions = Vec::new();
        mesh.positions
            .try_reserve(num_of_vertices as usize)
            .map_err(|_| XFileCommonParseError::InsufficientMemory)?;

        // read vertices
        for _ in 0..num_of_vertices {
            let v = parser.read_vec3()?;
            mesh.positions.push(v);
        }

        // read position faces
        let num_of_faces = parser.read_int()?;
        mesh.pos_faces = vec![Face::default(); num_of_faces as usize];
        for face in mesh.pos_faces.iter_mut() {
            // read indices
            let num_indices = parser.read_int()?;
            for _ in 0..num_indices {
                let idx = parser.read_int()?;
                if idx <= num_of_vertices {
                    face.indices.push(idx);
                }
            }
            parser.test_for_separator();
        }
        loop {
            let token = parser.next_token();
            if token.is_empty() {
                return Err(XFileCommonParseError::unexpected_end_of_file("Mesh"));
            }
            if token == b"}" {
                return Ok(mesh);
            }
            if token == b"MeshNormals" {
                mesh.parse_mesh_normals(parser)?;
            } else if token == b"MeshTextureCoords" {
                mesh.parse_mesh_texture_coords(parser)?;
            } else if token == b"MeshVertexColors" {
                mesh.parse_mesh_vertex_colors(parser)?;
            } else if token == b"MeshMaterialList" {
                mesh.parse_mesh_material_list(parser)?;
            } else if token == b"VertexDuplicationIndices" {
                UnknownDataObject::parse(parser)?;
            } else if token == b"XSkinMeshHeader" {
                SkinMeshHeader::parse(parser)?;
            } else if token == b"SkinWeights" {
                mesh.bones.push(Bone::parse(parser)?);
            }
            // else if token == b"DeclData" {
            //     DeclData::parse(parser)?;
            // }
            else {
                UnknownDataObject::parse(parser)?;
            }
        }
    }
}
struct SkinMeshHeader;

impl<'source> XFileParse<'source> for SkinMeshHeader {
    type Output = ();
    fn parse<P: XFileParser<'source>>(
        parser: &mut ParserCtx<'source, P>,
    ) -> Result<Self::Output, XFileCommonParseError> {
        ObjectHeader::parse(parser)?;
        let _max_skin_weights_per_vertex = parser.read_int()?;
        let _max_skin_weights_per_face = parser.read_int()?;
        let _num_bones_in_mesh = parser.read_int()?;
        parser.check_for_closing_brace()?;
        Ok(())
    }
}

/// Helper structure to represent a matrix key in a XFile
#[derive(Clone, Copy, Debug, Default)]
pub struct MatrixKey {
    /// The time of the matrix key.
    pub time: f64,
    /// The matrix of the matrix key.
    pub matrix: AiMat4,
}

/// Helper structure representing a single animated bone in a XFile
#[derive(Clone, Debug, Default)]
pub struct AnimBone {
    /// The name of the animated bone.
    pub name: String,
    /// The position keys of the animated bone.
    pub pos_keys: Vec<AiVectorKey>, /* either three separate key sequences for position,
                                     * rotation, scaling */
    /// The rotation keys of the animated bone.
    pub rot_keys: Vec<AiQuatKey>,
    /// The scale keys of the animated bone.
    pub scale_keys: Vec<AiVectorKey>,
    /// The transformation keys of the animated bone.
    pub trafo_keys: Vec<MatrixKey>, // or a combined key sequence of transformation matrices.
}

impl AnimBone {
    /// Constructor for the animated bone.
    pub fn new(name: String) -> Self {
        Self {
            name,
            pos_keys: Vec::new(),
            rot_keys: Vec::new(),
            scale_keys: Vec::new(),
            trafo_keys: Vec::new(),
        }
    }

    fn parse_anim_keys<'source, P: XFileParser<'source>>(
        &mut self,
        parser: &mut ParserCtx<'source, P>,
    ) -> Result<(), XFileCommonParseError> {
        ObjectHeader::parse(parser)?;

        // read key type
        let key_type = parser.read_int()?;

        // read number of keys
        let num_keys = parser.read_int()?;

        match key_type {
            0 => {
                self.rot_keys
                    .try_reserve(num_keys as usize)
                    .map_err(|_| XFileCommonParseError::InsufficientMemory)?;
            }
            1 => {
                self.scale_keys
                    .try_reserve(num_keys as usize)
                    .map_err(|_| XFileCommonParseError::InsufficientMemory)?;
            }
            2 => {
                self.pos_keys
                    .try_reserve(num_keys as usize)
                    .map_err(|_| XFileCommonParseError::InsufficientMemory)?;
            }
            3 | 4 => {
                self.trafo_keys
                    .try_reserve(num_keys as usize)
                    .map_err(|_| XFileCommonParseError::InsufficientMemory)?;
            }
            _ => {}
        }

        for _ in 0..num_keys {
            // read time
            let time = parser.read_int()?;
            // read keys
            match key_type {
                // rotation quaternion
                0 => {
                    // read count
                    let count = parser.read_int()? as usize;
                    if count != 4 {
                        return Err(
                            XFileCommonParseError::InvalidNumberOfArgumentsForKeyInAnimation {
                                key_type: "quaternion",
                                expected: 4,
                                found: count,
                            },
                        );
                    }
                    let w = parser.read_float()?;
                    let x = parser.read_float()?;
                    let y = parser.read_float()?;
                    let z = parser.read_float()?;
                    let key = AiQuatKey {
                        time: time as f64,
                        value: AiQuat::from_xyzw(x, y, z, w),
                        interpolation: AiAnimInterpolation::default(),
                    };

                    parser.check_for_semicolon()?;

                    self.rot_keys.push(key);
                }
                // scale vector | position vector
                1 | 2 => {
                    // read count
                    let count = parser.read_int()? as usize;
                    if count != 3 {
                        return Err(
                            XFileCommonParseError::InvalidNumberOfArgumentsForKeyInAnimation {
                                key_type: "vector",
                                expected: 3,
                                found: count,
                            },
                        );
                    }

                    let key = AiVectorKey {
                        time: time as f64,
                        value: parser.read_vec3()?,
                        interpolation: AiAnimInterpolation::default(),
                    };

                    if key_type == 2 {
                        self.pos_keys.push(key);
                    } else {
                        self.scale_keys.push(key);
                    }
                }

                // combined transformation matrix | denoted both as 3 or as 4
                3 | 4 => {
                    // read count
                    let count = parser.read_int()? as usize;
                    if count != 16 {
                        return Err(
                            XFileCommonParseError::InvalidNumberOfArgumentsForKeyInAnimation {
                                key_type: "matrix",
                                expected: 16,
                                found: count,
                            },
                        );
                    }

                    // read matrix
                    let x1 = parser.read_float()?;
                    let y1 = parser.read_float()?;
                    let z1 = parser.read_float()?;
                    let w1 = parser.read_float()?;
                    let x2 = parser.read_float()?;
                    let y2 = parser.read_float()?;
                    let z2 = parser.read_float()?;
                    let w2 = parser.read_float()?;
                    let x3 = parser.read_float()?;
                    let y3 = parser.read_float()?;
                    let z3 = parser.read_float()?;
                    let w3 = parser.read_float()?;
                    let x4 = parser.read_float()?;
                    let y4 = parser.read_float()?;
                    let z4 = parser.read_float()?;
                    let w4 = parser.read_float()?;
                    parser.check_for_semicolon()?;

                    self.trafo_keys.push(MatrixKey {
                        time: time as f64,
                        matrix: AiMat4::from_cols(
                            AiVec4::new(x1, x2, x3, x4),
                            AiVec4::new(y1, y2, y3, y4),
                            AiVec4::new(z1, z2, z3, z4),
                            AiVec4::new(w1, w2, w3, w4),
                        ),
                    });
                }

                _ => {
                    return Err(XFileCommonParseError::UnknownKeyTypeInAnimation(key_type));
                }
            }
            // key separator
            parser.check_for_separator()?;
        }
        parser.check_for_closing_brace()?;
        Ok(())
    }
}

impl<'source> XFileParse<'source> for AnimBone {
    type Output = Self;
    fn parse<P: XFileParser<'source>>(
        parser: &mut ParserCtx<'source, P>,
    ) -> Result<Self::Output, XFileCommonParseError> {
        let name_token = ObjectHeader::parse(parser)?;
        let name = str::from_utf8(name_token)
            .map(|s| s.to_owned())
            .unwrap_or_default();
        let mut banim = AnimBone::new(name);

        loop {
            let token = parser.next_token();
            if token.is_empty() {
                return Err(XFileCommonParseError::unexpected_end_of_file("Animation"));
            }
            if token == b"}" {
                break; // animation finished
            }
            if token == b"AnimationKey" {
                banim.parse_anim_keys(parser)?;
            } else if token == b"AnimationOptions" {
                UnknownDataObject::parse(parser)?; // not interested
            } else if token == b"{" {
                // read frame name
                let name = parser.next_token();
                banim.name = str::from_utf8(name)?.to_owned();
                parser.check_for_closing_brace()?;
            } else {
                UnknownDataObject::parse(parser)?;
            }
        }
        Ok(banim)
    }
}

/// Helper structure to represent an animation set in a XFile
#[derive(Clone, Debug, Default)]
pub struct Animation {
    /// The name of the animation.
    pub name: String,
    /// The animated bones of the animation.
    pub anims: Vec<AnimBone>,
}

impl Animation {
    /// Constructor for the animation.
    pub fn new(name: String) -> Self {
        Self {
            name,
            anims: Vec::new(),
        }
    }
}

impl<'source> XFileParse<'source> for Animation {
    type Output = Self;
    fn parse<P: XFileParser<'source>>(
        parser: &mut ParserCtx<'source, P>,
    ) -> Result<Self::Output, XFileCommonParseError> {
        let anim_name_token = ObjectHeader::parse(parser)?;
        let anim_name = str::from_utf8(anim_name_token)
            .map(|s| s.to_owned())
            .unwrap_or_default();
        let mut anim = Animation::new(anim_name);

        loop {
            let token = parser.next_token();
            if token.is_empty() {
                return Err(XFileCommonParseError::unexpected_end_of_file(
                    "AnimationSet",
                ));
            }
            if token == b"}" {
                break; // animation set finished
            } else if token == b"Animation" {
                anim.anims.push(AnimBone::parse(parser)?);
            } else {
                UnknownDataObject::parse(parser)?;
            }
        }
        Ok(anim)
    }
}

#[test]
fn test_animation_parse() {
    let mut parser = ParserCtx::new(TextParser::<true>::new(
        r#"
AnimationSet as {
    Animation a1 {
        {f_1}
        AnimationKey {
            0;
            1;
            0; 4; 1.000000, 0.000000, 0.000000, 0.000000;;;
        }
        AnimationKey {
            2;
            1;
            0; 3; 0.000000, 0.000000, 0.000000;;;
        }
    }
    Animation a2 {
        {f_2}
        AnimationKey {
            0;
            1;
            0; 4; 1.000000, 0.000000, 0.000000, 0.000000;;;
        }
        AnimationKey {
            2;
            1;
            0; 3; 2.500000, 0.000000, 40.000000;;;
        }
    }
}"#
        .as_bytes(),
    ));
    assert_eq!(parser.next_token(), b"AnimationSet");
    let animation = Animation::parse(&mut parser).unwrap();
    assert_eq!(animation.name, "as");
    assert_eq!(animation.anims.len(), 2);
    assert_eq!(animation.anims[0].name, "a1");
    assert_eq!(animation.anims[0].pos_keys.len(), 1);
    assert_eq!(animation.anims[0].pos_keys[0].time, 0.0);
    assert_eq!(animation.anims[0].pos_keys[0].value, AiVec3::ZERO);
    assert_eq!(
        animation.anims[0].pos_keys[0].interpolation,
        AiAnimInterpolation::default()
    );
}

pub(super) struct AnimTicksPerSecond;

impl<'source> XFileParse<'source> for AnimTicksPerSecond {
    type Output = ();
    fn parse<P: XFileParser<'source>>(
        parser: &mut ParserCtx<'source, P>,
    ) -> Result<Self::Output, XFileCommonParseError> {
        ObjectHeader::parse(parser)?;
        parser.scene.anim_ticks_per_second = parser.read_int()?;
        parser.check_for_closing_brace()?;
        Ok(())
    }
}
/// Helper structure to represent a XFile frame
#[derive(Clone, Debug)]
pub struct Node {
    /// The name of the node.
    pub name: String,
    /// The transformation matrix of the node.
    pub transformation_matrix: AiMat4,
    /// The parent of the node.
    pub parent: Index<Node>,
    /// The children of the node.
    pub children: Vec<Index<Node>>,
    /// The meshes of the node.
    pub meshes: Vec<Mesh>,
}

impl Default for Node {
    fn default() -> Self {
        Self {
            name: String::new(),
            transformation_matrix: AiMat4::IDENTITY,
            parent: Index::new(0),
            children: Vec::new(),
            meshes: Vec::new(),
        }
    }
}

impl Node {
    /// Constructor for the node.
    pub fn new(parent: Index<Node>) -> Self {
        Self {
            name: String::new(),
            transformation_matrix: AiMat4::IDENTITY,
            parent,
            children: Vec::new(),
            meshes: Vec::new(),
        }
    }
}

/// Helper structure analogue to aiScene
#[derive(Clone, Debug, Default)]
pub struct Scene {
    /// Nodes of the scene.
    pub nodes: Vec<Node>,

    /// Global meshes found outside of any frames.
    pub global_meshes: Vec<Mesh>,

    /// Global materials found outside of any meshes.
    pub global_materials: Vec<Material>,

    /// Animations of the scene.
    pub animations: Vec<Animation>,

    /// Ticks per second of the scene.
    pub anim_ticks_per_second: u32,
}

impl Scene {
    /// Check if the scene has a root node.
    pub fn has_root_node(&self) -> bool {
        !self.nodes.is_empty()
    }

    /// Push a node to the scene.
    pub fn push_node(&mut self, parent: Index<Node>, node: Node) -> Index<Node> {
        let index = Index::push(&mut self.nodes, node);
        if let Some(parent) = Index::get_mut(parent, &mut self.nodes) {
            parent.children.push(index);
        }
        index
    }
}

// pub struct DeclData;

// impl<'source> XFileParse<'source> for DeclData {
//     type Output = ();
//     fn parse<P: XFileParser<'source>>(
//         parser: &mut ParserCtx<'source, P>,
//     ) -> Result<Self::Output, XFileCommonParseError> {
//         use super::parser::constants::*;
//         ObjectHeader::parse(parser)?;
//         let dcnt = parser.read_int()?;
//         let mut size = 0;
//         let mut normalpos = 0;
//         let mut uvpos = 0;
//         let mut uv2pos = 0;
//         let mut tangentpos = 0;
//         let mut binormalpos = 0;
//         let mut normaltype = 0;
//         let mut uvtype = 0;
//         let mut uv2type = 0;
//         let mut tangenttype = 0;
//         let mut binormaltype = 0;
//         struct VertexElement {
//             r#type: u32,
//             tesselator: u32,
//             usage: u32,
//             usageindex: u32,
//         }
//         let mut vertex_elements = Vec::new();
//         vertex_elements
//             .try_reserve(dcnt as usize)
//             .map_err(|_| XFileGenericParseError::InsufficientMemory)?;
//         for _ in 0..dcnt {
//             let r#type = parser.read_int()?;
//             let tesselator = parser.read_int()?;
//             let usage = parser.read_int()?;
//             let usageindex = parser.read_int()?;
//             parser.test_for_separator();
//             vertex_elements.push(VertexElement {
//                 r#type,
//                 tesselator,
//                 usage,
//                 usageindex,
//             });
//             match usage {
//                 D3DDECLUSAGE_NORMAL => {
//                     normalpos = size;
//                     normaltype = r#type;
//                 }
//                 D3DDECLUSAGE_TEXCOORD => {
//                     if usageindex == D3DDECLUSAGE_POSITION {
//                         uvpos = size;
//                         uvtype = r#type;
//                     } else if usageindex == D3DDECLUSAGE_BLENDWEIGHT {
//                         uv2pos = size;
//                         uv2type = r#type;
//                     }
//                 }
//                 D3DDECLUSAGE_TANGENT => {
//                     tangentpos = size;
//                     tangenttype = r#type;
//                 }
//                 D3DDECLUSAGE_BINORMAL => {
//                     binormalpos = size;
//                     binormaltype = r#type;
//                 }
//                 _ => {}
//             }
//             match r#type {
//                 D3DDECLTYPE_FLOAT1
//                 | D3DDECLTYPE_D3DCOLOR
//                 | D3DDECLTYPE_UBYTE4
//                 | D3DDECLTYPE_SHORT2
//                 | D3DDECLTYPE_UBYTE4N
//                 | D3DDECLTYPE_SHORT2N
//                 | D3DDECLTYPE_USHORT2N
//                 | D3DDECLTYPE_UDEC3
//                 | D3DDECLTYPE_DEC3N
//                 | D3DDECLTYPE_FLOAT16_2 => size += 1,
//                 D3DDECLTYPE_FLOAT2
//                 | D3DDECLTYPE_SHORT4
//                 | D3DDECLTYPE_SHORT4N
//                 | D3DDECLTYPE_USHORT4N
//                 | D3DDECLTYPE_FLOAT16_4 => size += 2,
//                 D3DDECLTYPE_FLOAT3 => size += 3,
//                 D3DDECLTYPE_FLOAT4 => size += 4,
//                 _ => {}
//             }
//         }
//         let data_size = parser.read_int()?;
//         let mut tangents: Vec<Vec3> = Vec::new();
//         let mut binormals: Vec<Vec2> = Vec::new();
//         let mut normals: Vec<Vec3> = Vec::new();
//         let mut uvs: Vec<Vec2> = Vec::new();
//         let mut uv2s: Vec<Vec2> = Vec::new();
//         let mut colors: Vec<Color4D> = Vec::new();
//         for _ in 0..size {
//             for vertex_element in vertex_elements.iter() {
//                 match vertex_element.r#type {
//                     D3DDECLTYPE_FLOAT1
//                     | D3DDECLTYPE_D3DCOLOR
//                     | D3DDECLTYPE_UBYTE4
//                     | D3DDECLTYPE_SHORT2
//                     | D3DDECLTYPE_UBYTE4N
//                     | D3DDECLTYPE_SHORT2N
//                     | D3DDECLTYPE_USHORT2N
//                     | D3DDECLTYPE_UDEC3
//                     | D3DDECLTYPE_DEC3N
//                     | D3DDECLTYPE_FLOAT16_2 => {
//                         let _ = parser.read_int()?;
//                     }
//                     D3DDECLTYPE_FLOAT2
//                     | D3DDECLTYPE_SHORT4
//                     | D3DDECLTYPE_SHORT4N
//                     | D3DDECLTYPE_USHORT4N
//                     | D3DDECLTYPE_FLOAT16_4 => {
//                         let x = f32::from_bits(parser.read_int()?);
//                         let y = f32::from_bits(parser.read_int()?);
//                         match vertex_element.usage {
//                             D3DDECLUSAGE_BINORMAL => {
//                                 binormals.push(Vec2::new(x as AiReal, y as AiReal));
//                             }
//                             _ => {}
//                         }
//                     }
//                     D3DDECLTYPE_FLOAT3 => {
//                         let x = f32::from_bits(parser.read_int()?);
//                         let y = f32::from_bits(parser.read_int()?);
//                         let z = f32::from_bits(parser.read_int()?);
//                         match vertex_element.usage {
//                             D3DDECLUSAGE_TANGENT => {
//                                 tangents.push(Vec3::new(x as AiReal, y as AiReal, z as AiReal));
//                             }
//                             _ => {}
//                         }
//                     }
//                     D3DDECLTYPE_FLOAT4 => {
//                         let _ = parser.read_int()?;
//                         let _ = parser.read_int()?;
//                         let _ = parser.read_int()?;
//                         let _ = parser.read_int()?;
//                     }
//                     _ => {}
//                 }
//             }
//         }
//         parser.check_for_closing_brace()?;
//         Ok(())
//     }
// }
