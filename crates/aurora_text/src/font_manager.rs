use crate::errors::fonts::FontError;
use cosmic_text::fontdb::{Database, Source};
use std::path::Path;
use std::sync::Arc;

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
    pub fn load_from_bytes(&mut self, bytes: &[u8]) -> Option<String> {
        let db = self.font_system.db_mut();
        let bytes = Arc::new(bytes.to_vec());
        let font_source = Source::Binary(bytes);
        let id = db.load_font_source(font_source);
        id.first()
            .and_then(|face_id| db.face(*face_id).map(|face| face.families[0].0.clone()))
    }

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
