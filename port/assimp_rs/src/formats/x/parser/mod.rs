use core::marker::PhantomData;
use std::{borrow::Cow, mem};

#[cfg(feature = "compression")]
use zlib_rs::{InflateFlush, MAX_WBITS};

mod binary_parser;
mod text_parser;

use binary_parser::BinaryParser;
use text_parser::TextParser;

use crate::{
    AiReal,
    formats::x::{
        errors::{XFileImportError, XFileParseError},
        structs::{
            AnimBone, Animation, Bone, BoneWeight, Face, Material, MatrixKey, Mesh, Node, Scene,
            TexEntry,
        },
    },
    structs::{
        anim::AiAnimInterpolation,
        color::{Color3D, Color4D},
        key::{AiQuatKey, AiVectorKey},
        nodes::Index,
    },
    utils::{
        compression::{Compression, Format},
        float_precision::{Mat4, Quat, Vec2, Vec3, Vec4},
        read::parse_4digits_decimal,
    },
};

const AI_MAX_NUMBER_OF_TEXTURECOORDS: usize = 0x8;

const MSZIP_BLOCK: usize = 32786;
const MSZIP_MAGIC: u16 = u16::from_le_bytes([b'C', b'K']);

pub struct Parser;

#[derive(Debug, Clone, Copy)]
pub struct XFileHeader {
    pub major_version: u8,
    pub minor_version: u8,
    pub is_compressed: bool,
    pub is_binary_format: bool,
    pub binary_float_size: u8,
}

impl XFileHeader {
    const HEADER_BINARY_SIZE: usize = 16;
}

#[derive(Debug, Clone)]
pub struct XFile {
    pub header: XFileHeader,
    pub scene: Scene,
}

