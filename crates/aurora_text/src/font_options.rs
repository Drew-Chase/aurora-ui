/// Font configuration for text rendering.
///
/// All fields are optional — unset fields inherit from the global
/// [`App::font_options`](aurora_platform::app::App) at layout time, falling
/// back to hard-coded defaults (16 px, normal weight/style/stretch).
///
/// Works as both the global default (set on `App`) and per-widget overrides
/// (set on `Text`). Use [`resolve`](Self::resolve) to merge a per-widget
/// instance over a global base.
///
/// # Examples
///
/// ```
/// use aurora_text::font_options::{FontOptions, FontWeight, FontStyle};
///
/// let opts = FontOptions::new()
///     .family("Inter")
///     .size(20.0)
///     .weight(FontWeight::Bold)
///     .style(FontStyle::Italic);
/// ```
#[derive(Debug, Clone, Default)]
pub struct FontOptions {
    pub family: Option<String>,
    pub size: Option<f32>,
    pub weight: Option<FontWeight>,
    pub style: Option<FontStyle>,
    pub stretch: Option<FontStretch>,
    pub line_height: Option<f32>,
}

impl FontOptions {
    /// Creates a new `FontOptions` with all fields unset.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the font family by name (e.g. `"Roboto"`, `"Inter"`).
    pub fn family(mut self, family: impl Into<String>) -> Self {
        self.family = Some(family.into());
        self
    }

    /// Sets the font size in pixels.
    pub fn size(mut self, size: f32) -> Self {
        debug_assert!(size.is_finite() && size > 0.0, "font size must be finite and positive");
        self.size = Some(size);
        self
    }

    /// Sets the font weight.
    pub fn weight(mut self, weight: FontWeight) -> Self {
        self.weight = Some(weight);
        self
    }

    /// Shorthand for [`FontWeight::Bold`].
    pub fn bold(self) -> Self {
        self.weight(FontWeight::Bold)
    }

    /// Sets the font style.
    pub fn style(mut self, style: FontStyle) -> Self {
        self.style = Some(style);
        self
    }

    /// Shorthand for [`FontStyle::Italic`].
    pub fn italic(self) -> Self {
        self.style(FontStyle::Italic)
    }

    /// Sets the font stretch (width).
    pub fn stretch(mut self, stretch: FontStretch) -> Self {
        self.stretch = Some(stretch);
        self
    }

    /// Sets a custom line height in pixels. When unset, defaults to
    /// `size * 1.2`.
    pub fn line_height(mut self, line_height: f32) -> Self {
        self.line_height = Some(line_height);
        self
    }

    /// Returns the effective font size, defaulting to `16.0`.
    pub fn effective_size(&self) -> f32 {
        self.size.unwrap_or(16.0)
    }

    /// Returns the effective line height, defaulting to `size * 1.2`.
    pub fn effective_line_height(&self) -> f32 {
        self.line_height.unwrap_or(self.effective_size() * 1.2)
    }

    /// Merges this instance over `base`, using `base` values for any field
    /// that is `None` in `self`.
    pub fn resolve(&self, base: &FontOptions) -> FontOptions {
        FontOptions {
            family: self.family.clone().or_else(|| base.family.clone()),
            size: self.size.or(base.size),
            weight: self.weight.or(base.weight),
            style: self.style.or(base.style),
            stretch: self.stretch.or(base.stretch),
            line_height: self.line_height.or(base.line_height),
        }
    }

    /// Builds a `cosmic_text::Attrs` from the resolved options.
    pub fn to_cosmic_attrs(&self) -> cosmic_text::Attrs<'_> {
        let mut attrs = cosmic_text::Attrs::new();
        if let Some(ref family) = self.family {
            attrs = attrs.family(cosmic_text::Family::Name(family));
        }
        if let Some(weight) = self.weight {
            attrs = attrs.weight(weight.into());
        }
        if let Some(style) = self.style {
            attrs = attrs.style(style.into());
        }
        if let Some(stretch) = self.stretch {
            attrs = attrs.stretch(stretch.into());
        }
        attrs
    }
}

