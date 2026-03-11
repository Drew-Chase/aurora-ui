use aurora_core::geometry::size::Size;
use aurora_core::geometry::rect::Rect;
use aurora_render::canvas::Canvas;

pub trait Widget {
	fn layout(&mut self, available: Size) -> Size;
	fn paint(&self, canvas: &mut Canvas, rect: Rect);
	fn children(&self) -> &[Box<dyn Widget>];
}