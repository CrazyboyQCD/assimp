use core::fmt::Write;
use core::{
    fmt::{Display, Formatter},
    ops::Range,
};

use crate::{
    formats::{Level, x::errors::XFileExportError},
    structs::{
        exporter::ExportProperties,
        material::AiStringPropertyType,
        mesh::AiMesh,
        scene::{AiNode, AiScene},
    },
    utils::float_precision::{Mat4, PRECISION},
};

pub struct Exporter<'source> {
    properties: &'source ExportProperties,
    scene: &'source AiScene,
}

macro_rules! _writeln {
    ($stream:expr $(,)?) => {
        writeln!($stream).map_err(XFileExportError::from)?;
    };
    ($stream:expr, $($arg:tt)*) => {
        writeln!($stream, $($arg)*).map_err(XFileExportError::from)?;
    };
}

macro_rules! _write {
    ($stream:expr, $($arg:tt)*) => {
        write!($stream, $($arg)*).map_err(XFileExportError::from)?;
    };
}

impl<'source> Exporter<'source> {
    pub fn new(scene: &'source AiScene, properties: &'source ExportProperties) -> Self {
        Self { scene, properties }
    }

    pub fn write_to_stream(&self, stream: &mut impl Write) -> Result<(), XFileExportError> {
        self.write_header(stream)?;
        let level = Level(1);
        _writeln!(stream, "Frame DXCC_ROOT {{");
        _write!(stream, "{}", XFileMat4Wrapper(&Mat4::IDENTITY, level));

        _write!(
            stream,
            "{}",
            XFileNodeWrapper(
                &self.scene.root.unwrap().get(&self.scene.nodes).unwrap(),
                &self.scene.nodes,
                &self.scene,
                level
            )
        );

        _writeln!(stream, "}}");
        Ok(())
    }

