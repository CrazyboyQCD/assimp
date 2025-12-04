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

//! Implements MMD(PMX format only for now) format importer for the library

use alloc::{collections::BTreeMap, vec::Vec};
use core::mem;
#[cfg(feature = "std")]
use std::{fs::File, io::Read, path::Path};

use crate::{
    AiMat4, AiReal, AiVec3,
    assets::{
        formats::mmd::{
            error::MMD_OUT_OF_MEMORY_ERROR,
            parser::pmx::{
                PMXParser,
                structs::{
                    PmxBone, PmxMaterial, PmxModel, PmxSetting, PmxVertex, PmxVertexSkinning,
                },
            },
        },
        postprocess::{
            PostProcess,
            convert_to_left_hand_process::{
                ConvertToLeftHandProcess, flip_winding_order_process::FlipWindingOrderProcess,
            },
        },
    },
    io::importer::traits::{EmptyConfig, FormatHeader, InternalImporter},
    structs::{
        color::Color3D,
        importer_desc::{ImporterDesc, ImporterFlags},
        index::Index,
        material::{AiMaterial, property::AiProperty},
        mesh::{
            AI_MAX_NUMBER_OF_TEXTURECOORDS, AiMesh, AiVertexWeight, bone::AiBone, face::AiFace,
        },
        node::AiNode,
        scene::AiScene,
    },
};
#[cfg(feature = "std")]
use crate::{
    assets::formats::mmd::error::MMDImportError,
    io::{importer::traits::FormatHeaderValidator, utils::encoding::convert_to_utf8},
};

static DESC: ImporterDesc = ImporterDesc {
    name: "MMD PMX Importer",
    author: "",
    maintainer: "",
    comments: "",
    flags: ImporterFlags::from_bits_retain(ImporterFlags::SUPPORT_TEXT_FLAVOUR.bits()),
    min_major: 0,
    min_minor: 0,
    max_major: 0,
    max_minor: 0,
    file_extensions: &["pmx"],
};

/// ## Importer for the MMD PMX file format.
pub struct PmxFormatImporter;

