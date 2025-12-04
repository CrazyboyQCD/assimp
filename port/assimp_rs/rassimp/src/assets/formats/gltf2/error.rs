use gltf::Error as GltfError;
use thiserror::Error;
#[derive(Debug, Error)]
pub enum GltfImportError {
    #[error("GLTF importing error: {0}")]
    Gltf(#[from] GltfError),

    #[error("Failed to decode GLTF URI: {0}")]
    GltfUriUTF8DecodeError(#[from] core::str::Utf8Error),

    #[error("Unsupported GLTF URI format")]
    GltfUriUnsupportedFormat(String),
}
