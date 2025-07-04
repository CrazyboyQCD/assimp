use crate::utils::float_precision::{Mat4, Quat, Vec2, Vec3, Vec4};
use zlib_rs::{InflateFlush, MAX_WBITS};

use crate::{
    AiReal,
    formats::x::errors::XFileImportError,
    structs::{
        anim::AiAnimInterpolation,
        color::{Color3D, Color4D},
        key::{AiQuatKey, AiVectorKey},
        mesh::AI_MAX_NUMBER_OF_COLOR_SETS,
        nodes::Index,
    },
    utils::{
        compression::{Compression, Format},
        fast_atof::fast_atoreal_move,
    },
};

use super::structs::{
    AnimBone, Animation, Bone, BoneWeight, Face, Material, MatrixKey, Mesh, Node, Scene, TexEntry,
};

pub struct Parser<'a> {
    source: &'a [u8],
    major_version: u8,
    minor_version: u8,
    is_binary_format: bool,
    binary_float_size: u8,
    binary_num_count: u32,
    line_number: u32,
    scene: Scene,
}

const AI_MAX_NUMBER_OF_TEXTURECOORDS: usize = 0x8;

const MSZIP_BLOCK: usize = 32786;
const MSZIP_MAGIC: u16 = 0x4b43;

impl<'a> Parser<'a> {
    /// Source should be bytes of valid UTF-8 text.
    #[inline]
    pub fn new(source: &'a [u8]) -> Result<Self, XFileImportError> {
        // if !str::from_utf8(source).is_ok() {
        //     return Err(XFileImportError::from(EncodingError::NotValidUtf8));
        // }
        // SAFETY: source is guaranteed to be a valid UTF-8 string.
        Ok(unsafe { Self::new_unchecked(source) })
    }

    /// # Safety
    ///
    /// This function is unsafe because it does not check if the source is a valid UTF-8 string.
    /// Callers must ensure that the source is a valid UTF-8 string.
    #[inline]
    pub unsafe fn new_unchecked(source: &'a [u8]) -> Self {
        Self {
            source,
            major_version: 0,
            minor_version: 0,
            is_binary_format: false,
            binary_float_size: 0,
            binary_num_count: 0,
            line_number: 0,
            scene: Scene::default(),
        }
    }

    fn new_from_decompressed_source<'b: 'a>(&self, uncompressed_source: &'b [u8]) -> Self {
        Self {
            source: uncompressed_source,
            major_version: self.major_version,
            minor_version: self.minor_version,
            is_binary_format: self.is_binary_format,
            binary_float_size: self.binary_float_size,
            binary_num_count: self.binary_num_count,
            line_number: self.line_number,
            scene: Scene::default(),
        }
    }

    #[inline(always)]
    fn rest(&self) -> usize {
        self.source.len()
    }

    pub fn get_scene(self) -> Scene {
        self.scene
    }

