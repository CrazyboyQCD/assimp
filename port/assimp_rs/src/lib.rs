pub mod camera;
pub mod core;
pub mod errors;
pub mod formats;
pub mod material;
pub mod postprocess;
pub mod shims;
pub mod socket;
pub mod structs;
pub mod traits;
pub(crate) mod utils;

#[cfg(feature = "double_precision")]
pub type AiReal = f64;
#[cfg(not(feature = "double_precision"))]
pub type AiReal = f32;
