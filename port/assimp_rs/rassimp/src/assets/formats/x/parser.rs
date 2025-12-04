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

//! Implements X file format parser for the library

#[cfg(not(feature = "std"))]
use alloc::collections::BTreeMap as Map;
use alloc::{
    borrow::{Cow, ToOwned},
    string::String,
    vec::Vec,
};
use core::{marker::PhantomData, mem};
#[cfg(feature = "std")]
use std::collections::HashMap as Map;

#[cfg(feature = "compression")]
use zlib_rs::{InflateFlush, MAX_WBITS};

mod binary_parser;
#[allow(unused)]
pub(super) mod constants;
pub(super) mod text_parser;

use crate::{
    AiReal, AiVec2, AiVec3,
    assets::formats::x::{
        error::{XFileCommonParseError, XFileImportError, XFileParseError},
        importer::XFileImporterConfig,
        parser::{
            binary_parser::BinaryParser, constants::ms_compression::*, text_parser::TextParser,
        },
        structs::{
            AnimTicksPerSecond, Animation, Frame, Material, Mesh, Node, Scene, Template,
            UnknownDataObject,
        },
    },
    io::{
        reader::parse_4digits,
        utils::decompression::{Compression, Format},
    },
    structs::{
        color::{Color3D, Color4D},
        index::Index,
    },
};

/// X file format parser trait
///
/// This trait defines the interface for parsing X file format data, supporting both
/// text and binary formats. It provides methods for reading various data types,
/// tokenization, and navigation through the source data.
///
/// The trait is designed to be implemented by different parser types (text and binary)
/// while providing a unified interface for the parsing logic.
pub(super) trait XFileParser<'source> {
    /// Get the current parsing position for error reporting
    fn get_position_info(&self) -> String;

    /// Get the raw position of the parser
    fn get_position(&self) -> usize;

    /// Forward the parser by n bytes, returning the skipped data
    fn forward(&mut self, _n: usize) -> Result<&'source [u8], XFileCommonParseError> {
        Ok(&[])
    }

    /// Forward the parser by n bytes without bounds checking
    ///
    /// # Safety
    /// The caller must ensure that there are at least n bytes remaining in the source
    unsafe fn forward_unchecked(&mut self, _n: usize) -> &'source [u8];

    /// Peek at the next N bytes without advancing the parser
    fn peek<const N: usize>(&self) -> Option<&'source [u8; N]>;

    /// Peek at the next single byte without advancing the parser
    fn peek_one(&self) -> Option<u8>;

    /// Advance the parser by one byte if the next byte equals the test byte
    fn next_byte_if_eq(&mut self, test_byte: u8) {
        if self.peek_one() == Some(test_byte) {
            // SAFETY: we know that the next byte is the test byte
            unsafe { self.forward_unchecked(1) };
        }
    }

    /// Skip all characters until the next line break
    fn skip_until_next_line(&mut self) {}

    /// Skip all whitespace characters
    fn skip_whitespace(&mut self) {}

    /// Read and parse a 32-bit unsigned integer
    fn read_int(&mut self) -> Result<u32, XFileCommonParseError>;

    /// Read and parse a floating-point number
    fn read_float(&mut self) -> Result<AiReal, XFileCommonParseError>;

    /// Read and parse a 2D vector (x, y coordinates)
    fn read_vec2(&mut self) -> Result<AiVec2, XFileCommonParseError> {
        let x = self.read_float()?;
        let y = self.read_float()?;
        self.test_for_separator();
        Ok(AiVec2::new(x, y))
    }

    /// Read and parse a 3D vector (x, y, z coordinates)
    fn read_vec3(&mut self) -> Result<AiVec3, XFileCommonParseError> {
        let x = self.read_float()?;
        let y = self.read_float()?;
        let z = self.read_float()?;
        self.test_for_separator();
        Ok(AiVec3::new(x as AiReal, y as AiReal, z as AiReal))
    }

    /// Read and parse an RGB color (red, green, blue components)
    fn read_rgb(&mut self) -> Result<Color3D, XFileCommonParseError> {
        let r = self.read_float()?;
        let g = self.read_float()?;
        let b = self.read_float()?;
        self.test_for_separator();
        Ok(Color3D::new(r, g, b))
    }

    /// Read and parse an RGBA color (red, green, blue, alpha components)
    fn read_rgba(&mut self) -> Result<Color4D, XFileCommonParseError> {
        let r = self.read_float()?;
        let g = self.read_float()?;
        let b = self.read_float()?;
        let a = self.read_float()?;
        self.test_for_separator();
        Ok(Color4D::new(r, g, b, a))
    }

    /// Read the next token from the source data
    fn next_token(&mut self) -> &'source [u8];

    /// Read the next token and convert it to a UTF-8 string
    fn next_token_as_str(&mut self) -> Result<Cow<'source, str>, XFileCommonParseError>;

    /// Check for and consume a separator character (comma, semicolon, etc.)
    fn check_for_separator(&mut self) -> Result<(), XFileCommonParseError> {
        Ok(())
    }

    /// Check for and consume a semicolon
    fn check_for_semicolon(&mut self) -> Result<(), XFileCommonParseError> {
        Ok(())
    }

    /// Check for and consume a closing brace
    fn check_for_closing_brace(&mut self) -> Result<(), XFileCommonParseError> {
        let next = self.next_token();
        if next != b"}" {
            return Err(XFileCommonParseError::ClosingBraceExpected(
                match str::from_utf8(next) {
                    Ok(s) => s.to_owned(),
                    Err(_) => format!("bytes: {next:?}"),
                },
            ));
        }
        Ok(())
    }

    /// Test for and optionally consume a separator character
    fn test_for_separator(&mut self) {}

    /// This method consumes the extra semicolons.
    ///
    /// In version 03.02, the face indices end with two semicolons.
    ///
    /// Version 03.03 exported from blender also has 2 semicolons.
    fn consume_version_specific_semicolon(&mut self) {}
}

