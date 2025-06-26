use std::{fs::File, io::Read};

use glam::{Mat3, Quat, Vec3};

use crate::{
    postprocess::{
        PostProcess,
        convert_to_left_hand_process::{
            ConvertToLeftHandProcess, flip_winding_order_process::FlipWindingOrderProcess,
        },
    },
    structs::{
        anim::{AiAnimation, anim::AiNodeAnim},
        bone::AiBone,
        color::Color4D,
        face::AiFace,
        importer_desc::{ImporterDesc, ImporterFlags},
        key::{AiQuatKey, AiVectorKey},
        material::{
            AI_MATKEY_COLOR_DIFFUSE, AI_MATKEY_COLOR_EMISSIVE, AI_MATKEY_COLOR_SPECULAR,
            AI_MATKEY_NAME, AI_MATKEY_SHADING_MODEL, AI_MATKEY_SHININESS, AI_MATKEY_TEXTURE,
            AddProperty, AiColorDiffuseProperty, AiMaterial, AiProperty, AiShadingMode,
            AiStringPropertyType,
        },
        mesh::{
            AI_MAX_NUMBER_OF_COLOR_SETS, AI_MAX_NUMBER_OF_TEXTURECOORDS, AiMesh, AiVertexWeight,
        },
        nodes::Index,
        scene::{AiNode, AiScene},
    },
    traits::importer::trait_define::{
        FormatHeader, FormatValidator, InternalImporter, encoding::convert_to_utf8,
    },
};

use super::{
    errors::XFileImportError,
    parser::Parser,
    structs::{Animation, Material, Mesh, Node, Scene},
};

static DESC: ImporterDesc = ImporterDesc {
    name: "Direct3D XFile Importer",
    author: "",
    maintainer: "",
    comments: "",
    flags: ImporterFlags::SUPPORT_TEXT_FLAVOUR.bits()
        | ImporterFlags::SUPPORT_BINARY_FLAVOUR.bits()
        | ImporterFlags::SUPPORT_COMPRESSED_FLAVOUR.bits(),
    min_major: 1,
    min_minor: 3,
    max_major: 1,
    max_minor: 5,
    file_extensions: "x",
};

pub struct Importer;

impl Importer {
    pub fn get_info(&self) -> &ImporterDesc {
        &DESC
    }

