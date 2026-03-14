use crate::layout::{Align, Justify};
use crate::widgets::{LayoutCtx, Widget};
use aurora_core::color::Color;
use aurora_core::geometry::edges::Edges;
use aurora_core::geometry::rect::Rect;
use aurora_core::geometry::size::Size;
use aurora_render::canvas::Canvas;
use aurora_text::cosmic_text;
use aurora_text::font_options::{FontOptions, FontStyle, FontWeight};
use aurora_text::text_layout::TextLayout;

/// A widget that displays styled text.
///
/// Uses the builder pattern for configuration. Implements [`Widget`] so it can
/// be composed with layout containers like `column!` and `row!`.
///
/// Font properties (size, family, weight, style, stretch) are stored in a
/// [`FontOptions`] instance. Unset fields inherit from the global
/// [`App::font_options`] at layout time.
#[derive(Clone)]
pub struct Text {
    pub text: String,
    pub font: FontOptions,
    pub color: Color,
    pub padding: Edges,
    pub align: Align,
    pub justify: Justify,
    pub width: Option<f32>,
    pub height: Option<f32>,
    pub text_layout: Option<TextLayout>,
    pub vertical_offset: f32,
}

impl Text {
    /// Creates a new text widget with default styling.
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            ..Self::default()
        }
    }

    /// Sets full font options, replacing any previously set font properties.
    pub fn font(mut self, font: FontOptions) -> Self {
        self.font = font;
        self
    }

    /// Sets the font size in pixels.
    pub fn font_size(mut self, size: f32) -> Self {
        self.font.size = Some(size);
        self
    }

    /// Sets the font family by name (e.g. `"Roboto"`, `"Inter"`).
    pub fn font_family(mut self, family: impl Into<String>) -> Self {
        self.font.family = Some(family.into());
        self
    }

    /// Sets the font weight.
    pub fn font_weight(mut self, weight: FontWeight) -> Self {
        self.font.weight = Some(weight);
        self
    }

    /// Shorthand for [`FontWeight::Bold`].
    pub fn bold(mut self) -> Self {
        self.font.weight = Some(FontWeight::Bold);
        self
    }

    /// Sets the font style.
    pub fn font_style(mut self, style: FontStyle) -> Self {
        self.font.style = Some(style);
        self
    }

    /// Shorthand for [`FontStyle::Italic`].
    pub fn italic(mut self) -> Self {
        self.font.style = Some(FontStyle::Italic);
        self
    }

    /// Sets a custom line height in pixels.
    pub fn line_height(mut self, line_height: f32) -> Self {
        self.font.line_height = Some(line_height);
        self
    }

    /// Sets the text color. Defaults to [`Color::BLACK`].
    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    /// Sets padding around the text.
    pub fn padding(mut self, padding: Edges) -> Self {
        self.padding = padding;
        self
    }

    /// Sets horizontal text alignment.
    pub fn align(mut self, align: Align) -> Self {
        self.align = align;
        self
    }

    /// Sets vertical text justification.
    pub fn justify(mut self, justify: Justify) -> Self {
        self.justify = justify;
        self
    }

    /// Sets an explicit width in pixels. When `None`, fills the available width.
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    /// Sets an explicit height in pixels. When `None`, fills the available height.
    pub fn height(mut self, height: f32) -> Self {
        self.height = Some(height);
        self
    }
}

impl Default for Text {
    fn default() -> Self {
        Self {
            text: String::new(),
            font: FontOptions::default(),
            color: Color::BLACK,
            padding: Edges::zero(),
            align: Align::Start,
            justify: Justify::Start,
            width: None,
            height: None,
            text_layout: None,
            vertical_offset: 0.0,
        }
    }
}

impl Widget for Text {
    fn layout(&mut self, available: Size, ctx: &mut LayoutCtx) -> Size {
        let resolved_font = self.font.resolve(ctx.font_options);
        let font_manager = &mut ctx.font_manager;
        let max_width = (available.width - self.padding.horizontal()).max(0.0);
        let align: cosmic_text::Align = match self.align {
            Align::Start => cosmic_text::Align::Left,
            Align::Center => cosmic_text::Align::Center,
            Align::End => cosmic_text::Align::Right,
            Align::Stretch => cosmic_text::Align::Left,
        };

        let mut text_layout = TextLayout::new(
            font_manager,
            &self.text,
            &resolved_font,
            self.color,
            Some(align),
        );
        text_layout.set_max_width(font_manager, max_width);

        let text_size = text_layout.size();

        let width = self.width.unwrap_or(available.width).min(available.width);
        let height = self
            .height
            .unwrap_or(available.height)
            .min(available.height);

        let available_height = height - self.padding.top - self.padding.bottom;
        let leftover = (available_height - text_size.height).max(0.0);
        self.vertical_offset = match self.justify {
            Justify::Start | Justify::SpaceBetween => 0.0,
            Justify::Center => leftover / 2.0,
            Justify::End => leftover,
        };

        self.text_layout = Some(text_layout);

        Size::new(width, height)
    }

    fn paint(&self, canvas: &mut Canvas, rect: Rect) {
        if let Some(text_layout) = &self.text_layout {
            canvas.draw_text(
                text_layout,
                (rect.x1 + self.padding.left) as i32,
                (rect.y1 + self.padding.top + self.vertical_offset) as i32,
            );
        }
    }

    fn children(&self) -> &[Box<dyn Widget>] {
        &[]
    }
}

impl From<String> for Text {
    fn from(text: String) -> Self {
        Self::new(text)
    }
}

impl From<&str> for Text {
    fn from(text: &str) -> Self {
        Self::new(text)
    }
}