/// X File part parsing trait
///
/// This trait defines the interface for parsing a part of the X file.
///
/// The trait is designed to be implemented by different part types (e.g., Frame, Mesh, Material,
/// etc.) while providing a unified interface for the parsing logic.
pub(super) trait XFileParse<'source> {
    /// The output type of the parsed part.
    type Output;
    fn parse<P: XFileParser<'source>>(
        parser: &mut ParserCtx<'source, P>,
    ) -> Result<Self::Output, XFileCommonParseError>;
}

/// X file format parser.
pub struct Parser;

/// X file header.
#[derive(Clone, Copy, Debug)]
pub struct XFileHeader {
    /// Major version of the X file format.
    pub major_version: u8,

    /// Minor version of the X file format.
    pub minor_version: u8,

    /// Whether the file is compressed.
    pub is_compressed: bool,

    /// Whether the file is in binary format.
    pub is_binary_format: bool,

    /// Size of the floating-point number in the file.
    pub is_64_bits_float: bool,
}

impl XFileHeader {
    const HEADER_BINARY_SIZE: usize = 16;
}

/// X file format parser result.
#[derive(Clone, Debug)]
pub struct XFile {
    /// Header of the X file.
    pub header: XFileHeader,

    /// Scene of the X file.
    pub scene: Scene,
}

impl Parser {
    /// Parse the X file from the source.
    pub fn parse(source: &[u8], config: XFileImporterConfig) -> Result<XFile, XFileImportError> {
        let (header, source) = Self::parse_header(source)?;

        let XFileHeader {
            is_compressed,
            is_binary_format,
            is_64_bits_float,
            ..
        } = header;

        Ok(XFile {
            header,
            scene: {
                let mut scene = if is_compressed {
                    Self::parse_compressed_file(source, is_binary_format, is_64_bits_float, config)?
                } else {
                    Self::parse_by_format(source, is_binary_format, None, is_64_bits_float, config)?
                };
                Self::filter_hierarchy(&mut scene)?;
                scene
            },
        })
    }

