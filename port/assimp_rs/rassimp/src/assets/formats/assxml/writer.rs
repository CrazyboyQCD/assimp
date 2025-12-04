use core::fmt::{self, Debug, Write};
use std::path::Path;

use crate::{
    assets::formats::{AscTimeFormatter, CustomRepeatedString},
    structs::{
        material::property::AiProperty, mesh::primitive_type::AiPrimitiveType, node::AiNode,
        scene::AiScene,
    },
    utils::{ai_get_version_major, ai_get_version_minor, ai_get_version_patch},
};

pub struct AssxmlWriter<'source, W: Write> {
    writer: &'source mut W,
    scene: &'source AiScene,
}

impl<'source, W: Write> AssxmlWriter<'source, W> {
    pub fn new(scene: &'source AiScene, writer: &'source mut W) -> Self {
        Self { scene, writer }
    }
    pub fn write_dump(&mut self, path: &Path, cmd: &str, shortened: bool) -> fmt::Result {
        #[rustfmt::skip]
        write!(
            self.writer,
            concat!(
                "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n",
                "<ASSIMP format_id=\"1\">\n\n",
                "<!-- XML Model dump produced by rassimp dump\n",
                "  Library version: {}.{}.{}\n",
                "  Source: {}\n",
                "  Command line: {}\n",
                "  {}\n",
                "-->\n\n",
                "<Scene flags=\"{}\" postprocessing=\"{}\">\n",
            ),
            ai_get_version_major(), ai_get_version_minor(), ai_get_version_patch(),
            path.display(),
            cmd,
            AscTimeFormatter::now(),
            self.scene.flags.bits(), 0
        )?;
        if !self.scene.nodes.is_empty() {
            for node in self.scene.nodes.iter().filter(|node| node.is_root()) {
                self.write_node(node, &self.scene.nodes, 0)?;
            }
        }

        if !self.scene.textures.is_empty() {
            write!(
                self.writer,
                "<TextureList num=\"{}\">\n",
                self.scene.textures.len()
            )?;
            for tex in &self.scene.textures {
                let compressed = tex.height == 0;
                write!(
                    self.writer,
                    "\t<Texture name=\"{}\" width=\"{}\" height=\"{}\" compressed=\"{compressed}\">\n",
                    if tex.file_name.is_empty() {
                        "unknown"
                    } else {
                        &tex.file_name
                    },
                    if compressed { -1 } else { tex.width as i64 },
                    if compressed { -1 } else { tex.height as i64 },
                )?;

                if compressed {
                    write!(self.writer, "\t\t<Data length=\"{}\">\n", tex.width)?;
                    if !shortened {
                        for i in 0..tex.width {
                            // TODO: print by format
                            // let index = (i / 4) as usize;
                            // let color_index = (i % 4) as usize;
                            // let texel = &tex.pc_data[index];
                            // write!(
                            //     self.writer,
                            //     "\t\t\t{:2x}",
                            //     match color_index {
                            //         0 => texel.b,
                            //         1 => texel.r,
                            //         2 => texel.g,
                            //         3 => texel.a,
                            //         _ => unreachable!(),
                            //     }
                            // )?;
                            if i > 0 && i % 50 == 0 {
                                self.writer.write_str("\n")?;
                            }
                        }
                    }
                } else if !shortened {
                    write!(
                        self.writer,
                        "\t\t<Data length=\"{}\">\n",
                        tex.width * tex.height * 4
                    )?;

                    for y in 0..tex.height {
                        let row = y * tex.width;
                        for x in 0..tex.width {
                            // TODO: print by format
                            // let tx = &tex.pc_data[(row + x) as usize];
                            // write!(
                            //     self.writer,
                            //     "\t\t\t{:2x} {:2x} {:2x} {:2x}",
                            //     tx.r, tx.g, tx.b, tx.a
                            // )?;

                            // // group by four for readability
                            // if (x + y * tex.width) % 4 == 0 {
                            //     self.writer.write_str("\n")?;
                            // }
                        }
                    }
                }
                self.writer.write_str("\t\t</Data>\n\t</Texture>\n")?;
            }
            self.writer.write_str("</TextureList>\n")?;
        }

        if !self.scene.materials.is_empty() {
            write!(
                self.writer,
                "<MaterialList num=\"{}\">\n",
                self.scene.materials.len()
            )?;
            for mat in &self.scene.materials {
                self.writer.write_str("\t<Material>\n")?;
                write!(
                    self.writer,
                    "\t\t<MatPropertyList  num=\"{}\">\n",
                    mat.properties.len()
                )?;
                for prop in &mat.properties {
                    write!(
                        self.writer,
                        "\t\t\t<MatProperty key=\"{}\"\n\t\t\ttype=\"{:?}\" tex_index=\"{}\" value=\"{}\">",
                        prop.property.get_field_name(),
                        prop.r#type,
                        prop.index,
                        XmlAiProperty(&prop.property)
                    )?;
                    self.writer.write_str("\n\n\t\t\t</MatProperty>\n")?;
                }
                self.writer.write_str("\t\t</MatPropertyList>\n")?;
                self.writer.write_str("\t</Material>\n")?;
            }
            self.writer.write_str("</MaterialList>\n")?;
        }

        if !self.scene.animations.is_empty() {
            write!(
                self.writer,
                "<AnimationList num=\"{}\">\n",
                self.scene.animations.len()
            )?;
            for anim in &self.scene.animations {
                // anim header
                write!(
                    self.writer,
                    "\t<Animation name=\"{}\" duration=\"{}\" tick_cnt=\"{}\">\n",
                    XmlString(&anim.name),
                    anim.duration,
                    anim.ticks_per_second
                )?;

                // write bone animation channels
                if !anim.channels.is_empty() {
                    write!(
                        self.writer,
                        "\t\t<NodeAnimList num=\"{}\">\n",
                        anim.channels.len()
                    )?;
                    for channel in &anim.channels {
                        // node anim header
                        write!(
                            self.writer,
                            "\t\t\t<NodeAnim node=\"{}\">\n",
                            XmlString(&channel.node_name)
                        )?;
                        if !shortened {
                            // write position keys
                            if !channel.position_keys.is_empty() {
                                write!(
                                    self.writer,
                                    "\t\t\t\t<PositionKeyList num=\"{}\">\n",
                                    channel.position_keys.len()
                                )?;
                                for pk in &channel.position_keys {
                                    write!(
                                        self.writer,
                                        "\t\t\t\t\t<PositionKey time=\"{}\">\n\t\t\t\t\t\t{:.8} {:.8} {:.8}\n\t\t\t\t\t</PositionKey>\n",
                                        pk.time, pk.value.x, pk.value.y, pk.value.z
                                    )?;
                                }
                                self.writer.write_str("\t\t\t\t\t</PositionKeyList>\n")?;
                            }

                            // write scaling keys
                            if !channel.scaling_keys.is_empty() {
                                write!(
                                    self.writer,
                                    "\t\t\t\t<ScalingKeyList num=\"{}\">\n",
                                    channel.scaling_keys.len()
                                )?;
                                for sk in &channel.scaling_keys {
                                    write!(
                                        self.writer,
                                        "\t\t\t\t\t<ScalingKey time=\"{}\">\n\t\t\t\t\t\t{:.8} {:.8} {:.8}\n\t\t\t\t\t</ScalingKey>\n",
                                        sk.time, sk.value.x, sk.value.y, sk.value.z
                                    )?;
                                }
                                self.writer.write_str("\t\t\t\t\t</ScalingKeyList>\n")?;
                            }

                            // write rotation keys
                            if !channel.rotation_keys.is_empty() {
                                write!(
                                    self.writer,
                                    "\t\t\t\t<RotationKeyList num=\"{}\">\n",
                                    channel.rotation_keys.len()
                                )?;
                                for rk in &channel.rotation_keys {
                                    write!(
                                        self.writer,
                                        "\t\t\t\t\t<RotationKey time=\"{}\">\n\t\t\t\t\t\t{:.8} {:.8} {:.8}\n\t\t\t\t\t</RotationKey>\n",
                                        rk.time, rk.value.x, rk.value.y, rk.value.z
                                    )?;
                                }
                                self.writer.write_str("\t\t\t\t\t</RotationKeyList>\n")?;
                            }
                        }
                        self.writer.write_str("\t\t\t</NodeAnim>\n")?;
                    }
                    self.writer.write_str("\t\t</NodeAnimList>\n")?;
                }
                self.writer.write_str("\t</Animation>\n")?;
            }
            self.writer.write_str("</AnimationList>\n")?;
        }

        // write meshes
        if !self.scene.meshes.is_empty() {
            write!(
                self.writer,
                "<MeshList num=\"{}\">\n",
                self.scene.meshes.len()
            )?;
            for mesh in &self.scene.meshes {
                // mesh header
                write!(
                    self.writer,
                    "\t<Mesh types=\"{} {} {} {}\" material_index=\"{}\">\n",
                    if mesh.primitive_types.contains(AiPrimitiveType::Point) {
                        "points"
                    } else {
                        ""
                    },
                    if mesh.primitive_types.contains(AiPrimitiveType::Line) {
                        "lines"
                    } else {
                        ""
                    },
                    if mesh.primitive_types.contains(AiPrimitiveType::Triangle) {
                        "triangles"
                    } else {
                        ""
                    },
                    if mesh.primitive_types.contains(AiPrimitiveType::Polygon) {
                        "polygons"
                    } else {
                        ""
                    },
                    mesh.material_index
                )?;

                // bones
                if !mesh.bones.is_empty() {
                    write!(self.writer, "\t\t<BoneList num=\"{}\">\n", mesh.bones.len())?;
                    for bone in &mesh.bones {
                        let m = &bone.offset_matrix;
                        #[rustfmt::skip]
                        write!(
                            self.writer,
                            concat!(
                                "\t\t\t<Bone name=\"{}\">\n",
                                "\t\t\t\t<Matrix4>\n",
                                "\t\t\t\t\t{:.6} {:.6} {:.6} {:.6}\n",
                                "\t\t\t\t\t{:.6} {:.6} {:.6} {:.6}\n",
                                "\t\t\t\t\t{:.6} {:.6} {:.6} {:.6}\n",
                                "\t\t\t\t\t{:.6} {:.6} {:.6} {:.6}\n",
                                "\t\t\t\t</Matrix4>\n"
                            ),
                            XmlString(&bone.name),
                            m.x_axis.x, m.y_axis.x, m.z_axis.x, m.w_axis.x,
                            m.x_axis.y, m.y_axis.y, m.z_axis.y, m.w_axis.y,
                            m.x_axis.z, m.y_axis.z, m.z_axis.z, m.w_axis.z,
                            m.x_axis.w, m.y_axis.w, m.z_axis.w, m.w_axis.w,
                        )?;
                        if !shortened && !bone.weights.is_empty() {
                            write!(
                                self.writer,
                                "\t\t\t\t<WeightList num=\"{}\">\n",
                                bone.weights.len()
                            )?;
                            for weight in &bone.weights {
                                write!(
                                    self.writer,
                                    "\t\t\t\t\t<Weight index=\"{}\">\n\t\t\t\t\t\t{}\n\t\t\t\t\t</Weight>\n",
                                    weight.vertex_id, weight.weight
                                )?;
                            }
                            self.writer.write_str("\t\t\t\t</WeightList>\n")?;
                        }
                        self.writer.write_str("\t\t\t</Bone>\n")?;
                    }
                    self.writer.write_str("\t\t</BoneList>\n")?;
                }

                // faces
                if !shortened && !mesh.faces.is_empty() {
                    write!(self.writer, "\t\t<FaceList num=\"{}\">\n", mesh.faces.len())?;
                    for face in &mesh.faces {
                        write!(
                            self.writer,
                            "\t\t\t<Face num=\"{}\">\n\t\t\t\t",
                            face.indices.len()
                        )?;
                        for index in &face.indices {
                            write!(self.writer, "{} ", index)?;
                        }
                        self.writer.write_str("\n\t\t\t</Face>\n")?;
                    }
                    self.writer.write_str("\t\t</FaceList>\n")?;
                }

                // vertex positions
                if mesh.has_positions() {
                    write!(
                        self.writer,
                        "\t\t<Positions num=\"{}\" set=\"0\" num_components=\"3\">\n",
                        mesh.vertices.len()
                    )?;
                    if !shortened {
                        for v in &mesh.vertices {
                            write!(self.writer, "\t\t{:.8} {:.8} {:.8}\n", v.x, v.y, v.z)?;
                        }
                    }
                    self.writer.write_str("\t\t</Positions>\n")?;
                }

                // vertex normals
                if mesh.has_normals() {
                    write!(
                        self.writer,
                        "\t\t<Normals num=\"{}\" set=\"0\" num_components=\"3\">\n",
                        mesh.vertices.len()
                    )?;
                    if !shortened {
                        for normal in &mesh.normals[..mesh.vertices.len()] {
                            write!(
                                self.writer,
                                "\t\t{:.8} {:.8} {:.8}\n",
                                normal.x, normal.y, normal.z
                            )?;
                        }
                    }
                    self.writer.write_str("\t\t</Normals>\n")?;
                }

                // vertex tangents and bitangents
                if mesh.has_tangents_and_bitangents() {
                    write!(
                        self.writer,
                        "\t\t<Tangents num=\"{}\" set=\"0\" num_components=\"3\">\n",
                        mesh.vertices.len()
                    )?;
                    if !shortened {
                        for tangent in &mesh.tangents[..mesh.vertices.len()] {
                            write!(
                                self.writer,
                                "\t\t{:.8} {:.8} {:.8}\n",
                                tangent.x, tangent.y, tangent.z
                            )?;
                        }
                    }
                    self.writer.write_str("\t\t</Tangents>\n")?;

                    write!(
                        self.writer,
                        "\t\t<Bitangents num=\"{}\" set=\"0\" num_components=\"3\">\n",
                        mesh.vertices.len()
                    )?;
                    if !shortened {
                        for n in 0..mesh.vertices.len() {
                            write!(
                                self.writer,
                                "\t\t{:.8} {:.8} {:.8}\n",
                                mesh.bitangents[n].x, mesh.bitangents[n].y, mesh.bitangents[n].z
                            )?;
                        }
                    }
                    self.writer.write_str("\t\t</Bitangents>\n")?;
                }

                // texture coordinates
                for (a, (texture_coords, &num_of_uv_component)) in mesh
                    .texture_coords
                    .iter()
                    .zip(mesh.num_of_uv_components.iter())
                    .enumerate()
                {
                    if texture_coords.is_empty() {
                        break;
                    }

                    write!(
                        self.writer,
                        "\t\t<TextureCoords num=\"{}\" set=\"{}\" name=\"{}\" num_components=\"{}\">\n",
                        mesh.vertices.len(),
                        a,
                        mesh.get_texture_coords_name(a).unwrap_or_default(),
                        num_of_uv_component
                    )?;

                    if !shortened {
                        if num_of_uv_component == 3 {
                            for texture_coord in texture_coords.iter().take(mesh.vertices.len()) {
                                write!(
                                    self.writer,
                                    "\t\t{:.8} {:.8} {:.8}\n",
                                    texture_coord.x, texture_coord.y, texture_coord.z
                                )?;
                            }
                        } else {
                            for texture_coord in texture_coords.iter().take(mesh.vertices.len()) {
                                write!(
                                    self.writer,
                                    "\t\t{:.8} {:.8}\n",
                                    texture_coord.x, texture_coord.y
                                )?;
                            }
                        }
                    }
                    self.writer.write_str("\t\t</TextureCoords>\n")?;
                }

                // vertex colors
                for (a, colors) in mesh.colors.iter().enumerate() {
                    if colors.is_empty() {
                        break;
                    }
                    write!(
                        self.writer,
                        "\t\t<Colors num=\"{}\" set=\"{}\" num_components=\"4\">\n",
                        mesh.vertices.len(),
                        a
                    )?;
                    if !shortened {
                        for color in &colors[..mesh.vertices.len()] {
                            write!(
                                self.writer,
                                "\t\t{:.8} {:.8} {:.8} {:.8}\n",
                                color.x, color.y, color.z, color.w
                            )?;
                        }
                    }
                    self.writer.write_str("\t\t</Colors>\n")?;
                }
                self.writer.write_str("\t</Mesh>\n")?;
            }
            self.writer.write_str("\t</MeshList>\n")?;
        }
        self.writer.write_str("</Scene>\n</ASSIMP>")?;
        Ok(())
    }

