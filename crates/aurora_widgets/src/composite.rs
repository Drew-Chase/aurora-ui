use crate::widgets::{EventResponse, LayoutCtx, Widget};
use aurora_core::geometry::rect::Rect;
use aurora_core::geometry::size::Size;
use aurora_core::kmi::WidgetEvent;
use aurora_render::canvas::Canvas;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

type BuildFn<S> = Box<dyn Fn(&S, StateSetter<S>) -> Box<dyn Widget>>;

/// A stateful widget that rebuilds its child tree when state changes.
///
/// Pairs a state value of type `S` with a build function that produces the
/// widget tree. When state is mutated via a [`StateSetter`], the composite
/// marks itself dirty and rebuilds its children on the next layout pass.
///
/// The state is protected by an [`Arc<Mutex<S>>`], making [`StateSetter`]
/// thread-safe (`Send + Sync`). Background threads can call
/// [`StateSetter::set`] to mutate state and the next layout pass will
/// pick up the change.
pub struct Composite<S: 'static> {
    state: Arc<Mutex<S>>,
    build_fn: BuildFn<S>,
    inner: Option<Box<dyn Widget>>,
    dirty: Arc<AtomicBool>,
}

/// A handle for mutating a [`Composite`]'s state from event callbacks or
/// background threads.
///
/// `StateSetter` is `Send + Sync` (when `S: Send`), so it can be moved
/// into `std::thread::spawn` closures or shared across threads. After
/// calling [`set`](Self::set), the composite's dirty flag is set and
/// the child tree will rebuild on the next layout pass.
///
/// Implements `Clone` so it can be shared across multiple handlers.
pub struct StateSetter<S: 'static> {
    state: Arc<Mutex<S>>,
    dirty: Arc<AtomicBool>,
}

impl<S> StateSetter<S> {
    /// Mutates the state and marks the composite as dirty for rebuild.
    ///
    /// The closure receives `&mut S`. After it returns, the composite's
    /// child tree will be rebuilt on the next layout pass.
    ///
    /// This method is safe to call from any thread. If called from a
    /// background thread, pair with a [`TaskSpawner`] to ensure a redraw
    /// is requested on the main thread.
    ///
    /// [`TaskSpawner`]: aurora_platform::app::TaskSpawner
    pub fn set(&self, f: impl FnOnce(&mut S)) {
        f(&mut self.state.lock().unwrap());
        self.dirty.store(true, Ordering::Release);
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
        let state = Arc::new(Mutex::new(state));
        let dirty = Arc::new(AtomicBool::new(true));
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
        if self.dirty.load(Ordering::Acquire) || self.inner.is_none() {
            let state = self.state.lock().unwrap();
            self.inner = Some((self.build_fn)(&state, self.setter()));
            self.dirty.store(false, Ordering::Release);
        }
        self.inner.as_mut().unwrap().layout(available, ctx)
    }
    fn paint(&self, canvas: &mut Canvas, rect: Rect) {
        if let Some(ref inner) = self.inner {
            inner.paint(canvas, rect);
        }
    }

    fn paint_overlay(&self, canvas: &mut Canvas, rect: Rect) {
        if let Some(ref inner) = self.inner {
            inner.paint_overlay(canvas, rect);
        }
    }

    fn children(&self) -> &[Box<dyn Widget>] {
        match &self.inner {
            Some(inner) => inner.children(),
            None => &[],
        }
    }

    fn event_overlay(&mut self, event: &WidgetEvent, rect: Rect) -> EventResponse {
        match &mut self.inner {
            Some(inner) => inner.event_overlay(event, rect),
            None => EventResponse::default(),
        }
    }

    fn event(&mut self, event: &WidgetEvent, rect: Rect) -> EventResponse {
        match &mut self.inner {
            Some(inner) => inner.event(event, rect),
            None => EventResponse::default(),
        }
    }

    fn needs_animation(&self) -> bool {
        self.inner.as_ref().is_some_and(|w| w.needs_animation())
    }

    #[cfg(feature = "a11y")]
    fn access_info(&self) -> aurora_a11y::NodeInfo {
        match &self.inner {
            Some(inner) => inner.access_info(),
            None => aurora_a11y::NodeInfo::transparent(),
        }
    }
}

/// Implement this trait on a config struct to define a composite widget.
///
/// Pair with `#[derive(CompositeWidget)]` to auto-generate builder setters
/// and a `new()` constructor. The derive creates a [`CompositeWrapper`] that
/// implements [`Widget`] by lazily calling your [`build`](CompositeBuilder::build)
/// method on first layout.
///
/// # Example
///
/// ```ignore
/// #[derive(Default, CompositeWidget)]
/// pub struct MyToggle {
///     pub on_color: Color,
///     pub off_color: Color,
/// }
///
/// impl CompositeBuilder for MyToggle {
///     fn build(&self) -> Box<dyn Widget> {
///         let on_color = self.on_color;
///         let off_color = self.off_color;
///         Box::new(Composite::new(false, move |&is_on, set_state| {
///             // ... compose widget tree using on_color, off_color
///         }))
///     }
/// }
///
/// // Usage: MyToggle::new().on_color(Color::RED)
/// ```
pub trait CompositeBuilder {
    /// Builds the composed widget tree from the current configuration.
    ///
    /// Called once on the first layout pass. The returned widget handles
    /// all subsequent layout, paint, and event processing.
    fn build(&self) -> Box<dyn Widget>;

