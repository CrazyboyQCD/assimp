#[cfg(feature = "compression")]
pub mod compression;
pub mod fast_atof;
pub mod float_precision;
#[allow(unused)]
pub mod read;

use std::{env, ffi::OsString, fs::read_dir, io, io::ErrorKind, path::PathBuf};

pub use float_precision::AiReal;

pub(crate) fn get_project_root() -> io::Result<PathBuf> {
    let path = env::current_dir()?;
    let mut path_ancestors = path.as_path().ancestors();

    while let Some(p) = path_ancestors.next() {
        let has_cargo = read_dir(p)?
            .into_iter()
            .any(|p| p.unwrap().file_name() == OsString::from("Cargo.toml"));
        if has_cargo {
            return Ok(PathBuf::from(p));
        }
    }
    Err(io::Error::new(
        ErrorKind::NotFound,
        "Ran out of places to find Cargo.toml",
    ))
}

pub(crate) fn get_model_path(model_format: &str, model_name: &str) -> PathBuf {
    let project_root = get_project_root().unwrap();
    let mut path_ancestors = project_root.as_path().ancestors();
    path_ancestors
        .nth(2)
        .unwrap()
        .join("test")
        .join("models")
        .join(model_format)
        .join(model_name)
}
