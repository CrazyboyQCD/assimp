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

//! Implements X format exporter for the library

use alloc::{collections::BTreeMap, string::String, vec::Vec};
use core::fmt::{Display, Error as FmtError, Formatter, Result as FmtResult, Write};

use crate::{
    AiMat4,
    assets::formats::{DefaultRepeatedIndent, RepeatedFormatter, x::error::XFileExportError},
    io::utils::float_precision::PRECISION,
    structs::{
        material::{
            AiMaterial,
            property::{AiColorDiffuseProperty, AiProperty},
        },
        mesh::AiMesh,
        node::AiNode,
        scene::AiScene,
    },
};

type KeyType = u64;

// typedefs for our four configuration maps.
// We don't need more, so there is no need for a generic solution
type IntPropertyMap = BTreeMap<KeyType, i32>;
type FloatPropertyMap = BTreeMap<KeyType, f32>;
type StringPropertyMap = BTreeMap<KeyType, String>;
type MatrixPropertyMap = BTreeMap<KeyType, AiMat4>;
// typedef std::map<KeyType, std::function<void *(void *)>> CallbackPropertyMap;

/// ## Export properties for the X file format.
#[allow(unused)]
#[derive(Debug, Default)]
pub struct ExportProperties {
    int_properties: IntPropertyMap,
    float_properties: FloatPropertyMap,
    string_properties: StringPropertyMap,
    matrix_properties: MatrixPropertyMap,
    // callback_properties: CallbackPropertyMap,
}

impl ExportProperties {
    // pub fn get_bool(&self, key: &str) -> bool {
    //     let mut hasher = DefaultHasher::new();
    //     key.hash(&mut hasher);
    //     *self.int_properties.get(&hasher.finish()).unwrap_or(&0) != 0
    // }

    // pub fn get_int(&self, key: &str) -> i32 {
    //     let mut hasher = DefaultHasher::new();
    //     key.hash(&mut hasher);
    //     *self.int_properties.get(&hasher.finish()).unwrap_or(&0)
    // }
}

/// ## Exporter for the X file format.
pub struct Exporter<'source, W: Write> {
    writer: &'source mut W,
    scene: &'source AiScene,
}

impl<'source, W: Write> Exporter<'source, W> {
    /// Constructor for the exporter.
    pub fn new(scene: &'source AiScene, writer: &'source mut W) -> Self {
        Self { scene, writer }
    }

    /// Write the scene to a stream.
    pub fn write_to_stream(&mut self) -> Result<(), XFileExportError> {
        self.write_header()?;
        self.writer
            .write_fmt(format_args!("{}", XFileAiSceneWrapper(self.scene)))?;
        Ok(())
    }