impl Parser {
    pub fn parse<'source>(source: &'source [u8]) -> Result<XFile, XFileImportError> {
        let (header, source) = Self::parse_header(source)?;

        let XFileHeader {
            is_compressed,
            is_binary_format,
            binary_float_size,
            ..
        } = header;

        Ok(XFile {
            header,
            scene: {
                let mut scene = if is_compressed {
                    Self::parse_compressed_file(source, is_binary_format, binary_float_size)?
                } else {
                    Self::parse_by_format(source, is_binary_format, binary_float_size)?
                };
                Self::filter_hierarchy(&mut scene);
                scene
            },
        })
    }

    /// Filters the imported hierarchy for some degenerated cases that some exporters produce.
    fn filter_hierarchy(scene: &mut Scene) {
        if let Some(root) = scene.root_node {
            let mut filter = vec![];
            let mut stack = vec![root];
            while let Some(node) = stack.pop() {
                let mut node =
                    mem::replace(scene.nodes.get_mut(node.value()).unwrap(), Node::default());
                // if the node has just a single unnamed child containing a mesh, remove
                // the anonymous node between. The 3DSMax kwXport plugin seems to produce this
                // mess in some cases
                if node.meshes.is_empty() && node.children.len() == 1 {
                    let child = *node.children.first().unwrap();
                    let child = scene.nodes.get_mut(child.value()).unwrap();
                    if child.name.is_empty() && !child.meshes.is_empty() {
                        // transfer its meshes to us
                        node.meshes.extend(child.meshes.drain(..));
                        node.transformation_matrix *= child.transformation_matrix;
                    }
                }
                stack.extend(node.children.drain(..));
                filter.push(node);
            }
            scene.nodes = filter;
        }
    }

    fn parse_header<'source>(
        source: &'source [u8],
    ) -> Result<(XFileHeader, &'source [u8]), XFileImportError> {
        let (header, rest): (&[u8; 16], &'source [u8]) = match source.split_at_checked(16) {
            Some((header, rest)) => (header.try_into().unwrap(), rest),
            None => {
                return Err(XFileImportError::XFileParseError {
                    position: 0.to_string(),
                    error: XFileParseError::NotEnoughDataToReadHeader(source.len()),
                });
            }
        };
        if &header[..4] != b"xof " {
            return Err(XFileImportError::XFileParseError {
                position: 0.to_string(),
                error: XFileParseError::UnsupportedFileFormat(header[..4].try_into().unwrap()),
            });
        }

        let major_version = (header[4] - b'0') * 10 + (header[5] - b'0');
        let minor_version = (header[6] - b'0') * 10 + (header[7] - b'0');

        let file_format_signature: &[u8; 4] = &header[8..12].try_into().unwrap();
        let is_compressed;
        let is_binary_format;
        if file_format_signature == b"txt " {
            is_binary_format = false;
            is_compressed = false
        } else if file_format_signature == b"bin " {
            is_binary_format = true;
            is_compressed = false
        } else if file_format_signature == b"tzip" {
            is_binary_format = false;
            is_compressed = true
        } else if file_format_signature == b"bzip" {
            is_binary_format = true;
            is_compressed = true
        } else {
            return Err(XFileImportError::XFileParseError {
                position: 8.to_string(),
                error: XFileParseError::UnsupportedFileFormat(*file_format_signature),
            });
        };

        let binary_format_size = parse_4digits_decimal(u32::from_le_bytes([
            header[12], header[13], header[14], header[15],
        ]));
        if binary_format_size != 32 && binary_format_size != 64 {
            return Err(XFileImportError::XFileParseError {
                position: 12.to_string(),
                error: XFileParseError::UnsupportedFloatSize(binary_format_size),
            });
        }
        let binary_float_size = (binary_format_size / u8::BITS) as u8;
        Ok((
            XFileHeader {
                major_version,
                minor_version,
                is_compressed,
                is_binary_format,
                binary_float_size,
            },
            rest,
        ))
    }

    fn parse_by_format<'source>(
        source: &'source [u8],
        is_binary_format: bool,
        binary_float_size: u8,
    ) -> Result<Scene, XFileImportError> {
        if is_binary_format {
            let mut parser = ParserImpl::new(
                BinaryParser::new(source, binary_float_size),
                is_binary_format,
            );
            if let Err(e) = parser.parse_file() {
                Err(XFileImportError::XFileParseError {
                    position: parser.get_position(),
                    error: e,
                })
            } else {
                Ok(parser.scene)
            }
        } else {
            let mut parser = ParserImpl::new(TextParser::new(source), is_binary_format);
            if let Err(e) = parser.parse_file() {
                Err(XFileImportError::XFileParseError {
                    position: parser.get_position(),
                    error: e,
                })
            } else {
                Ok(parser.scene)
            }
        }
    }

    fn parse_compressed_file<'source>(
        mut source: &'source [u8],
        is_binary_format: bool,
        binary_float_size: u8,
    ) -> Result<Scene, XFileImportError> {
        let start = source.as_ptr() as usize;
        let error_handler = |error: XFileParseError| XFileImportError::XFileParseError {
            position: format!("Offset {}", source.as_ptr() as usize - start),
            error,
        };
        #[cfg(feature = "compression")]
        {
            /* ///////////////////////////////////////////////////////////////////////
             * COMPRESSED X FILE FORMAT
             * ///////////////////////////////////////////////////////////////////////
             *    [xhead]
             *    2 major
             *    2 minor
             *    4 type    // bzip,tzip
             *    [mszip_master_head]
             *    4 unkn    // checksum?
             *    2 unkn    // flags? (seems to be constant)
             *    [mszip_head]
             *    2 ofs     // offset to next section
             *    2 magic   // 'CK'
             *    ... ofs bytes of data
             *    ... next mszip_head
             *
             *  http://www.kdedevelopers.org/node/3181 has been very helpful.
             * ///////////////////////////////////////////////////////////////////////
             */
            // skip unknown data (checksum, flags?)
            if let Some((_, rest)) = source.split_at_checked(6) {
                source = rest;
            } else {
                return Err(error_handler(XFileParseError::NotEnoughDataToReadHeader(6)));
            }

            // First find out how much storage we'll need. Count sections.
            let mut cloned_source = source;
            let mut est_out = 0;

            while let &[a, b, c, d, ..] = cloned_source {
                // read next offset
                let ofs = u16::from_le_bytes([a, b]) as usize;
                if ofs >= MSZIP_BLOCK {
                    return Err(error_handler(
                        XFileParseError::InvalidOffsetToNextMszipCompressedBlock(ofs),
                    ));
                }

                // check magic word
                let magic = u16::from_le_bytes([c, d]);
                if magic != MSZIP_MAGIC {
                    return Err(error_handler(XFileParseError::UnsupportedCompressedFormat(
                        [c, d],
                    )));
                }

                // and advance to the next offset
                if let Some(s) = cloned_source.get(ofs..) {
                    cloned_source = s;
                } else {
                    return Err(error_handler(XFileParseError::TooSmallZipFile {
                        left: cloned_source.len(),
                        offset: ofs,
                    }));
                }
                est_out += MSZIP_BLOCK; // one decompressed block is 32786 in size
            }
            let mut decompressed_source: Vec<u8> = vec![0u8; est_out + 1];
            let mut compression = Compression::new();
            compression
                .open(
                    if is_binary_format {
                        Format::Binary
                    } else {
                        Format::Text
                    },
                    InflateFlush::SyncFlush,
                    -MAX_WBITS,
                )
                .map_err(|e| error_handler(XFileParseError::DecompressionError(e)))?;
            let mut out = decompressed_source.as_mut_slice();
            while let &[a, b, _c, _d, ref rest @ ..] = source {
                let ofs = u16::from_le_bytes([a, b]) as usize;
                source = rest;
                if source.len() + 2 < ofs as usize {
                    return Err(XFileImportError::FileTooSmall);
                }

                let size = compression
                    .decompress_block(source, &mut out[..MSZIP_BLOCK])
                    .map_err(|e| error_handler(XFileParseError::DecompressionError(e)))?;
                // SAFETY: size is guaranteed to be less than MSZIP_BLOCK
                out = unsafe { out.get_unchecked_mut(size..) };
                if let Some(s) = source.get(ofs..) {
                    source = s;
                } else {
                    break;
                }
            }
            compression
                .close()
                .map_err(|e| error_handler(XFileParseError::DecompressionError(e)))?;
            drop(compression);

            Self::parse_by_format(&decompressed_source, is_binary_format, binary_float_size)
        }
        #[cfg(not(feature = "compression"))]
        {
            Err(XFileImportError::XFileParseError {
                position: format!("Offset {}", start),
                error: XFileParseError::CompressionFeatureNotEnabled,
            })
        }
    }
}