impl PmxFormatImporter {
    /// Get the importer description.
    pub const fn get_info() -> &'static ImporterDesc {
        &DESC
    }

    fn convert_material(
        material: &PmxMaterial,
        textures: &[String],
    ) -> Result<AiMaterial, MMDImportError> {
        let mut mat = AiMaterial::default();
        mat.add_property(
            AiProperty::MaterialName(material.material_english_name.clone()),
            0,
        );
        mat.add_property(
            AiProperty::MaterialColorDiffuse(
                Color3D::new(
                    material.diffuse.x as AiReal,
                    material.diffuse.y as AiReal,
                    material.diffuse.z as AiReal,
                )
                .into(),
            ),
            0,
        );

        mat.add_property(
            AiProperty::MaterialColorSpecular(Color3D::new(
                material.specular.x as AiReal,
                material.specular.y as AiReal,
                material.specular.z as AiReal,
            )),
            0,
        );

        mat.add_property(
            AiProperty::MaterialColorAmbient(
                Color3D::new(
                    material.ambient.x as AiReal,
                    material.ambient.y as AiReal,
                    material.ambient.z as AiReal,
                )
                .into(),
            ),
            0,
        );

        mat.add_property(AiProperty::MaterialOpacity(material.diffuse.w as AiReal), 0);
        mat.add_property(
            AiProperty::MaterialShininess(material.specularlity as AiReal),
            0,
        );

        if material.diffuse_texture_index >= 0 {
            mat.add_property(
                AiProperty::TextureDiffuse(
                    textures[material.diffuse_texture_index as usize].clone(),
                ),
                0,
            );
        }

        mat.add_property(AiProperty::TextureUvwsrc(0), 0);

        Ok(mat)
    }

    fn create_mesh(
        setting: PmxSetting,
        old_vertices: &[PmxVertex],
        old_bones: &[PmxBone],
        old_indices: &[i32],
        start: usize,
        count: usize,
    ) -> Result<AiMesh, MMDImportError> {
        let num_faces = count as u32 / 3;
        let mut faces = Vec::new();
        faces
            .try_reserve(num_faces as usize)
            .map_err(|_| MMD_OUT_OF_MEMORY_ERROR)?;
        const NUM_OF_INDICES: u32 = 3;
        for i in 0..(count as u32 / 3) {
            faces.push(AiFace {
                indices: vec![
                    NUM_OF_INDICES * i,
                    NUM_OF_INDICES * i + 1,
                    NUM_OF_INDICES * i + 2,
                ],
            });
        }
        let mut texture_coords: [Vec<AiVec3>; 8] = Default::default();
        let mut num_of_uv_components = [0; AI_MAX_NUMBER_OF_TEXTURECOORDS];
        num_of_uv_components[0] = 2;
        // additional UVs
        let range = 1..setting.uv as usize;
        for (texture_coord, num_of_uv_component) in texture_coords[range.clone()]
            .iter_mut()
            .zip(num_of_uv_components[range.clone()].iter_mut())
        {
            texture_coord
                .try_reserve(count)
                .map_err(|_| MMD_OUT_OF_MEMORY_ERROR)?;
            *num_of_uv_component = 4
        }
        let mut vertices = Vec::new();
        vertices
            .try_reserve(count)
            .map_err(|_| MMD_OUT_OF_MEMORY_ERROR)?;
        let mut normals = Vec::new();
        normals
            .try_reserve(count)
            .map_err(|_| MMD_OUT_OF_MEMORY_ERROR)?;
        let mut bone_map = BTreeMap::new();
        for index in 0..count {
            let v = &old_vertices[old_indices[start + index] as usize];
            let position = v.position;
            vertices.push(AiVec3::new(position[0], position[1], position[2]));
            let normal = v.normal;
            normals.push(AiVec3::new(normal[0], normal[1], normal[2]));
            texture_coords[0].push(AiVec3::new(v.uv.x, v.uv.y, 0.0));

            for (texture_coord, uva) in texture_coords[range.clone()]
                .iter_mut()
                .zip(v.uva[range.clone()].iter())
            {
                texture_coord.push(AiVec3::new(uva.x, uva.y, 0.0));
            }
            match &v.skinning {
                PmxVertexSkinning::BDEF1(pmx_vertex_skinning_bdef1) => {
                    bone_map
                        .entry(pmx_vertex_skinning_bdef1.bone_index)
                        .or_insert(Vec::new())
                        .push(AiVertexWeight {
                            vertex_id: index as u32,
                            weight: 1.0,
                        });
                }
                PmxVertexSkinning::BDEF2(pmx_vertex_skinning_bdef2) => {
                    bone_map
                        .entry(pmx_vertex_skinning_bdef2.bone_index_1)
                        .or_insert(Vec::new())
                        .push(AiVertexWeight {
                            vertex_id: index as u32,
                            weight: pmx_vertex_skinning_bdef2.bone_weight,
                        });
                    bone_map
                        .entry(pmx_vertex_skinning_bdef2.bone_index_2)
                        .or_insert(Vec::new())
                        .push(AiVertexWeight {
                            vertex_id: index as u32,
                            weight: 1.0 - pmx_vertex_skinning_bdef2.bone_weight,
                        });
                }
                PmxVertexSkinning::BDEF4(pmx_vertex_skinning_bdef4) => {
                    bone_map
                        .entry(pmx_vertex_skinning_bdef4.bone_index_1)
                        .or_insert(Vec::new())
                        .push(AiVertexWeight {
                            vertex_id: index as u32,
                            weight: pmx_vertex_skinning_bdef4.bone_weight_1,
                        });
                    bone_map
                        .entry(pmx_vertex_skinning_bdef4.bone_index_2)
                        .or_insert(Vec::new())
                        .push(AiVertexWeight {
                            vertex_id: index as u32,
                            weight: pmx_vertex_skinning_bdef4.bone_weight_2,
                        });
                    bone_map
                        .entry(pmx_vertex_skinning_bdef4.bone_index_3)
                        .or_insert(Vec::new())
                        .push(AiVertexWeight {
                            vertex_id: index as u32,
                            weight: pmx_vertex_skinning_bdef4.bone_weight_3,
                        });
                    bone_map
                        .entry(pmx_vertex_skinning_bdef4.bone_index_4)
                        .or_insert(Vec::new())
                        .push(AiVertexWeight {
                            vertex_id: index as u32,
                            weight: pmx_vertex_skinning_bdef4.bone_weight_4,
                        });
                }
                PmxVertexSkinning::SDEF(pmx_vertex_skinning_sdef) => {
                    bone_map
                        .entry(pmx_vertex_skinning_sdef.bone_index_1)
                        .or_insert(Vec::new())
                        .push(AiVertexWeight {
                            vertex_id: index as u32,
                            weight: pmx_vertex_skinning_sdef.bone_weight,
                        });
                    bone_map
                        .entry(pmx_vertex_skinning_sdef.bone_index_2)
                        .or_insert(Vec::new())
                        .push(AiVertexWeight {
                            vertex_id: index as u32,
                            weight: 1.0 - pmx_vertex_skinning_sdef.bone_weight,
                        });
                }
                PmxVertexSkinning::QDEF(pmx_vertex_skinning_qdef) => {
                    bone_map
                        .entry(pmx_vertex_skinning_qdef.bone_index_1)
                        .or_insert(Vec::new())
                        .push(AiVertexWeight {
                            vertex_id: index as u32,
                            weight: pmx_vertex_skinning_qdef.bone_weight_1,
                        });
                    bone_map
                        .entry(pmx_vertex_skinning_qdef.bone_index_2)
                        .or_insert(Vec::new())
                        .push(AiVertexWeight {
                            vertex_id: index as u32,
                            weight: pmx_vertex_skinning_qdef.bone_weight_2,
                        });
                    bone_map
                        .entry(pmx_vertex_skinning_qdef.bone_index_3)
                        .or_insert(Vec::new())
                        .push(AiVertexWeight {
                            vertex_id: index as u32,
                            weight: pmx_vertex_skinning_qdef.bone_weight_3,
                        });
                    bone_map
                        .entry(pmx_vertex_skinning_qdef.bone_index_4)
                        .or_insert(Vec::new())
                        .push(AiVertexWeight {
                            vertex_id: index as u32,
                            weight: pmx_vertex_skinning_qdef.bone_weight_4,
                        });
                }
            }
        }
        // make all bones for each mesh
        // assign bone weights to skinned bones (otherwise just initialize)
        let mut bones = Vec::new();
        bones
            .try_reserve(old_bones.len())
            .map_err(|_| MMD_OUT_OF_MEMORY_ERROR)?;
        for (index, bone) in old_bones.iter().enumerate() {
            let mut new_bone = AiBone {
                name: bone.bone_name.clone(),
                offset_matrix: AiMat4::from_translation(bone.position),
                ..Default::default()
            };
            if let Some(vertex_weights) = bone_map.get_mut(&(index as i32)) {
                mem::swap(&mut new_bone.weights, vertex_weights);
            }
            bones.push(new_bone);
        }
        Ok(AiMesh {
            faces,
            texture_coords,
            num_of_uv_components,
            vertices,
            normals,
            bones,
            ..Default::default()
        })
    }

    fn to_ai_scene(scene: &PmxModel, ai_scene: &mut AiScene) -> Result<(), MMDImportError> {
        let PmxModel {
            setting,
            model_name,
            vertices,
            indices,
            textures,
            materials,
            bones,
            ..
        } = scene;
        let mut root = AiNode {
            name: model_name.clone(),
            ..Default::default()
        };

        let mesh_node = AiNode {
            name: format!("{}_mesh", root.name),
            meshes: (0..materials.len() as u32).collect(),
            ..Default::default()
        };
        let mut start = 0;
        for (index, mat) in materials.iter().enumerate() {
            let count = mat.index_count as usize;
            let mut mesh = Self::create_mesh(*setting, vertices, bones, indices, start, count)?;
            mesh.name = mat.material_name.clone();
            mesh.material_index = index as u32;
            ai_scene.meshes.push(mesh);
            start += count;
        }
        // create node hierarchy for bone position
        root.children.push(Index::new(1));
        let mut nodes = vec![root, mesh_node];
        let start_index = nodes.len();
        for old_bone in bones.iter() {
            nodes.push(AiNode::from_name(old_bone.bone_name.clone()));
        }
        for (index, old_bone) in bones.iter().enumerate() {
            if old_bone.parent_index < 0 {
                nodes[0]
                    .children
                    .push(Index::new(index as u32 + start_index as u32));
            } else {
                let parent_index = old_bone.parent_index as usize;
                let v3 = old_bone.position - bones[parent_index].position;
                nodes[index + start_index].transformation = AiMat4::from_translation(v3);
                nodes[parent_index + start_index]
                    .children
                    .push(Index::new(index as u32 + start_index as u32));
            }
        }

        let mut new_materials = Vec::new();
        new_materials
            .try_reserve(materials.len())
            .map_err(|_| MMD_OUT_OF_MEMORY_ERROR)?;
        for mat in materials.iter() {
            new_materials.push(Self::convert_material(mat, textures)?);
        }

        ai_scene.nodes = nodes;
        ai_scene.materials = new_materials;
        // Convert everything to OpenGL space
        ConvertToLeftHandProcess::execute(ai_scene);

        FlipWindingOrderProcess::execute(ai_scene);

        Ok(())
    }
}

