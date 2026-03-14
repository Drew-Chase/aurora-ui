use crate::errors::fonts::FontError;
use cosmic_text::fontdb::{Database, Source};
use std::path::Path;
use std::sync::Arc;

/// Manages font loading and provides access to the underlying `cosmic_text::FontSystem`.
///
/// Fonts can be loaded from file paths or byte slices. The font system is then
/// passed to [`TextLayout`](crate::text_layout::TextLayout) for text shaping and rasterisation.
pub struct FontManager {
    font_system: cosmic_text::FontSystem,
}

impl FontManager {
    /// Creates a new font manager with an empty font database.
    pub fn new() -> Self {
        Self::default()
    }
    pub fn new_with_system_db() -> Self {
        let mut manager = Self::new();
        manager.font_system.db_mut().load_system_fonts();
        manager
    }

    /// Loads a font from a file path on disk.
    pub fn load(&mut self, path: impl AsRef<Path>) -> Result<(), FontError> {
        let db = self.font_system.db_mut();
        db.load_font_file(path)?;

        Ok(())
    }
    /// Loads a font from a byte slice and returns the font family name on success.
    ///
    /// This is the preferred method when embedding fonts with `include_bytes!`.
    pub fn load_from_bytes(&mut self, bytes: &[u8]) -> Option<String> {
        let db = self.font_system.db_mut();
        let bytes = Arc::new(bytes.to_vec());
        let font_source = Source::Binary(bytes);
        let id = db.load_font_source(font_source);
        id.first()
            .and_then(|face_id| db.face(*face_id).map(|face| face.families[0].0.clone()))
    }

    /// Returns a mutable reference to the underlying [`cosmic_text::FontSystem`].
    pub fn font_system_mut(&mut self) -> &mut cosmic_text::FontSystem {
        &mut self.font_system
    }

}

impl Default for FontManager {
    fn default() -> Self {
        Self {
            font_system: cosmic_text::FontSystem::new_with_locale_and_db(
                "".into(),
                Database::default(),
            ),
        }
    }
}