pub(super) trait XFileParser<'source> {
    fn get_position(&self) -> String;

    fn rest(&self) -> usize {
        0
    }

    fn forward(&mut self, _n: usize) -> Result<&'source [u8], XFileParseError> {
        Ok(&[])
    }

    unsafe fn forward_unchecked(&mut self, _n: usize) -> &'source [u8] {
        &[]
    }

    fn peek<const N: usize>(&self) -> Option<&'source [u8; N]> {
        None
    }

    fn peek_one(&self) -> Option<u8> {
        None
    }

    fn next_byte_if_eq(&mut self, test_byte: u8) {
        if self.peek_one() == Some(test_byte) {
            // SAFETY: we know that the next byte is the test byte
            unsafe { self.forward_unchecked(1) };
        }
    }

    fn skip_until_next_line(&mut self) {}

    fn skip_whitespace(&mut self) {}

    fn read_int(&mut self) -> Result<u32, XFileParseError>;

    fn read_float(&mut self) -> Result<AiReal, XFileParseError>;

    fn read_vec2(&mut self) -> Result<Vec2, XFileParseError> {
        let x = self.read_float()?;
        let y = self.read_float()?;
        self.test_for_separator();
        Ok(Vec2::new(x, y))
    }

    fn read_vec3(&mut self) -> Result<Vec3, XFileParseError> {
        let x = self.read_float()?;
        let y = self.read_float()?;
        let z = self.read_float()?;
        self.test_for_separator();
        Ok(Vec3::new(x, y, z))
    }

    fn read_rgb(&mut self) -> Result<Color3D, XFileParseError> {
        let r = self.read_float()?;
        let g = self.read_float()?;
        let b = self.read_float()?;
        self.test_for_separator();
        Ok(Color3D::new(r as f32, g as f32, b as f32))
    }

    fn read_rgba(&mut self) -> Result<Color4D, XFileParseError> {
        let r = self.read_float()?;
        let g = self.read_float()?;
        let b = self.read_float()?;
        let a = self.read_float()?;
        self.test_for_separator();
        Ok(Color4D::new(r as f32, g as f32, b as f32, a as f32))
    }

    fn next_token(&mut self) -> Result<&'source [u8], XFileParseError>;

    fn next_token_as_str(&mut self) -> Result<Cow<'source, str>, XFileParseError>;

    fn check_for_separator(&mut self) -> Result<(), XFileParseError> {
        Ok(())
    }

    fn check_for_semicolon(&mut self) -> Result<(), XFileParseError> {
        Ok(())
    }

    fn check_for_closing_brace(&mut self) -> Result<(), XFileParseError> {
        let next = self.next_token()?;
        if next != b"}" {
            return Err(XFileParseError::ClosingBraceExpected(
                String::from_utf8_lossy(next).into_owned(),
            ));
        }
        Ok(())
    }

    fn test_for_separator(&mut self) {}
}

struct ParserImpl<'source, P: XFileParser<'source>> {
    inner_parser: P,
    is_binary_format: bool,
    line_number: u32,
    scene: Scene,
    _marker: PhantomData<&'source [u8]>,
}

impl<'source, P: XFileParser<'source>> XFileParser<'source> for ParserImpl<'source, P> {
    fn get_position(&self) -> String {
        self.inner_parser.get_position()
    }