    /// Writes the asset header
    pub(crate) fn write_header(&mut self) -> Result<(), XFileExportError> {
        let is_64_bits = false /*self.properties.get_bool("AI_CONFIG_EXPORT_XFILE_64BIT")*/;
        if is_64_bits {
            self.writer.write_str("xof 0303txt 0064\n")?;
        } else {
            self.writer.write_str("xof 0303txt 0032\n")?;
        }
        #[rustfmt::skip]
        self.writer.write_str(
r#"
template Frame {{
  <3d82ab46-62da-11cf-ab39-0020af71e433>
  [...]
}}

template Matrix4x4 {{
  <f6f23f45-7686-11cf-8f52-0040333594a3>
  array FLOAT matrix[16];
}}

template FrameTransformMatrix {{
  <f6f23f41-7686-11cf-8f52-0040333594a3>
  Matrix4x4 frameMatrix;
}}

template Vector {{
  <3d82ab5e-62da-11cf-ab39-0020af71e433>
  FLOAT x;
  FLOAT y;
  FLOAT z;
}}

template MeshFace {{
  <3d82ab5f-62da-11cf-ab39-0020af71e433>
  DWORD nFaceVertexIndices;
  array DWORD faceVertexIndices[nFaceVertexIndices];
}}

template Mesh {{
  <3d82ab44-62da-11cf-ab39-0020af71e433>
  DWORD nVertices;
  array Vector vertices[nVertices];
  DWORD nFaces;
  array MeshFace faces[nFaces];
  [...]
}}

template MeshNormals {{
  <f6f23f43-7686-11cf-8f52-0040333594a3>
  DWORD nNormals;
  array Vector normals[nNormals];
  DWORD nFaceNormals;
  array MeshFace faceNormals[nFaceNormals];
}}

template Coords2d {{
  <f6f23f44-7686-11cf-8f52-0040333594a3>
  FLOAT u;
  FLOAT v;
}}

template MeshTextureCoords {{
  <f6f23f40-7686-11cf-8f52-0040333594a3>
  DWORD nTextureCoords;
  array Coords2d textureCoords[nTextureCoords];
}}

template ColorRGBA {{
  <35ff44e0-6c7c-11cf-8f52-0040333594a3>
  FLOAT red;
  FLOAT green;
  FLOAT blue;
  FLOAT alpha;
}}

template IndexedColor {{
  <1630b820-7842-11cf-8f52-0040333594a3>
  DWORD index;
  ColorRGBA indexColor;
}}

template MeshVertexColors {{
  <1630b821-7842-11cf-8f52-0040333594a3>
  DWORD nVertexColors;
  array IndexedColor vertexColors[nVertexColors];
}}

template VertexElement {{
  <f752461c-1e23-48f6-b9f8-8350850f336f>
  DWORD Type;
  DWORD Method;
  DWORD Usage;
  DWORD UsageIndex;
}}

template DeclData {{
  <bf22e553-292c-4781-9fea-62bd554bdd93>
  DWORD nElements;
  array VertexElement Elements[nElements];
  DWORD nDWords;
  array DWORD data[nDWords];
}}

"#)?;
        Ok(())
    }
}

struct XFileAiSceneWrapper<'a>(&'a AiScene);

impl<'a> Display for XFileAiSceneWrapper<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let scene = self.0;
        let level = DefaultRepeatedIndent::new(1);
        f.write_str("Frame DXCC_ROOT {\n")?;
        XFileMat4Wrapper(&AiMat4::IDENTITY, level).fmt(f)?;
        for node in scene
            .nodes
            .iter()
            // write top-level nodes(nodes that have no parent)
            .filter(|node| node.is_root())
        {
            XFileNodeWrapper(node, &scene.nodes, scene, level).fmt(f)?;
        }

        f.write_str("}\n")
    }
}

struct XFileNodeWrapper<'a>(
    &'a AiNode,
    &'a Vec<AiNode>,
    &'a AiScene,
    DefaultRepeatedIndent,
);

impl<'a> Display for XFileNodeWrapper<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let XFileNodeWrapper(node, nodes, scene, level) = self;
        let mut level = *level;

        level.fmt(f)?;
        f.write_str("Frame ")?;
        if node.name.is_empty() {
            f.write_str("Node_")?;
            (node as *const _ as usize).fmt(f)?;
        } else {
            XFileStringWrapper(&node.name).fmt(f)?;
        }
        f.write_str("{\n")?;

        level = level.next();
        XFileMat4Wrapper(&node.transformation, level).fmt(f)?;
        for mesh_index in &node.meshes {
            XFileAiMeshWrapper(scene, &scene.meshes[*mesh_index as usize], level).fmt(f)?;
        }

        // recursive call the Nodes
        for i in &node.children {
            XFileNodeWrapper(i.get(nodes).ok_or(FmtError)?, nodes, scene, level).fmt(f)?;
        }

        level = level.back();
        write!(f, "{level}}}\n")?;
        f.write_str("\n")?;
        Ok(())
    }
}
struct XFileMat4Wrapper<'a>(&'a AiMat4, DefaultRepeatedIndent);