    /// Returns accessibility info for this composite widget.
    ///
    /// Override this to declare a semantic role. The default returns a
    /// transparent node (children promoted to parent).
    #[cfg(feature = "a11y")]
    fn access_info(&self) -> aurora_a11y::NodeInfo {
        aurora_a11y::NodeInfo::transparent()
    }
}

/// Wraps a [`CompositeBuilder`] config and lazily builds its widget tree.
///
/// Created automatically by `#[derive(CompositeWidget)]` via the generated
/// `new()` constructor. You should not need to construct this manually.
pub struct CompositeWrapper<T> {
    /// The configuration struct. Public so the derive macro can generate
    /// setters that access it.
    pub config: T,
    inner: Option<Box<dyn Widget>>,
}

impl<T> CompositeWrapper<T> {
    /// Creates a new wrapper around the given config.
    pub fn new(config: T) -> Self {
        Self {
            config,
            inner: None,
        }
    }
}

impl<T: CompositeBuilder> Widget for CompositeWrapper<T> {
    fn layout(&mut self, available: Size, ctx: &mut LayoutCtx) -> Size {
        if self.inner.is_none() {
            self.inner = Some(self.config.build());
        }
        self.inner.as_mut().unwrap().layout(available, ctx)
    }

    fn paint(&self, canvas: &mut Canvas, rect: Rect) {
        if let Some(ref inner) = self.inner {
            inner.paint(canvas, rect);
        }
    }

    fn paint_overlay(&self, canvas: &mut Canvas, rect: Rect) {
        if let Some(ref inner) = self.inner {
            inner.paint_overlay(canvas, rect);
        }
    }

    fn children(&self) -> &[Box<dyn Widget>] {
        match &self.inner {
            Some(inner) => inner.children(),
            None => &[],
        }
    }

    fn event_overlay(&mut self, event: &WidgetEvent, rect: Rect) -> EventResponse {
        match &mut self.inner {
            Some(inner) => inner.event_overlay(event, rect),
            None => EventResponse::default(),
        }
    }

    fn event(&mut self, event: &WidgetEvent, rect: Rect) -> EventResponse {
        match &mut self.inner {
            Some(inner) => inner.event(event, rect),
            None => EventResponse::default(),
        }
    }

    fn needs_animation(&self) -> bool {
        self.inner.as_ref().is_some_and(|w| w.needs_animation())
    }

    #[cfg(feature = "a11y")]
    fn access_info(&self) -> aurora_a11y::NodeInfo {
        self.config.access_info()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::Ordering;

    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}

    #[test]
    fn state_setter_is_send_sync() {
        assert_send::<StateSetter<i32>>();
        assert_sync::<StateSetter<i32>>();
        assert_send::<StateSetter<String>>();
        assert_sync::<StateSetter<String>>();
    }

    #[test]
    fn state_setter_set_mutates_state() {
        let state = Arc::new(Mutex::new(0i32));
        let dirty = Arc::new(AtomicBool::new(false));
        let setter = StateSetter {
            state: state.clone(),
            dirty: dirty.clone(),
        };

        setter.set(|s| *s = 42);

        assert_eq!(*state.lock().unwrap(), 42);
        assert!(dirty.load(Ordering::Acquire));
    }

    #[test]
    fn state_setter_set_from_another_thread() {
        let state = Arc::new(Mutex::new(0i32));
        let dirty = Arc::new(AtomicBool::new(false));
        let setter = StateSetter {
            state: state.clone(),
            dirty: dirty.clone(),
        };

        let handle = std::thread::spawn(move || {
            setter.set(|s| *s = 99);
        });
        handle.join().unwrap();

        assert_eq!(*state.lock().unwrap(), 99);
        assert!(dirty.load(Ordering::Acquire));
    }

    #[test]
    fn concurrent_set_calls() {
        let state = Arc::new(Mutex::new(0i32));
        let dirty = Arc::new(AtomicBool::new(false));

        let handles: Vec<_> = (0..10)
            .map(|_| {
                let setter = StateSetter {
                    state: state.clone(),
                    dirty: dirty.clone(),
                };
                std::thread::spawn(move || {
                    for _ in 0..100 {
                        setter.set(|s| *s += 1);
                    }
                })
            })
            .collect();

        for h in handles {
            h.join().unwrap();
        }

        assert_eq!(*state.lock().unwrap(), 1000);
        assert!(dirty.load(Ordering::Acquire));
    }

    #[test]
    fn state_setter_clone_shares_state() {
        let state = Arc::new(Mutex::new(0i32));
        let dirty = Arc::new(AtomicBool::new(false));
        let setter1 = StateSetter {
            state: state.clone(),
            dirty: dirty.clone(),
        };
        let setter2 = setter1.clone();

        setter1.set(|s| *s += 10);
        setter2.set(|s| *s += 20);

        assert_eq!(*state.lock().unwrap(), 30);
    }
}
