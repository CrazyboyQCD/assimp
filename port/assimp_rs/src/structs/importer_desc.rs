use bitflags::bitflags;
bitflags! {
    /// Mixed set of flags for [`ImporterDesc`], indicating some features
    /// common to many importers
    ///
    #[derive(Debug, Clone, Copy, Default)]
    pub struct ImporterFlags: u32 {
        /// Indicates that there is a textual encoding of the
        /// file format; and that it is supported.
        const SUPPORT_TEXT_FLAVOUR = 1 << 0;

        /// Indicates that there is a binary encoding of the
        /// file format; and that it is supported.
        const SUPPORT_BINARY_FLAVOUR = 1 << 1;

        /// Indicates that there is a compressed encoding of the
        /// file format; and that it is supported.
        const SUPPORT_COMPRESSED_FLAVOUR = 1 << 2;

        /// Indicates that the importer reads only a very particular
        /// subset of the file format.
        ///
        /// This happens commonly for
        /// declarative or procedural formats which cannot easily
        /// be mapped to [`Scene`](crate::structs::scene::Scene).
        ///
        const LIMITED_SUPPORT = 1 << 3;

        /// Indicates that the importer is highly experimental and
        /// should be used with care. This only happens for trunk
        /// (i.e. SVN) versions, experimental code is not included
        /// in releases.
        const EXPERIMENTAL = 1 << 4;
    }
}

/// Meta information about a particular importer. Importers need to fill
/// this structure, but they can freely decide how talkative they are.
///
/// A common use case for loader meta info is a user interface
/// in which the user can choose between various import/export file
/// formats.
///
/// Building such an UI by hand means a lot of maintenance
/// as importers/exporters are added to Assimp, so it might be useful
/// to have a common mechanism to query some rough importer
/// characteristics.
#[derive(Debug, Clone, Copy, Default)]
pub struct ImporterDesc {
    /// Full name of the importer (i.e. Blender3D importer)
    pub name: &'static str,

    /// Original author (left blank if unknown or whole assimp team)
    pub author: &'static str,

    /// Current maintainer, left blank if the author maintains
    pub maintainer: &'static str,

    /// Implementation comments, i.e. unimplemented features
    pub comments: &'static str,

    /// These flags indicate some characteristics common to many
    /// importers.
    pub flags: u32,

    /// Minimum format version that can be loaded im major.minor format,
    /// both are set to 0 if there is either no version scheme
    /// or if the loader doesn't care.
    pub min_major: u32,
    pub min_minor: u32,

    /// Maximum format version that can be loaded im major.minor format,
    /// both are set to 0 if there is either no version scheme
    /// or if the loader doesn't care.
    ///
    /// Loaders that expect to be forward-compatible to potential future format versions should
    /// indicate zero, otherwise they should specify the current
    /// maximum version.
    pub max_major: u32,
    pub max_minor: u32,

    /// List of file extensions this importer can handle.
    ///
    /// List entries are separated by space characters.
    ///
    /// All entries are lower case without a leading dot (i.e.
    /// **"xml dae"** would be a valid value.
    ///
    /// Note that multiple importers may respond to the same file extension -
    /// assimp calls all importers in the order in which they
    /// are registered and each importer gets the opportunity
    /// to load the file until one importer **claims** the file.
    ///
    /// Apart from file extension checks, importers typically use
    /// other methods to quickly reject files (i.e. **magic
    /// words**) so this does not mean that common or generic
    /// file extensions such as XML would be tediously slow.
    pub file_extensions: &'static str,
}
