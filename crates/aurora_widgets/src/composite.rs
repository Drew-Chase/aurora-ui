use crate::widgets::{EventResponse, LayoutCtx, Widget};
use aurora_core::geometry::rect::Rect;
use aurora_core::geometry::size::Size;
use aurora_core::kmi::mouse::MouseEvent;
use aurora_render::canvas::Canvas;

use std::rc::Rc;
use std::cell::{Cell, RefCell};

pub struct Composite<S: 'static> {
	state: Rc<RefCell<S>>,
	build_fn: Box<dyn Fn(&S, StateSetter<S>) -> Box<dyn Widget>>,
	inner: Option<Box<dyn Widget>>,
	dirty: Rc<Cell<bool>>,
}

pub struct StateSetter<S: 'static> {
	state: Rc<RefCell<S>>,
	dirty: Rc<Cell<bool>>,
}

impl<S> StateSetter<S> {
	pub fn set(&self, f: impl FnOnce(&mut S)) {
		f(&mut self.state.borrow_mut());
		self.dirty.set(true);
	}
}

impl<S> Clone for StateSetter<S> {
	fn clone(&self) -> Self {
		Self {
			state: self.state.clone(),
			dirty: self.dirty.clone(),
		}
	}
}

impl<S: 'static> Composite<S> {
	pub fn new(
		state: S,
		build_fn: impl Fn(&S, StateSetter<S>) -> Box<dyn Widget> + 'static,
	) -> Self {
		let state = Rc::new(RefCell::new(state));
		let dirty = Rc::new(Cell::new(true));
		Self {
			state,
			build_fn: Box::new(build_fn),
			inner: None,
			dirty,
		}
	}

	fn setter(&self) -> StateSetter<S> {
		StateSetter {
			state: self.state.clone(),
			dirty: self.dirty.clone(),
		}
	}
}

impl<S: 'static> Widget for Composite<S> {
	fn layout(&mut self, available: Size, ctx: &mut LayoutCtx) -> Size {
		if self.dirty.get() || self.inner.is_none() {
			let state = self.state.borrow();
			self.inner = Some((self.build_fn)(&state, self.setter()));
			self.dirty.set(false);
		}
		self.inner.as_mut().unwrap().layout(available, ctx)
	}
	fn paint(&self, canvas: &mut Canvas, rect: Rect) {
		if let Some(ref inner) = self.inner {
			inner.paint(canvas, rect);
		}
	}

	fn children(&self) -> &[Box<dyn Widget>] {
		match &self.inner {
			Some(inner) => inner.children(),
			None => &[],
		}
	}

	fn event(&mut self, event: &MouseEvent, rect: Rect) -> EventResponse {
		match &mut self.inner {
			Some(inner) => inner.event(event, rect),
			None => EventResponse::default(),
		}
	}
}
