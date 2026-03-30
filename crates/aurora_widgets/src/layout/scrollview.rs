use aurora_core::color::Color;
use aurora_core::geometry::corners::Corners;
use aurora_core::geometry::edges::Edges;
use aurora_core::geometry::point::Point;
use aurora_core::geometry::rect::Rect;
use aurora_core::geometry::size::Size;
use aurora_core::kmi::WidgetEvent;
use aurora_core::kmi::mouse::{MouseEvent, MouseState};
use aurora_render::canvas::Canvas;

use std::cell::Cell;
use std::rc::Rc;

use crate::widgets::{EventResponse, EventStatus, LayoutCtx, Widget};

/// Shared scroll offset that persists across widget tree rebuilds.
///
/// Create a `ScrollState` once and pass it to each rebuilt [`ScrollView`] via
/// [`.state()`](ScrollView::state). The scroll offset is stored in an
/// `Rc<Cell<f32>>`, so cloning is cheap and both the old and new ScrollView
/// instances share the same value.
///
/// # Example
///
/// ```ignore
/// let scroll = ScrollState::new();
/// // In a Composite build fn:
/// ScrollView::new()
///     .state(scroll.clone())
///     .child(my_content)
/// ```
#[derive(Clone)]
pub struct ScrollState {
    offset: Rc<Cell<f32>>,
}

impl ScrollState {
    /// Creates a new scroll state starting at offset 0.
    pub fn new() -> Self {
        Self {
            offset: Rc::new(Cell::new(0.0)),
        }
    }

    /// Returns the current scroll offset.
    pub fn get(&self) -> f32 {
        self.offset.get()
    }

    /// Sets the scroll offset.
    pub fn set(&self, value: f32) {
        self.offset.set(value);
    }
}

impl Default for ScrollState {
    fn default() -> Self {
        Self::new()
    }
}

/// A scrollable container that clips its child to a viewport.
///
/// The child lays out at its natural height (given unlimited vertical space).
/// During paint the child is offset by `scroll_offset` and clipped to the
/// viewport bounds. Mouse wheel events adjust the offset.
///
/// When content overflows, a scrollbar track and thumb are drawn on the right
/// edge. The thumb can be dragged, and clicking the track jumps to that position.
///
/// # Example
///
/// ```ignore
/// use aurora_ui::prelude::*;
/// use aurora_ui::aurora_widgets::layout::scrollview::ScrollView;
///
/// ScrollView::new()
///     .height(300.0)
///     .scrollbar_thumb_color(Color::from_rgb(100, 100, 100))
///     .child(
///         col!()
///             .spacing(4.0)
///             .child(BoxWidget::new().height(100).background_color(Color::RED))
///             .child(BoxWidget::new().height(100).background_color(Color::GREEN))
///             .child(BoxWidget::new().height(100).background_color(Color::BLUE))
///             .child(BoxWidget::new().height(100).background_color(Color::BLACK))
///     )
/// ```
pub struct ScrollView {
    child: Option<Box<dyn Widget>>,
    scroll_offset: f32,
    max_scroll: f32,
    viewport_height: f32,
    child_size: Size,
    child_rect: Rect,
    width: Option<f32>,
    height: Option<f32>,
    padding: Edges,
    last_mouse_pos: Point,

    // Scrollbar configuration
    scrollbar_width: f32,
    scrollbar_track_color: Color,
    scrollbar_thumb_color: Color,
    scrollbar_thumb_corners: Option<Corners>,
    min_thumb_height: f32,

    // Internal drag state
    scrollbar_dragging: bool,
    scrollbar_drag_anchor: f32,

    // Layout-computed
    content_width: f32,

    // Shared persistent state (survives widget rebuilds)
    state: Option<ScrollState>,
}

const SCROLL_SPEED: f32 = 40.0;
const DEFAULT_SCROLLBAR_WIDTH: f32 = 6.0;
const DEFAULT_MIN_THUMB_HEIGHT: f32 = 20.0;

impl Default for ScrollView {
    fn default() -> Self {
        Self {
            child: None,
            scroll_offset: 0.0,
            max_scroll: 0.0,
            viewport_height: 0.0,
            child_size: Size::zero(),
            child_rect: Rect::zero(),
            width: None,
            height: None,
            padding: Edges::zero(),
            last_mouse_pos: Point::new(0.0, 0.0),

            scrollbar_width: DEFAULT_SCROLLBAR_WIDTH,
            scrollbar_track_color: Color::TRANSPARENT,
            scrollbar_thumb_color: Color::new(0, 0, 0, 100),
            scrollbar_thumb_corners: None,
            min_thumb_height: DEFAULT_MIN_THUMB_HEIGHT,

            scrollbar_dragging: false,
            scrollbar_drag_anchor: 0.0,

            content_width: 0.0,

            state: None,
        }
    }
}