    pub fn parse(&mut self) -> Result<(), XFileImportError> {
        let header: &[u8; 16] = self
            .forward(16)
            .map_err(|_| XFileImportError::FileTooSmall)?
            .try_into()
            .unwrap();
        if &header[..4] != b"xof " {
            return Err(XFileImportError::InvalidHeader(
                header[..4].try_into().unwrap(),
            ));
        }

        self.major_version = (header[4] - 48) * 10 + (header[5] - 48);
        self.minor_version = (header[6] - 48) * 10 + (header[7] - 48);

        let mut is_compressed = false;
        let file_format_signature: &[u8; 4] = &header[8..12].try_into().unwrap();
        if file_format_signature == b"txt " {
            self.is_binary_format = false;
        } else if file_format_signature == b"bin " {
            self.is_binary_format = true;
        } else if file_format_signature == b"tzip" {
            self.is_binary_format = false;
            is_compressed = true;
        } else if file_format_signature == b"bzip" {
            self.is_binary_format = true;
            is_compressed = true;
        } else {
            return Err(XFileImportError::InvalidFormatSignature(
                *file_format_signature,
            ));
        }

        let binary_format_size = ((header[12] - b'0') as u32) * 1000
            + ((header[13] - b'0') as u32) * 100
            + ((header[14] - b'0') as u32) * 10
            + ((header[15] - b'0') as u32);
        if binary_format_size != 32 && binary_format_size != 64 {
            return Err(XFileImportError::InvalidBinaryFormatSize(
                binary_format_size,
            ));
        }
        self.binary_float_size = (binary_format_size / 8) as u8;
        if is_compressed {
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
            self.forward(6)?;

            // First find out how much storage we'll need. Count sections.
            let mut cloned_source = self.source;
            let mut est_out = 0;

            while let &[a, b, c, d, ..] = cloned_source {
                // read next offset
                let ofs = u16::from_le_bytes([a, b]) as usize;
                if ofs >= MSZIP_BLOCK {
                    return Err(XFileImportError::InvalidOffsetToNextMszipCompressedBlock);
                }

                // check magic word
                let magic = u16::from_le_bytes([c, d]);
                if magic != MSZIP_MAGIC {
                    return Err(XFileImportError::UnsupportedCompressedFormat(magic));
                }

                // and advance to the next offset
                cloned_source = match cloned_source.get(ofs..) {
                    Some(s) => s,
                    None => return Err(XFileImportError::InvalidCompressedFormat),
                };
                est_out += MSZIP_BLOCK; // one decompressed block is 32786 in size
            }
            let mut decompressed_source: Vec<u8> = vec![0u8; est_out + 1];
            let mut compression = Compression::new();
            compression
                .open(
                    if self.is_binary_format {
                        Format::Binary
                    } else {
                        Format::Text
                    },
                    InflateFlush::SyncFlush,
                    -MAX_WBITS,
                )
                .map_err(XFileImportError::from)?;
            let mut out = decompressed_source.as_mut_slice();
            while let &[a, b, _c, _d, ref rest @ ..] = self.source {
                let ofs = u16::from_le_bytes([a, b]) as usize;
                self.source = rest;
                if self.rest() + 2 < ofs as usize {
                    return Err(XFileImportError::FileTooSmall);
                }

                let size = compression
                    .decompress_block(self.source, &mut out[..MSZIP_BLOCK])
                    .map_err(XFileImportError::from)?;
                // SAFETY: size is guaranteed to be less than MSZIP_BLOCK
                out = unsafe { out.get_unchecked_mut(size..) };
                self.source = match self.source.get(ofs..) {
                    Some(s) => s,
                    None => break,
                };
            }
            compression.close().map_err(XFileImportError::from)?;

            let mut new_parser = self.new_from_decompressed_source(&decompressed_source);
            new_parser.parse_file()?;
            self.scene = new_parser.scene;
        } else {
            self.parse_file()?;
        }
        Ok(())
    }