/// Font weight (boldness).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FontWeight {
    /// 100
    Thin,
    /// 200
    ExtraLight,
    /// 300
    Light,
    /// 400
    Normal,
    /// 500
    Medium,
    /// 600
    SemiBold,
    /// 700
    Bold,
    /// 800
    ExtraBold,
    /// 900
    Black,
}

impl From<FontWeight> for cosmic_text::Weight {
    fn from(w: FontWeight) -> Self {
        match w {
            FontWeight::Thin => cosmic_text::Weight(100),
            FontWeight::ExtraLight => cosmic_text::Weight(200),
            FontWeight::Light => cosmic_text::Weight(300),
            FontWeight::Normal => cosmic_text::Weight(400),
            FontWeight::Medium => cosmic_text::Weight(500),
            FontWeight::SemiBold => cosmic_text::Weight(600),
            FontWeight::Bold => cosmic_text::Weight(700),
            FontWeight::ExtraBold => cosmic_text::Weight(800),
            FontWeight::Black => cosmic_text::Weight(900),
        }
    }
}

/// Font style (posture).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FontStyle {
    /// Upright glyphs.
    Normal,
    /// True italic glyphs designed by the type designer.
    Italic,
    /// Mechanically slanted upright glyphs.
    Oblique,
}

impl From<FontStyle> for cosmic_text::Style {
    fn from(s: FontStyle) -> Self {
        match s {
            FontStyle::Normal => cosmic_text::Style::Normal,
            FontStyle::Italic => cosmic_text::Style::Italic,
            FontStyle::Oblique => cosmic_text::Style::Oblique,
        }
    }
}

/// Font stretch (width).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FontStretch {
    /// 50% width.
    UltraCondensed,
    /// 62.5% width.
    ExtraCondensed,
    /// 75% width.
    Condensed,
    /// 87.5% width.
    SemiCondensed,
    /// 100% width (default).
    Normal,
    /// 112.5% width.
    SemiExpanded,
    /// 125% width.
    Expanded,
    /// 150% width.
    ExtraExpanded,
    /// 200% width.
    UltraExpanded,
}

