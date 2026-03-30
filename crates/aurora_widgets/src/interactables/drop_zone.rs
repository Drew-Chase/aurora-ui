use crate::widgets::{EventResponse, EventStatus, LayoutCtx, Widget};
use aurora_core::color::Color;
use aurora_core::geometry::corners::Corners;
use aurora_core::geometry::point::Point;
use aurora_core::geometry::rect::Rect;
use aurora_core::geometry::size::Size;
use aurora_core::kmi::WidgetEvent;
use aurora_core::kmi::drag::{DragEvent, FileDropEvent};
use aurora_render::canvas::Canvas;
use std::path::PathBuf;

/// Callback invoked when a drop is accepted. Receives the drop position.
pub type OnDropCallback = Box<dyn FnMut(Point)>;
/// Callback invoked when the drag enters the zone.
pub type OnDragEnterCallback = Box<dyn FnMut()>;
/// Callback invoked when the drag leaves the zone.
pub type OnDragLeaveCallback = Box<dyn FnMut()>;
/// Callback invoked when a file is hovered over the zone.
pub type OnFileHoverCallback = Box<dyn FnMut(PathBuf, Point)>;
/// Callback invoked when a file is dropped onto the zone.
pub type OnFileDropCallback = Box<dyn FnMut(PathBuf, Point)>;

/// A wrapper widget that accepts drag-and-drop payloads.
///
/// Listens for [`DragEvent`]s and [`FileDropEvent`]s to track when a drag is
/// hovering over it. When the drop lands inside its bounds, the configured
/// callbacks fire and a visual highlight is shown.
///
/// # Usage
///
/// Pair with [`Draggable`](crate::interactables::draggable::Draggable) for
/// internal widget-to-widget drag-and-drop, or use standalone for OS file drops.
///
/// ```ignore
/// DropZone::new()
///     .child(my_target_widget)
///     .hover_color(Color::from_rgba(0, 120, 215, 60))
///     .on_drop(|pos| println!("widget dropped at {pos:?}"))
///     .on_file_drop(|path, pos| println!("file {path:?} dropped at {pos:?}"))
/// ```
pub struct DropZone {
    child: Option<Box<dyn Widget>>,
    child_rect: Rect,
    // Runtime state
    drag_hovered: bool,
    file_hovered: bool,
    // Callbacks
    on_drop: Option<OnDropCallback>,
    on_drag_enter: Option<OnDragEnterCallback>,
    on_drag_leave: Option<OnDragLeaveCallback>,
    on_file_hover: Option<OnFileHoverCallback>,
    on_file_drop: Option<OnFileDropCallback>,
    // Visual styling
    /// Background color when a drag is hovering over this zone.
    hover_color: Color,
    /// Border color when a drag is hovering over this zone.
    hover_border_color: Color,
    /// Border thickness in pixels. Only drawn when `hover_border_color` has non-zero alpha.
    hover_border_width: u32,
    /// Corner radii for the hover highlight.
    hover_corners: Corners,
}

impl Default for DropZone {
    fn default() -> Self {
        Self::new()
    }
}

impl DropZone {
    /// Creates a new drop zone with default settings.
    pub fn new() -> Self {
        Self {
            child: None,
            child_rect: Rect::default(),
            drag_hovered: false,
            file_hovered: false,
            on_drop: None,
            on_drag_enter: None,
            on_drag_leave: None,
            on_file_hover: None,
            on_file_drop: None,
            hover_color: Color::from_rgba(0, 120, 215, 40),
            hover_border_color: Color::from_rgba(0, 120, 215, 200),
            hover_border_width: 2,
            hover_corners: Corners::all(4.0),
        }
    }

    /// Sets the child widget displayed inside the drop zone.
    pub fn child(mut self, child: impl Widget + 'static) -> Self {
        self.child = Some(Box::new(child));
        self
    }

    /// Sets the background highlight color shown when a drag is over this zone.
    pub fn hover_color(mut self, color: impl Into<Color>) -> Self {
        self.hover_color = color.into();
        self
    }

    /// Sets the border color shown when a drag is over this zone.
    pub fn hover_border_color(mut self, color: impl Into<Color>) -> Self {
        self.hover_border_color = color.into();
        self
    }

    /// Sets the border thickness in pixels for the drag-hover highlight.
    pub fn hover_border_width(mut self, width: u32) -> Self {
        self.hover_border_width = width;
        self
    }

    /// Sets the corner radii for the drag-hover highlight.
    pub fn hover_corners(mut self, corners: Corners) -> Self {
        self.hover_corners = corners;
        self
    }