    fn convert_material(
        ai_scene: &mut AiScene,
        materials: Vec<Material>,
    ) -> Result<Vec<u32>, XFileImportError> {
        let mut material_indices = materials.iter().map(|m| m.scene_index).collect::<Vec<_>>();
        // count the non-referrer materials in the array
        let mut num_new_materials = 0;
        for material in materials.iter() {
            num_new_materials += (!material.is_reference) as usize;
        }
        // resize the scene's material list to offer enough space for the new materials
        if num_new_materials > 0 {
            ai_scene
                .materials
                .try_reserve(num_new_materials)
                .map_err(|_| XFileImportError::InsufficientMemory)?;
        }
        for (i, (mut old_mat, scene_index)) in materials
            .into_iter()
            .zip(material_indices.iter_mut())
            .enumerate()
        {
            if old_mat.is_reference {
                for mat in &ai_scene.materials {
                    if mat.get_string_property(AI_MATKEY_NAME, 0, AiStringPropertyType::Name)
                        == Some(&old_mat.name)
                    {
                        *scene_index = i as u32;
                        break;
                    }
                }
                if *scene_index == u32::MAX {
                    *scene_index = 0;
                }
            }

            let mut new_materials = AiMaterial::default();
            new_materials.add_string_property(
                AI_MATKEY_NAME,
                old_mat.name,
                0,
                AiStringPropertyType::Name,
            );

            // Shading model: hard-coded to PHONG, there is no such information in an XFile
            // FIX (aramis): If the specular exponent is 0, use gouraud shading. This is a bugfix
            // for some models in the SDK (e.g. good old tiny.x)
            let shade_mode = if old_mat.specular_exponent == 0.0 {
                AiShadingMode::Gouraud
            } else {
                AiShadingMode::Phong
            };
            new_materials.add_property_v2(AiProperty::ShadingModel(shade_mode), 0);

            // material colours
            // Unclear: there's no ambient colour, but emissive. What to put for ambient?
            // Probably nothing at all, let the user select a suitable default.
            new_materials.add_property_v2(AiProperty::ColorEmissive(old_mat.emissive), 0);
            new_materials.add_property_v2(AiProperty::ColorDiffuse(old_mat.diffuse.into()), 0);
            new_materials.add_property_v2(AiProperty::ColorSpecular(old_mat.specular), 0);
            new_materials.add_property_v2(AiProperty::Shiness(old_mat.specular_exponent), 0);

            // texture, if there is one
            if old_mat.textures.len() == 1 {
                let otex = old_mat.textures.remove(0);
                if otex.name.len() > 0 {
                    // if there is only one texture assume it contains the diffuse color
                    let tex = otex.name;
                    if otex.is_normal_map {
                        new_materials.add_string_property(
                            AI_MATKEY_TEXTURE,
                            tex,
                            0,
                            AiStringPropertyType::TextureNormals,
                        );
                    } else {
                        new_materials.add_string_property(
                            AI_MATKEY_TEXTURE,
                            tex,
                            0,
                            AiStringPropertyType::TextureDiffuse,
                        );
                    }
                }
            } else {
                // Otherwise ... try to search for typical strings in the
                // texture's file name like 'bump' or 'diffuse'
                let mut index_of_height_property = 0;
                let mut index_of_normal_map_property = 0;
                let mut index_of_specular_property = 0;
                let mut index_of_ambient_property = 0;
                let mut index_of_emissive_property = 0;
                let mut index_of_diffuse_property = 0;
                for otex in old_mat.textures.into_iter() {
                    let mut sz = otex.name.clone();
                    if sz.len() == 0 {
                        continue;
                    }

                    // cut off the file extension
                    if let Some(s_ext) = sz.rfind('.') {
                        sz.truncate(s_ext);
                    }

                    // find the file name
                    let s = sz.rfind("\\/").unwrap_or(0);

                    let (_, sz) = sz.split_at_mut(s);

                    // convert to lower case for easier comparison
                    sz.make_ascii_lowercase();

                    // Place texture filename property under the corresponding name
                    let tex = otex.name;

                    // bump map
                    if sz.contains("bump") || sz.contains("height") {
                        new_materials.add_string_property(
                            AI_MATKEY_TEXTURE,
                            tex,
                            index_of_height_property,
                            AiStringPropertyType::TextureHeight,
                        );
                        index_of_height_property += 1;
                    } else if otex.is_normal_map || sz.contains("normal") || sz.contains("nm") {
                        new_materials.add_string_property(
                            AI_MATKEY_TEXTURE,
                            tex,
                            index_of_normal_map_property,
                            AiStringPropertyType::TextureNormals,
                        );
                        index_of_normal_map_property += 1;
                    } else if sz.contains("spec") || sz.contains("glanz") {
                        new_materials.add_string_property(
                            AI_MATKEY_TEXTURE,
                            tex,
                            index_of_specular_property,
                            AiStringPropertyType::TextureSpecular,
                        );
                        index_of_specular_property += 1;
                    } else if sz.contains("ambi") || sz.contains("env") {
                        new_materials.add_string_property(
                            AI_MATKEY_TEXTURE,
                            tex,
                            index_of_ambient_property,
                            AiStringPropertyType::TextureAmbient,
                        );
                        index_of_ambient_property += 1;
                    } else if sz.contains("emissive") || sz.contains("self") {
                        new_materials.add_string_property(
                            AI_MATKEY_TEXTURE,
                            tex,
                            index_of_emissive_property,
                            AiStringPropertyType::TextureEmissive,
                        );
                        index_of_emissive_property += 1;
                    } else {
                        // Assume it is a diffuse texture
                        new_materials.add_string_property(
                            AI_MATKEY_TEXTURE,
                            tex,
                            index_of_diffuse_property,
                            AiStringPropertyType::TextureDiffuse,
                        );
                        index_of_diffuse_property += 1;
                    }
                }
            }
            ai_scene.materials.push(new_materials);
            *scene_index = (ai_scene.materials.len() - 1) as u32;
        }
        Ok(material_indices)
    }

    fn create_node(
        scene: &mut AiScene,
        nodes: Vec<Node>,
    ) -> Result<Index<AiNode>, XFileImportError> {
        let len = nodes.len();
        if len == 0 {
            return Ok(Index::GUARD_INDEX);
        }
        let mut new_nodes = Vec::with_capacity(len);
        for node in nodes {
            let mut new_node = AiNode {
                name: node.name,
                transformation: node.transformation_matrix,
                parent: Index::new(node.parent.value() as u32),
                children: node
                    .children
                    .into_iter()
                    .map(|c| Index::new(c.value() as u32))
                    .collect(),
                meshes: Vec::new(),
                metadata: Box::default(),
            };
            new_node.meshes = Self::create_mesh(scene, node.meshes)?;
            new_nodes.push(new_node);
        }
        scene.nodes = new_nodes;
        Ok(Index::new(1))
    }

