use thiserror::Error;

#[derive(Debug, Clone, Error)]
pub enum FastAtofError {
    #[error("Unexpected end of file")]
    UnexpectedEndOfFile,
    #[error("Cannot parse string \"{0}\" cannot be converted into a value")]
    InvalidNumericString(String),
    #[error(
        "Cannot parse string \"{0}\" as a real number: does not start with digit or decimal point followed by digit"
    )]
    InvalidRealNumber(String),

    #[error("Parse error: {0}")]
    LexicalParseFloatError(lexical_parse_float::Error),
}

impl From<lexical_parse_float::Error> for FastAtofError {
    fn from(e: lexical_parse_float::Error) -> Self {
        FastAtofError::LexicalParseFloatError(e)
    }
}
