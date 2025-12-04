/// Checks if a error variant is an end of stream error
pub(crate) trait EndOfStreamError {
    fn is_eos(&self) -> bool;
}

/// Convert an end of stream error to a specific parsing part error
pub(crate) trait IntoParsingPartEndOfStreamError {
    /// Convert an end of stream error to a specific parsing part error
    fn unexpected_end_of_stream(part: &'static str) -> Self;
}

/// Convert an end of stream error to a specific parsing part sub error
pub(crate) trait MappingPartEndOfStreamError<E> {
    /// Convert an end of stream error to a specific parsing part error
    fn map_end_of_stream_error(e: E, part: &'static str) -> E;
}

impl<E: EndOfStreamError, T: IntoParsingPartEndOfStreamError + Into<E>>
    MappingPartEndOfStreamError<E> for T
{
    fn map_end_of_stream_error(e: E, part: &'static str) -> E {
        if e.is_eos() {
            Self::unexpected_end_of_stream(part).into()
        } else {
            e
        }
    }
}
