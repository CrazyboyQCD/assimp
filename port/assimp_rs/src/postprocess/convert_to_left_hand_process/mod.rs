use crate::{
    structs::scene::{AiNode, AiScene},
    structs::{
        anim::anim::AiNodeAnim,
        camera::AiCamera,
        material::{AI_MATKEY_TEXMAP_AXIS, AiMaterial, AiProperty},
        mesh::AiMesh,
    },
};

use super::{AiPostProcessSteps, PostProcess};

pub mod flip_uvs_process;
pub mod flip_winding_order_process;

pub struct ConvertToLeftHandProcess;

impl ConvertToLeftHandProcess {
    fn process_node(nodes: &mut Vec<AiNode>) {
        for node in nodes {
            // mirror all base vectors at the local Z axis
            node.transformation.z_axis *= -1.0;

            // now invert the Z axis again to keep the matrix determinant positive.
            // The local meshes will be inverted accordingly so that the result should look just fine again.
            node.transformation.x_axis.z *= -1.0;
            node.transformation.y_axis.z *= -1.0;
            node.transformation.z_axis.z *= -1.0;
            node.transformation.w_axis.z *= -1.0; // useless, but anyways...
        }
    }

    fn process_mesh(mesh: &mut AiMesh) {
        // mirror positions, normals and stuff along the Z axis
        for v in mesh.vertices.iter_mut() {
            v.z *= -1.0;
        }
        for v in mesh.normals.iter_mut() {
            v.z *= -1.0;
        }
        for v in mesh.tangents.iter_mut() {
            v.z *= -1.0;
        }
        // mirror bitangents as well as they're derived from the texture coords
        for v in mesh.bitangents.iter_mut() {
            v.z *= -1.0;
        }

        // mirror anim meshes positions, normals and stuff along the Z axis
        for anim_mesh in mesh.anim_meshes.iter_mut() {
            for v in anim_mesh.vertices.iter_mut() {
                v.z *= -1.0;
            }
            for v in anim_mesh.normals.iter_mut() {
                v.z *= -1.0;
            }
            for v in anim_mesh.tangents.iter_mut() {
                v.z *= -1.0;
            }
            for v in anim_mesh.bitangents.iter_mut() {
                v.z *= -1.0;
            }
        }

        // mirror offset matrices of all bones
        for bone in mesh.bones.iter_mut() {
            bone.offset_matrix.x_axis.z *= -1.0;
            bone.offset_matrix.y_axis.z *= -1.0;
            bone.offset_matrix.w_axis.z *= -1.0;

            bone.offset_matrix.z_axis.x *= -1.0;
            bone.offset_matrix.z_axis.y *= -1.0;
            bone.offset_matrix.z_axis.w *= -1.0;
        }
    }

    fn process_material(material: &mut AiMaterial) {
        for p in material.properties.iter_mut() {
            // Mapping axis for UV mappings?
            if p.key == AI_MATKEY_TEXMAP_AXIS {
                if let AiProperty::Vec3(ref mut v) = p.property {
                    v.z *= -1.0;
                }
            }
        }
    }

    fn process_animation(animation: &mut AiNodeAnim) {
        // position keys
        for p in animation.position_keys.iter_mut() {
            p.value.z *= -1.0;
        }

        // rotation keys
        for p in animation.rotation_keys.iter_mut() {
            p.value.x *= -1.0;
            p.value.y *= -1.0;
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
        flag.contains(AiPostProcessSteps::MakeLeftHanded)
    }
}
