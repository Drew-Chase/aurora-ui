use aurora_core::geometry::rect::Rect;
use aurora_core::geometry::size::Size;
use aurora_render::canvas::Canvas;

pub trait Widget {
    fn layout(&mut self, available: Size, ctx: &mut LayoutCtx) -> Size;
    fn paint(&self, canvas: &mut Canvas, rect: Rect);
    fn children(&self) -> &[Box<dyn Widget>];
}

#[cfg(feature = "text")]
pub struct LayoutCtx<'a> {
    pub font_manager: &'a mut aurora_text::font_manager::FontManager,
}

#[cfg(not(feature = "text"))]
pub struct LayoutCtx;