impl ScrollView {
    /// Creates a new empty scroll view.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the scrollable child widget.
    pub fn child(mut self, child: impl Widget + 'static) -> Self {
        self.child = Some(Box::new(child));
        self
    }

    /// Sets the initial scroll offset in pixels.
    pub fn scroll_offset(mut self, scroll_offset: f32) -> Self {
        self.scroll_offset = scroll_offset;
        self
    }

    /// Sets a fixed width, overriding the available width.
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    /// Sets a fixed height for the viewport, overriding the available height.
    pub fn height(mut self, height: f32) -> Self {
        self.height = Some(height);
        self
    }

    /// Sets the inner padding around the viewport.
    pub fn padding(mut self, padding: Edges) -> Self {
        self.padding = padding;
        self
    }

    /// Sets the scrollbar width in pixels. Defaults to 6.
    ///
    /// Set to `0.0` to hide the scrollbar entirely.
    pub fn scrollbar_width(mut self, width: f32) -> Self {
        self.scrollbar_width = width;
        self
    }

    /// Sets the scrollbar track (background) color. Defaults to transparent.
    pub fn scrollbar_track_color(mut self, color: Color) -> Self {
        self.scrollbar_track_color = color;
        self
    }

    /// Sets the scrollbar thumb (handle) color. Defaults to semi-transparent black.
    pub fn scrollbar_thumb_color(mut self, color: Color) -> Self {
        self.scrollbar_thumb_color = color;
        self
    }

    /// Sets the scrollbar thumb corner radii. Defaults to fully rounded (pill shape).
    pub fn scrollbar_thumb_corners(mut self, corners: Corners) -> Self {
        self.scrollbar_thumb_corners = Some(corners);
        self
    }

    /// Sets the minimum thumb height in pixels. Defaults to 20.
    pub fn min_thumb_height(mut self, height: f32) -> Self {
        self.min_thumb_height = height;
        self
    }

    /// Attaches a shared [`ScrollState`] so the scroll offset persists across
    /// widget tree rebuilds (e.g. inside a [`Composite`](crate::composite::Composite)).
    pub fn state(mut self, state: ScrollState) -> Self {
        self.scroll_offset = state.get();
        self.state = Some(state);
        self
    }

    /// Whether content overflows the viewport (scrollbar should be visible).
    fn has_overflow(&self) -> bool {
        self.max_scroll > 0.0
    }

    /// Returns scroll_offset clamped to the current valid range.
    ///
    /// The raw `scroll_offset` is intentionally NOT clamped during layout
    /// because the Column flex algorithm may call layout multiple times with
    /// different available sizes. Clamping during an intermediate pass with a
    /// temporarily smaller `max_scroll` would permanently lose the real offset.
    fn clamped_offset(&self) -> f32 {
        self.scroll_offset.clamp(0.0, self.max_scroll)
    }

    /// Effective scrollbar width — zero when content fits.
    fn effective_scrollbar_width(&self) -> f32 {
        if self.has_overflow() {
            self.scrollbar_width
        } else {
            0.0
        }
    }

    /// Computes the thumb height and Y offset relative to the track top.
    fn thumb_geometry(&self) -> (f32, f32) {
        if !self.has_overflow() || self.child_size.height <= 0.0 {
            return (0.0, 0.0);
        }
        let ratio = self.viewport_height / self.child_size.height;
        let thumb_height = (ratio * self.viewport_height).max(self.min_thumb_height);
        let scrollable_track = self.viewport_height - thumb_height;
        let thumb_y = if self.max_scroll > 0.0 {
            (self.clamped_offset() / self.max_scroll) * scrollable_track
        } else {
            0.0
        };
        (thumb_height, thumb_y)
    }

    /// Returns the scrollbar track rect in absolute coordinates.
    fn track_rect(&self, rect: &Rect) -> Rect {
        let sb_width = self.effective_scrollbar_width();
        Rect::new(
            rect.x1 + self.padding.left + self.content_width,
            rect.y1 + self.padding.top,
            rect.x1 + self.padding.left + self.content_width + sb_width,
            rect.y1 + self.padding.top + self.viewport_height,
        )
    }

    /// Returns the scrollbar thumb rect in absolute coordinates.
    fn thumb_rect(&self, rect: &Rect) -> Rect {
        let (thumb_height, thumb_y) = self.thumb_geometry();
        let track = self.track_rect(rect);
        Rect::new(
            track.x1,
            track.y1 + thumb_y,
            track.x2,
            track.y1 + thumb_y + thumb_height,
        )
    }
}