impl<'a> Display for XFileMat4Wrapper<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let XFileMat4Wrapper(m, level) = self;
        let level1 = *level;
        let level2 = level.next();
        #[rustfmt::skip]
        write!(
            f,
            concat!(
                "{}FrameTransformMatrix {{\n",
                "{}{:.*}, {:.*}, {:.*}, {:.*},\n",
                "{}{:.*}, {:.*}, {:.*}, {:.*},\n",
                "{}{:.*}, {:.*}, {:.*}, {:.*},\n",
                "{}{:.*}, {:.*}, {:.*}, {:.*};;\n",
                "{}}}\n\n"
            ),
            level1,
            level2, PRECISION, m.x_axis.x, PRECISION, m.y_axis.x, PRECISION, m.z_axis.x, PRECISION, m.w_axis.x,
            level2, PRECISION, m.x_axis.y, PRECISION, m.y_axis.y, PRECISION, m.z_axis.y, PRECISION, m.w_axis.y,
            level2, PRECISION, m.x_axis.z, PRECISION, m.y_axis.z, PRECISION, m.z_axis.z, PRECISION, m.w_axis.z,
            level2, PRECISION, m.x_axis.w, PRECISION, m.y_axis.w, PRECISION, m.z_axis.w, PRECISION, m.w_axis.w,
            level1
        )?;
        Ok(())
    }
}

struct XFileMaterialWrapper<'a>(&'a AiMaterial, DefaultRepeatedIndent);

impl<'a> Display for XFileMaterialWrapper<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let XFileMaterialWrapper(m, level) = self;
        let level1 = *level;
        let level2 = level1.next();
        let level3 = level2.next();
        let name = m
            .get_property(0, AiProperty::is_material_name_property)
            .unwrap_or_default();
        let Some(diffuse) = m.get_property(0, AiProperty::is_material_color_diffuse_property)
        else {
            panic!("Material in X File should have diffuse property");
        };
        let diffuse = match diffuse {
            AiColorDiffuseProperty::Color4D(vec4) => vec4,
            AiColorDiffuseProperty::Color3D(_) => {
                unreachable!("X File should not have RGB colors in Material")
            }
        };
        let Some(shininess) = m.get_property(0, AiProperty::is_material_shininess_property) else {
            panic!("Material in X File should have shininess property");
        };
        let Some(specular) = m.get_property(0, AiProperty::is_material_color_specular_property)
        else {
            panic!("Material in X File should have specular property");
        };
        let Some(emissive) = m.get_property(0, AiProperty::is_material_color_emissive_property)
        else {
            panic!("Material in X File should have emissive property");
        };
        let tex_file = m
            .get_property(0, AiProperty::is_texture_diffuse_property)
            .unwrap_or_default();

        #[rustfmt::skip]
        write!(
            f,
            concat!(
                "{}Material {} {{\n",
                "{}{:.*}; {:.*}; {:.*}; {:.*};;\n",
                "{}{:.*};\n",
                "{}{:.*}; {:.*}; {:.*};;\n",
                "{}{:.*}; {:.*}; {:.*};;\n",
                "{}TextureFilename {{\n",
                "{}\"{}\";\n",
                "{}}}\n",
                "{}}}\n",
            ),
            level1, XFileStringWrapper(name),
            level2, PRECISION, diffuse.x, PRECISION, diffuse.y, PRECISION, diffuse.z, PRECISION, diffuse.w,
            level2, PRECISION, shininess,
            level2, PRECISION, specular.x, PRECISION, specular.y, PRECISION, specular.z,
            level2, PRECISION, emissive.x, PRECISION, emissive.y, PRECISION, emissive.z,
            level2,
            level3, XFileStringPathWrapper(tex_file),
            level2,
            level1
        )?;
        Ok(())
    }
}
struct XFileAiMeshWrapper<'a>(&'a AiScene, &'a AiMesh, DefaultRepeatedIndent);

