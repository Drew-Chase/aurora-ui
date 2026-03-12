use crate::widgets::{LayoutCtx, Widget};
use aurora_core::geometry::edges::Edges;
use aurora_core::geometry::rect::Rect;
use aurora_core::geometry::size::Size;
use aurora_render::canvas::Canvas;
use aurora_text::cosmic_text::Align;
use aurora_text::text_layout::TextLayout;

#[derive(Clone)]
pub struct Text {
    pub text: String,
    pub font_size: f32,
    pub color: aurora_core::color::Color,
    pub padding: Edges,
    pub align: Align,
    pub(crate) text_layout: Option<TextLayout>,
}

impl Text {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            ..Self::default()
        }
    }
    pub fn font_size(mut self, font_size: f32) -> Self {
        self.font_size = font_size;
        self
    }
    pub fn color(mut self, color: aurora_core::color::Color) -> Self {
        self.color = color;
        self
    }
    pub fn padding(mut self, padding: Edges) -> Self {
        self.padding = padding;
        self
    }
    pub fn align(mut self, align: Align) -> Self {
        self.align = align;
        self
    }
}

impl Default for Text {
    fn default() -> Self {
        Self {
            text: String::new(),
            font_size: 16.0,
            color: aurora_core::color::Color::BLACK,
            text_layout: None,
            padding: Edges::zero(),
            align: Align::Left,
        }
    }
}

impl Widget for Text {
    fn layout(&mut self, _available: Size, ctx: &mut LayoutCtx) -> Size {
        let font_manager = &mut ctx.font_manager;
        let text_layout = TextLayout::new(
            font_manager,
            &self.text,
            self.font_size,
            self.color,
            Some(self.align),
        );
        let size = text_layout.size();
        self.text_layout = Some(text_layout);

        Size::new(
            size.width + self.padding.horizontal(),
            size.height + self.padding.vertical(),
        )
    }

    fn paint(&self, canvas: &mut Canvas, rect: Rect) {
        if let Some(text_layout) = &self.text_layout {
            canvas.draw_text(
                text_layout,
                (rect.x1 + self.padding.left) as i32,
                (rect.y1 + self.padding.top) as i32,
            );
        }
    }

    fn children(&self) -> &[Box<dyn Widget>] {
        &[]
    }
}
