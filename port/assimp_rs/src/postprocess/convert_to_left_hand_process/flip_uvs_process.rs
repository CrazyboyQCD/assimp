use crate::{
    postprocess::{AiPostProcessSteps, PostProcess},
    structs::{
        material::{AI_MATKEY_UVTRANSFORM, AiMaterial, AiProperty},
        mesh::{AI_MAX_NUMBER_OF_TEXTURECOORDS, AiMesh},
        scene::AiScene,
    },
    utils::float_precision::Vec3,
};

/// Postprocessing step to flip the UV coordinate system of the import data
pub struct FlipUVsProcess;

impl FlipUVsProcess {
    fn flip_uvs(texture_coords: &mut Box<[Vec<Vec3>; AI_MAX_NUMBER_OF_TEXTURECOORDS]>) {
        for texture_coord in texture_coords.iter_mut() {
            for uv in texture_coord.iter_mut() {
                uv.y = 1.0 - uv.y;
            }
        }
    }

    fn process_mesh(mesh: &mut AiMesh) {
        Self::flip_uvs(&mut mesh.texture_coords);
        for anim_mesh in mesh.anim_meshes.iter_mut() {
            Self::flip_uvs(&mut anim_mesh.texture_coords);
        }
    }

    fn process_material(material: &mut AiMaterial) {
        for p in material.properties.iter_mut() {
            if p.key == AI_MATKEY_UVTRANSFORM {
                if let AiProperty::UvTransform(ref mut uv_transform) = p.property {
                    // just flip it, that's everything
                    uv_transform.translation.y *= -1.0;
                    uv_transform.rotation *= -1.0;
                }
            }
        }
    }
}

impl PostProcess for FlipUVsProcess {
    fn execute(scene: &mut AiScene) {
        for mesh in scene.meshes.iter_mut() {
            Self::process_mesh(mesh);
        }
        for material in scene.materials.iter_mut() {
            Self::process_material(material);
        }
    }

    fn is_active(flag: AiPostProcessSteps) -> bool {
        flag.contains(AiPostProcessSteps::FlipUVs)
    }
}