    /// Writes the asset header
    pub(crate) fn write_header(&self, stream: &mut impl Write) -> Result<(), XFileExportError> {
        let is_64_bits = self.properties.get_bool("AI_CONFIG_EXPORT_XFILE_64BIT");
        if is_64_bits {
            _writeln!(stream, "xof 0303txt 0064");
        } else {
            _writeln!(stream, "xof 0303txt 0032");
        }
        let level = Level(1);
        _writeln!(stream);
        _writeln!(stream, "template Frame {{");

        _writeln!(stream, "{level}<3d82ab46-62da-11cf-ab39-0020af71e433>");
        _writeln!(stream, "{level}[...]");
        _writeln!(stream, "}}");
        _writeln!(stream);

        _writeln!(stream, "template Matrix4x4 {{");
        _writeln!(stream, "{level}<f6f23f45-7686-11cf-8f52-0040333594a3>");
        _writeln!(stream, "{level}array FLOAT matrix[16];");
        _writeln!(stream, "}}");
        _writeln!(stream);

        _writeln!(stream, "template FrameTransformMatrix {{");
        _writeln!(stream, "{level}<f6f23f41-7686-11cf-8f52-0040333594a3>");
        _writeln!(stream, "{level}Matrix4x4 frameMatrix;");
        _writeln!(stream, "}}");
        _writeln!(stream);

        _writeln!(stream, "template Vector {{");
        _writeln!(stream, "{level}<3d82ab5e-62da-11cf-ab39-0020af71e433>");
        _writeln!(stream, "{level}FLOAT x;");
        _writeln!(stream, "{level}FLOAT y;");
        _writeln!(stream, "{level}FLOAT z;");
        _writeln!(stream, "}}");
        _writeln!(stream);

        _writeln!(stream, "template MeshFace {{");
        _writeln!(stream, "{level}<3d82ab5f-62da-11cf-ab39-0020af71e433>");
        _writeln!(stream, "{level}DWORD nFaceVertexIndices;");
        _writeln!(
            stream,
            "{level}array DWORD faceVertexIndices[nFaceVertexIndices];"
        );
        _writeln!(stream, "}}");
        _writeln!(stream);

        _writeln!(stream, "template Mesh {{");
        _writeln!(stream, "{level}<3d82ab44-62da-11cf-ab39-0020af71e433>");
        _writeln!(stream, "{level}DWORD nVertices;");
        _writeln!(stream, "{level}array Vector vertices[nVertices];");
        _writeln!(stream, "{level}DWORD nFaces;");
        _writeln!(stream, "{level}array MeshFace faces[nFaces];");
        _writeln!(stream, "{level}[...]");
        _writeln!(stream, "}}");
        _writeln!(stream);

        _writeln!(stream, "template MeshNormals {{");
        _writeln!(stream, "{level}<f6f23f43-7686-11cf-8f52-0040333594a3>");
        _writeln!(stream, "{level}DWORD nNormals;");
        _writeln!(stream, "{level}array Vector normals[nNormals];");
        _writeln!(stream, "{level}DWORD nFaceNormals;");
        _writeln!(stream, "{level}array MeshFace faceNormals[nFaceNormals];");
        _writeln!(stream, "}}");
        _writeln!(stream);

        _writeln!(stream, "template Coords2d {{");
        _writeln!(stream, "{level}<f6f23f44-7686-11cf-8f52-0040333594a3>");
        _writeln!(stream, "{level}FLOAT u;");
        _writeln!(stream, "{level}FLOAT v;");
        _writeln!(stream, "}}");
        _writeln!(stream);

        _writeln!(stream, "template MeshTextureCoords {{");
        _writeln!(stream, "{level}<f6f23f40-7686-11cf-8f52-0040333594a3>");
        _writeln!(stream, "{level}DWORD nTextureCoords;");
        _writeln!(
            stream,
            "{level}array Coords2d textureCoords[nTextureCoords];"
        );
        _writeln!(stream, "}}");
        _writeln!(stream);

        _writeln!(stream, "template ColorRGBA {{");
        _writeln!(stream, "{level}<35ff44e0-6c7c-11cf-8f52-0040333594a3>");
        _writeln!(stream, "{level}FLOAT red;");
        _writeln!(stream, "{level}FLOAT green;");
        _writeln!(stream, "{level}FLOAT blue;");
        _writeln!(stream, "{level}FLOAT alpha;");
        _writeln!(stream, "}}");
        _writeln!(stream);

        _writeln!(stream, "template IndexedColor {{");
        _writeln!(stream, "{level}<1630b820-7842-11cf-8f52-0040333594a3>");
        _writeln!(stream, "{level}DWORD index;");
        _writeln!(stream, "{level}ColorRGBA indexColor;");
        _writeln!(stream, "}}");
        _writeln!(stream);

        _writeln!(stream, "template MeshVertexColors {{");
        _writeln!(stream, "{level}<1630b821-7842-11cf-8f52-0040333594a3>");
        _writeln!(stream, "{level}DWORD nVertexColors;");
        _writeln!(
            stream,
            "{level}array IndexedColor vertexColors[nVertexColors];"
        );
        _writeln!(stream, "}}");
        _writeln!(stream);

        _writeln!(stream, "template VertexElement {{");
        _writeln!(stream, "{level}<f752461c-1e23-48f6-b9f8-8350850f336f>");
        _writeln!(stream, "{level}DWORD Type;");
        _writeln!(stream, "{level}DWORD Method;");
        _writeln!(stream, "{level}DWORD Usage;");
        _writeln!(stream, "{level}DWORD UsageIndex;");
        _writeln!(stream, "}}");
        _writeln!(stream);

        _writeln!(stream, "template DeclData {{");
        _writeln!(stream, "{level}<bf22e553-292c-4781-9fea-62bd554bdd93>");
        _writeln!(stream, "{level}DWORD nElements;");
        _writeln!(stream, "{level}array VertexElement Elements[nElements];");
        _writeln!(stream, "{level}DWORD nDWords;");
        _writeln!(stream, "{level}array DWORD data[nDWords];");
        _writeln!(stream, "}}");
        _writeln!(stream);

        Ok(())
    }
}

struct XFileNodeWrapper<'a>(&'a AiNode, &'a Vec<AiNode>, &'a AiScene, Level);

impl<'a> Display for XFileNodeWrapper<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let XFileNodeWrapper(node, nodes, scene, level) = self;
        let mut level = *level;
        if node.name.is_empty() {
            writeln!(
                f,
                "{level}Frame {} {{",
                XFileStringWrapper(&format!("Node_{:p}", node))
            )?;
        } else {
            writeln!(f, "{level}Frame {} {{", XFileStringWrapper(&node.name))?;
        }
        level = level.next();
        write!(f, "{}", XFileMat4Wrapper(&node.transformation, level))?;
        let Range { start, end } = node.meshes;
        for mesh in &scene.meshes[start as usize..end as usize] {
            write!(f, "{}", XFileAiMeshWrapper(scene, mesh, level))?;
        }

        // recursive call the Nodes
        for i in &node.children {
            write!(
                f,
                "{}",
                XFileNodeWrapper(i.get(nodes).unwrap(), nodes, scene, level)
            )?;
        }

        level = level.back();
        writeln!(f, "{level}}}")?;
        writeln!(f)?;
        Ok(())
    }
}
struct XFileMat4Wrapper<'a>(&'a Mat4, Level);

