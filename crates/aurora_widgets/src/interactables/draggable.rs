use crate::widgets::{EventResponse, EventStatus, LayoutCtx, Widget};
use aurora_core::color::Color;
use aurora_core::geometry::corners::Corners;
use aurora_core::geometry::point::Point;
use aurora_core::geometry::rect::Rect;
use aurora_core::geometry::size::Size;
use aurora_core::kmi::WidgetEvent;
use aurora_core::kmi::cursor_icon::CursorIcon;
use aurora_core::kmi::drag::DragEvent;
use aurora_render::canvas::Canvas;

/// Callback invoked when a drag gesture starts. Receives the origin position.
pub type OnDragStartCallback = Box<dyn FnMut(Point)>;
/// Callback invoked during a drag gesture. Receives (origin, current) positions.
pub type OnDragMoveCallback = Box<dyn FnMut(Point, Point)>;
/// Callback invoked when a drag gesture ends. Receives (origin, drop) positions.
pub type OnDragEndCallback = Box<dyn FnMut(Point, Point)>;

/// A wrapper widget that makes its child draggable.
///
/// Listens for [`DragEvent`]s dispatched by the platform and tracks whether the
/// drag originated inside its bounds. When active, a ghost preview rectangle is
/// rendered in the overlay pass at the current cursor position so the user can
/// see what they're dragging.
///
/// # Usage
///
/// Pair with [`DropZone`](crate::interactables::drop_zone::DropZone) to build
/// complete drag-and-drop interactions.
///
/// ```ignore
/// Draggable::new()
///     .child(my_card_widget)
///     .on_drag_start(|origin| println!("drag started at {origin:?}"))
///     .on_drag_end(|origin, drop_pos| println!("dropped at {drop_pos:?}"))
/// ```
pub struct Draggable {
    child: Option<Box<dyn Widget>>,
    child_rect: Rect,
    // Runtime drag state
    drag_active: bool,
    drag_origin: Option<Point>,
    drag_current: Option<Point>,
    // Callbacks
    on_drag_start: Option<OnDragStartCallback>,
    on_drag_move: Option<OnDragMoveCallback>,
    on_drag_end: Option<OnDragEndCallback>,
    // Ghost appearance
    /// Color used to render the drag ghost rectangle. Defaults to a semi-transparent grey.
    ghost_color: Color,
    /// Corner radii for the ghost rectangle.
    ghost_corners: Corners,
    /// Whether to show a ghost rectangle at the cursor during drag.
    show_ghost: bool,
}

impl Default for Draggable {
    fn default() -> Self {
        Self::new()
    }
}

impl Draggable {
    /// Creates a new draggable wrapper with default settings.
    pub fn new() -> Self {
        Self {
            child: None,
            child_rect: Rect::default(),
            drag_active: false,
            drag_origin: None,
            drag_current: None,
            on_drag_start: None,
            on_drag_move: None,
            on_drag_end: None,
            ghost_color: Color::from_rgba(128, 128, 128, 160),
            ghost_corners: Corners::all(4.0),
            show_ghost: true,
        }
    }

    /// Sets the child widget that will be made draggable.
    pub fn child(mut self, child: impl Widget + 'static) -> Self {
        self.child = Some(Box::new(child));
        self
    }

    /// Registers a callback invoked once when the drag gesture starts.
    ///
    /// Receives the position where the mouse was originally pressed.
    pub fn on_drag_start(mut self, f: impl FnMut(Point) + 'static) -> Self {
        self.on_drag_start = Some(Box::new(f));
        self
    }

    /// Registers a callback invoked on every move during a drag.
    ///
    /// Receives `(origin, current)` — the original press position and the
    /// current cursor position.
    pub fn on_drag_move(mut self, f: impl FnMut(Point, Point) + 'static) -> Self {
        self.on_drag_move = Some(Box::new(f));
        self
    }

    /// Registers a callback invoked when the drag gesture ends.
    ///
    /// Receives `(origin, drop_position)` — the original press position and
    /// where the mouse was released.
    pub fn on_drag_end(mut self, f: impl FnMut(Point, Point) + 'static) -> Self {
        self.on_drag_end = Some(Box::new(f));
        self
    }

    /// Sets the ghost rectangle color shown during drag. Supports transparency.
    pub fn ghost_color(mut self, color: impl Into<Color>) -> Self {
        self.ghost_color = color.into();
        self
    }

