use crate::widgets::{LayoutCtx, Widget};
use aurora_core::geometry::corners::Corners;
use aurora_core::geometry::edges::Edges;
use aurora_core::geometry::rect::Rect;
use aurora_core::geometry::size::Size;
use aurora_render::canvas::Canvas;

pub struct BoxWidget {
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub background_color: aurora_core::color::Color,
    pub corners: Corners,
    pub padding: Edges,
}

impl BoxWidget {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn width(mut self, width: u32) -> Self {
        self.width = Some(width);
        self
    }
    pub fn height(mut self, height: u32) -> Self {
        self.height = Some(height);
        self
    }
    pub fn background_color(mut self, color: aurora_core::color::Color) -> Self {
        self.background_color = color;
        self
    }
    pub fn corners(mut self, corners: Corners) -> Self {
        self.corners = corners;
        self
    }
    pub fn padding(mut self, padding: Edges) -> Self {
        self.padding = padding;
        self
    }
}

impl Default for BoxWidget {
    fn default() -> Self {
        Self {
            width: None,
            height: None,
            background_color: aurora_core::color::Color::TRANSPARENT,
            corners: Corners::zero(),
            padding: Edges::zero(),
        }
    }
}

impl Widget for BoxWidget {
    fn layout(&mut self, available: Size, _ctx: &mut LayoutCtx) -> Size {
        let width = match self.width {
            Some(w) => w as f32,
            None => available.width,
        };
        let height = match self.height {
            Some(h) => h as f32,
            None => available.height,
        };
        Size::new(width, height)
    }

    fn paint(&self, canvas: &mut Canvas, rect: Rect) {
        canvas.fill_rounded_rect(rect, self.corners, self.background_color)
    }

    fn children(&self) -> &[Box<dyn Widget>] {
        &[]
    }
}
