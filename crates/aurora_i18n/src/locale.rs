use aurora_core::direction::TextDirection;

/// A resolved locale with its associated text direction.
///
/// Created via [`Locale::system`] to detect the OS locale, or
/// [`Locale::new`] to specify one explicitly from a BCP 47 tag.
///
/// # Examples
///
/// ```
/// use aurora_i18n::locale::Locale;
/// use aurora_core::direction::TextDirection;
///
/// let en = Locale::new("en-US");
/// assert_eq!(en.direction(), TextDirection::Ltr);
///
/// let ar = Locale::new("ar-SA");
/// assert_eq!(ar.direction(), TextDirection::Rtl);
/// ```
#[derive(Debug, Clone)]
pub struct Locale {
    tag: String,
    direction: TextDirection,
}

impl Locale {
    /// Creates a locale from a BCP 47 tag, auto-detecting text direction
    /// from the primary language subtag.
    pub fn new(tag: impl Into<String>) -> Self {
        let tag = tag.into();
        let direction = detect_direction(&tag);
        Self { tag, direction }
    }

    /// Detects the system locale via the operating system.
    ///
    /// Falls back to `"en-US"` (LTR) if detection fails.
    pub fn system() -> Self {
        let tag = sys_locale::get_locale().unwrap_or_else(|| "en-US".to_string());
        Self::new(tag)
    }

    /// Returns the BCP 47 tag as a string slice.
    pub fn tag(&self) -> &str {
        &self.tag
    }

    /// Returns the text direction for this locale.
    pub fn direction(&self) -> TextDirection {
        self.direction
    }

    /// Overrides the text direction (useful for testing or user preference).
    pub fn with_direction(mut self, dir: TextDirection) -> Self {
        self.direction = dir;
        self
    }
}

impl Default for Locale {
    fn default() -> Self {
        Self::new("en-US")
    }
}

/// Detects text direction from a BCP 47 tag's primary language subtag.
///
/// RTL languages: Arabic, Hebrew, Persian, Urdu, Pashto, Sindhi,
/// Kurdish (Sorani), Dhivehi, Yiddish, Syriac.
fn detect_direction(tag: &str) -> TextDirection {
    let primary = tag.split(['-', '_']).next().unwrap_or("en");
    match primary {
        "ar" | "he" | "fa" | "ur" | "ps" | "sd" | "ckb" | "dv" | "yi" | "syc" => TextDirection::Rtl,
        _ => TextDirection::Ltr,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn english_is_ltr() {
        let l = Locale::new("en-US");
        assert!(l.direction().is_ltr());
        assert_eq!(l.tag(), "en-US");
    }

    #[test]
    fn arabic_is_rtl() {
        let l = Locale::new("ar-SA");
        assert!(l.direction().is_rtl());
    }

    #[test]
    fn hebrew_is_rtl() {
        let l = Locale::new("he-IL");
        assert!(l.direction().is_rtl());
    }

    #[test]
    fn persian_is_rtl() {
        let l = Locale::new("fa-IR");
        assert!(l.direction().is_rtl());
    }

    #[test]
    fn japanese_is_ltr() {
        let l = Locale::new("ja-JP");
        assert!(l.direction().is_ltr());
    }

    #[test]
    fn underscore_separator() {
        let l = Locale::new("ar_SA");
        assert!(l.direction().is_rtl());
    }

    #[test]
    fn default_is_en_us() {
        let l = Locale::default();
        assert_eq!(l.tag(), "en-US");
        assert!(l.direction().is_ltr());
    }

    #[test]
    fn with_direction_override() {
        let l = Locale::new("en-US").with_direction(TextDirection::Rtl);
        assert!(l.direction().is_rtl());
    }
}
