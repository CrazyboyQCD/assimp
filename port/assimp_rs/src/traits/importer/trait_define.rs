#[cfg(feature = "std")]
use std::{fs::File, io::Read, path::Path};

use super::error::{EncodingError, ImportError};
use crate::structs::scene::AiScene;

/// UTF encoding conversion utilities
pub mod encoding {
    use core::mem;

    use super::EncodingError;

    /// Convert bytes of different encodings to UTF-8 string
    ///
    /// Supported encoding formats:
    /// - UTF-8 (with/without BOM)
    /// - UTF-16 BE/LE (with BOM)
    /// - UTF-32 BE/LE (with BOM)
    pub fn convert_to_utf8(mut buf: Vec<u8>) -> Result<String, EncodingError> {
        if buf.len() < 8 {
            return Err(EncodingError::UnknownEncoding);
        }

        // UTF-8 with BOM
        if buf.len() >= 3 && buf[0] == 0xEF && buf[1] == 0xBB && buf[2] == 0xBF {
            buf.rotate_left(3);
            buf.truncate(buf.len() - 3);
            return String::from_utf8(buf).map_err(|_| EncodingError::NotValidUtf8);
        }

        // UTF-32 with BOM
        if let Some(b) = buf.get(0..4) {
            let b = u32::from_le_bytes(b.try_into().unwrap());
            if b == 0xFFFE0000 || b == 0x0000FFFE {
                return convert_utf32_to_string(&buf, b == 0xFFFE0000);
            }
        }

        // UTF-16 with BOM
        if let Some(b) = buf.get(0..2) {
            let b = u16::from_le_bytes(b.try_into().unwrap());
            if b == 0xFFFE || b == 0xFEFF {
                return convert_utf16_to_string(&buf, b == 0xFFFE);
            }
        }

        // Default to UTF-8
        String::from_utf8(buf).map_err(|_| EncodingError::UnknownEncoding)
    }

    fn convert_utf32_to_string(buf: &[u8], is_big_endian: bool) -> Result<String, EncodingError> {
        if buf.len() % mem::size_of::<u32>() != 0 {
            return Err(EncodingError::NotValidUtf32Length(buf.len()));
        }

        let mut s = String::with_capacity(buf.len() / 4);
        for chunk in buf.chunks_exact(4) {
            let bytes: [u8; 4] = chunk.try_into().unwrap();
            let code_point = if is_big_endian {
                u32::from_be_bytes(bytes)
            } else {
                u32::from_le_bytes(bytes)
            };

            let c =
                char::from_u32(code_point).ok_or(EncodingError::NotValidCodePoint(code_point))?;
            s.push(c);
        }
        Ok(s)
    }

    fn convert_utf16_to_string(buf: &[u8], is_big_endian: bool) -> Result<String, EncodingError> {
        let len = buf.len();
        if len % mem::size_of::<u16>() != 0 {
            return Err(EncodingError::NotValidUtf16Length(len));
        }

        let result = if is_big_endian {
            char::decode_utf16(
                buf.chunks_exact(2)
                    .map(|v| u16::from_be_bytes(v.try_into().unwrap())),
            )
            .collect::<Result<String, _>>()
            .map_err(|e| EncodingError::NotValidUtf16Be(e))
        } else {
            char::decode_utf16(
                buf.chunks_exact(2)
                    .map(|v| u16::from_le_bytes(v.try_into().unwrap())),
            )
            .collect::<Result<String, _>>()
            .map_err(|e| EncodingError::NotValidUtf16Le(e))
        };

        result
    }

    /// Convert UTF-8 to ISO-8859-1(Latin-1)
    pub fn convert_utf8_to_iso8859_1(buf: &mut Vec<u8>) -> Result<(), EncodingError> {
        let len = buf.len();
        let mut i = 0;
        let mut j = 0;

        while i < len {
            if buf[i] < 0x80 {
                buf[j] = buf[i];
            } else if i < len - 1 {
                if buf[i] == 0xC2 {
                    i += 1;
                    buf[j] = buf[i];
                } else if buf[i] == 0xC3 {
                    i += 1;
                    buf[j] = buf[i] + 0x40;
                } else {
                    return Err(EncodingError::NotValidUtf8ToIso8859_1(buf[i], buf[i + 1]));
                }
            } else {
                return Err(EncodingError::NotValidUtf8OnlyOneCharacterRemaining);
            }

            i += 1;
            j += 1;
        }

        buf.truncate(j);
        Ok(())
    }
}