    fn read_head_of_data_object(&mut self) -> Result<&'a [u8], XFileImportError> {
        let name_or_brace = self.next_token()?;
        if name_or_brace != b"{" {
            let next = self.next_token()?;
            if next != b"{" {
                return Err(XFileImportError::unexpected_token(
                    "{",
                    String::from_utf8_lossy(name_or_brace).into_owned(),
                ));
            } else {
                return Ok(name_or_brace);
            }
        }
        Ok(name_or_brace)
    }

    fn parse_data_object_template(&mut self) -> Result<(), XFileImportError> {
        let _name = self.read_head_of_data_object()?;
        let _guid = self.next_token()?;
        loop {
            let token = self.next_token()?;
            if token.is_empty() {
                return Err(XFileImportError::unexpected_end_of_file(
                    "parse_data_object_template",
                ));
            }
            if token == b"}" {
                return Ok(());
            }
        }
    }

    fn parse_data_object_transformation_matrix(&mut self) -> Result<Mat4, XFileImportError> {
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

    fn parse_data_object_mesh_normals(&mut self, m: &mut Mesh) -> Result<(), XFileImportError> {
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
            return Err(XFileImportError::NormalFaceCountDoesNotMatchVertexFaceCount);
        }

        if num_of_indices > 0 {
            m.norm_faces
                .resize(num_of_indices as usize, Face::default());
            for face in m.norm_faces.iter_mut() {
                let num_indices = self.read_int()?;
                *face = Face::default();
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
    ) -> Result<(), XFileImportError> {
        self.read_head_of_data_object()?;
        if m.num_textures + 1 > AI_MAX_NUMBER_OF_TEXTURECOORDS as u32 {
            return Err(XFileImportError::TooManySetsOfTextureCoordinates);
        }

        let tex_coords = &mut m.tex_coords[m.num_textures as usize];
        m.num_textures += 1;
        let num_coords = self.read_int()?;
        if num_coords != m.positions.len() as u32 {
            return Err(XFileImportError::TextureCoordCountDoesNotMatchVertexCount);
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
    ) -> Result<(), XFileImportError> {
        self.read_head_of_data_object()?;
        if m.num_color_sets + 1 > AI_MAX_NUMBER_OF_COLOR_SETS as u32 {
            return Err(XFileImportError::TooManyColorSets);
        }
        let colors = &mut m.colors[m.num_color_sets as usize];
        m.num_color_sets += 1;
        let num_colors = self.read_int()?;
        if num_colors != m.positions.len() as u32 {
            return Err(XFileImportError::VertexColorCountDoesNotMatchVertexCount);
        }

        colors.resize(num_colors as usize, Color4D::default());
        let len = colors.len();
        for _ in 0..num_colors {
            let index = self.read_int()? as usize;
            if index >= len {
                return Err(XFileImportError::VertexColorIndexOutOfBounds);
            }

            colors[index] = self.read_rgba()?;
            // HACK: (thom) Maxon Cinema XPort plugin puts a third separator here, kwxPort puts a comma.
            // Ignore gracefully.
            if !self.is_binary_format {
                self.skip_whitespace();
                let Some(b) = self.peek_one() else {
                    return Err(XFileImportError::NotEnoughDataToRead(1));
                };
                if b == b';' || b == b',' {
                    self.forward(1)?;
                }
            }
        }

        self.check_for_closing_brace()?;
        Ok(())
    }

    fn parse_data_object_mesh_material_list(
        &mut self,
        m: &mut Mesh,
    ) -> Result<(), XFileImportError> {
        self.read_head_of_data_object()?;
        // read material count
        let _num_materials = self.read_int()?;
        // read non triangulated face material index count
        let num_mat_indices = self.read_int()?;

        // some models have a material index count of 1... to be able to read them we
        // replicate this single material index on every face
        if num_mat_indices != m.pos_faces.len() as u32 && num_mat_indices != 1 {
            return Err(XFileImportError::PerFaceMaterialIndexCountDoesNotMatchFaceCount);
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
                return Err(XFileImportError::unexpected_end_of_file(
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

                // SAFETY: source is guaranteed to be a valid UTF-8 string.
                material.name = unsafe { String::from_utf8_unchecked(mat_name.to_vec()) };
                m.materials.push(material);

                self.check_for_closing_brace()?; // skip }
            } else if token == b"Material" {
                let mut material = Material::default();
                self.parse_data_object_material(&mut material)?;
                m.materials.push(material);
            } else if token == b";" {
                // ignore
            } else {
                self.parse_unknown_data_object()?;
            }
        }
        Ok(())
    }

    fn parse_data_object_material(&mut self, m: &mut Material) -> Result<(), XFileImportError> {
        let mat_name = self.read_head_of_data_object()?;
        if mat_name.is_empty() {
            m.name = "material".to_string() + &self.line_number.to_string();
        } else {
            // SAFETY: source is guaranteed to be a valid UTF-8 string.
            m.name = unsafe { String::from_utf8_unchecked(mat_name.to_vec()) };
        }
        m.is_reference = false;
        m.diffuse = self.read_rgba()?;
        m.specular_exponent = self.read_float()?;
        m.specular = self.read_rgb()?;
        m.emissive = self.read_rgb()?;
        // read other data objects
        loop {
            let token = self.next_token()?;
            if token.is_empty() {
                return Err(XFileImportError::unexpected_end_of_file(
                    "parse_data_object_material",
                ));
            }
            if token == b"}" {
                break; // material finished
            }

            if token == b"TextureFilename" || token == b"TextureFileName" {
                // some exporters write "TextureFileName" instead.
                let tex_name = self.parse_data_object_material_texture_filename()?;
                m.textures.push(TexEntry::new(tex_name, false));
            } else if token == b"NormalmapFilename" || token == b"NormalmapFileName" {
                // one exporter writes out the normal map in a separate filename tag
                let tex_name = self.parse_data_object_material_texture_filename()?;
                m.textures.push(TexEntry::new(tex_name, true));
            } else {
                self.parse_unknown_data_object()?;
            }
        }
        Ok(())
    }

    fn parse_data_object_material_texture_filename(&mut self) -> Result<String, XFileImportError> {
        self.read_head_of_data_object()?;
        let name = self.next_token_as_str()?.replace("\\\\", "\\");
        self.check_for_closing_brace()?;
        Ok(name)
    }

    fn parse_data_object_skin_mesh_header(&mut self) -> Result<(), XFileImportError> {
        self.read_head_of_data_object()?;
        let _max_skin_weights_per_vertex = self.read_int()?;
        let _max_skin_weights_per_face = self.read_int()?;
        let _num_bones_in_mesh = self.read_int()?;
        self.check_for_closing_brace()?;
        Ok(())
    }

    fn parse_data_object_skin_weights(&mut self, m: &mut Mesh) -> Result<(), XFileImportError> {
        self.read_head_of_data_object()?;

        let transform_node_name = self.next_token_as_str()?;
        let mut bone = Bone::new(transform_node_name.to_owned());

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

    fn parse_data_object_mesh(&mut self, m: &mut Mesh) -> Result<(), XFileImportError> {
        self.read_head_of_data_object()?;
        let num_of_vertices = self.read_int()?;
        m.positions.resize(num_of_vertices as usize, Vec3::ZERO);
        for pos in m.positions.iter_mut() {
            *pos = self.read_vec3()?;
        }
        let num_of_faces = self.read_int()?;
        m.pos_faces.resize(num_of_faces as usize, Face::default());
        for face in m.pos_faces.iter_mut() {
            let num_indices = self.read_int()?;
            for _ in 0..num_indices {
                let idx = self.read_int()?;
                if idx <= num_indices {
                    face.indices.push(idx);
                }
            }
            self.test_for_separator();
        }
        loop {
            let token = self.next_token()?;
            if token.is_empty() {
                return Err(XFileImportError::unexpected_end_of_file(
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

    fn parse_data_object_frame(&mut self, parent: Index<Node>) -> Result<(), XFileImportError> {
        let name = self
            .read_head_of_data_object()
            .unwrap_or_default()
            .to_owned();
        let mut node = Node::new(parent);
        let name = String::from_utf8(name).unwrap_or_default();
        node.name = name.clone();

        let node_index = self.scene.push_node(parent, node);
        loop {
            let token = self.next_token()?;
            if token.is_empty() {
                return Err(XFileImportError::unexpected_end_of_file(
                    "parse_data_object_frame",
                ));
            }
            if token == b"}" {
                break; // frame finished
            } else if token == b"Frame" {
                self.parse_data_object_frame(node_index)?; // child frame
            } else if token == b"FrameTransformMatrix" {
                let matrix = self.parse_data_object_transformation_matrix()?;
                let node = node_index.get_mut(&mut self.scene.nodes).unwrap();
                node.transformation_matrix = matrix;
            } else if token == b"Mesh" {
                let mut mesh = Mesh::new(name.clone());
                self.parse_data_object_mesh(&mut mesh)?;
                let node = node_index.get_mut(&mut self.scene.nodes).unwrap();
                node.meshes.push(mesh);
            } else {
                self.parse_unknown_data_object()?;
            }
        }
        Ok(())
    }

    fn parse_data_object_anim_ticks_per_second(&mut self) -> Result<(), XFileImportError> {
        self.read_head_of_data_object()?;
        self.scene.anim_ticks_per_second = self.read_int()?;
        self.check_for_closing_brace()?;
        Ok(())
    }

    fn parse_data_object_animation_set(&mut self) -> Result<(), XFileImportError> {
        let anim_name = self.read_head_of_data_object()?;
        let anim_name = String::from_utf8(anim_name.to_owned()).unwrap_or_default();

        let mut anim = Animation::new(anim_name);

        loop {
            let token = self.next_token()?;
            if token.is_empty() {
                return Err(XFileImportError::unexpected_end_of_file(
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

    fn parse_data_object_animation(
        &mut self,
        anim: &mut Animation,
    ) -> Result<(), XFileImportError> {
        self.read_head_of_data_object()?;
        let mut banim = AnimBone::new();

        loop {
            let token = self.next_token()?;
            if token.is_empty() {
                return Err(XFileImportError::unexpected_end_of_file(
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
                // SAFETY: source is guaranteed to be a valid UTF-8 string.
                banim.name = unsafe { String::from_utf8_unchecked(self.next_token()?.to_vec()) };
                self.check_for_closing_brace()?;
            } else {
                self.parse_unknown_data_object()?;
            }
        }
        anim.anims.push(banim);
        Ok(())
    }

    fn parse_data_object_animation_key(
        &mut self,
        banim: &mut AnimBone,
    ) -> Result<(), XFileImportError> {
        self.read_head_of_data_object()?;
        // read key type
        let key_type = self.read_int()?;

        // read number of keys
        let num_keys = self.read_int()?;

        for _ in 0..num_keys {
            // read time
            let time = self.read_int()?;

            // read keys
            match key_type {
                // rotation quaternion
                0 => {
                    // read count
                    let count = self.read_int()?;
                    if count != 4 {
                        return Err(
                            XFileImportError::InvalidNumberOfArgumentsForKeyInAnimation {
                                key_type: "quaternion",
                                expected: 4,
                            },
                        );
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
                    if self.read_int()? != 3 {
                        return Err(
                            XFileImportError::InvalidNumberOfArgumentsForKeyInAnimation {
                                key_type: "vector",
                                expected: 3,
                            },
                        );
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
                    if self.read_int()? != 16 {
                        return Err(
                            XFileImportError::InvalidNumberOfArgumentsForKeyInAnimation {
                                key_type: "matrix",
                                expected: 16,
                            },
                        );
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

                    break;
                }

                _ => {
                    return Err(XFileImportError::UnknownKeyTypeInAnimation(key_type));
                }
            }
            // key separator
            self.check_for_separator()?;
        }
        self.check_for_closing_brace()?;
        Ok(())
    }

    fn parse_unknown_data_object(&mut self) -> Result<(), XFileImportError> {
        // find opening delimiter
        loop {
            let token = self.next_token()?;
            if token.is_empty() {
                return Err(XFileImportError::unexpected_end_of_file(
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
                return Err(XFileImportError::unexpected_end_of_file(
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

    fn parse_file(&mut self) -> Result<(), XFileImportError> {
        loop {
            let token = self.next_token()?;
            if token.is_empty() {
                break;
            }
            // parse specific object
            if token == b"template" {
                self.parse_data_object_template()?;
            } else if token == b"Frame" {
                self.parse_data_object_frame(Index::GUARD_INDEX)?;
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
                let mut material = Material::default();
                self.parse_data_object_material(&mut material)?;
                self.scene.global_materials.push(material);
            } else if token == b"}" {
                // whatever?
            } else {
                self.parse_unknown_data_object()?;
            }
        }
        Ok(())
    }

    fn check_for_separator(&mut self) -> Result<(), XFileImportError> {
        if self.is_binary_format {
            return Ok(());
        }
        let next = self.next_token()?;
        if !matches!(next, b"," | b";") {
            return Err(XFileImportError::SeparatorCharacterExpected(
                String::from_utf8_lossy(next).into_owned(),
            ));
        }
        Ok(())
    }

    fn check_for_semicolon(&mut self) -> Result<(), XFileImportError> {
        if self.is_binary_format {
            return Ok(());
        }

        let next = self.next_token()?;
        if next != b";" {
            return Err(XFileImportError::SemicolonExpected(
                String::from_utf8_lossy(next).into_owned(),
            ));
        }
        Ok(())
    }

    fn check_for_closing_brace(&mut self) -> Result<(), XFileImportError> {
        let next = self.next_token()?;
        if next != b"}" {
            return Err(XFileImportError::ClosingBraceExpected(
                String::from_utf8_lossy(next).into_owned(),
            ));
        }
        Ok(())
    }

    fn test_for_separator(&mut self) {
        if self.is_binary_format {
            return;
        }
        self.skip_whitespace();
        let Some(b) = self.peek_one() else {
            return;
        };
        if matches!(b, b',' | b';') {
            // SAFETY: we know that the next byte is a separator
            unsafe { self.forward_unchecked(1) };
        }
    }

    fn read_binary_word(&mut self) -> Result<u16, XFileImportError> {
        let word = self
            .forward(2)
            .map_err(|_| XFileImportError::NotEnoughDataToRead(2))?;
        Ok(u16::from_le_bytes([word[0], word[1]]))
    }

    fn read_binary_dword(&mut self) -> Result<u32, XFileImportError> {
        let dword = self
            .forward(4)
            .map_err(|_| XFileImportError::NotEnoughDataToRead(4))?;
        Ok(u32::from_le_bytes([dword[0], dword[1], dword[2], dword[3]]))
    }

    fn read_int(&mut self) -> Result<u32, XFileImportError> {
        if self.is_binary_format {
            if self.binary_num_count == 0 && self.rest() >= 2 {
                let tmp = self.read_binary_word()?;
                if tmp == 0x06 && self.rest() >= 4 {
                    // array of floats following
                    self.binary_num_count = self.read_binary_dword()?;
                } else {
                    // single float following
                    self.binary_num_count = 1;
                }
            }
            self.binary_num_count -= 1;
            if self.rest() >= 4 {
                return self.read_binary_dword();
            } else {
                self.source = &[];
                return Ok(0);
            }
        } else {
            self.skip_whitespace();
            let Some(b) = self.peek_one() else {
                return Err(XFileImportError::NotEnoughDataToRead(1));
            };
            let is_neg = if b == b'-' {
                self.forward(1)?;
                true
            } else {
                false
            };
            let mut value = 0;
            while let &[b, ref rest @ ..] = self.source {
                if b.is_ascii_digit() {
                    value = value * 10 + (b - b'0') as u32;
                    self.source = rest;
                } else {
                    break;
                }
            }
            self.check_for_separator()?;
            return Ok(if is_neg {
                (-(value as i32)) as u32
            } else {
                value
            });
        }
    }

    fn read_float(&mut self) -> Result<AiReal, XFileImportError> {
        if self.is_binary_format {
            if self.binary_num_count == 0 && self.rest() >= 2 {
                let tmp = self.read_binary_word()?;
                if tmp == 0x07 && self.rest() >= 4 {
                    // array of floats following
                    self.binary_num_count = self.read_binary_dword()?;
                } else {
                    // single float following
                    self.binary_num_count = 1;
                }
            }
            self.binary_num_count -= 1;
            if self.binary_float_size == 8 {
                if self.rest() >= 8 {
                    return Ok(f64::from_le_bytes(self.forward(8)?.try_into().unwrap()) as f32);
                } else {
                    self.source = &[];
                    return Ok(0.0);
                }
            } else {
                if self.rest() >= 4 {
                    return Ok(f32::from_le_bytes(self.forward(4)?.try_into().unwrap()));
                } else {
                    self.source = &[];
                    return Ok(0.0);
                }
            }
        }
        self.skip_whitespace();

        // check for various special strings to allow reading files from faulty exporters
        // I mean you, Blender!
        let special_string = self.peek::<9>();

        if special_string == Some(b"-1.#IND00") {
            // SAFETY: we know that the next 8 bytes are a special string
            unsafe { self.forward_unchecked(8) };
            self.check_for_separator()?;
            return Ok(0.0);
        } else if matches!(self.peek::<8>(), Some(b"1.#IND00") | Some(b"1.#QNAN0")) {
            // SAFETY: we know that the next 8 bytes are a special string
            unsafe { self.forward_unchecked(8) };
            self.check_for_separator()?;
            return Ok(0.0);
        }
        let (rest, f) =
            fast_atoreal_move(self.source, true).map_err(|e| XFileImportError::FastAtofError(e))?;

        self.source = rest;
        self.check_for_separator()?;
        Ok(f)
    }

    fn read_vec2(&mut self) -> Result<Vec2, XFileImportError> {
        let x = self.read_float()?;
        let y = self.read_float()?;
        self.test_for_separator();
        Ok(Vec2::new(x, y))
    }

    fn read_vec3(&mut self) -> Result<Vec3, XFileImportError> {
        let x = self.read_float()?;
        let y = self.read_float()?;
        let z = self.read_float()?;
        self.test_for_separator();
        Ok(Vec3::new(x, y, z))
    }

    fn read_rgb(&mut self) -> Result<Color3D, XFileImportError> {
        let r = self.read_float()?;
        let g = self.read_float()?;
        let b = self.read_float()?;
        self.test_for_separator();
        Ok(Color3D::new(r, g, b))
    }

    fn read_rgba(&mut self) -> Result<Color4D, XFileImportError> {
        let r = self.read_float()?;
        let g = self.read_float()?;
        let b = self.read_float()?;
        let a = self.read_float()?;
        self.test_for_separator();
        Ok(Color4D::new(r, g, b, a))
    }

    fn peek_one(&self) -> Option<u8> {
        let Some((&b, _)) = self.source.split_first() else {
            return None;
        };
        Some(b)
    }

    fn next_byte_if_eq(&mut self, test_byte: u8) {
        if self.peek_one() == Some(test_byte) {
            // SAFETY: we know that the next byte is the test byte
            unsafe { self.forward_unchecked(1) };
        }
    }

    fn peek<const N: usize>(&self) -> Option<&'a [u8; N]> {
        let (data, _) = self.source.split_at_checked(N)?;
        Some(data.try_into().unwrap())
    }

    fn forward(&mut self, n: usize) -> Result<&'a [u8], XFileImportError> {
        let (data, rest) = self
            .source
            .split_at_checked(n)
            .ok_or(XFileImportError::unexpected_end_of_file("forward"))?;
        self.source = rest;
        Ok(data)
    }

    unsafe fn forward_unchecked(&mut self, n: usize) -> &'a [u8] {
        let (data, rest) = unsafe { self.source.split_at_unchecked(n) };
        self.source = rest;
        data
    }

    fn skip_until_next_line(&mut self) {
        if self.is_binary_format {
            return;
        }
        while let &[b, ref rest @ ..] = self.source {
            self.source = rest;
            if b == b'\n' || b == b'\r' {
                // process '\r\n' on windows
                self.next_byte_if_eq(b'\n');
                self.line_number += 1;
                break;
            }
        }
    }

    fn skip_whitespace(&mut self) {
        if self.is_binary_format {
            return;
        }
        loop {
            while let &[b, ref rest @ ..] = self.source {
                if b.is_ascii_whitespace() {
                    self.line_number += (b == b'\n') as u32;
                    self.source = rest;
                } else {
                    break;
                }
            }
            if self.rest() == 0 {
                return;
            }
            if let &[a, b, ref rest @ ..] = self.source {
                if a == b'/' && b == b'/' || a == b'#' {
                    self.source = rest;
                    self.skip_until_next_line();
                } else {
                    break;
                }
            } else {
                break;
            }
        }
    }

    fn next_token_as_str(&mut self) -> Result<&str, XFileImportError> {
        if self.is_binary_format {
            let token = self.next_token()?;
            Ok(unsafe { str::from_utf8_unchecked(token) })
        } else {
            self.skip_whitespace();
            if self.rest() == 0 {
                return Err(XFileImportError::unexpected_end_of_file(
                    "next_token_as_str",
                ));
            }
            let b = self.peek_one();
            if b != Some(b'"') {
                return Err(XFileImportError::unexpected_token(
                    "\"",
                    String::from_utf8_lossy(self.next_token()?).into_owned(),
                ));
            }
            // SAFETY: we know that the next byte is '"'
            unsafe { self.forward_unchecked(1) };
            let mut cnt = 0;
            for b in self.source {
                if *b == b'"' {
                    break;
                }
                cnt += 1;
            }
            // SAFETY: cnt is within the bounds of the source.
            let token = unsafe { self.forward_unchecked(cnt) };
            let deliminator = self
                .forward(2)
                .map_err(|_| XFileImportError::unexpected_end_of_file("next_token_as_str"))?;
            if deliminator != b"\";" {
                return Err(XFileImportError::unexpected_token(
                    "\";",
                    String::from_utf8_lossy(deliminator).into_owned(),
                ));
            }
            Ok(unsafe { str::from_utf8_unchecked(token) })
        }
    }

    fn next_token(&mut self) -> Result<&'a [u8], XFileImportError> {
        if self.is_binary_format {
            if self.rest() < 2 {
                return Ok(&[]);
            }
            let tok = self.read_binary_word()?;
            match tok {
                1 => {
                    let len = self.read_binary_dword()?;
                    if self.rest() < len as usize {
                        return Err(XFileImportError::NotEnoughDataToRead(len as usize));
                    }
                    return self.forward(len as usize);
                }
                2 => {
                    let len = self.read_binary_dword()?;
                    if self.rest() < len as usize {
                        return Err(XFileImportError::NotEnoughDataToRead(len as usize));
                    }
                    let s = self.forward(len as usize + 2)?;
                    return Ok(&s[..s.len() - 2]);
                }
                3 => {
                    let _ = self.forward(4);
                    return Ok(b"<integer>");
                }
                5 => {
                    let _ = self.forward(16);
                    return Ok(b"<guid>");
                }
                6 => {
                    let len = self.read_binary_dword()?;
                    if self.rest() < len as usize {
                        return Err(XFileImportError::NotEnoughDataToRead(len as usize));
                    }
                    self.forward((len as usize) * 4)?;
                    return Ok(b"<int_list>");
                }
                7 => {
                    let len = self.read_binary_dword()?;
                    if self.rest() < len as usize {
                        return Err(XFileImportError::NotEnoughDataToRead(len as usize));
                    }
                    self.forward((len * self.binary_float_size as u32) as usize)?;
                    return Ok(b"<flt_list>");
                }
                0x0a => {
                    return Ok(b"{");
                }
                0x0b => {
                    return Ok(b"}");
                }
                0x0c => {
                    return Ok(b"(");
                }
                0x0d => {
                    return Ok(b")");
                }
                0x0e => {
                    return Ok(b"[");
                }
                0x0f => {
                    return Ok(b"]");
                }
                0x10 => {
                    return Ok(b"<");
                }
                0x11 => {
                    return Ok(b">");
                }
                0x12 => {
                    return Ok(b".");
                }
                0x13 => {
                    return Ok(b",");
                }
                0x14 => {
                    return Ok(b";");
                }
                0x1f => {
                    return Ok(b"template");
                }
                0x28 => {
                    return Ok(b"WORD");
                }
                0x29 => {
                    return Ok(b"DWORD");
                }
                0x2a => {
                    return Ok(b"FLOAT");
                }
                0x2b => {
                    return Ok(b"DOUBLE");
                }
                0x2c => {
                    return Ok(b"CHAR");
                }
                0x2d => {
                    return Ok(b"UCHAR");
                }
                0x2e => {
                    return Ok(b"SWORD");
                }
                0x2f => {
                    return Ok(b"SDWORD");
                }
                0x30 => {
                    return Ok(b"void");
                }
                0x31 => {
                    return Ok(b"string");
                }
                0x32 => {
                    return Ok(b"unicode");
                }
                0x33 => {
                    return Ok(b"cstring");
                }
                0x34 => {
                    return Ok(b"array");
                }
                _ => {
                    return Ok(&[]);
                }
            }
        } else {
            self.skip_whitespace();
            if self.rest() == 0 {
                return Ok(&[]);
            }
            let mut index = 0;
            let mut _rest = self.source;
            while let &[b, ref rest @ ..] = _rest {
                if !b.is_ascii_whitespace() {
                    _rest = rest;
                    if matches!(b, b';' | b'}' | b'{' | b',') {
                        if index == 0 {
                            index += 1;
                        }
                        break;
                    }
                    index += 1;
                } else {
                    break;
                }
            }
            let token = &self.source[..index];
            self.source = _rest;
            return Ok(token);
        }
    }
}