impl<'a> Display for XFileAiMeshWrapper<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let XFileAiMeshWrapper(scene, mesh, level) = self;
        let mut level = *level;
        write!(
            f,
            "{level}Mesh {}_mShape {{\n",
            XFileStringWrapper(&mesh.name)
        )?;

        level = level.next();

        let vertices_len = mesh.vertices.len();
        write!(f, "{level}{};\n", vertices_len)?;
        if let Some((last_vertex, pre_vertices)) = mesh.vertices.split_last() {
            for vertex in pre_vertices.iter() {
                write!(
                    f,
                    "{level}{:.*};{:.*};{:.*};,\n",
                    PRECISION, vertex.x, PRECISION, vertex.y, PRECISION, vertex.z
                )?;
            }
            write!(
                f,
                "{level}{:.*};{:.*};{:.*};;\n",
                PRECISION, last_vertex.x, PRECISION, last_vertex.y, PRECISION, last_vertex.z
            )?;
        }

        // write all the faces
        let faces_len = mesh.faces.len();
        write!(f, "{level}{};\n", faces_len)?;
        if let Some((last_face, pre_faces)) = mesh.faces.split_last() {
            for face in pre_faces.iter() {
                let indices_len = face.indices.len();
                write!(f, "{level}{indices_len};")?;
                if let Some((last_index, pre_indices)) = face.indices.split_last() {
                    for index in pre_indices.iter() {
                        write!(f, "{index},")?;
                    }
                    write!(f, "{last_index};,\n")?;
                }
            }
            let indices_len = last_face.indices.len();
            write!(f, "{level}{indices_len};")?;
            if let Some((last_index, pre_indices)) = last_face.indices.split_last() {
                for index in pre_indices.iter() {
                    write!(f, "{index},")?;
                }
                write!(f, "{last_index};;\n")?;
            }
        }
        f.write_str("\n")?;

        // if mesh.has_texture_coords(0) {
        let mat: &AiMaterial = &scene.materials[mesh.material_index as usize];
        write!(f, "{level}MeshMaterialList {{\n")?;
        level = level.next();
        write!(f, "{level}1;\n")?;
        write!(f, "{level}{faces_len};\n")?;
        if faces_len > 0 {
            write!(f, "{level}")?;
            (0..faces_len - 1).try_for_each(|_| f.write_str("0, "))?;
            f.write_str("0;\n")?;
        }
        if mesh.has_texture_coords(0) {
            XFileMaterialWrapper(mat, level).fmt(f)?;
        }
        level = level.back();
        write!(f, "{level}}}")?;

        if mesh.has_normals() {
            f.write_str("\n")?;
            write!(f, "{level}MeshNormals {{\n")?;
            level = level.next();
            write!(f, "{level}{vertices_len};\n")?;
            if let Some((last_normal, pre_normals)) = mesh.normals.split_last() {
                for normal in pre_normals.iter() {
                    // because we have a LHS and also changed wth winding, we need to invert the
                    // normals again
                    write!(
                        f,
                        "{level}{:.*};{:.*};{:.*};,\n",
                        PRECISION, -normal.x, PRECISION, -normal.y, PRECISION, -normal.z
                    )?;
                }
                // because we have a LHS and also changed wth winding, we need to invert the normals
                // again
                write!(
                    f,
                    "{level}{:.*};{:.*};{:.*};;\n",
                    PRECISION, -last_normal.x, PRECISION, -last_normal.y, PRECISION, -last_normal.z
                )?;
            }

            write!(f, "{level}{};\n", mesh.faces.len())?;
            if let Some((last_face, pre_faces)) = mesh.faces.split_last() {
                for face in pre_faces.iter() {
                    let indices_len = face.indices.len();
                    write!(f, "{level}{indices_len};")?;
                    if let Some((last_index, pre_indices)) = face.indices.split_last() {
                        for index in pre_indices.iter() {
                            write!(f, "{index},")?;
                        }
                        write!(f, "{last_index};,\n")?;
                    }
                }
                let indices_len = last_face.indices.len();
                write!(f, "{level}{indices_len};")?;
                if let Some((last_index, pre_indices)) = last_face.indices.split_last() {
                    for index in pre_indices.iter() {
                        write!(f, "{index},")?;
                    }
                    write!(f, "{last_index};;\n")?;
                }
            }
            level = level.back();
            write!(f, "{level}}}\n")?;
        }

        // write texture UVs if available
        if mesh.has_texture_coords(0) {
            f.write_str("\n")?;
            write!(f, "{level}MeshTextureCoords {{\n")?;
            level = level.next();
            write!(f, "{level}{vertices_len};\n")?;
            if let Some((last_uv, pre_uvs)) = mesh.texture_coords[0].split_last() {
                for uv in pre_uvs.iter() {
                    write!(
                        f,
                        "{level}{:.*};{:.*};,\n",
                        PRECISION, uv.x, PRECISION, uv.y
                    )?;
                }
                write!(
                    f,
                    "{level}{:.*};{:.*};;\n",
                    PRECISION, last_uv.x, PRECISION, last_uv.y
                )?;
            }
            level = level.back();
            write!(f, "{level}}}\n")?;
        }

        // write color channel if available
        if mesh.has_vertex_colors(0) {
            f.write_str("\n")?;
            write!(f, "{level}MeshVertexColors {{\n")?;
            level = level.next();
            write!(f, "{level}{vertices_len};\n")?;
            if let Some((last_color, pre_colors)) = mesh.colors[0].split_last() {
                for (i, color) in pre_colors.iter().enumerate() {
                    write!(
                        f,
                        "{level}{};{:.*};{:.*};{:.*};{:.*};,\n",
                        i,
                        PRECISION,
                        color.x,
                        PRECISION,
                        color.y,
                        PRECISION,
                        color.z,
                        PRECISION,
                        color.w
                    )?;
                }
                write!(
                    f,
                    "{level}{};{:.*};{:.*};{:.*};{:.*};;\n",
                    vertices_len - 1,
                    PRECISION,
                    last_color.x,
                    PRECISION,
                    last_color.y,
                    PRECISION,
                    last_color.z,
                    PRECISION,
                    last_color.w
                )?;
            }
            level = level.back();
            write!(f, "{level}}}\n")?;
        }
        level = level.back();
        write!(f, "{level}}}\n")?;
        f.write_str("\n")?;

        Ok(())
    }
}

