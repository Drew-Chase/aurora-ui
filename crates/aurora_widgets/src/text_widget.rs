use crate::layout::Align;
use crate::widgets::{LayoutCtx, Widget};
use aurora_core::color::Color;
use aurora_core::geometry::edges::Edges;
use aurora_core::geometry::rect::Rect;
use aurora_core::geometry::size::Size;
use aurora_render::canvas::Canvas;
use aurora_text::cosmic_text;
use aurora_text::text_layout::TextLayout;

/// A widget that displays styled text.
///
/// Uses the builder pattern for configuration. Implements [`Widget`] so it can
/// be composed with layout containers like `column!` and `row!`.
#[derive(Clone)]
pub struct Text {
    pub text: String,
    pub font_size: f32,
    pub color: aurora_core::color::Color,
    pub padding: Edges,
    pub align: Align,
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
    /// Sets the font size in pixels. Defaults to `16.0`.
    pub fn font_size(mut self, font_size: f32) -> Self {
        self.font_size = font_size;
        self
    }
    /// Sets the text color. Defaults to [`Color::BLACK`](aurora_core::color::Color::BLACK).
    pub fn color(mut self, color: aurora_core::color::Color) -> Self {
        self.color = color;
        self
    }
    /// Sets padding around the text.
    pub fn padding(mut self, padding: Edges) -> Self {
        self.padding = padding;
        self
    }
    /// Sets horizontal text alignment. Defaults to [`Align::Left`].
    pub fn align(mut self, align: Align) -> Self {
        self.align = align;
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
            font_size: 16.0,
            color: Color::BLACK,
            padding: Edges::zero(),
            align: Align::Start,
            width: None,
            height: None,
            text_layout: None,
            vertical_offset: 0.0,
        }
    }
}

impl Widget for Text {
    fn layout(&mut self, available: Size, ctx: &mut LayoutCtx) -> Size {
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
            self.font_size,
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

        // Center text vertically
        let available_height = height - self.padding.top - self.padding.bottom;
        self.vertical_offset = ((available_height - text_size.height) / 2.0).max(0.0);

        self.text_layout = Some(text_layout);

        Size::new(width, height)
    }

    fn paint(&self, canvas: &mut Canvas, rect: Rect) {
//        canvas.fill_rect(rect, Color::RED);
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