    fn create_mesh(scene: &mut AiScene, meshes: Vec<Mesh>) -> Result<Vec<u32>, XFileImportError> {
        if meshes.len() == 0 {
            return Ok(Vec::new());
        }
        let old_meshes_cnt = scene.meshes.len();
        for mesh in meshes {
            let Mesh {
                name,
                positions,
                pos_faces,
                normals,
                norm_faces,
                tex_coords,
                colors,
                face_materials,
                materials,
                bones,
                ..
            } = mesh;
            let name = name.into_boxed_str();
            let material_indices = Self::convert_material(scene, materials)?;
            let num_materials = scene.materials.len().max(1) as u32;
            for b in 0..num_materials {
                let mut new_faces = Vec::new();
                let mut num_vertices = 0;
                if !face_materials.is_empty() {
                    // if there is a per-face material defined, select the faces with the corresponding material
                    for (c, (face_material, face)) in
                        face_materials.iter().zip(pos_faces.iter()).enumerate()
                    {
                        if *face_material == b {
                            new_faces.push(c as u32);
                            num_vertices += face.indices.len() as u32;
                        }
                    }
                } else {
                    // if there is no per-face material, place everything into one mesh
                    for (c, face) in pos_faces.iter().enumerate() {
                        new_faces.push(c as u32);
                        num_vertices += face.indices.len() as u32;
                    }
                }

                // no faces/vertices using this material? strange...
                if num_vertices == 0 {
                    continue;
                }

                let mut new_mesh = AiMesh::default();
                // find the material in the scene's material list. Either own material
                // or referenced material, it should already have a valid index
                if !face_materials.is_empty() {
                    new_mesh.material_index = material_indices[b as usize];
                } else {
                    new_mesh.material_index = 0;
                }

                // Create properly sized data arrays in the mesh. We store unique vertices per face,
                // as specified
                new_mesh.vertices = vec![Vec3::default(); num_vertices as usize].into_boxed_slice();
                new_mesh.faces =
                    vec![AiFace::default(); new_faces.len() as usize].into_boxed_slice();

                new_mesh.name = name.clone();

                // normals?
                if !normals.is_empty() {
                    new_mesh.normals =
                        vec![Vec3::default(); num_vertices as usize].into_boxed_slice();
                }
                // texture coords
                for c in 0..AI_MAX_NUMBER_OF_TEXTURECOORDS {
                    if !tex_coords[c].is_empty() {
                        new_mesh.texture_coords[c] =
                            vec![Vec3::default(); num_vertices as usize].into_boxed_slice();
                    }
                }
                // vertex colors
                for c in 0..AI_MAX_NUMBER_OF_COLOR_SETS {
                    if !colors[c].is_empty() {
                        new_mesh.colors[c] =
                            vec![Color4D::default(); num_vertices as usize].into_boxed_slice();
                    }
                }

                let mut new_index: usize = 0;

                let mut org_points = vec![0u32; num_vertices as usize];

                for (c, &f) in new_faces.iter().enumerate() {
                    let pf = &pos_faces[f as usize]; // position source face

                    // create face. either triangle or triangle fan depending on the index count
                    let df = &mut new_mesh.faces[c]; // destination face
                    df.indices = vec![0u32; pf.indices.len() as usize].into_boxed_slice();

                    // collect vertex data for indices of this face
                    for (d, new_idx) in df.indices.iter_mut().enumerate() {
                        *new_idx = new_index as u32;
                        let new_idx = pf.indices[d];
                        if new_idx >= positions.len() as u32 {
                            continue;
                        }

                        org_points[new_index] = pf.indices[d];

                        // Position
                        new_mesh.vertices[new_index] = positions[new_idx as usize];
                        // Normal, if present
                        if !normals.is_empty() {
                            if norm_faces[f as usize].indices.len() > d {
                                let idx = norm_faces[f as usize].indices[d];
                                if idx < normals.len() as u32 {
                                    new_mesh.normals[new_index] = normals[idx as usize];
                                }
                            }
                        }

                        // texture coord sets
                        for e in 0..AI_MAX_NUMBER_OF_TEXTURECOORDS {
                            if !tex_coords[e].is_empty() {
                                let tex = tex_coords[e][pf.indices[d] as usize];
                                new_mesh.texture_coords[e][new_index] =
                                    Vec3::new(tex.x, 1.0 - tex.y, 0.0);
                            }
                        }
                        // vertex color sets
                        for e in 0..AI_MAX_NUMBER_OF_COLOR_SETS {
                            if !colors[e].is_empty() {
                                new_mesh.colors[e][new_index] = colors[e][pf.indices[d] as usize];
                            }
                        }

                        new_index += 1;
                    }
                }
                let mut new_bones = Vec::new();
                for bone in bones.iter() {
                    let mut old_weights = vec![0.0; positions.len() as usize];
                    for weight in bone.weights.iter() {
                        // The conditional against boneIdx which was added in commit f844c33
                        //     (https://github.com/assimp/assimp/commit/f844c3397d7726477ab0fdca8efd3df56c18366b)
                        // causes massive breakage as detailed in:
                        //     https://github.com/assimp/assimp/issues/5332
                        // In cases like this unit tests are less useful, since the model still has
                        // meshes, textures, animations etc. and asserts against these values may pass;
                        // when touching importer code, it is crucial that developers also run manual, visual
                        // checks to ensure there's no obvious breakage _before_ commiting to main branch
                        old_weights[weight.vertex as usize] = weight.weight;
                    }
                    // collect all vertex weights that influence a vertex in the new mesh
                    let mut new_weights = Vec::with_capacity(num_vertices as usize);
                    for (d, &org_point) in org_points.iter().enumerate() {
                        // does the new vertex stem from an old vertex which was influenced by this bone?
                        let w = old_weights[org_point as usize];
                        if w > 0.0 {
                            new_weights.push(AiVertexWeight {
                                vertex_id: d as u32,
                                weight: w,
                            });
                        }
                    }

                    // if the bone has no weights in the newly created mesh, ignore it
                    if new_weights.is_empty() {
                        continue;
                    }

                    // create
                    let mut new_bone = AiBone::default();
                    new_bone.name = bone.name.clone().into();
                    new_bone.offset_matrix = bone.offset_matrix;
                    new_bone.weights = new_weights.into_boxed_slice();
                    new_bones.push(new_bone);
                }
                if !new_bones.is_empty() {
                    new_mesh.bones = new_bones.into_boxed_slice();
                }
                scene.meshes.push(new_mesh);
            }
        }
        Ok((old_meshes_cnt..old_meshes_cnt + scene.meshes.len())
            .map(|i| i as u32)
            .collect())
    }

