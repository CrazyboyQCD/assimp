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

//! Implementation of the post processing step to convert all imported data to a left-handed
//! coordinate system.
//!
//! Face order & UV flip are also implemented in sub module here.

use crate::{
    assets::postprocess::{AiPostProcessSteps, PostProcess},
    structs::{
        animation::anim::AiNodeAnim,
        camera::AiCamera,
        material::{AiMaterial, property::AiProperty},
        mesh::AiMesh,
        node::AiNode,
        scene::AiScene,
    },
};

pub mod flip_uvs_process;
pub mod flip_winding_order_process;

/// ## Convert to left-handed process.
pub struct ConvertToLeftHandProcess;

impl ConvertToLeftHandProcess {
    fn process_node(nodes: &mut [AiNode]) {
        for node in nodes {
            // mirror all base vectors at the local Z axis
            node.transformation.z_axis = -node.transformation.z_axis;

            // now invert the Z axis again to keep the matrix determinant positive.
            // The local meshes will be inverted accordingly so that the result should
            // look just fine again.
            node.transformation.x_axis.z = -node.transformation.x_axis.z;
            node.transformation.y_axis.z = -node.transformation.y_axis.z;
            node.transformation.z_axis.z = -node.transformation.z_axis.z;
            node.transformation.w_axis.z = -node.transformation.w_axis.z; // useless, but anyways...
        }
    }

    fn process_mesh(mesh: &mut AiMesh) {
        // mirror positions, normals and stuff along the Z axis
        for v in mesh.vertices.iter_mut() {
            v.z = -v.z;
        }
        for v in mesh.normals.iter_mut() {
            v.z = -v.z;
        }
        for v in mesh.tangents.iter_mut() {
            v.z = -v.z;
        }
        // mirror bitangents as well as they're derived from the texture coords
        for v in mesh.bitangents.iter_mut() {
            v.z = -v.z;
        }

        // mirror anim meshes positions, normals and stuff along the Z axis
        for anim_mesh in mesh.anim_meshes.iter_mut() {
            for v in anim_mesh.vertices.iter_mut() {
                v.z = -v.z;
            }
            for v in anim_mesh.normals.iter_mut() {
                v.z = -v.z;
            }
            for v in anim_mesh.tangents.iter_mut() {
                v.z = -v.z;
            }
            for v in anim_mesh.bitangents.iter_mut() {
                v.z = -v.z;
            }
        }

        // mirror offset matrices of all bones
        for bone in mesh.bones.iter_mut() {
            bone.offset_matrix.x_axis.z = -bone.offset_matrix.x_axis.z;
            bone.offset_matrix.y_axis.z = -bone.offset_matrix.y_axis.z;
            bone.offset_matrix.w_axis.z = -bone.offset_matrix.w_axis.z;

            bone.offset_matrix.z_axis.x = -bone.offset_matrix.z_axis.x;
            bone.offset_matrix.z_axis.y = -bone.offset_matrix.z_axis.y;
            bone.offset_matrix.z_axis.w = -bone.offset_matrix.z_axis.w;
        }
    }

    fn process_material(material: &mut AiMaterial) {
        for p in material.properties.iter_mut() {
            // Mapping axis for UV mappings?
            if let AiProperty::TextureMapAxis(v) = &mut p.property {
                v.z = -v.z;
            }
        }
    }

    fn process_animation(animation: &mut AiNodeAnim) {
        // position keys
        for p in animation.position_keys.iter_mut() {
            p.value.z = -p.value.z;
        }

        // rotation keys
        for p in animation.rotation_keys.iter_mut() {
            p.value.x = -p.value.x;
            p.value.y = -p.value.y;
        }
    }

    fn process_camera(camera: &mut AiCamera) {
        camera.look_at = (camera.position * 2.0) - camera.look_at;
    }
}

impl PostProcess for ConvertToLeftHandProcess {
    fn execute(scene: &mut AiScene) {
        Self::process_node(&mut scene.nodes);
        for mesh in scene.meshes.iter_mut() {
            Self::process_mesh(mesh);
        }
        for material in scene.materials.iter_mut() {
            Self::process_material(material);
        }
        for animation in scene.animations.iter_mut() {
            for node_anim in animation.channels.iter_mut() {
                Self::process_animation(node_anim);
            }
        }
        for camera in scene.cameras.iter_mut() {
            Self::process_camera(camera);
        }
    }

    fn is_active(flag: AiPostProcessSteps) -> bool {
        flag.contains(AiPostProcessSteps::Make_Left_Handed)
    }
}
