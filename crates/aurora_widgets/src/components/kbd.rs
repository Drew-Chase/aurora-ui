use crate::widgets::{EventResponse, LayoutCtx, Widget};
use aurora_core::color::Color;
use aurora_core::geometry::corners::Corners;
use aurora_core::geometry::rect::Rect;
use aurora_core::geometry::size::Size;
use aurora_core::kmi::WidgetEvent;
use aurora_render::canvas::Canvas;
use aurora_text::font_options::FontWeight;
use aurora_text::text_layout::TextLayout;

use super::colors;

/// A keyboard shortcut indicator.
///
/// Displays a key combination in a styled inline box.
///
/// # Example
/// ```ignore
/// Kbd::new("Ctrl+S")
/// ```
pub struct Kbd {
    text: String,
    font_size: f32,
    background_color: Color,
    foreground_color: Color,
    border_color: Color,
    corners: Corners,
    layout: Option<TextLayout>,
}

impl Kbd {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            font_size: 12.0,
            background_color: colors::muted(),
            foreground_color: colors::muted_foreground(),
            border_color: colors::border(),
            corners: Corners::all(4.0),
            layout: None,
        }
    }

    pub fn font_size(mut self, size: f32) -> Self {
        self.font_size = size;
        self
    }

    pub fn background_color(mut self, color: Color) -> Self {
        self.background_color = color;
        self
    }

    pub fn foreground_color(mut self, color: Color) -> Self {
        self.foreground_color = color;
        self
    }
}

impl Widget for Kbd {
    fn layout(&mut self, _available: Size, ctx: &mut LayoutCtx) -> Size {
        let mut opts = ctx.font_options.clone();
        opts.size = Some(self.font_size);
        opts.weight = Some(FontWeight::Normal);

        let mut tl = TextLayout::new(ctx.font_manager, &self.text, &opts, self.foreground_color, None);
        tl.set_max_width(ctx.font_manager, f32::MAX);
        let ts = tl.size();
        self.layout = Some(tl);

        let pad_x = 6.0;
        let pad_y = 2.0;
        Size::new(ts.width + pad_x * 2.0, ts.height + pad_y * 2.0)
    }

    fn paint(&self, canvas: &mut Canvas, rect: Rect) {
        canvas.fill_rounded_rect(rect, self.corners, self.background_color);
        canvas.stroke_rounded_rect(rect, self.corners, 1, self.border_color);
        // Bottom shadow line
        let shadow_rect = Rect::new(rect.x1 + 1.0, rect.y2 - 2.0, rect.x2 - 1.0, rect.y2);
        canvas.fill_rect(shadow_rect, Color::new(0, 0, 0, 15));

        if let Some(ref tl) = self.layout {
            let ts = tl.size();
            let x = rect.x1 + (rect.width() - ts.width) / 2.0;
            let y = rect.y1 + (rect.height() - ts.height) / 2.0;
            canvas.draw_text(tl, x as i32, y as i32);
        }
    }

    fn children(&self) -> &[Box<dyn Widget>] {
        &[]
    }

    fn event(&mut self, _event: &WidgetEvent, _rect: Rect) -> EventResponse {
        EventResponse::default()
    }
}
