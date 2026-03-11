use aurora_core::geometry::corners::Corners;
use aurora_core::geometry::edges::Edges;
use aurora_core::geometry::rect::Rect;
use aurora_core::geometry::size::Size;
use aurora_render::canvas::Canvas;
use crate::widgets::Widget;

pub struct BoxWidget {
	pub width: Option<u32>,
	pub height: Option<u32>,
	pub background_color: aurora_core::color::Color,
	pub corners: Corners,
	pub padding: Edges,
	pub children: Vec<Box<dyn Widget>>,
}

impl Widget for BoxWidget {
	fn layout(&mut self, available: Size) -> Size {
		todo!()
	}

	fn paint(&self, canvas: &mut Canvas, rect: Rect) {
		todo!()
	}

	fn children(&self) -> &[Box<dyn Widget>] {
		todo!()
	}
}