    fn write_node(&mut self, node: &AiNode, nodes: &[AiNode], depth: usize) -> fmt::Result {
        let prefix = CustomRepeatedString::new(depth, "\t");
        let m = &node.transformation;

        #[rustfmt::skip]
        write!(
            self.writer,
            concat!(
                "{}<Node name=\"{}\">",
                "{}\t<Matrix4>\n",
                "{}\t\t{:.6} {:.6} {:.6} {:.6}\n",
                "{}\t\t{:.6} {:.6} {:.6} {:.6}\n",
                "{}\t\t{:.6} {:.6} {:.6} {:.6}\n",
                "{}\t\t{:.6} {:.6} {:.6} {:.6}\n",
                "{}\t</Matrix4>\n"
            ),
            prefix, XmlString(&node.name),
            prefix,
            prefix, m.x_axis.x, m.y_axis.x, m.z_axis.x, m.w_axis.x,
            prefix, m.x_axis.y, m.y_axis.y, m.z_axis.y, m.w_axis.y,
            prefix, m.x_axis.z, m.y_axis.z, m.z_axis.z, m.w_axis.z,
            prefix, m.x_axis.w, m.y_axis.w, m.z_axis.w, m.w_axis.w,
            prefix,
        )?;

        if !node.meshes.is_empty() {
            write!(
                self.writer,
                "{prefix}\t<MeshRefs num=\"{}\">\n{prefix}\t",
                node.meshes.len()
            )?;
            for &m in &node.meshes {
                write!(self.writer, "{m} ")?;
            }
            write!(self.writer, "\n{prefix}\t</MeshRefs>\n")?;
        }
        if !node.children.is_empty() {
            write!(
                self.writer,
                "{prefix}\t<NodeList num=\"{}\">\n",
                node.children.len()
            )?;
            for &i in &node.children {
                self.write_node(
                    i.get(nodes)
                        .expect("each node's children should correspond to a node in nodes"),
                    nodes,
                    depth + 2,
                )?;
            }
            write!(self.writer, "{prefix}\t</NodeList>\n")?;
        }
        write!(self.writer, "{prefix}</Node>\n")?;
        Ok(())
    }
}

struct XmlString<'a>(&'a str);

impl<'a> fmt::Display for XmlString<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let src = self.0;
        let mut last_end = 0;
        for (start, part) in src.match_indices(['<', '>', '&', '"', '\'']) {
            // SAFETY: last_end and start should be within the string and char boundary
            f.write_str(unsafe { src.get_unchecked(last_end..start) })?;
            f.write_str(match part {
                "<" => "&lt;",
                ">" => "&gt;",
                "&" => "&amp;",
                "\"" => "&quot;",
                "'" => "&apos;",
                // SAFETY: part should be one of the above characters
                _ => unsafe { core::hint::unreachable_unchecked() },
            })?;
            last_end = start + part.len();
        }
        // SAFETY: last_end should be within the string and char boundary
        f.write_str(unsafe { src.get_unchecked(last_end..src.len()) })?;
        Ok(())
    }
}

struct XmlAiProperty<'a>(&'a AiProperty);

impl<'a> fmt::Display for XmlAiProperty<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(s) = self.0.get_inner_string() {
            XmlString(s).fmt(f)
        } else {
            self.0.fmt(f)
        }
    }
}