impl From<FontStretch> for cosmic_text::Stretch {
    fn from(s: FontStretch) -> Self {
        match s {
            FontStretch::UltraCondensed => cosmic_text::Stretch::UltraCondensed,
            FontStretch::ExtraCondensed => cosmic_text::Stretch::ExtraCondensed,
            FontStretch::Condensed => cosmic_text::Stretch::Condensed,
            FontStretch::SemiCondensed => cosmic_text::Stretch::SemiCondensed,
            FontStretch::Normal => cosmic_text::Stretch::Normal,
            FontStretch::SemiExpanded => cosmic_text::Stretch::SemiExpanded,
            FontStretch::Expanded => cosmic_text::Stretch::Expanded,
            FontStretch::ExtraExpanded => cosmic_text::Stretch::ExtraExpanded,
            FontStretch::UltraExpanded => cosmic_text::Stretch::UltraExpanded,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_all_none() {
        let opts = FontOptions::new();
        assert!(opts.family.is_none());
        assert!(opts.size.is_none());
        assert!(opts.weight.is_none());
        assert!(opts.style.is_none());
        assert!(opts.stretch.is_none());
        assert!(opts.line_height.is_none());
    }

    #[test]
    fn builder_chain() {
        let opts = FontOptions::new()
            .family("Inter")
            .size(20.0)
            .weight(FontWeight::Bold)
            .style(FontStyle::Italic)
            .stretch(FontStretch::Condensed)
            .line_height(24.0);
        assert_eq!(opts.family.as_deref(), Some("Inter"));
        assert_eq!(opts.size, Some(20.0));
        assert_eq!(opts.weight, Some(FontWeight::Bold));
        assert_eq!(opts.style, Some(FontStyle::Italic));
        assert_eq!(opts.stretch, Some(FontStretch::Condensed));
        assert_eq!(opts.line_height, Some(24.0));
    }

    #[test]
    fn bold_shorthand() {
        let opts = FontOptions::new().bold();
        assert_eq!(opts.weight, Some(FontWeight::Bold));
    }

    #[test]
    fn italic_shorthand() {
        let opts = FontOptions::new().italic();
        assert_eq!(opts.style, Some(FontStyle::Italic));
    }

    #[test]
    fn effective_size_default() {
        assert_eq!(FontOptions::new().effective_size(), 16.0);
    }

    #[test]
    fn effective_size_custom() {
        assert_eq!(FontOptions::new().size(24.0).effective_size(), 24.0);
    }

    #[test]
    fn effective_line_height_default() {
        // Default: 16.0 * 1.2 = 19.2
        let lh = FontOptions::new().effective_line_height();
        assert!((lh - 19.2).abs() < 0.01);
    }

    #[test]
    fn effective_line_height_with_custom_size() {
        let lh = FontOptions::new().size(20.0).effective_line_height();
        assert!((lh - 24.0).abs() < 0.01); // 20 * 1.2
    }

    #[test]
    fn effective_line_height_custom() {
        let lh = FontOptions::new().line_height(30.0).effective_line_height();
        assert_eq!(lh, 30.0);
    }

    #[test]
    fn resolve_inherits_from_base() {
        let base = FontOptions::new().family("Roboto").size(16.0);
        let widget = FontOptions::new().bold();
        let resolved = widget.resolve(&base);
        assert_eq!(resolved.family.as_deref(), Some("Roboto"));
        assert_eq!(resolved.size, Some(16.0));
        assert_eq!(resolved.weight, Some(FontWeight::Bold));
    }

    #[test]
    fn resolve_widget_overrides_base() {
        let base = FontOptions::new().family("Roboto").size(16.0);
        let widget = FontOptions::new().family("Inter").size(24.0);
        let resolved = widget.resolve(&base);
        assert_eq!(resolved.family.as_deref(), Some("Inter"));
        assert_eq!(resolved.size, Some(24.0));
    }

    #[test]
    fn resolve_both_none_stays_none() {
        let base = FontOptions::new();
        let widget = FontOptions::new();
        let resolved = widget.resolve(&base);
        assert!(resolved.family.is_none());
        assert!(resolved.size.is_none());
    }

    #[test]
    fn font_weight_to_cosmic() {
        assert_eq!(cosmic_text::Weight::from(FontWeight::Thin).0, 100);
        assert_eq!(cosmic_text::Weight::from(FontWeight::Normal).0, 400);
        assert_eq!(cosmic_text::Weight::from(FontWeight::Bold).0, 700);
        assert_eq!(cosmic_text::Weight::from(FontWeight::Black).0, 900);
    }

    #[test]
    fn font_style_to_cosmic() {
        assert_eq!(cosmic_text::Style::from(FontStyle::Normal), cosmic_text::Style::Normal);
        assert_eq!(cosmic_text::Style::from(FontStyle::Italic), cosmic_text::Style::Italic);
        assert_eq!(cosmic_text::Style::from(FontStyle::Oblique), cosmic_text::Style::Oblique);
    }

    #[test]
    fn font_stretch_to_cosmic() {
        assert_eq!(
            cosmic_text::Stretch::from(FontStretch::UltraCondensed),
            cosmic_text::Stretch::UltraCondensed
        );
        assert_eq!(
            cosmic_text::Stretch::from(FontStretch::Normal),
            cosmic_text::Stretch::Normal
        );
        assert_eq!(
            cosmic_text::Stretch::from(FontStretch::UltraExpanded),
            cosmic_text::Stretch::UltraExpanded
        );
    }

    #[test]
    fn to_cosmic_attrs_applies_fields() {
        let opts = FontOptions::new()
            .family("Inter")
            .weight(FontWeight::Bold)
            .style(FontStyle::Italic)
            .stretch(FontStretch::Condensed);
        let attrs = opts.to_cosmic_attrs();
        // Just verify it doesn't panic — cosmic_text::Attrs doesn't expose getters easily
        let _ = attrs;
    }
}