impl<'a> Display for XFileMat4Wrapper<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let XFileMat4Wrapper(m, level) = self;
        let mut level = *level;
        writeln!(f, "{}FrameTransformMatrix {{", level)?;
        level = level.next();
        write!(f, "{level}{:.*}, ", PRECISION, m.x_axis.x)?;
        write!(f, "{:.*}, ", PRECISION, m.y_axis.x)?;
        write!(f, "{:.*}, ", PRECISION, m.z_axis.x)?;
        writeln!(f, "{:.*},", PRECISION, m.w_axis.x)?;

        write!(f, "{level}{:.*}, ", PRECISION, m.x_axis.y)?;
        write!(f, "{:.*}, ", PRECISION, m.y_axis.y)?;
        write!(f, "{:.*}, ", PRECISION, m.z_axis.y)?;
        writeln!(f, "{:.*},", PRECISION, m.w_axis.y)?;

        write!(f, "{level}{:.*}, ", PRECISION, m.x_axis.z)?;
        write!(f, "{:.*}, ", PRECISION, m.y_axis.z)?;
        write!(f, "{:.*}, ", PRECISION, m.z_axis.z)?;
        writeln!(f, "{:.*},", PRECISION, m.w_axis.z)?;

        write!(f, "{level}{:.*}, ", PRECISION, m.x_axis.w)?;
        write!(f, "{:.*}, ", PRECISION, m.y_axis.w)?;
        write!(f, "{:.*}, ", PRECISION, m.z_axis.w)?;
        writeln!(f, "{:.*};;", PRECISION, m.w_axis.w)?;
        level = level.back();
        writeln!(f, "{}}}", level)?;
        writeln!(f)?;
        Ok(())
    }
}

struct XFileAiMeshWrapper<'a>(&'a AiScene, &'a AiMesh, Level);