    /// Filters the imported hierarchy for some degenerated cases that some exporters produce.
    fn filter_hierarchy(scene: &mut Scene) -> Result<(), XFileImportError> {
        if scene.has_root_node() {
            // check if there are any nodes that need merging
            let mut need_merge = false;
            {
                let mut stack = vec![Index::ROOT_INDEX];
                // Read-only traversal to check for merge candidates
                while let Some(node_id) = stack.pop() {
                    let node = &scene.nodes[node_id.value()];

                    // if the node has just a single unnamed child containing a mesh, remove
                    // the anonymous node between. The 3DSMax kwXport plugin seems to produce this
                    // mess in some cases
                    if node.meshes.is_empty() && node.children.len() == 1 {
                        let child_id = node.children[0];
                        let child = &scene.nodes[child_id.value()];

                        if child.name.is_empty() && !child.meshes.is_empty() {
                            need_merge = true;
                            break; // Early exit if any merge candidate is found
                        }
                    }
                    stack.extend(node.children.iter());
                }
            }

            // Only proceed with full processing if merge is needed
            if !need_merge {
                return Ok(());
            }

            // New node storage and ID mapping
            let mut new_nodes = vec![];
            let mut id_map = Map::new(); // Maps old NodeId -> new NodeId

            // Traversal stack (using old IDs)
            let mut stack = vec![Index::ROOT_INDEX];

            // First pass: process nodes and build new node list
            while let Some(old_id) = stack.pop() {
                // Take ownership of the node (replace with default)
                let mut node = mem::take(scene.nodes.get_mut(old_id.value()).ok_or(
                    XFileImportError::Other(
                        "each node's children should correspond to a node in nodes",
                    ),
                )?);

                // if the node has just a single unnamed child containing a mesh, remove
                // the anonymous node between. The 3DSMax kwXport plugin seems to produce this
                // mess in some cases
                if node.meshes.is_empty()
                    && let [child_id] = node.children.as_slice()
                {
                    if let Some(child) = scene.nodes.get_mut(child_id.value())
                        && child.name.is_empty()
                        && !child.meshes.is_empty()
                    {
                        // Transfer meshes and transform
                        node.meshes.append(&mut child.meshes);
                        node.transformation_matrix *= child.transformation_matrix;
                    }
                } else {
                    // Push children for normal processing
                    stack.extend(node.children.iter());
                }

                // Map old ID to new position and store node
                let new_id = Index::new(new_nodes.len() as u32);
                id_map.insert(old_id, new_id);
                new_nodes.push(node);
            }

            // Second pass: update child references with new IDs
            for node in &mut new_nodes {
                let new_children: Vec<_> = node
                    .children
                    .iter()
                    .filter_map(|old_id| id_map.get(old_id).copied())
                    .collect();
                node.children = new_children;
            }

            // Replace node storage
            scene.nodes = new_nodes;
        }
        Ok(())
    }