    fn create_animation(
        scene: &mut AiScene,
        animations: Vec<Animation>,
        ticks_per_second: u32,
    ) -> Result<(), XFileImportError> {
        let mut new_animations = Vec::new();
        for anim in animations {
            if anim.anims.is_empty() {
                continue;
            }
            let mut new_anim = AiAnimation::default();
            new_anim.ticks_per_second = ticks_per_second as f64;
            let mut new_channels = Vec::new();
            for bone in anim.anims {
                let mut new_bone = AiNodeAnim::default();
                new_bone.node_name = bone.name.into();
                if let Some(last) = bone.trafo_keys.last() {
                    let len = bone.trafo_keys.len();
                    new_bone.position_keys.reserve(len);
                    new_bone.rotation_keys.reserve(len);
                    new_bone.scaling_keys.reserve(len);
                    for key in bone.trafo_keys.iter() {
                        let time = key.time;
                        let trafo = key.matrix;

                        // extract position
                        new_bone.position_keys.push(AiVectorKey {
                            time,
                            value: Vec3::new(trafo.x_axis.w, trafo.y_axis.w, trafo.z_axis.w),
                            interpolation: Default::default(),
                        });

                        // extract scaling
                        let scale = Vec3::new(
                            Vec3::new(trafo.x_axis.x, trafo.y_axis.x, trafo.z_axis.x).length(),
                            Vec3::new(trafo.x_axis.y, trafo.y_axis.y, trafo.z_axis.y).length(),
                            Vec3::new(trafo.x_axis.z, trafo.y_axis.z, trafo.z_axis.z).length(),
                        );
                        new_bone.scaling_keys.push(AiVectorKey {
                            time,
                            value: scale,
                            interpolation: Default::default(),
                        });

                        // extract rotation
                        let mut rotmat = Mat3::from_mat4(trafo);
                        rotmat.x_axis /= scale.x;
                        rotmat.y_axis /= scale.y;
                        rotmat.z_axis /= scale.z;
                        new_bone.rotation_keys.push(AiQuatKey {
                            time,
                            value: Quat::from_mat3(&rotmat),
                            interpolation: Default::default(),
                        });
                    }
                    // longest lasting key sequence determines duration
                    new_anim.duration = new_anim.duration.max(last.time);
                } else {
                    // separate key sequences for position, rotation, scaling
                    if !bone.pos_keys.is_empty() {
                        new_bone.position_keys = bone.pos_keys;
                    }
                    // rotation
                    if !bone.rot_keys.is_empty() {
                        new_bone.rotation_keys = bone.rot_keys;
                    }
                    // scaling
                    if !bone.scale_keys.is_empty() {
                        new_bone.scaling_keys = bone.scale_keys;
                    }

                    // longest lasting key sequence determines duration
                    if let Some(last) = new_bone.position_keys.last() {
                        new_anim.duration = new_anim.duration.max(last.time);
                    }
                    if let Some(last) = new_bone.rotation_keys.last() {
                        new_anim.duration = new_anim.duration.max(last.time);
                    }
                    if let Some(last) = new_bone.scaling_keys.last() {
                        new_anim.duration = new_anim.duration.max(last.time);
                    }
                }
                new_channels.push(new_bone);
            }
            new_anim.channels = new_channels;
            new_animations.push(new_anim);
        }
        if !new_animations.is_empty() {
            scene.animations = new_animations;
        }
        Ok(())
    }