impl<'a> Display for XFileAiMeshWrapper<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let XFileAiMeshWrapper(scene, mesh, level) = self;
        let mut level = *level;
        writeln!(
            f,
            "{level}Mesh {}_mShape {{",
            XFileStringWrapper(&mesh.name)
        )?;

        level = level.next();

        let vertices_len = mesh.vertices.len();
        writeln!(f, "{level}{};", vertices_len)?;
        if let Some((last_vertex, pre_vertices)) = mesh.vertices.split_last() {
            for vertex in pre_vertices.iter() {
                writeln!(
                    f,
                    "{level}{:.*};{:.*};{:.*};,",
                    PRECISION, vertex.x, PRECISION, vertex.y, PRECISION, vertex.z
                )?;
            }
            writeln!(
                f,
                "{level}{:.*};{:.*};{:.*};;",
                PRECISION, last_vertex.x, PRECISION, last_vertex.y, PRECISION, last_vertex.z
            )?;
        }

        let faces_len = mesh.faces.len();
        writeln!(f, "{level}{};", faces_len)?;
        if let Some((last_face, pre_faces)) = mesh.faces.split_last() {
            for face in pre_faces.iter() {
                let indices_len = face.indices.len();
                write!(f, "{level}{};", indices_len)?;
                if let Some((last_index, pre_indices)) = face.indices.split_last() {
                    for index in pre_indices.iter() {
                        write!(f, "{index},")?;
                    }
                    writeln!(f, "{last_index};,")?;
                }
            }
            let indices_len = last_face.indices.len();
            write!(f, "{level}{};", indices_len)?;
            if let Some((last_index, pre_indices)) = last_face.indices.split_last() {
                for index in pre_indices.iter() {
                    write!(f, "{index},")?;
                }
                writeln!(f, "{last_index};;")?;
            }
        }
        writeln!(f)?;

        if mesh.has_texture_coords(0) {
            let mat = &scene.materials[mesh.material_index as usize];
            let tex_file = mat
                .get_string_property("", 0, AiStringPropertyType::TextureDiffuse)
                .unwrap_or_default();
            writeln!(f, "{}MeshMaterialList {{", level)?;
            level = level.next();
            writeln!(f, "{level}1;")?;
            writeln!(f, "{level}{faces_len};")?;
            if faces_len > 0 {
                write!(f, "{level}")?;
                (0..faces_len - 1).try_for_each(|_| write!(f, "0, "))?;
                writeln!(f, "0;")?;
            }
            writeln!(f, "{level}Material {{")?;
            level = level.next();
            writeln!(f, "{level}1.0; 1.0; 1.0; 1.000000;;")?;
            writeln!(f, "{level}1.000000;")?;
            writeln!(f, "{level}0.000000; 0.000000; 0.000000;;")?;
            writeln!(f, "{level}0.000000; 0.000000; 0.000000;;")?;
            write!(f, "{level}TextureFilename {{ \"")?;
            write!(f, "{}", XFileStringPathWrapper(&tex_file))?;

            writeln!(f, "\"; }}")?;

            level = level.back();

            writeln!(f, "{level}}}")?;
            level = level.back();
            writeln!(f, "{level}}}")?;
        }

        if mesh.has_normals() {
            writeln!(f)?;
            writeln!(f, "{level}MeshNormals {{")?;
            writeln!(f, "{level}{};", vertices_len)?;
            if let Some((last_normal, pre_normals)) = mesh.normals.split_last() {
                for normal in pre_normals.iter() {
                    // because we have a LHS and also changed wth winding, we need to invert the normals again
                    writeln!(
                        f,
                        "{level}{:.*};{:.*};{:.*};,",
                        PRECISION, -normal.x, PRECISION, -normal.y, PRECISION, -normal.z
                    )?;
                }
                // because we have a LHS and also changed wth winding, we need to invert the normals again
                writeln!(
                    f,
                    "{level}{:.*};{:.*};{:.*};;",
                    PRECISION, -last_normal.x, PRECISION, -last_normal.y, PRECISION, -last_normal.z
                )?;
            }

            writeln!(f, "{level}{};", mesh.faces.len())?;
            if let Some((last_face, pre_faces)) = mesh.faces.split_last() {
                for face in pre_faces.iter() {
                    let indices_len = face.indices.len();
                    write!(f, "{level}{};", indices_len)?;
                    if let Some((last_index, pre_indices)) = face.indices.split_last() {
                        for index in pre_indices.iter() {
                            write!(f, "{index},")?;
                        }
                        writeln!(f, "{last_index};,")?;
                    }
                }
                let indices_len = last_face.indices.len();
                write!(f, "{level}{};", indices_len)?;
                if let Some((last_index, pre_indices)) = last_face.indices.split_last() {
                    for index in pre_indices.iter() {
                        write!(f, "{index},")?;
                    }
                    writeln!(f, "{last_index};;")?;
                }
            }
            writeln!(f, "{level}}}")?;
        }

        // write texture UVs if available
        if mesh.has_texture_coords(0) {
            writeln!(f)?;
            writeln!(f, "{level}MeshTextureCoords {{")?;
            writeln!(f, "{level}{};", vertices_len)?;
            if let Some((last_uv, pre_uvs)) = mesh.texture_coords[0].split_last() {
                for uv in pre_uvs.iter() {
                    writeln!(
                        f,
                        "{level}{:.*};{:.*};,",
                        PRECISION,
                        uv.x,
                        PRECISION,
                        1.0 - uv.y
                    )?;
                }
                writeln!(
                    f,
                    "{level}{:.*};{:.*};;",
                    PRECISION,
                    last_uv.x,
                    PRECISION,
                    1.0 - last_uv.y
                )?;
            }
            writeln!(f, "{level}}}")?;
        }

        // write color channel if available
        if mesh.has_vertex_colors(0) {
            writeln!(f)?;
            writeln!(f, "{level}MeshVertexColors {{")?;
            writeln!(f, "{level}{};", vertices_len)?;
            if let Some((last_color, pre_colors)) = mesh.colors[0].split_last() {
                for (i, color) in pre_colors.iter().enumerate() {
                    writeln!(
                        f,
                        "{level}{};{};{};{};{},",
                        i, color.x, color.y, color.z, color.w
                    )?;
                }
                writeln!(
                    f,
                    "{level}{};{};{};{};{};",
                    vertices_len - 1,
                    last_color.x,
                    last_color.y,
                    last_color.z,
                    last_color.w
                )?;
            }
            writeln!(f, "{level}}}")?;
        }
        level = level.back();
        writeln!(f, "{}}}", level)?;
        writeln!(f)?;

        Ok(())
    }
}

struct XFileStringWrapper<'a>(&'a str);

impl<'a> Display for XFileStringWrapper<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "{}",
            self.0.replace(
                |c: char| !(c.is_ascii_alphabetic() || c.is_ascii_digit()),
                "_"
            )
        )?;
        Ok(())
    }
}

struct XFileStringPathWrapper<'a>(&'a str);

impl<'a> Display for XFileStringPathWrapper<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let XFileStringPathWrapper(s) = self;
        let mut dst = " ".repeat(s.len());
        let len = encoding_rs::mem::convert_utf8_to_latin1_lossy(s.as_bytes(), unsafe {
            dst.as_mut_vec()
        });
        f.write_str(&dst[..len].replace("\\", "/"))?;
        Ok(())
    }
}