impl Widget for ScrollView {
    fn layout(&mut self, available: Size, ctx: &mut LayoutCtx) -> Size {
        let width = self.width.unwrap_or(available.width);
        let height = self.height.unwrap_or(available.height);
        self.viewport_height = (height - self.padding.vertical()).max(0.0);

        // First pass: lay out child at full width to determine overflow
        let full_content_width = (width - self.padding.horizontal()).max(0.0);
        if let Some(child) = &mut self.child {
            let probe = Size::new(full_content_width, f32::MAX);
            self.child_size = child.layout(probe, ctx);
        }
        self.max_scroll = (self.child_size.height - self.viewport_height).max(0.0);

        // Now that we know overflow, compute the real content width
        let sb_width = self.effective_scrollbar_width();
        self.content_width = (full_content_width - sb_width).max(0.0);

        // Second pass: re-lay out child at the narrower width (only if scrollbar appeared)
        if sb_width > 0.0 {
            if let Some(child) = &mut self.child {
                let child_available = Size::new(self.content_width, f32::MAX);
                self.child_size = child.layout(child_available, ctx);
            }
            self.max_scroll = (self.child_size.height - self.viewport_height).max(0.0);
        }

        if let Some(_child) = &self.child {
            self.child_rect = Rect::new(
                self.padding.left,
                self.padding.top,
                self.padding.left + self.child_size.width,
                self.padding.top + self.child_size.height,
            );
        }

        Size::new(width, height)
    }

    fn paint(&self, canvas: &mut Canvas, rect: Rect) {
        if let Some(child) = &self.child {
            let offset = self.clamped_offset();

            // Clip to content area (excludes scrollbar)
            let viewport = Rect::new(
                rect.x1 + self.padding.left,
                rect.y1 + self.padding.top,
                rect.x1 + self.padding.left + self.content_width,
                rect.y1 + self.padding.top + self.viewport_height,
            );

            canvas.push_clip(viewport);

            let scrolled_rect = Rect::new(
                rect.x1 + self.padding.left,
                rect.y1 + self.padding.top - offset,
                rect.x1 + self.padding.left + self.child_size.width,
                rect.y1 + self.padding.top - offset + self.child_size.height,
            );

            child.paint(canvas, scrolled_rect);
            canvas.pop_clip();
        }

        // Paint scrollbar (outside the content clip)
        if self.has_overflow() && self.scrollbar_width > 0.0 {
            let track = self.track_rect(&rect);
            let thumb = self.thumb_rect(&rect);
            let thumb_corners = self
                .scrollbar_thumb_corners
                .unwrap_or_else(|| Corners::all(self.scrollbar_width / 2.0));

            canvas.fill_rect(track, self.scrollbar_track_color);
            canvas.fill_rounded_rect(thumb, thumb_corners, self.scrollbar_thumb_color);
        }
    }

    fn paint_overlay(&self, canvas: &mut Canvas, rect: Rect) {
        if let Some(child) = &self.child {
            let offset = self.clamped_offset();
            let scrolled_rect = Rect::new(
                rect.x1 + self.padding.left,
                rect.y1 + self.padding.top - offset,
                rect.x1 + self.padding.left + self.child_size.width,
                rect.y1 + self.padding.top - offset + self.child_size.height,
            );
            child.paint_overlay(canvas, scrolled_rect);
        }
    }

    fn children(&self) -> &[Box<dyn Widget>] {
        match &self.child {
            Some(child) => std::slice::from_ref(child),
            None => &[],
        }
    }

    fn event_overlay(&mut self, event: &WidgetEvent, rect: Rect) -> EventResponse {
        let offset = self.clamped_offset();
        if let Some(child) = &mut self.child {
            let scrolled_rect = Rect::new(
                rect.x1 + self.padding.left,
                rect.y1 + self.padding.top - offset,
                rect.x1 + self.padding.left + self.child_size.width,
                rect.y1 + self.padding.top - offset + self.child_size.height,
            );
            return child.event_overlay(event, scrolled_rect);
        }
        EventResponse::default()
    }

