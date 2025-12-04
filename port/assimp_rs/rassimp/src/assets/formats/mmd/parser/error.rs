use thiserror::Error;

use crate::{
    assets::formats::mmd::parser::{
        pmd::error::PmdParseError, pmx::error::PmxParseError, vmd::error::VmdParseError,
        vpd::error::VpdParseError,
    },
    io::{importer::error::CommonImportError, reader::error::EndOfStreamError},
};

pub(super) const MMD_COMMON_ERROR_OUT_OF_MEMORY: MMDParseCommonError =
    MMDParseCommonError::CommonError(CommonImportError::OutOfMemory);

pub(super) const MMD_COMMON_ERROR_UNEXPECTED_EOS: MMDParseCommonError =
    MMDParseCommonError::CommonError(CommonImportError::UnexpectedEOS);

#[derive(Debug, Error)]
pub enum MMDParseCommonError {
    #[error(transparent)]
    CommonError(#[from] CommonImportError),

    #[error("Unsupported version {0}")]
    UnsupportedVersion(f32),
}

#[derive(Debug, Error)]
pub enum MMDParseError {
    #[error(transparent)]
    Common(#[from] MMDParseCommonError),

    #[error(transparent)]
    Pmd(#[from] PmdParseError),

    #[error(transparent)]
    Pmx(#[from] PmxParseError),

    #[error(transparent)]
    Vmd(#[from] VmdParseError),

    #[error(transparent)]
    Vpd(#[from] VpdParseError),
}

impl EndOfStreamError for MMDParseError {
    fn is_eos(&self) -> bool {
        matches!(
            self,
            MMDParseError::Common(MMDParseCommonError::CommonError(
                CommonImportError::UnexpectedEOS,
            ))
        )
    }
}
