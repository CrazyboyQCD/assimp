use crate::{
    postprocess::{AiPostProcessSteps, PostProcess},
    structs::{mesh::AiMesh, scene::AiScene},
};

pub struct FlipWindingOrderProcess;

impl FlipWindingOrderProcess {
    pub fn process_mesh(mesh: &mut AiMesh) {
        // invert the order of all faces in this mesh
        for face in mesh.faces.iter_mut() {
            face.indices.reverse();
        }
        // invert the order of all components in this mesh anim meshes
        for anim_mesh in mesh.anim_meshes.iter_mut() {
            if !anim_mesh.vertices.is_empty() {
                anim_mesh.vertices.reverse();
            }
            if !anim_mesh.normals.is_empty() {
                anim_mesh.normals.reverse();
            }
            for texture_coord in anim_mesh.texture_coords.iter_mut() {
                texture_coord.reverse();
            }
            if !anim_mesh.tangents.is_empty() {
                anim_mesh.tangents.reverse();
            }
            if !anim_mesh.bitangents.is_empty() {
                anim_mesh.bitangents.reverse();
            }
            for color in anim_mesh.colors.iter_mut() {
                color.reverse();
            }
        }
    }
}

impl PostProcess for FlipWindingOrderProcess {
    fn execute(scene: &mut AiScene) {
        for mesh in scene.meshes.iter_mut() {
            Self::process_mesh(mesh);
        }
    }

    fn is_active(flag: AiPostProcessSteps) -> bool {
        flag.contains(AiPostProcessSteps::FlipWindingOrder)
    }
}