    fn parse_header(source: &[u8]) -> Result<(XFileHeader, &[u8]), XFileImportError> {
        let Some((header, rest)) = source.split_first_chunk::<16>() else {
            return Err(XFileImportError::XFileParseError {
                position: "0".into(),
                error: XFileCommonParseError::NotEnoughDataToReadForHeader(source.len()).into(),
            });
        };
        if &header[..4] != b"xof " {
            return Err(XFileImportError::XFileParseError {
                position: "0".into(),
                error: XFileCommonParseError::UnsupportedFileFormat([
                    header[0], header[1], header[2], header[3],
                ])
                .into(),
            });
        }

        let major_version = (header[4] - b'0') * 10 + (header[5] - b'0');
        let minor_version = (header[6] - b'0') * 10 + (header[7] - b'0');

        let file_format_signature = &header[8..12];
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
                position: "8".into(),
                error: XFileCommonParseError::UnsupportedFileFormat([
                    file_format_signature[0],
                    file_format_signature[1],
                    file_format_signature[2],
                    file_format_signature[3],
                ])
                .into(),
            });
        };

        let binary_format_size = parse_4digits::<10>(u32::from_le_bytes([
            header[12], header[13], header[14], header[15],
        ]));
        if binary_format_size != 32 && binary_format_size != 64 {
            return Err(XFileImportError::XFileParseError {
                position: "12".into(),
                error: XFileCommonParseError::UnsupportedFloatSize(binary_format_size).into(),
            });
        }
        let is_64_bits_float = binary_format_size == 64;
        Ok((
            XFileHeader {
                major_version,
                minor_version,
                is_compressed,
                is_binary_format,
                is_64_bits_float,
            },
            rest,
        ))
    }

    fn parse_by_format<'source>(
        source: &'source [u8],
        is_binary_format: bool,
        decompressed_source: Option<&'source [u8]>,
        is_64_bits_float: bool,
        config: XFileImporterConfig,
    ) -> Result<Scene, XFileImportError> {
        fn parse_file_by_format_inner<'source, P: XFileParser<'source>>(
            mut parser: ParserCtx<'source, P>,
            decompressed_source: Option<&'source [u8]>,
        ) -> Result<Scene, XFileImportError> {
            match parser.parse() {
                Ok(_) => Ok(parser.scene),
                Err(e) => match decompressed_source {
                    Some(decompressed_source) => Err(XFileImportError::XFileParseError {
                        position: format!("{} in decompressed X file", parser.get_position_info())
                            .into(),
                        error: XFileParseError::CompressedFileParseError {
                            decompressed_source: decompressed_source.to_vec(),
                            error: e,
                        },
                    }),
                    None => Err(XFileImportError::XFileParseError {
                        position: parser.get_position_info().into(),
                        error: e.into(),
                    }),
                },
            }
        }

        if is_binary_format {
            if is_64_bits_float {
                parse_file_by_format_inner(
                    ParserCtx::new(BinaryParser::<true>::new(source)),
                    decompressed_source,
                )
            } else {
                parse_file_by_format_inner(
                    ParserCtx::new(BinaryParser::<false>::new(source)),
                    decompressed_source,
                )
            }
        } else if config.check_ill_float_for_faulty_exporters {
            parse_file_by_format_inner(
                ParserCtx::new(TextParser::<true>::new(source)),
                decompressed_source,
            )
        } else {
            parse_file_by_format_inner(
                ParserCtx::new(TextParser::<false>::new(source)),
                decompressed_source,
            )
        }
    }

    fn parse_compressed_file(
        mut source: &[u8],
        is_binary_format: bool,
        is_64_bits_float: bool,
        config: XFileImporterConfig,
    ) -> Result<Scene, XFileImportError> {
        let start = source.as_ptr() as usize;
        let error_handler = |error: XFileCommonParseError| XFileImportError::XFileParseError {
            position: format!("Offset {}", source.as_ptr() as usize - start).into(),
            error: error.into(),
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
                return Err(error_handler(
                    XFileCommonParseError::NotEnoughDataToReadForHeader(6),
                ));
            }

            // First find out how much storage we'll need. Count sections.
            let mut cloned_source = source;
            let mut est_out = 0;

            while let &[a, b, c, d, ..] = cloned_source {
                // read next offset
                let ofs = u16::from_le_bytes([a, b]) as usize;
                if ofs >= MSZIP_BLOCK {
                    return Err(error_handler(
                        XFileCommonParseError::InvalidOffsetToNextMszipCompressedBlock(ofs),
                    ));
                }

                // check magic word
                let magic = u16::from_le_bytes([c, d]);
                if magic != MSZIP_MAGIC {
                    return Err(error_handler(
                        XFileCommonParseError::UnsupportedCompressedFormat([c, d]),
                    ));
                }

                // and advance to the next offset
                if let Some(s) = cloned_source.get(ofs..) {
                    cloned_source = s;
                } else {
                    return Err(error_handler(XFileCommonParseError::TooSmallZipFile {
                        left: cloned_source.len(),
                        offset: ofs,
                    }));
                }
                est_out += MSZIP_BLOCK; // one decompressed block is 32786 in size
            }
            let decompressed_source = {
                let mut decompressed_source: Vec<u8> = vec![0u8; est_out + 1];
                #[cfg(feature = "zlib_custom_allocator")]
                let mut compression = Compression::new_with_custom_allocator(
                    config.custom_zlib_alloc_fn,
                    config.custom_zlib_free_fn,
                );
                #[cfg(not(feature = "zlib_custom_allocator"))]
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
                    .map_err(|e| error_handler(XFileCommonParseError::DecompressionError(e)))?;
                let mut out = decompressed_source.as_mut_slice();
                while let &[a, b, _, _, ref rest @ ..] = source {
                    let ofs = u16::from_le_bytes([a, b]) as usize;
                    source = rest;
                    if source.len() + 2 < ofs {
                        return Err(XFileImportError::FileTooSmall);
                    }

                    let size = compression
                        .decompress_block(source, &mut out[..MSZIP_BLOCK])
                        .map_err(|e| error_handler(XFileCommonParseError::DecompressionError(e)))?;
                    // SAFETY: size is guaranteed to be less than MSZIP_BLOCK
                    out = unsafe { out.get_unchecked_mut(size..) };
                    if let Some(s) = source.get(ofs..) {
                        source = s;
                    } else {
                        break;
                    }
                }
                decompressed_source
            };
            Self::parse_by_format(
                &decompressed_source,
                is_binary_format,
                Some(&decompressed_source),
                is_64_bits_float,
                config,
            )
        }
        #[cfg(not(feature = "compression"))]
        {
            Err(XFileImportError::XFileParseError {
                position: format!("Offset {}", start),
                error: XFileCommonParseError::CompressionFeatureNotEnabled,
            })
        }
    }
}

