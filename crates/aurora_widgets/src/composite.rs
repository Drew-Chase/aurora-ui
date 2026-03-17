use crate::widgets::{EventResponse, LayoutCtx, Widget};
use aurora_core::geometry::rect::Rect;
use aurora_core::geometry::size::Size;
use aurora_core::kmi::WidgetEvent;
use aurora_render::canvas::Canvas;

use std::rc::Rc;
use std::cell::{Cell, RefCell};

/// A stateful widget that rebuilds its child tree when state changes.
///
/// Pairs a state value of type `S` with a build function that produces the
/// widget tree. When state is mutated via a [`StateSetter`], the composite
/// marks itself dirty and rebuilds its children on the next layout pass.
pub struct Composite<S: 'static> {
	state: Rc<RefCell<S>>,
	build_fn: Box<dyn Fn(&S, StateSetter<S>) -> Box<dyn Widget>>,
	inner: Option<Box<dyn Widget>>,
	dirty: Rc<Cell<bool>>,
}

/// A handle for mutating a [`Composite`]'s state from event callbacks.
///
/// Implements `Clone` so it can be shared across multiple handlers.
pub struct StateSetter<S: 'static> {
	state: Rc<RefCell<S>>,
	dirty: Rc<Cell<bool>>,
}

impl<S> StateSetter<S> {
	/// Mutates the state and marks the composite as dirty for rebuild.
	///
	/// The closure receives `&mut S`. After it returns, the composite's
	/// child tree will be rebuilt on the next layout pass.
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
	/// Creates a new composite widget with initial state and a build function.
	///
	/// The build function is called with the current state and a [`StateSetter`]
	/// whenever the composite needs to (re)build its child tree.
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

	fn event(&mut self, event: &WidgetEvent, rect: Rect) -> EventResponse {
		match &mut self.inner {
			Some(inner) => inner.event(event, rect),
			None => EventResponse::default(),
		}
	}
}
