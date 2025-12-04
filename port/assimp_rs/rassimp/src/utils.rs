const VER_MAJOR: &str = env!("CARGO_PKG_VERSION_MAJOR");
const VER_MINOR: &str = env!("CARGO_PKG_VERSION_MINOR");
const VER_PATCH: &str = env!("CARGO_PKG_VERSION_PATCH");
#[allow(unused)]
const VER_BUILD: &str = env!("CARGO_PKG_VERSION");

// ------------------------------------------------------------------------------------------------
// Get Assimp patch version
pub const fn ai_get_version_patch() -> &'static str {
    VER_PATCH
}

// ------------------------------------------------------------------------------------------------
// Get Assimp minor version
pub const fn ai_get_version_minor() -> &'static str {
    VER_MINOR
}

// ------------------------------------------------------------------------------------------------
// Get Assimp major version
pub const fn ai_get_version_major() -> &'static str {
    VER_MAJOR
}
