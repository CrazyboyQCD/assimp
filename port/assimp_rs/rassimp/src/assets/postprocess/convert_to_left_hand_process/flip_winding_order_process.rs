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

//! Implementation of the post processing step to flip the winding order of the import data.

use crate::{
    assets::postprocess::{AiPostProcessSteps, PostProcess},
    structs::{mesh::AiMesh, scene::AiScene},
};

/// ## Flip winding order process.
pub struct FlipWindingOrderProcess;

impl FlipWindingOrderProcess {
    fn process_mesh(mesh: &mut AiMesh) {
        // invert the order of all faces in this mesh
        for face in mesh.faces.iter_mut() {
            face.indices.reverse();
        }
        // invert the order of all components in this mesh anim meshes
        for anim_mesh in mesh.anim_meshes.iter_mut() {
            anim_mesh.vertices.reverse();
            anim_mesh.normals.reverse();
            for texture_coord in anim_mesh.texture_coords.iter_mut() {
                texture_coord.reverse();
            }
            anim_mesh.tangents.reverse();
            anim_mesh.bitangents.reverse();
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
        flag.contains(AiPostProcessSteps::Flip_Winding_Order)
    }
}