    fn event(&mut self, event: &WidgetEvent, rect: Rect) -> EventResponse {
        let offset = self.clamped_offset();
        let mouse = match event {
            WidgetEvent::Mouse(e) => e,
            _ => {
                if let Some(child) = &mut self.child {
                    let scrolled_rect = Rect::new(
                        rect.x1 + self.padding.left,
                        rect.y1 + self.padding.top - offset,
                        rect.x1 + self.padding.left + self.child_size.width,
                        rect.y1 + self.padding.top - offset + self.child_size.height,
                    );
                    return child.event(event, scrolled_rect);
                }
                return EventResponse::default();
            }
        };
        let viewport = Rect::new(
            rect.x1 + self.padding.left,
            rect.y1 + self.padding.top,
            rect.x1 + self.padding.left + self.content_width + self.effective_scrollbar_width(),
            rect.y1 + self.padding.top + self.viewport_height,
        );

        match mouse {
            MouseEvent::MouseMoveEvent(pos) => {
                self.last_mouse_pos = *pos;

                // Handle thumb drag
                if self.scrollbar_dragging {
                    let track = self.track_rect(&rect);
                    let (thumb_height, _) = self.thumb_geometry();
                    let scrollable_track = self.viewport_height - thumb_height;
                    if scrollable_track > 0.0 {
                        let new_thumb_top = pos.y - self.scrollbar_drag_anchor - track.y1;
                        let ratio = new_thumb_top / scrollable_track;
                        self.scroll_offset = (ratio * self.max_scroll).clamp(0.0, self.max_scroll);
                        if let Some(ref state) = self.state {
                            state.set(self.scroll_offset);
                        }
                    }
                    return EventResponse {
                        status: EventStatus::Consumed,
                        ..Default::default()
                    };
                }

                // Forward to child
                if let Some(child) = &mut self.child {
                    let scrolled_rect = Rect::new(
                        rect.x1 + self.padding.left,
                        rect.y1 + self.padding.top - offset,
                        rect.x1 + self.padding.left + self.child_size.width,
                        rect.y1 + self.padding.top - offset + self.child_size.height,
                    );
                    return child.event(&WidgetEvent::Mouse(*mouse), scrolled_rect);
                }
                EventResponse::default()
            }
            MouseEvent::MouseClickEvent(click) => {
                // Stop drag on any release
                if click.state == MouseState::Released && self.scrollbar_dragging {
                    self.scrollbar_dragging = false;
                    return EventResponse {
                        status: EventStatus::Consumed,
                        ..Default::default()
                    };
                }

                if click.state == MouseState::Pressed && self.has_overflow() {
                    let track = self.track_rect(&rect);
                    let thumb = self.thumb_rect(&rect);

                    if thumb.contains(&click.position) {
                        // Start dragging — anchor is distance from thumb top to click
                        self.scrollbar_dragging = true;
                        self.scrollbar_drag_anchor = click.position.y - thumb.y1;
                        return EventResponse {
                            status: EventStatus::Consumed,
                            ..Default::default()
                        };
                    } else if track.contains(&click.position) {
                        // Click on track — jump to that position
                        let (thumb_height, _) = self.thumb_geometry();
                        let click_center = click.position.y - track.y1 - thumb_height / 2.0;
                        let scrollable_track = self.viewport_height - thumb_height;
                        if scrollable_track > 0.0 {
                            let ratio = click_center / scrollable_track;
                            self.scroll_offset =
                                (ratio * self.max_scroll).clamp(0.0, self.max_scroll);
                            if let Some(ref state) = self.state {
                                state.set(self.scroll_offset);
                            }
                        }
                        return EventResponse {
                            status: EventStatus::Consumed,
                            ..Default::default()
                        };
                    }
                }

                // Forward to child — only if click is within viewport
                if viewport.contains(&click.position)
                    && let Some(child) = &mut self.child
                {
                    let scrolled_rect = Rect::new(
                        rect.x1 + self.padding.left,
                        rect.y1 + self.padding.top - offset,
                        rect.x1 + self.padding.left + self.child_size.width,
                        rect.y1 + self.padding.top - offset + self.child_size.height,
                    );
                    return child.event(&WidgetEvent::Mouse(*mouse), scrolled_rect);
                }
                EventResponse::default()
            }
            MouseEvent::MouseScrollEvent(delta) => {
                if viewport.contains(&self.last_mouse_pos) {
                    // Forward to child first so nested ScrollViews get priority
                    if let Some(child) = &mut self.child {
                        let scrolled_rect = Rect::new(
                            rect.x1 + self.padding.left,
                            rect.y1 + self.padding.top - offset,
                            rect.x1 + self.padding.left + self.child_size.width,
                            rect.y1 + self.padding.top - offset + self.child_size.height,
                        );
                        let resp = child.event(&WidgetEvent::Mouse(*mouse), scrolled_rect);
                        if resp.status.stops_propagation() {
                            return resp;
                        }
                    }
                    // Child didn't handle it — scroll ourselves
                    let old_offset = self.scroll_offset;
                    self.scroll_offset =
                        (self.scroll_offset - delta * SCROLL_SPEED).clamp(0.0, self.max_scroll);
                    if let Some(ref state) = self.state {
                        state.set(self.scroll_offset);
                    }
                    // Only consume if we actually moved
                    if (self.scroll_offset - old_offset).abs() > 0.001 {
                        return EventResponse {
                            status: EventStatus::Consumed,
                            ..Default::default()
                        };
                    }
                }
                EventResponse::default()
            }
        }
    }
    #[cfg(feature = "a11y")]
    fn access_info(&self) -> aurora_a11y::NodeInfo {
        aurora_a11y::NodeInfo::new(aurora_a11y::accesskit::Role::ScrollView)
    }
}
