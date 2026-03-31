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
    /// Creates a new font manager pre-loaded with system fonts.
    ///
    /// This can add ~200 ms to startup but enables system font discovery.
    pub fn new_with_system_db() -> Self {
        let mut manager = Self::new();
        manager.font_system.db_mut().load_system_fonts();
        manager
    }

    /// Creates a new font manager with the specified BCP 47 locale tag.
    ///
    /// The locale is used by cosmic-text for locale-aware font fallback
    /// (e.g. choosing Japanese vs Chinese variants of shared Han characters).
    pub fn new_with_locale(locale: &str) -> Self {
        Self {
            font_system: cosmic_text::FontSystem::new_with_locale_and_db(
                locale.into(),
                Database::default(),
            ),
        }
    }

    /// Creates a new font manager with a locale and pre-loaded system fonts.
    pub fn new_with_locale_and_system_db(locale: &str) -> Self {
        let mut manager = Self::new_with_locale(locale);
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
        let byte_len = bytes.len();
        let db = self.font_system.db_mut();
        let bytes = Arc::new(bytes.to_vec());
        let font_source = Source::Binary(bytes);
        let id = db.load_font_source(font_source);
        let result = id
            .first()
            .and_then(|face_id| db.face(*face_id).map(|face| face.families[0].0.clone()));
        if result.is_none() {
            log::warn!("Failed to load font from {byte_len} bytes");
        }
        result
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
