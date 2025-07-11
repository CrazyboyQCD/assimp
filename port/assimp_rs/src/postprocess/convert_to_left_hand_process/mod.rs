use super::{AiPostProcessSteps, PostProcess};
use crate::{
    structs::{
        anim::anim::AiNodeAnim,
        camera::AiCamera,
        material::{AI_MATKEY_TEXMAP_AXIS, AiMaterial, AiProperty},
        mesh::AiMesh,
        nodes::Index,
        scene::{AiNode, AiScene},
    },
    utils::float_precision::Mat4,
};

pub mod flip_uvs_process;
pub mod flip_winding_order_process;

pub struct ConvertToLeftHandProcess;

impl ConvertToLeftHandProcess {
    fn process_node(root: Option<Index<AiNode>>, nodes: &mut [AiNode], root_transformataion: Mat4) {
        if let Some(root) = root {
            let root = [root];
            let nodes_ptr = nodes.as_mut_ptr();
            let mut stack = vec![(&root[..], root_transformataion)];
            while let Some((inner_nodes_index, current_parent_transformataion)) = stack.pop() {
                for node in inner_nodes_index.iter() {
                    let index = node.value();
                    let node = unsafe { nodes_ptr.add(index) };
                    {
                        // Trick borrow checker as we won't modify the children vector.
                        // SAFETY: indexes should be unique and valid
                        let node = unsafe { node.as_mut().unwrap_unchecked() };
                        // let node = node.get_mut(nodes).unwrap();
                        // mirror all base vectors at the local Z axis
                        node.transformation.z_axis = -node.transformation.z_axis;

                        // now invert the Z axis again to keep the matrix determinant positive.
                        // The local meshes will be inverted accordingly so that the result should look just fine again.
                        node.transformation.x_axis.z = -node.transformation.x_axis.z;
                        node.transformation.y_axis.z = -node.transformation.y_axis.z;
                        node.transformation.z_axis.z = -node.transformation.z_axis.z;
                        node.transformation.w_axis.z = -node.transformation.w_axis.z; // useless, but anyways...
                    }
                    let node = unsafe { node.as_ref().unwrap() };
                    // let node = node.get(nodes).unwrap();
                    stack.push((
                        node.children.as_slice(),
                        node.transformation * current_parent_transformataion,
                    ));
                }
            }
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
            if p.key == AI_MATKEY_TEXMAP_AXIS {
                if let AiProperty::Vec3(ref mut v) = p.property {
                    v.z = -v.z;
                }
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
        Self::process_node(scene.root, &mut scene.nodes, Mat4::IDENTITY);
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
