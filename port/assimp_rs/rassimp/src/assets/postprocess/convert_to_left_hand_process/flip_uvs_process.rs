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

//! Implementation of the post processing step to flip the UV coordinate system of the import data.

use alloc::vec::Vec;

use crate::{
    AiVec3,
    assets::postprocess::{AiPostProcessSteps, PostProcess},
    structs::{
        material::{AiMaterial, property::AiProperty},
        mesh::{AI_MAX_NUMBER_OF_TEXTURECOORDS, AiMesh},
        scene::AiScene,
    },
};

/// Postprocessing step to flip the UV coordinate system of the import data
pub struct FlipUVsProcess;

impl FlipUVsProcess {
    fn flip_uvs(texture_coords: &mut [Vec<AiVec3>; AI_MAX_NUMBER_OF_TEXTURECOORDS]) {
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
            if let AiProperty::UvTransform(ref mut uv_transform) = p.property {
                // just flip it, that's everything
                uv_transform.translation.y = -uv_transform.translation.y;
                uv_transform.rotation = -uv_transform.rotation;
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
        flag.contains(AiPostProcessSteps::Flip_UVs)
    }
}