impl TryFrom<&PmxModel> for AiScene {
    type Error = MMDImportError;
    fn try_from(value: &PmxModel) -> Result<Self, Self::Error> {
        let mut ai_scene = AiScene::default();
        PmxFormatImporter::to_ai_scene(value, &mut ai_scene)?;
        Ok(ai_scene)
    }
}

impl FormatHeader<4> for PmxFormatImporter {
    const HEADER: [u8; 4] = *b"pmx ";
}

impl InternalImporter<MMDImportError> for PmxFormatImporter {
    type ExtraConfig = EmptyConfig;

    #[cfg(feature = "std")]
    fn import_from_file(
        file_path: &Path,
        ai_scene: &mut AiScene,
        config: Self::ExtraConfig,
    ) -> Result<(), MMDImportError> {
        use crate::io::importer::error::CommonImportError;

        let mut buf = {
            use crate::io::importer::error::CommonImportError;

            let mut file = File::open(file_path).map_err(CommonImportError::from)?;
            let file_size = file.metadata().map_err(CommonImportError::from)?.len();
            if file_size < 16 {
                Err(CommonImportError::FileTooSmall)?;
            }
            let mut buf = Vec::new();
            file.read_to_end(&mut buf)
                .map_err(CommonImportError::from)?;
            buf
        };

        // in the hope that binary files will never start with a BOM ...
        convert_to_utf8(&mut buf).map_err(CommonImportError::from)?;
        let buf = buf.as_slice();
        if Self::check_header_from_buf(buf) {
            Self::import_from_buf(buf, ai_scene, config)
        } else {
            Err(CommonImportError::InvalidFormat)?
        }
    }

    fn import_from_buf(
        buf: &[u8],
        ai_scene: &mut AiScene,
        _config: Self::ExtraConfig,
    ) -> Result<(), MMDImportError> {
        Self::to_ai_scene(&PMXParser::new(buf).parse()?, ai_scene)
    }
}

impl PmxFormatImporter {
    #[allow(unused)]
    pub(crate) fn get_tokens(buf: &[u8]) -> Result<Vec<&[u8]>, MMDImportError> {
        // let parser = Parser::new(buf)?;
        // parser.get_tokens()
        Ok(vec![])
    }
}