/// Format header (magic signature) trait
///
/// Each format with a fixed header should implement this trait
pub trait FormatHeader<const N: usize> {
    /// Magic byte sequence of the format
    const HEADER: [u8; N];

    /// Check if given byte sequence matches format header
    fn check_header(buf: &[u8]) -> bool {
        buf.get(..N).map_or(false, |b| b == Self::HEADER)
    }
}

/// Format validator trait
///
/// Unified handling of format validation from different sources (file, Reader, buffer)
pub trait FormatValidator<const N: usize>: FormatHeader<N> {
    /// Validate format from buffer
    fn can_read_from_buf(buf: &[u8]) -> bool {
        Self::check_header(buf)
    }

    /// Validate format from Reader
    #[cfg(feature = "std")]
    fn can_read_from_reader<R: Read>(reader: &mut R) -> Result<bool, std::io::Error> {
        let mut buffer = [0u8; N];
        reader.read_exact(&mut buffer)?;
        Ok(Self::check_header(&buffer))
    }

    /// Validate format from file
    #[cfg(feature = "std")]
    fn can_read_from_file<P: AsRef<Path>>(file_path: P) -> Result<bool, std::io::Error> {
        match File::open(file_path) {
            Ok(mut file) => Ok(Self::can_read_from_reader(&mut file)?),
            Err(e) => Err(e),
        }
    }
}

// Automatically implement FormatValidator for all types that implement FormatHeader
impl<const N: usize, T: FormatHeader<N>> FormatValidator<N> for T {}

/// Internal importer trait
///
/// Focus on core import logic, excluding format validation and encoding conversion
pub trait InternalImporter<E> {
    /// Import from byte buffer to scene
    fn import_from_buf(buf: &[u8], scene: &mut AiScene) -> Result<(), E>;

    /// Import from file to scene
    #[cfg(feature = "std")]
    fn import_from_file(file_name: &str, scene: &mut AiScene) -> Result<(), E>;
}

/// Public importer trait
///
/// Provide high-level import API, returning complete scene objects
pub trait Importer<E>: InternalImporter<E> {
    /// Read from file and create scene
    #[cfg(feature = "std")]
    fn read_from_file(file_name: &str) -> Result<Box<AiScene>, E> {
        let mut scene = Box::<AiScene>::default();
        Self::import_from_file(file_name, &mut scene)?;
        Ok(scene)
    }

    /// Read from byte buffer and create scene
    fn read_from_buf(buf: &[u8]) -> Result<Box<AiScene>, E> {
        let mut scene = Box::<AiScene>::default();
        Self::import_from_buf(buf, &mut scene)?;
        Ok(scene)
    }
}

// Automatically implement Importer for all types that implement InternalImporter
impl<E, T: InternalImporter<E>> Importer<E> for T {}

/// Complete format importer trait
///
/// Combines format validation and import functionality
pub trait FormatImporter<const N: usize, E>:
    FormatValidator<N> + InternalImporter<E> + Importer<E>
{
    /// Try importing from file (including format validation)
    #[cfg(feature = "std")]
    fn try_import_from_file(file_name: &str) -> Result<Box<AiScene>, E>
    where
        E: From<ImportError>,
    {
        if Self::can_read_from_file(file_name).map_err(ImportError::from)? {
            Self::read_from_file(file_name)
        } else {
            Err(ImportError::InvalidFormat.into())
        }
    }

    /// Try importing from buffer (including format validation)
    fn try_import_from_buf(buf: &[u8]) -> Result<Box<AiScene>, E>
    where
        E: From<ImportError>,
    {
        if Self::can_read_from_buf(buf) {
            Self::read_from_buf(buf)
        } else {
            Err(ImportError::InvalidFormat.into())
        }
    }
}

// Automatically implement FormatImporter for types that meet the conditions
impl<const N: usize, E, T> FormatImporter<N, E> for T where
    T: FormatValidator<N> + InternalImporter<E> + Importer<E>
{
}
