use thiserror::Error;

use crate::{
    assets::formats::mmd::parser::error::MMDParseError, io::importer::error::CommonImportError,
};

pub(super) const MMD_OUT_OF_MEMORY_ERROR: MMDImportError =
    MMDImportError::CommonError(CommonImportError::OutOfMemory);

#[derive(Debug, Error)]
pub enum MMDImportError {
    #[error(transparent)]
    CommonError(#[from] CommonImportError),

    #[error("mmd file parse error: {error} at {position}")]
    ParseError {
        error: MMDParseError,
        position: String,
    },
}