    /// Registers a callback invoked when a widget is dropped onto this zone.
    ///
    /// Receives the cursor position at the time of the drop.
    pub fn on_drop(mut self, f: impl FnMut(Point) + 'static) -> Self {
        self.on_drop = Some(Box::new(f));
        self
    }

    /// Registers a callback invoked when a drag enters this zone's bounds.
    pub fn on_drag_enter(mut self, f: impl FnMut() + 'static) -> Self {
        self.on_drag_enter = Some(Box::new(f));
        self
    }

    /// Registers a callback invoked when a drag leaves this zone's bounds.
    pub fn on_drag_leave(mut self, f: impl FnMut() + 'static) -> Self {
        self.on_drag_leave = Some(Box::new(f));
        self
    }

    /// Registers a callback invoked when an OS file is being hovered over this zone.
    ///
    /// Receives the file path and cursor position.
    pub fn on_file_hover(mut self, f: impl FnMut(PathBuf, Point) + 'static) -> Self {
        self.on_file_hover = Some(Box::new(f));
        self
    }

    /// Registers a callback invoked when an OS file is dropped onto this zone.
    ///
    /// Receives the file path and cursor position at the time of the drop.
    pub fn on_file_drop(mut self, f: impl FnMut(PathBuf, Point) + 'static) -> Self {
        self.on_file_drop = Some(Box::new(f));
        self
    }
}

impl Widget for DropZone {
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

        // Draw hover highlight on top of the child.
        let hovered = self.drag_hovered || self.file_hovered;
        if hovered {
            canvas.fill_rounded_rect(rect, self.hover_corners, self.hover_color);
            if self.hover_border_color.alpha > 0 && self.hover_border_width > 0 {
                canvas.stroke_rounded_rect(
                    rect,
                    self.hover_corners,
                    self.hover_border_width,
                    self.hover_border_color,
                );
            }
        }
    }

    fn paint_overlay(&self, canvas: &mut Canvas, rect: Rect) {
        if let Some(child) = &self.child {
            let translated = self.child_rect.translate(&rect.origin());
            child.paint_overlay(canvas, translated);
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
                DragEvent::DragMove { current, .. } => {
                    let now_hovered = rect.contains(current);
                    if now_hovered != self.drag_hovered {
                        self.drag_hovered = now_hovered;
                        if now_hovered {
                            if let Some(cb) = &mut self.on_drag_enter {
                                cb();
                            }
                        } else if let Some(cb) = &mut self.on_drag_leave {
                            cb();
                        }
                    }
                    EventResponse {
                        status: if now_hovered {
                            EventStatus::Consumed
                        } else {
                            EventStatus::Ignored
                        },
                        ..Default::default()
                    }
                }
                DragEvent::DragEnd { current, .. } => {
                    let was_hovered = self.drag_hovered;
                    self.drag_hovered = false;
                    if was_hovered && rect.contains(current) {
                        if let Some(cb) = &mut self.on_drop {
                            cb(*current);
                        }
                        EventResponse {
                            status: EventStatus::Consumed,
                            ..Default::default()
                        }
                    } else {
                        EventResponse::default()
                    }
                }
                DragEvent::DragStart { .. } => EventResponse::default(),
            },
            WidgetEvent::FileDrop(file_event) => match file_event {
                FileDropEvent::FileHovered { path, position } => {
                    if rect.contains(position) {
                        if !self.file_hovered {
                            self.file_hovered = true;
                        }
                        if let Some(cb) = &mut self.on_file_hover {
                            cb(path.clone(), *position);
                        }
                        EventResponse {
                            status: EventStatus::Consumed,
                            ..Default::default()
                        }
                    } else {
                        if self.file_hovered {
                            self.file_hovered = false;
                        }
                        EventResponse::default()
                    }
                }
                FileDropEvent::FileDropped { path, position } => {
                    let in_bounds = rect.contains(position);
                    self.file_hovered = false;
                    if in_bounds {
                        if let Some(cb) = &mut self.on_file_drop {
                            cb(path.clone(), *position);
                        }
                        EventResponse {
                            status: EventStatus::Consumed,
                            ..Default::default()
                        }
                    } else {
                        EventResponse::default()
                    }
                }
                FileDropEvent::FileCancelled => {
                    self.file_hovered = false;
                    EventResponse::default()
                }
            },
            _ => {
                if let Some(child) = &mut self.child {
                    let translated = self.child_rect.translate(&rect.origin());
                    return child.event(event, translated);
                }
                EventResponse::default()
            }
        }
    }
}
