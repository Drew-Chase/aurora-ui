use crate::errors::fonts::FontError;
use cosmic_text::fontdb::Database;
use std::path::Path;

pub struct FontManager {
    font_system: cosmic_text::FontSystem,
}

impl FontManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn load(&mut self, path: impl AsRef<Path>) -> Result<(), FontError> {
        let db = self.font_system.db_mut();
        db.load_font_file(path)?;

        Ok(())
    }
    pub fn load_from_bytes(&mut self, bytes: &[u8]) {
        let db = self.font_system.db_mut();
        db.load_font_data(bytes.to_vec());
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