    /// Sets the corner radii for the ghost rectangle.
    pub fn ghost_corners(mut self, corners: Corners) -> Self {
        self.ghost_corners = corners;
        self
    }

    /// Whether to render a ghost rectangle at the cursor position during drag.
    ///
    /// Defaults to `true`. Set to `false` if you want to handle visual feedback
    /// yourself via the [`on_drag_move`](Self::on_drag_move) callback.
    pub fn show_ghost(mut self, show: bool) -> Self {
        self.show_ghost = show;
        self
    }
}

impl Widget for Draggable {
    fn layout(&mut self, available: Size, ctx: &mut LayoutCtx) -> Size {
        let size = self
            .child
            .as_mut()
            .map(|c| c.layout(available, ctx))
            .unwrap_or(available);
        self.child_rect = Rect::new(0.0, 0.0, size.width, size.height);
        size
    }

    fn paint(&self, canvas: &mut Canvas, rect: Rect) {
        if let Some(child) = &self.child {
            let translated = self.child_rect.translate(&rect.origin());
            child.paint(canvas, translated);
        }
    }

    fn paint_overlay(&self, canvas: &mut Canvas, rect: Rect) {
        if let Some(child) = &self.child {
            let translated = self.child_rect.translate(&rect.origin());
            child.paint_overlay(canvas, translated);
        }

        // Draw the ghost preview when actively dragging.
        if self.show_ghost
            && self.drag_active
            && let (Some(origin), Some(current)) = (self.drag_origin, self.drag_current)
        {
            // Keep the ghost anchored to where within the widget the drag started.
            let widget_origin = rect.origin();
            let click_offset = origin - widget_origin;
            let ghost_origin = current - click_offset;
            let ghost_rect = Rect::new(
                ghost_origin.x,
                ghost_origin.y,
                ghost_origin.x + self.child_rect.width(),
                ghost_origin.y + self.child_rect.height(),
            );
            canvas.fill_rounded_rect(ghost_rect, self.ghost_corners, self.ghost_color);
        }
    }

    fn children(&self) -> &[Box<dyn Widget>] {
        match &self.child {
            Some(c) => std::slice::from_ref(c),
            None => &[],
        }
    }

    fn event_overlay(&mut self, event: &WidgetEvent, rect: Rect) -> EventResponse {
        if let Some(child) = &mut self.child {
            let translated = self.child_rect.translate(&rect.origin());
            return child.event_overlay(event, translated);
        }
        EventResponse::default()
    }

    fn event(&mut self, event: &WidgetEvent, rect: Rect) -> EventResponse {
        match event {
            WidgetEvent::Drag(drag_event) => match drag_event {
                DragEvent::DragStart { origin } => {
                    if rect.contains(origin) {
                        self.drag_active = true;
                        self.drag_origin = Some(*origin);
                        self.drag_current = Some(*origin);
                        if let Some(cb) = &mut self.on_drag_start {
                            cb(*origin);
                        }
                        EventResponse {
                            status: EventStatus::Consumed,
                            cursor: Some(CursorIcon::Grabbing),
                            ..Default::default()
                        }
                    } else {
                        EventResponse::default()
                    }
                }
                DragEvent::DragMove { origin, current } => {
                    if self.drag_active {
                        self.drag_current = Some(*current);
                        if let Some(cb) = &mut self.on_drag_move {
                            cb(*origin, *current);
                        }
                        EventResponse {
                            status: EventStatus::Consumed,
                            cursor: Some(CursorIcon::Grabbing),
                            ..Default::default()
                        }
                    } else {
                        EventResponse::default()
                    }
                }
                DragEvent::DragEnd { origin, current } => {
                    if self.drag_active {
                        self.drag_active = false;
                        self.drag_origin = None;
                        self.drag_current = None;
                        if let Some(cb) = &mut self.on_drag_end {
                            cb(*origin, *current);
                        }
                        EventResponse {
                            status: EventStatus::Consumed,
                            ..Default::default()
                        }
                    } else {
                        EventResponse::default()
                    }
                }
            },
            _ => {
                // Forward non-drag events to the child.
                if let Some(child) = &mut self.child {
                    let translated = self.child_rect.translate(&rect.origin());
                    return child.event(event, translated);
                }
                EventResponse::default()
            }
        }
    }
}
