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
    Normal,
    Italic,
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
    UltraCondensed,
    ExtraCondensed,
    Condensed,
    SemiCondensed,
    Normal,
    SemiExpanded,
    Expanded,
    ExtraExpanded,
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