    fn to_ai_scene(scene: Scene, ai_scene: &mut AiScene) -> Result<(), XFileImportError> {
        let Scene {
            nodes,
            global_meshes,
            global_materials,
            animations,
            anim_ticks_per_second,
            ..
        } = scene;

        Self::convert_material(ai_scene, global_materials)?;

        let root_node = Self::create_node(ai_scene, nodes)?;
        ai_scene.root = root_node;

        if !global_meshes.is_empty() {
            if !ai_scene.root.is_exist() {
                ai_scene.root = Index::new(1);
                ai_scene.nodes.push(AiNode::default());
            }
            ai_scene.nodes[1].meshes = Self::create_mesh(ai_scene, global_meshes)?;
        }

        if !root_node.is_exist() {
            return Err(XFileImportError::NoRootNode);
        }

        if !animations.is_empty() {
            Self::create_animation(ai_scene, animations, anim_ticks_per_second)?;
        }
        ConvertToLeftHandProcess::execute(ai_scene);
        FlipWindingOrderProcess::execute(ai_scene);

        if ai_scene.materials.is_empty() {
            let mut new_material = AiMaterial::default();
            new_material.add_property_v2(AiProperty::ShadingModel(AiShadingMode::Gouraud), 0);
            new_material.add_property_v2(AiProperty::ColorEmissive(Vec3::ZERO), 0);
            new_material.add_property_v2(AiProperty::ColorSpecular(Vec3::ZERO), 0);
            new_material
                .add_property_v2(AiProperty::ColorDiffuse(Vec3::new(0.5, 0.5, 0.5).into()), 0);
            new_material.add_property_v2(AiProperty::Shiness(1.0), 0);
            ai_scene.materials.push(new_material);
        }

        Ok(())
    }
}

impl FormatHeader<4> for Importer {
    const HEADER: [u8; 4] = *b"xof ";
}

impl InternalImporter<XFileImportError> for Importer {
    #[cfg(feature = "std")]
    fn import_from_file(file_name: &str, ai_scene: &mut AiScene) -> Result<(), XFileImportError> {
        let mut file = File::open(file_name)?;
        let file_size = file.metadata()?.len();
        if file_size < 16 {
            return Err(XFileImportError::FileTooSmall);
        }
        let mut buf = Vec::new();
        file.read_to_end(&mut buf)?;

        let text = convert_to_utf8(buf).map_err(|e| XFileImportError::from(e))?;
        let buf = text.as_bytes();
        if Self::can_read_from_buf(buf) {
            Self::import_from_buf(buf, ai_scene)
        } else {
            Err(XFileImportError::InvalidFormat)
        }
    }

    fn import_from_buf(buf: &[u8], ai_scene: &mut AiScene) -> Result<(), XFileImportError> {
        let mut parser = Parser::new(buf)?;
        parser.parse()?;
        Self::to_ai_scene(parser.get_scene(), ai_scene)?;
        Ok(())
    }
}