    fn peek<const N: usize>(&self) -> Option<&'source [u8; N]> {
        self.inner_parser.peek::<N>()
    }

    fn read_int(&mut self) -> Result<u32, XFileParseError> {
        self.inner_parser.read_int()
    }

    fn read_float(&mut self) -> Result<AiReal, XFileParseError> {
        self.inner_parser.read_float()
    }

    fn next_token(&mut self) -> Result<&'source [u8], XFileParseError> {
        self.inner_parser.next_token()
    }

    fn next_token_as_str(&mut self) -> Result<Cow<'source, str>, XFileParseError> {
        self.inner_parser.next_token_as_str()
    }

    fn check_for_separator(&mut self) -> Result<(), XFileParseError> {
        self.inner_parser.check_for_separator()
    }

    fn check_for_semicolon(&mut self) -> Result<(), XFileParseError> {
        self.inner_parser.check_for_semicolon()
    }

    fn test_for_separator(&mut self) {
        self.inner_parser.test_for_separator()
    }
}
impl<'source, P: XFileParser<'source>> ParserImpl<'source, P> {
    /// Source should be bytes of valid UTF-8 text.
    #[inline]
    pub fn new(inner_parser: P, is_binary_format: bool) -> Self {
        Self {
            inner_parser,
            is_binary_format,
            line_number: 0,
            scene: Scene::default(),
            _marker: PhantomData,
        }
    }

    fn parse_file(&mut self) -> Result<(), XFileParseError> {
        loop {
            let token = self.next_token()?;
            if token.is_empty() {
                break;
            }
            // parse specific object
            if token == b"template" {
                self.parse_data_object_template()?;
            } else if token == b"Frame" {
                self.parse_data_object_frame(None)?;
            } else if token == b"Mesh" {
                // some meshes have no frames at all
                let mut mesh = Mesh::default();
                self.parse_data_object_mesh(&mut mesh)?;
                self.scene.global_meshes.push(mesh);
            } else if token == b"AnimTicksPerSecond" {
                self.parse_data_object_anim_ticks_per_second()?;
            } else if token == b"AnimationSet" {
                self.parse_data_object_animation_set()?;
            } else if token == b"Material" {
                // Material outside of a mesh or node
                let material = self.parse_data_object_material()?;
                self.scene.global_materials.push(material);
            } else if token == b"}" {
                // whatever?
            } else {
                self.parse_unknown_data_object()?;
            }
        }
        Ok(())
    }

    fn parse_data_object_frame(
        &mut self,
        parent: Option<Index<Node>>,
    ) -> Result<(), XFileParseError> {
        let name = if let Ok(s) = self.read_head_of_data_object() {
            if let Ok(s) = str::from_utf8(s) { s } else { "" }
        } else {
            ""
        };
        let parent = parent.unwrap_or(Index::new(0));
        let mut node = Node::new(parent);
        node.name = name.to_owned();

        let node_index = self.scene.push_node(parent, node);
        loop {
            let token = self.next_token()?;
            if token.is_empty() {
                return Err(XFileParseError::unexpected_end_of_file(
                    "parse_data_object_frame",
                ));
            }
            if token == b"}" {
                break; // frame finished
            } else if token == b"Frame" {
                self.parse_data_object_frame(Some(node_index))?; // child frame
            } else if token == b"FrameTransformMatrix" {
                let matrix = self.parse_data_object_transformation_matrix()?;
                // SAFETY: node_index is guaranteed to be valid
                let node = unsafe { node_index.get_mut_unchecked(&mut self.scene.nodes) };
                node.transformation_matrix = matrix;
            } else if token == b"Mesh" {
                let mut mesh = Mesh::new(name.to_owned());
                self.parse_data_object_mesh(&mut mesh)?;
                // SAFETY: node_index is guaranteed to be valid
                let node = unsafe { node_index.get_mut_unchecked(&mut self.scene.nodes) };
                node.meshes.push(mesh);
            } else {
                self.parse_unknown_data_object()?;
            }
        }
        Ok(())
    }

    fn read_head_of_data_object(&mut self) -> Result<&'source [u8], XFileParseError> {
        let name_or_brace = self.next_token()?;
        if name_or_brace != b"{" {
            let next = self.next_token()?;
            if next != b"{" {
                return Err(XFileParseError::unexpected_token("{", name_or_brace));
            } else {
                return Ok(name_or_brace);
            }
        }
        Ok(&[])
    }

    fn parse_data_object_template(&mut self) -> Result<(), XFileParseError> {
        let _name = self.read_head_of_data_object()?;
        let _guid = self.next_token()?;

        loop {
            let token = self.next_token()?;
            if token.is_empty() {
                return Err(XFileParseError::unexpected_end_of_file(
                    "parse_data_object_template",
                ));
            }

            if token == b"}" {
                return Ok(());
            }
        }
    }

    fn parse_data_object_transformation_matrix(&mut self) -> Result<Mat4, XFileParseError> {
        self.read_head_of_data_object()?;
        let x1 = self.read_float()?;
        let y1 = self.read_float()?;
        let z1 = self.read_float()?;
        let w1 = self.read_float()?;
        let x2 = self.read_float()?;
        let y2 = self.read_float()?;
        let z2 = self.read_float()?;
        let w2 = self.read_float()?;
        let x3 = self.read_float()?;
        let y3 = self.read_float()?;
        let z3 = self.read_float()?;
        let w3 = self.read_float()?;
        let x4 = self.read_float()?;
        let y4 = self.read_float()?;
        let z4 = self.read_float()?;
        let w4 = self.read_float()?;
        let mat = Mat4::from_cols(
            Vec4::new(x1, x2, x3, x4),
            Vec4::new(y1, y2, y3, y4),
            Vec4::new(z1, z2, z3, z4),
            Vec4::new(w1, w2, w3, w4),
        );
        self.check_for_semicolon()?;
        self.check_for_closing_brace()?;
        Ok(mat)
    }

    fn parse_data_object_mesh(&mut self, m: &mut Mesh) -> Result<(), XFileParseError> {
        self.read_head_of_data_object()?;
        let num_of_vertices = self.read_int()?;
        m.positions = Vec::with_capacity(num_of_vertices as usize);
        for _ in 0..num_of_vertices {
            let v = self.read_vec3()?;
            m.positions.push(v);
        }
        let num_of_faces = self.read_int()?;
        m.pos_faces = vec![Face::default(); num_of_faces as usize];
        for face in m.pos_faces.iter_mut() {
            let num_indices = self.read_int()?;
            for _ in 0..num_indices {
                let idx = self.read_int()?;
                if idx <= num_of_vertices {
                    face.indices.push(idx);
                }
            }
            self.test_for_separator();
        }
        loop {
            let token = self.next_token()?;
            if token.is_empty() {
                return Err(XFileParseError::unexpected_end_of_file(
                    "parse_data_object_mesh",
                ));
            }
            if token == b"}" {
                return Ok(());
            }
            if token == b"MeshNormals" {
                self.parse_data_object_mesh_normals(m)?;
            } else if token == b"MeshTextureCoords" {
                self.parse_data_object_mesh_texture_coords(m)?;
            } else if token == b"MeshVertexColors" {
                self.parse_data_object_mesh_vertex_colors(m)?;
            } else if token == b"MeshMaterialList" {
                self.parse_data_object_mesh_material_list(m)?;
            } else if token == b"VertexDuplicationIndices" {
                self.parse_unknown_data_object()?;
            } else if token == b"XSkinMeshHeader" {
                self.parse_data_object_skin_mesh_header()?;
            } else if token == b"SkinWeights" {
                self.parse_data_object_skin_weights(m)?;
            } else {
                self.parse_unknown_data_object()?;
            }
        }
    }

    fn parse_data_object_mesh_normals(&mut self, m: &mut Mesh) -> Result<(), XFileParseError> {
        self.read_head_of_data_object()?;

        // read count
        let num_of_normals = self.read_int()?;
        if num_of_normals == 0 {
            return Ok(());
        }

        m.normals.resize(num_of_normals as usize, Vec3::ZERO);

        // read normal vectors
        for normal in m.normals.iter_mut() {
            *normal = self.read_vec3()?;
        }

        // read normal indices
        let num_of_indices = self.read_int()?;
        if num_of_indices != m.pos_faces.len() as u32 {
            return Err(XFileParseError::NormalFaceCountDoesNotMatchVertexFaceCount);
        }

        if num_of_indices > 0 {
            m.norm_faces
                .resize(num_of_indices as usize, Face::default());
            for face in m.norm_faces.iter_mut() {
                let num_indices = self.read_int()?;
                *face = Face::default();
                face.indices
                    .try_reserve(num_indices as usize)
                    .map_err(|_| XFileParseError::InsufficientMemory)?;
                for _ in 0..num_indices {
                    let idx = self.read_int()?;
                    // if idx <= num_indices {
                    face.indices.push(idx);
                    // }
                }
                self.test_for_separator();
            }
        }
        self.check_for_closing_brace()?;
        Ok(())
    }

    fn parse_data_object_mesh_texture_coords(
        &mut self,
        m: &mut Mesh,
    ) -> Result<(), XFileParseError> {
        self.read_head_of_data_object()?;
        if m.num_textures + 1 > AI_MAX_NUMBER_OF_TEXTURECOORDS as u32 {
            return Err(XFileParseError::TooManySetsOfTextureCoordinates);
        }

        let tex_coords = &mut m.tex_coords[m.num_textures as usize];
        m.num_textures += 1;
        let num_coords = self.read_int()?;
        if num_coords != m.positions.len() as u32 {
            return Err(XFileParseError::TextureCoordCountDoesNotMatchVertexCount);
        }

        tex_coords.resize(num_coords as usize, Vec2::ZERO);
        for coord in tex_coords.iter_mut() {
            *coord = self.read_vec2()?;
        }
        self.check_for_closing_brace()?;
        Ok(())
    }

    fn parse_data_object_mesh_vertex_colors(
        &mut self,
        m: &mut Mesh,
    ) -> Result<(), XFileParseError> {
        self.read_head_of_data_object()?;
        let Some(colors) = m.colors.get_mut(m.num_color_sets as usize) else {
            return Err(XFileParseError::TooManyColorSets);
        };
        m.num_color_sets += 1;
        let num_colors = self.read_int()? as usize;
        if num_colors != m.positions.len() {
            return Err(XFileParseError::VertexColorCountDoesNotMatchVertexCount);
        }

        *colors = vec![Color4D::default(); num_colors];
        for _ in 0..num_colors {
            let index = self.read_int()? as usize;

            match colors.get_mut(index) {
                Some(color) => *color = self.read_rgba()?,
                None => return Err(XFileParseError::VertexColorIndexOutOfBounds),
            }
            // HACK: (thom) Maxon Cinema XPort plugin puts a third separator here, kwxPort puts a comma.
            // Ignore gracefully.
            self.test_for_separator();
        }

        self.check_for_closing_brace()?;
        Ok(())
    }

    fn parse_data_object_mesh_material_list(
        &mut self,
        m: &mut Mesh,
    ) -> Result<(), XFileParseError> {
        self.read_head_of_data_object()?;
        // read material count
        let _num_materials = self.read_int()?;
        // read non triangulated face material index count
        let num_mat_indices = self.read_int()? as usize;

        // some models have a material index count of 1... to be able to read them we
        // replicate this single material index on every face
        if num_mat_indices != m.pos_faces.len() && num_mat_indices != 1 {
            return Err(XFileParseError::PerFaceMaterialIndexCountDoesNotMatchFaceCount);
        }

        // read per-face material indices
        for _ in 0..num_mat_indices {
            m.face_materials.push(self.read_int()?);
        }

        // in version 03.02, the face indices end with two semicolons.
        // commented out version check, as version 03.03 exported from blender also has 2 semicolons
        if !self.is_binary_format {
            self.next_byte_if_eq(b';');
        }

        // if there was only a single material index, replicate it on all faces
        if m.face_materials.len() < m.pos_faces.len() {
            m.face_materials.extend(
                core::iter::repeat(m.face_materials.get(0).copied().unwrap_or_default())
                    .take(m.pos_faces.len() - m.face_materials.len()),
            );
        }

        // read following data objects
        loop {
            let token = self.next_token()?;
            if token.is_empty() {
                return Err(XFileParseError::unexpected_end_of_file(
                    "parse_data_object_mesh_material_list",
                ));
            }
            if token == b"}" {
                break; // material list finished
            } else if token == b"{" {
                // template materials
                let mat_name = self.next_token()?;
                let mut material = Material::default();
                material.is_reference = true;

                material.name = String::from_utf8_lossy(mat_name).into_owned();
                m.materials.push(material);

                self.check_for_closing_brace()?; // skip }
            } else if token == b"Material" {
                m.materials.push(self.parse_data_object_material()?);
            } else if token == b";" {
                // ignore
            } else {
                self.parse_unknown_data_object()?;
            }
        }
        Ok(())
    }

    fn parse_data_object_material(&mut self) -> Result<Material, XFileParseError> {
        let mat_name = self.read_head_of_data_object()?;
        let name = if mat_name.is_empty() {
            format!("material{}", self.line_number)
        } else {
            String::from_utf8_lossy(mat_name).into_owned()
        };
        let is_reference = false;
        let diffuse = self.read_rgba()?;
        let specular_exponent = self.read_float()?;
        let specular = self.read_rgb()?;
        let emissive = self.read_rgb()?;
        let mut textures = Vec::new();
        // read other data objects
        loop {
            let token = self.next_token()?;
            if token.is_empty() {
                return Err(XFileParseError::unexpected_end_of_file(
                    "parse_data_object_material",
                ));
            }
            if token == b"}" {
                break; // material finished
            }

            if token == b"TextureFilename" || token == b"TextureFileName" {
                // some exporters write "TextureFileName" instead.
                let tex_name = self.parse_data_object_material_texture_filename()?;
                textures.push(TexEntry::new(tex_name, false));
            } else if token == b"NormalmapFilename" || token == b"NormalmapFileName" {
                // one exporter writes out the normal map in a separate filename tag
                let tex_name = self.parse_data_object_material_texture_filename()?;
                textures.push(TexEntry::new(tex_name, true));
            } else {
                self.parse_unknown_data_object()?;
            }
        }
        Ok(Material {
            name,
            is_reference,
            diffuse,
            specular_exponent,
            specular,
            emissive,
            textures,
            scene_index: 0,
        })
    }

    fn parse_data_object_material_texture_filename(&mut self) -> Result<String, XFileParseError> {
        self.read_head_of_data_object()?;
        let name = self.next_token_as_str()?.replace("\\\\", "\\");
        self.check_for_closing_brace()?;
        Ok(name)
    }

    fn parse_data_object_skin_mesh_header(&mut self) -> Result<(), XFileParseError> {
        self.read_head_of_data_object()?;
        let _max_skin_weights_per_vertex = self.read_int()?;
        let _max_skin_weights_per_face = self.read_int()?;
        let _num_bones_in_mesh = self.read_int()?;
        self.check_for_closing_brace()?;
        Ok(())
    }

    fn parse_data_object_skin_weights(&mut self, m: &mut Mesh) -> Result<(), XFileParseError> {
        self.read_head_of_data_object()?;

        let transform_node_name = self.next_token_as_str()?;
        let mut bone = Bone::new(transform_node_name.into_owned());

        // read vertex weights
        let num_weights = self.read_int()?;
        bone.weights.reserve(num_weights as usize);

        for _ in 0..num_weights {
            let mut weight = BoneWeight::default();
            weight.vertex = self.read_int()?;
            bone.weights.push(weight);
        }

        // read vertex weights
        for weight in bone.weights.iter_mut() {
            weight.weight = self.read_float()?;
        }

        // read matrix offset
        bone.offset_matrix.x_axis.x = self.read_float()?;
        bone.offset_matrix.y_axis.x = self.read_float()?;
        bone.offset_matrix.z_axis.x = self.read_float()?;
        bone.offset_matrix.w_axis.x = self.read_float()?;
        bone.offset_matrix.x_axis.y = self.read_float()?;
        bone.offset_matrix.y_axis.y = self.read_float()?;
        bone.offset_matrix.z_axis.y = self.read_float()?;
        bone.offset_matrix.w_axis.y = self.read_float()?;
        bone.offset_matrix.x_axis.z = self.read_float()?;
        bone.offset_matrix.y_axis.z = self.read_float()?;
        bone.offset_matrix.z_axis.z = self.read_float()?;
        bone.offset_matrix.w_axis.z = self.read_float()?;
        bone.offset_matrix.x_axis.w = self.read_float()?;
        bone.offset_matrix.y_axis.w = self.read_float()?;
        bone.offset_matrix.z_axis.w = self.read_float()?;
        bone.offset_matrix.w_axis.w = self.read_float()?;

        self.check_for_semicolon()?;
        self.check_for_closing_brace()?;

        m.bones.push(bone);
        Ok(())
    }

    fn parse_data_object_animation_set(&mut self) -> Result<(), XFileParseError> {
        let anim_name = self.read_head_of_data_object()?;
        let anim_name = String::from_utf8(anim_name.to_owned()).unwrap_or_default();
        let mut anim = Animation::new(anim_name);

        loop {
            let token = self.next_token()?;
            if token.is_empty() {
                return Err(XFileParseError::unexpected_end_of_file(
                    "parse_data_object_animation_set",
                ));
            }
            if token == b"}" {
                break; // animation set finished
            } else if token == b"Animation" {
                self.parse_data_object_animation(&mut anim)?;
            } else {
                self.parse_unknown_data_object()?;
            }
        }
        self.scene.animations.push(anim);
        Ok(())
    }

    fn parse_data_object_animation(&mut self, anim: &mut Animation) -> Result<(), XFileParseError> {
        self.read_head_of_data_object()?;
        let mut banim = AnimBone::new();

        loop {
            let token = self.next_token()?;
            if token.is_empty() {
                return Err(XFileParseError::unexpected_end_of_file(
                    "parse_data_object_animation",
                ));
            }
            if token == b"}" {
                break; // animation finished
            }
            if token == b"AnimationKey" {
                self.parse_data_object_animation_key(&mut banim)?;
            } else if token == b"AnimationOptions" {
                self.parse_unknown_data_object()?; // not interested
            } else if token == b"{" {
                // read frame name
                let name = self.next_token()?;
                banim.name = String::from_utf8_lossy(name).into_owned();
                self.check_for_closing_brace()?;
            } else {
                self.parse_unknown_data_object()?;
            }
        }
        anim.anims.push(banim);
        Ok(())
    }

    fn parse_data_object_anim_ticks_per_second(&mut self) -> Result<(), XFileParseError> {
        self.read_head_of_data_object()?;
        self.scene.anim_ticks_per_second = self.read_int()?;
        self.check_for_closing_brace()?;
        Ok(())
    }

    fn parse_data_object_animation_key(
        &mut self,
        banim: &mut AnimBone,
    ) -> Result<(), XFileParseError> {
        self.read_head_of_data_object()?;

        // read key type
        let key_type = self.read_int()?;

        // read number of keys
        let num_keys = self.read_int()?;

        match key_type {
            0 => {
                banim
                    .rot_keys
                    .try_reserve(num_keys as usize)
                    .map_err(|_| XFileParseError::InsufficientMemory)?;
            }
            1 => {
                banim
                    .scale_keys
                    .try_reserve(num_keys as usize)
                    .map_err(|_| XFileParseError::InsufficientMemory)?;
            }
            2 => {
                banim
                    .pos_keys
                    .try_reserve(num_keys as usize)
                    .map_err(|_| XFileParseError::InsufficientMemory)?;
            }
            3 | 4 => {
                banim
                    .trafo_keys
                    .try_reserve(num_keys as usize)
                    .map_err(|_| XFileParseError::InsufficientMemory)?;
            }
            _ => {}
        }

        for _ in 0..num_keys {
            // read time
            let time = self.read_int()?;
            // read keys
            match key_type {
                // rotation quaternion
                0 => {
                    // read count
                    let count = self.read_int()? as usize;
                    if count != 4 {
                        return Err(XFileParseError::InvalidNumberOfArgumentsForKeyInAnimation {
                            key_type: "quaternion",
                            expected: 4,
                            found: count,
                        });
                    }
                    let w = self.read_float()?;
                    let x = self.read_float()?;
                    let y = self.read_float()?;
                    let z = self.read_float()?;
                    let key = AiQuatKey {
                        time: time as f64,
                        value: Quat::from_xyzw(x, y, z, w),
                        interpolation: AiAnimInterpolation::default(),
                    };

                    self.check_for_semicolon()?;

                    banim.rot_keys.push(key);
                }
                // scale vector | position vector
                1 | 2 => {
                    // read count
                    let count = self.read_int()? as usize;
                    if count != 3 {
                        return Err(XFileParseError::InvalidNumberOfArgumentsForKeyInAnimation {
                            key_type: "vector",
                            expected: 3,
                            found: count,
                        });
                    }

                    let key = AiVectorKey {
                        time: time as f64,
                        value: self.read_vec3()?,
                        interpolation: AiAnimInterpolation::default(),
                    };

                    if key_type == 2 {
                        banim.pos_keys.push(key);
                    } else {
                        banim.scale_keys.push(key);
                    }
                }

                // combined transformation matrix | denoted both as 3 or as 4
                3 | 4 => {
                    // read count
                    let count = self.read_int()? as usize;
                    if count != 16 {
                        return Err(XFileParseError::InvalidNumberOfArgumentsForKeyInAnimation {
                            key_type: "matrix",
                            expected: 16,
                            found: count,
                        });
                    }

                    // read matrix
                    let x1 = self.read_float()?;
                    let y1 = self.read_float()?;
                    let z1 = self.read_float()?;
                    let w1 = self.read_float()?;
                    let x2 = self.read_float()?;
                    let y2 = self.read_float()?;
                    let z2 = self.read_float()?;
                    let w2 = self.read_float()?;
                    let x3 = self.read_float()?;
                    let y3 = self.read_float()?;
                    let z3 = self.read_float()?;
                    let w3 = self.read_float()?;
                    let x4 = self.read_float()?;
                    let y4 = self.read_float()?;
                    let z4 = self.read_float()?;
                    let w4 = self.read_float()?;
                    let key = MatrixKey {
                        time: time as f64,
                        matrix: Mat4::from_cols(
                            Vec4::new(x1, x2, x3, x4),
                            Vec4::new(y1, y2, y3, y4),
                            Vec4::new(z1, z2, z3, z4),
                            Vec4::new(w1, w2, w3, w4),
                        ),
                    };
                    self.check_for_semicolon()?;

                    banim.trafo_keys.push(key);
                }

                _ => {
                    return Err(XFileParseError::UnknownKeyTypeInAnimation(key_type));
                }
            }
            // key separator
            self.check_for_separator()?;
        }
        self.check_for_closing_brace()?;
        Ok(())
    }

    fn parse_unknown_data_object(&mut self) -> Result<(), XFileParseError> {
        // find opening delimiter
        loop {
            let token = self.next_token()?;
            if token.is_empty() {
                return Err(XFileParseError::unexpected_end_of_file(
                    "parse_data_object_animation_key",
                ));
            }
            if token == b"{" {
                break;
            }
        }

        let mut brace_left_match_cnt = 1;

        // parse until closing delimiter
        while brace_left_match_cnt > 0 {
            let token = self.next_token()?;
            if token.is_empty() {
                return Err(XFileParseError::unexpected_end_of_file(
                    "parse_unknown_data_object",
                ));
            }

            if token == b"{" {
                brace_left_match_cnt += 1;
            } else if token == b"}" {
                brace_left_match_cnt -= 1;
            }
        }
        Ok(())
    }
}
