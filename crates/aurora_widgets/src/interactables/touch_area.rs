use crate::widgets::{EventResponse, LayoutCtx, Widget};
use aurora_core::geometry::point::Point;
use aurora_core::geometry::rect::Rect;
use aurora_core::geometry::size::Size;
use aurora_core::kmi::WidgetEvent;
use aurora_core::kmi::cursor_icon::CursorIcon;
use aurora_core::kmi::mouse::{MouseButton, MouseClickEvent, MouseEvent, MouseState};
use aurora_render::canvas::Canvas;

/// Callback invoked on mouse click (press then release).
pub type OnClickCallback = Box<dyn FnMut(&MouseClickEvent)>;
/// Callback invoked when hover state changes. Receives the widget bounds and whether the cursor is inside.
pub type OnHoverCallback = Box<dyn FnMut(Rect, bool)>;
/// Callback invoked while dragging. Receives the current cursor position.
pub type OnDragCallback = Box<dyn FnMut(Point)>;
/// Callback invoked on mouse button down. Receives which button was pressed.
pub type MouseCallback = Box<dyn FnMut(MouseButton)>;

/// An invisible hit-testing region that dispatches mouse events to callbacks.
///
/// Wraps an optional child widget and forwards paint/layout to it. Use this
/// to add click, hover, drag, and mouse-down handling to any widget.
#[derive(Default)]
pub struct TouchArea {
    on_mouse_down: Option<MouseCallback>,
    on_click: Option<OnClickCallback>,
    on_hover: Option<OnHoverCallback>,
    on_drag: Option<OnDragCallback>,
    width: Option<f32>,
    height: Option<f32>,
    child: Option<Box<dyn Widget>>,
    child_rect: Rect,
    hovered: bool,
    dragging: bool,
    hover_cursor: Option<CursorIcon>,
}

impl TouchArea {
    /// Creates a new touch area with no callbacks and no child.
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a callback invoked when the area is clicked (released inside bounds).
    pub fn on_click(mut self, f: impl FnMut(&MouseClickEvent) + 'static) -> Self {
        self.on_click = Some(Box::new(f));
        self
    }

    /// Registers a callback invoked when hover state changes.
    pub fn on_hover(mut self, f: impl FnMut(Rect, bool) + 'static) -> Self {
        self.on_hover = Some(Box::new(f));
        self
    }

    /// Registers a callback invoked on mouse button press (before release).
    pub fn on_mouse_down(mut self, f: impl FnMut(MouseButton) + 'static) -> Self {
        self.on_mouse_down = Some(Box::new(f));
        self
    }

    /// Registers a callback invoked while the mouse is dragged after pressing inside bounds.
    pub fn on_drag(mut self, f: impl FnMut(Point) + 'static) -> Self {
        self.on_drag = Some(Box::new(f));
        self
    }

    /// Sets a fixed width for the touch area.
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    /// Sets a fixed height for the touch area.
    pub fn height(mut self, height: f32) -> Self {
        self.height = Some(height);
        self
    }

    /// Sets a child widget to render inside the touch area.
    pub fn child(mut self, child: impl Widget + 'static) -> Self {
        self.child = Some(Box::new(child));
        self
    }

    /// Sets the cursor icon shown when hovering over the touch area.
    pub fn hover_cursor(mut self, cursor: CursorIcon) -> Self {
        self.hover_cursor = Some(cursor);
        self
    }
}

impl Widget for TouchArea {
    fn layout(&mut self, available: Size, ctx: &mut LayoutCtx) -> Size {
        let child_size = self
            .child
            .as_mut()
            .map(|child| child.layout(available, ctx));
        let width = self
            .width
            .unwrap_or(child_size.unwrap_or(available).width)
            .min(available.width);
        let height = self
            .height
            .unwrap_or(child_size.unwrap_or(available).height)
            .min(available.height);

        self.child_rect = Rect::new(0f32, 0f32, width, height);

        Size::new(width, height)
    }

    fn paint(&self, canvas: &mut Canvas, rect: Rect) {
        if let Some(child) = self.child.as_ref() {
            let translated = self.child_rect.translate(&rect.origin());
            child.paint(canvas, translated);
        }
    }

    fn paint_overlay(&self, canvas: &mut Canvas, rect: Rect) {
        if let Some(child) = self.child.as_ref() {
            let translated = self.child_rect.translate(&rect.origin());
            child.paint_overlay(canvas, translated);
        }
    }

    fn children(&self) -> &[Box<dyn Widget>] {
        match &self.child {
            Some(child) => std::slice::from_ref(child),
            None => &[],
        }
    }

    fn event(&mut self, event: &WidgetEvent, rect: Rect) -> EventResponse {
        let event = match event {
            WidgetEvent::Mouse(e) => e,
            _ => return EventResponse::default(),
        };
        match event {
            MouseEvent::MouseMoveEvent(pos) => {
                self.hovered = rect.contains(pos);

                if self.dragging
                    && let Some(on_drag) = &mut self.on_drag
                {
                    on_drag(*pos)
                }

                if let Some(ref mut on_hover) = self.on_hover {
                    on_hover(rect, self.hovered);
                }
                if self.hovered {
                    EventResponse {
                        handled: true,
                        cursor: self.hover_cursor,
                        ..Default::default()
                    }
                } else {
                    EventResponse::default()
                }
            }
            MouseEvent::MouseClickEvent(click) => {
                if rect.contains(&click.position) {
                    if click.state == MouseState::Pressed {
                        self.dragging = true;
                        if let Some(mouse_down) = &mut self.on_mouse_down {
                            mouse_down(click.button);
                        }
                    } else if click.state == MouseState::Released {
                        self.dragging = false;
                        if let Some(ref mut on_click) = self.on_click {
                            on_click(click);
                        }
                        return EventResponse {
                            handled: true,
                            ..EventResponse::default()
                        };
                    }
                }
                if click.state == MouseState::Released {
                    self.dragging = false;
                }

                EventResponse {
                    handled: false,
                    ..EventResponse::default()
                }
            }
            _ => EventResponse {
                handled: false,
                ..EventResponse::default()
            },
        }
    }
}
