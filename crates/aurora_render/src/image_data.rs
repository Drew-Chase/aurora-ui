/// Decoded raster image data in RGBA format.
///
/// Load once (from `include_bytes!` or at runtime) and pass to
/// [`Canvas::draw_image`](crate::canvas::Canvas::draw_image) each frame.
/// Decoding happens at construction time, not during paint.
///
/// # Example
///
/// ```no_run
/// use aurora_render::image_data::ImageData;
///
/// let img = ImageData::from_bytes(include_bytes!("logo.png")).unwrap();
/// assert!(img.width > 0);
/// ```
pub struct ImageData {
    /// RGBA pixel data, 4 bytes per pixel.
    pub pixels: Vec<u8>,
    /// Image width in pixels.
    pub width: u32,
    /// Image height in pixels.
    pub height: u32,
}

impl ImageData {
    /// Decodes a raster image (PNG, JPEG) from raw bytes.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, image::ImageError> {
        let img = image::load_from_memory(bytes)?;
        let rgba = img.to_rgba8();
        Ok(Self {
            width: rgba.width(),
            height: rgba.height(),
            pixels: rgba.into_raw(),
        })
    }

    /// Creates an `ImageData` from pre-decoded RGBA pixels.
    ///
    /// # Panics
    /// Panics if `pixels.len() != width * height * 4`.
    pub fn from_raw(pixels: Vec<u8>, width: u32, height: u32) -> Self {
        assert_eq!(
            pixels.len(),
            (width as usize) * (height as usize) * 4,
            "pixel buffer length must match width * height * 4"
        );
        Self {
            pixels,
            width,
            height,
        }
    }
}
