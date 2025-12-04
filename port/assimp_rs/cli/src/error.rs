use thiserror::Error;

#[derive(Error, Debug)]
pub enum AssimpCmdError {
    #[error("Invalid number of arguments")]
    InvalidNumberOfArguments,

    #[error("Invalid combination of arguments")]
    InvalidCombinaisonOfArguments,

    #[error("Invalid argument")]
    InvalidArgument,

    #[error("File not found")]
    FileNotFound,

    #[error("Permission denied")]
    PermissionDenied,

    #[error("Invalid file format")]
    InvalidFileFormat,
}
