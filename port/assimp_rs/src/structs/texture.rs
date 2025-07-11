const HINT_MAX_TEXTURE_LEN: usize = 9;

#[derive(Clone, Debug, Copy)]
pub struct AiTexel {
    pub b: u8,
    pub g: u8,
    pub r: u8,
    pub a: u8,
}

impl AiTexel {
    pub const fn new(b: u8, g: u8, r: u8, a: u8) -> AiTexel {
        AiTexel { b, g, r, a }
    }
}

// --------------------------------------------------------------------------------
/** Helper structure to describe an embedded texture
 *
 * Normally textures are contained in external files but some file formats embed
 * them directly in the model file. There are two types of embedded textures:
 * 1. Uncompressed textures. The color data is given in an uncompressed format.
 * 2. Compressed textures stored in a file format like png or jpg. The raw file
 * bytes are given so the application must utilize an image decoder (e.g. DevIL) to
 * get access to the actual color data.
 *
 * Embedded textures are referenced from materials using strings like "*0", "*1", etc.
 * as the texture paths (a single asterisk character followed by the
 * zero-based index of the texture in the aiScene::mTextures array).
 */
#[derive(Default, Clone, Debug)]
pub struct AiTexture {
    /** Width of the texture, in pixels
     *
     * If mHeight is zero the texture is compressed in a format
     * like JPEG. In this case mWidth specifies the size of the
     * memory area pcData is pointing to, in bytes.
     */
    pub width: u32,

    /** Height of the texture, in pixels
     *
     * If this value is zero, pcData points to an compressed texture
     * in any format (e.g. JPEG).
     */
    pub height: u32,

    /** A hint from the loader to make it easier for applications
     *  to determine the type of embedded textures.
     *
     * If mHeight != 0 this member is show how data is packed. Hint will consist of
     * two parts: channel order and channel bitness (count of the bits for every
     * color channel). For simple parsing by the viewer it's better to not omit
     * absent color channel and just use 0 for bitness. For example:
     * 1. Image contain RGBA and 8 bit per channel, achFormatHint == "rgba8888";
     * 2. Image contain ARGB and 8 bit per channel, achFormatHint == "argb8888";
     * 3. Image contain RGB and 5 bit for R and B channels and 6 bit for G channel, achFormatHint == "rgba5650";
     * 4. One color image with B channel and 1 bit for it, achFormatHint == "rgba0010";
     * If mHeight == 0 then achFormatHint is set set to '\\0\\0\\0\\0' if the loader has no additional
     * information about the texture file format used OR the
     * file extension of the format without a trailing dot. If there
     * are multiple file extensions for a format, the shortest
     * extension is chosen (JPEG maps to 'jpg', not to 'jpeg').
     * E.g. 'dds\\0', 'pcx\\0', 'jpg\\0'.  All characters are lower-case.
     * The fourth character will always be '\\0'.
     */
    pub ash_format_hint: [u8; HINT_MAX_TEXTURE_LEN], // 8 for string + 1 for terminator.

    /** Data of the texture.
     *
     * Points to an array of mWidth * mHeight aiTexel's.
     * The format of the texture data shall always be ARGB8888 if the texture-hint of the type is empty.
     * If the hint is not empty you can interpret the format by looking into this hint.
     * make the implementation for user of the library as easy
     * as possible. If mHeight = 0 this is a pointer to a memory
     * buffer of size mWidth containing the compressed texture
     * data. Good luck, have fun!
     */
    pub data: Box<[Box<[AiTexel]>]>,

    /** Texture original filename
     *
     * Used to get the texture reference
     */
    pub filename: Box<str>,
}