pub(super) struct ParserCtx<'source, P: XFileParser<'source>> {
    inner_parser: P,
    pub(super) scene: Scene,
    _marker: PhantomData<&'source [u8]>,
}

impl<'source, P: XFileParser<'source>> XFileParser<'source> for ParserCtx<'source, P> {
    fn get_position_info(&self) -> String {
        self.inner_parser.get_position_info()
    }

    fn get_position(&self) -> usize {
        self.inner_parser.get_position()
    }

    unsafe fn forward_unchecked(&mut self, n: usize) -> &'source [u8] {
        unsafe { self.inner_parser.forward_unchecked(n) }
    }

    fn peek<const N: usize>(&self) -> Option<&'source [u8; N]> {
        self.inner_parser.peek::<N>()
    }

    fn peek_one(&self) -> Option<u8> {
        self.inner_parser.peek_one()
    }

    fn read_int(&mut self) -> Result<u32, XFileCommonParseError> {
        self.inner_parser.read_int()
    }

    fn read_float(&mut self) -> Result<AiReal, XFileCommonParseError> {
        self.inner_parser.read_float()
    }

    fn next_token(&mut self) -> &'source [u8] {
        self.inner_parser.next_token()
    }

    fn next_token_as_str(&mut self) -> Result<Cow<'source, str>, XFileCommonParseError> {
        self.inner_parser.next_token_as_str()
    }

    fn check_for_separator(&mut self) -> Result<(), XFileCommonParseError> {
        self.inner_parser.check_for_separator()
    }

    fn check_for_semicolon(&mut self) -> Result<(), XFileCommonParseError> {
        self.inner_parser.check_for_semicolon()
    }

    fn test_for_separator(&mut self) {
        self.inner_parser.test_for_separator()
    }

    fn consume_version_specific_semicolon(&mut self) {
        self.inner_parser.consume_version_specific_semicolon()
    }
}

impl<'source, P: XFileParser<'source>> ParserCtx<'source, P> {
    #[inline]
    pub fn new(inner_parser: P) -> Self {
        Self {
            inner_parser,
            scene: Scene::default(),
            _marker: PhantomData,
        }
    }

    pub(super) fn parse(&mut self) -> Result<(), XFileCommonParseError> {
        loop {
            let token = self.next_token();
            if token.is_empty() {
                return Ok(());
            }
            // parse specific object
            if token == b"template" {
                Template::parse(self)?;
            } else if token == b"Frame" {
                Frame::parse(self)?;
            } else if token == b"Mesh" {
                // some meshes have no frames at all
                let mesh = Mesh::parse(self)?;
                self.scene.global_meshes.push(mesh);
            } else if token == b"AnimTicksPerSecond" {
                AnimTicksPerSecond::parse(self)?;
            } else if token == b"AnimationSet" {
                let animation = Animation::parse(self)?;
                self.scene.animations.push(animation);
            } else if token == b"Material" {
                // Material outside of a mesh or node
                let material = Material::parse(self)?;
                self.scene.global_materials.push(material);
            } else if token == b"}" {
                // whatever?
            } else {
                UnknownDataObject::parse(self)?;
            }
        }
    }
}