struct XFileStringWrapper<'a>(&'a str);

impl<'a> Display for XFileStringWrapper<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let src = self.0;
        let mut last_end = 0;
        for (start, part) in
            src.match_indices(|c: char| !(c.is_ascii_alphabetic() || c.is_ascii_digit()))
        {
            // SAFETY: last_end and start should be within the string and char boundary
            f.write_str(unsafe { src.get_unchecked(last_end..start) })?;
            f.write_str("_")?;
            last_end = start + part.len();
        }
        // SAFETY: last_end should be within the string and char boundary
        f.write_str(unsafe { src.get_unchecked(last_end..src.len()) })?;
        Ok(())
    }
}

struct XFileStringPathWrapper<'a>(&'a str);

impl<'a> Display for XFileStringPathWrapper<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let XFileStringPathWrapper(src) = self;
        // let mut dst = String::with_capacity(s.len());
        // // SAFETY: dst is an uninitialized string with enough capacity
        // let len = unsafe {
        //     let dst = dst.as_mut_vec();
        //     dst.set_len(s.len());
        //     encoding_rs::mem::convert_utf8_to_latin1_lossy(s.as_bytes(), dst)
        // };
        // // SAFETY: len should be within the string and char boundary
        // let src = unsafe { dst.get_unchecked(..len) };
        let mut last_end = 0;
        for (start, part) in src.match_indices('\\') {
            // SAFETY: last_end and start should be within the string and char boundary
            f.write_str(unsafe { src.get_unchecked(last_end..start) })?;
            f.write_str("/")?;
            last_end = start + part.len();
        }
        // SAFETY: last_end should be within the string and char boundary
        f.write_str(unsafe { src.get_unchecked(last_end..src.len()) })?;
        Ok(())
    }
}
