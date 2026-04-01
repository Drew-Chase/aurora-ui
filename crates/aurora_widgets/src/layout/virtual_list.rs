//! Virtualized list for rendering large datasets efficiently.
//!
//! Only items within the visible viewport (plus a configurable buffer zone)
//! are constructed, laid out, and painted each frame.
//!
//! # Example
//!
//! ```ignore
//! use aurora_ui::prelude::*;
//!
//! let scroll = ScrollState::new();
//! VirtualList::new(100_000, 36.0, |index| {
//!     Box::new(Text::new(format!("Item {index}")))
//! })
//! .height(400.0)
//! .buffer_count(5)
//! .scroll_state(scroll)
//! .on_select(|idx| println!("selected: {idx}"))
//! ```

use aurora_core::color::Color;
use aurora_core::geometry::corners::Corners;
use aurora_core::geometry::point::Point;
use aurora_core::geometry::rect::Rect;
use aurora_core::geometry::size::Size;
use aurora_core::kmi::WidgetEvent;
use aurora_core::kmi::cursor_icon::CursorIcon;
use aurora_core::kmi::mouse::{MouseEvent, MouseState};
use aurora_render::canvas::Canvas;

use crate::layout::scrollview::ScrollState;
use crate::widgets::{EventResponse, EventStatus, LayoutCtx, Widget};

const SCROLL_SPEED: f32 = 40.0;
const DEFAULT_SCROLLBAR_WIDTH: f32 = 6.0;
const DEFAULT_MIN_THUMB_HEIGHT: f32 = 20.0;
const DEFAULT_BUFFER_COUNT: usize = 5;

/// A virtualized list that only constructs and renders visible items.
///
/// Items are created on-demand via a builder closure. Only items within the
/// visible viewport plus a configurable buffer zone are instantiated each
/// frame.
pub struct VirtualList {
    // Configuration
    item_count: usize,
    item_height: f32,
    #[allow(clippy::type_complexity)]
    item_builder: Box<dyn Fn(usize) -> Box<dyn Widget>>,
    width: Option<f32>,
    height: Option<f32>,
    buffer_count: usize,
    selected: Option<usize>,
    #[allow(clippy::type_complexity)]
    on_select: Option<Box<dyn FnMut(usize)>>,
    hover_bg: Color,
    selected_bg: Color,

    // Scroll state (f64 to avoid precision loss at large item counts;
    // 2M items * 36px = 72M pixels exceeds f32's ~7-digit precision)
    scroll_offset: f64,
    max_scroll: f64,
    viewport_height: f64,
    state: Option<ScrollState>,

    // Scrollbar
    scrollbar_width: f32,
    scrollbar_track_color: Color,
    scrollbar_thumb_color: Color,
    scrollbar_thumb_corners: Option<Corners>,
    min_thumb_height: f32,

    // Internal
    scrollbar_dragging: bool,
    scrollbar_drag_anchor: f32,
    last_mouse_pos: Point,
    content_width: f32,

    // Materialized visible items (rebuilt each layout)
    visible_range: (usize, usize),
    visible_items: Vec<Box<dyn Widget>>,
    visible_rects: Vec<Rect>,
}

impl VirtualList {
    /// Creates a new virtual list.
    ///
    /// - `item_count` — total number of items in the dataset
    /// - `item_height` — fixed height in pixels for each item
    /// - `item_builder` — closure called with an item index, returns the widget
    pub fn new(
        item_count: usize,
        item_height: f32,
        item_builder: impl Fn(usize) -> Box<dyn Widget> + 'static,
    ) -> Self {
        Self {
            item_count,
            item_height,
            item_builder: Box::new(item_builder),
            width: None,
            height: None,
            buffer_count: DEFAULT_BUFFER_COUNT,
            selected: None,
            on_select: None,
            hover_bg: Color::new(0, 0, 0, 15),
            selected_bg: Color::new(0, 120, 215, 40),

            scroll_offset: 0.0,
            max_scroll: 0.0,
            viewport_height: 0.0,
            state: None,

            scrollbar_width: DEFAULT_SCROLLBAR_WIDTH,
            scrollbar_track_color: Color::TRANSPARENT,
            scrollbar_thumb_color: Color::new(0, 0, 0, 100),
            scrollbar_thumb_corners: None,
            min_thumb_height: DEFAULT_MIN_THUMB_HEIGHT,

            scrollbar_dragging: false,
            scrollbar_drag_anchor: 0.0,
            last_mouse_pos: Point::new(0.0, 0.0),
            content_width: 0.0,

            visible_range: (0, 0),
            visible_items: Vec::new(),
            visible_rects: Vec::new(),
        }
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    pub fn height(mut self, height: f32) -> Self {
        self.height = Some(height);
        self
    }

    /// Number of extra items to render above and below the viewport.
    /// Default: 5.
    pub fn buffer_count(mut self, count: usize) -> Self {
        self.buffer_count = count;
        self
    }

    pub fn selected(mut self, selected: Option<usize>) -> Self {
        self.selected = selected;
        self
    }

    pub fn on_select(mut self, cb: impl FnMut(usize) + 'static) -> Self {
        self.on_select = Some(Box::new(cb));
        self
    }

    /// Persistent scroll position that survives widget rebuilds.
    pub fn scroll_state(mut self, state: ScrollState) -> Self {
        self.scroll_offset = state.get();
        self.state = Some(state);
        self
    }

    pub fn scrollbar_width(mut self, width: f32) -> Self {
        self.scrollbar_width = width;
        self
    }

    pub fn scrollbar_thumb_color(mut self, color: Color) -> Self {
        self.scrollbar_thumb_color = color;
        self
    }

    pub fn scrollbar_track_color(mut self, color: Color) -> Self {
        self.scrollbar_track_color = color;
        self
    }

    pub fn hover_bg(mut self, color: Color) -> Self {
        self.hover_bg = color;
        self
    }

    pub fn selected_bg(mut self, color: Color) -> Self {
        self.selected_bg = color;
        self
    }

    /// Update the total item count (e.g. after filtering).
    pub fn item_count(mut self, count: usize) -> Self {
        self.item_count = count;
        self
    }

    // -----------------------------------------------------------------------
    // Virtualization internals
    // -----------------------------------------------------------------------

    fn total_content_height(&self) -> f64 {
        self.item_count as f64 * self.item_height as f64
    }

    fn has_overflow(&self) -> bool {
        self.max_scroll > 0.0
    }

    fn clamped_offset(&self) -> f64 {
        self.scroll_offset.clamp(0.0, self.max_scroll)
    }

    fn effective_scrollbar_width(&self) -> f32 {
        if self.has_overflow() {
            self.scrollbar_width
        } else {
            0.0
        }
    }

    /// Calculate the visible range of item indices including buffer zone.
    fn compute_visible_range(&self) -> (usize, usize) {
        if self.item_count == 0 || self.viewport_height <= 0.0 {
            return (0, 0);
        }
        let offset = self.clamped_offset();
        let ih = self.item_height as f64;
        let first_visible = (offset / ih).floor() as usize;
        let visible_count = (self.viewport_height / ih).ceil() as usize + 1;
        let last_visible = (first_visible + visible_count).min(self.item_count);

        let start = first_visible.saturating_sub(self.buffer_count);
        let end = (last_visible + self.buffer_count).min(self.item_count);
        (start, end)
    }

    fn thumb_geometry(&self) -> (f32, f32) {
        if !self.has_overflow() {
            return (0.0, 0.0);
        }
        let total = self.total_content_height();
        if total <= 0.0 {
            return (0.0, 0.0);
        }
        let vp = self.viewport_height as f32;
        let ratio = vp / total as f32;
        let thumb_h = (ratio * vp).max(self.min_thumb_height);
        let scrollable = vp - thumb_h;
        let thumb_y = if self.max_scroll > 0.0 {
            (self.clamped_offset() as f32 / self.max_scroll as f32) * scrollable
        } else {
            0.0
        };
        (thumb_h, thumb_y)
    }

    fn track_rect(&self, rect: &Rect) -> Rect {
        let vp = self.viewport_height as f32;
        Rect::new(
            rect.x1 + self.content_width,
            rect.y1,
            rect.x1 + self.content_width + self.effective_scrollbar_width(),
            rect.y1 + vp,
        )
    }

    fn thumb_rect(&self, rect: &Rect) -> Rect {
        let (thumb_h, thumb_y) = self.thumb_geometry();
        let track = self.track_rect(rect);
        Rect::new(
            track.x1,
            track.y1 + thumb_y,
            track.x2,
            track.y1 + thumb_y + thumb_h,
        )
    }

    /// Compute an item's screen rect using f64 math to avoid precision loss
    /// at large item counts, then convert to f32 for the final Rect.
    fn item_screen_rect(
        rect: &Rect,
        global_idx: usize,
        item_height: f32,
        content_width: f32,
        offset: f64,
    ) -> Rect {
        let y = global_idx as f64 * item_height as f64;
        let screen_y = (rect.y1 as f64 + y - offset) as f32;
        Rect::new(
            rect.x1,
            screen_y,
            rect.x1 + content_width,
            screen_y + item_height,
        )
    }
}

impl Widget for VirtualList {
    fn layout(&mut self, available: Size, ctx: &mut LayoutCtx) -> Size {
        let width = self.width.unwrap_or(available.width);
        let height = self.height.unwrap_or(available.height);
        self.viewport_height = height.max(0.0) as f64;

        // Restore scroll from persistent state
        if let Some(ref state) = self.state {
            self.scroll_offset = state.get();
        }

        let total = self.total_content_height();
        self.max_scroll = (total - self.viewport_height).max(0.0);

        let sb = self.effective_scrollbar_width();
        self.content_width = (width - sb).max(0.0);

        let (start, end) = self.compute_visible_range();
        self.visible_range = (start, end);

        // Materialize and layout only visible items
        self.visible_items.clear();
        self.visible_rects.clear();
        let item_available = Size::new(self.content_width, self.item_height);

        for i in start..end {
            let mut item = (self.item_builder)(i);
            item.layout(item_available, ctx);
            let y = i as f32 * self.item_height;
            self.visible_rects
                .push(Rect::new(0.0, y, self.content_width, y + self.item_height));
            self.visible_items.push(item);
        }

        Size::new(width, height)
    }

    fn paint(&self, canvas: &mut Canvas, rect: Rect) {
        let offset = self.clamped_offset();
        let vp_h = self.viewport_height as f32;
        let viewport = Rect::new(
            rect.x1,
            rect.y1,
            rect.x1 + self.content_width,
            rect.y1 + vp_h,
        );
        canvas.push_clip(viewport);

        for (local_idx, item) in self.visible_items.iter().enumerate() {
            let global_idx = self.visible_range.0 + local_idx;
            let screen = Self::item_screen_rect(
                &rect,
                global_idx,
                self.item_height,
                self.content_width,
                offset,
            );

            // Skip items entirely outside viewport
            if screen.y2 < viewport.y1 || screen.y1 > viewport.y2 {
                continue;
            }

            // Selection highlight
            if self.selected == Some(global_idx) {
                canvas.fill_rect(screen, self.selected_bg);
            }

            item.paint(canvas, screen);
        }

        canvas.pop_clip();

        // Scrollbar
        if self.has_overflow() && self.scrollbar_width > 0.0 {
            let track = self.track_rect(&rect);
            let thumb = self.thumb_rect(&rect);
            let corners = self
                .scrollbar_thumb_corners
                .unwrap_or_else(|| Corners::all(self.scrollbar_width / 2.0));
            canvas.fill_rect(track, self.scrollbar_track_color);
            canvas.fill_rounded_rect(thumb, corners, self.scrollbar_thumb_color);
        }
    }

    fn paint_overlay(&self, canvas: &mut Canvas, rect: Rect) {
        let offset = self.clamped_offset();
        for (local_idx, item) in self.visible_items.iter().enumerate() {
            let global_idx = self.visible_range.0 + local_idx;
            let screen = Self::item_screen_rect(
                &rect,
                global_idx,
                self.item_height,
                self.content_width,
                offset,
            );
            item.paint_overlay(canvas, screen);
        }
    }

    fn children(&self) -> &[Box<dyn Widget>] {
        &self.visible_items
    }

    fn event(&mut self, event: &WidgetEvent, rect: Rect) -> EventResponse {
        let offset = self.clamped_offset();
        let item_height = self.item_height;
        let content_width = self.content_width;
        let range_start = self.visible_range.0;
        let vp_h = self.viewport_height as f32;
        let viewport = Rect::new(
            rect.x1,
            rect.y1,
            rect.x1 + self.content_width + self.effective_scrollbar_width(),
            rect.y1 + vp_h,
        );

        let mouse = match event {
            WidgetEvent::Mouse(m) => m,
            _ => {
                // Forward non-mouse events to all visible children
                let mut result = EventResponse::default();
                for (local_idx, item) in self.visible_items.iter_mut().enumerate() {
                    let global_idx = range_start + local_idx;
                    let screen = Self::item_screen_rect(
                        &rect,
                        global_idx,
                        item_height,
                        content_width,
                        offset,
                    );
                    let resp = item.event(event, screen);
                    if resp.status.is_handled() {
                        result.status = EventStatus::Consumed;
                    }
                    if resp.request_focus.is_some() {
                        result.request_focus = resp.request_focus;
                    }
                    if resp.focus_next {
                        result.focus_next = true;
                    }
                    if resp.focus_prev {
                        result.focus_prev = true;
                    }
                }
                return result;
            }
        };

        match mouse {
            MouseEvent::MouseMoveEvent(pos) => {
                self.last_mouse_pos = *pos;

                // Scrollbar drag
                if self.scrollbar_dragging {
                    let track = self.track_rect(&rect);
                    let (thumb_h, _) = self.thumb_geometry();
                    let scrollable = vp_h - thumb_h;
                    if scrollable > 0.0 {
                        let top = pos.y - self.scrollbar_drag_anchor - track.y1;
                        let ratio = (top / scrollable) as f64;
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

                // Forward to visible children
                if viewport.contains(pos) {
                    for (local_idx, item) in self.visible_items.iter_mut().enumerate() {
                        let global_idx = range_start + local_idx;
                        let screen = Self::item_screen_rect(
                            &rect,
                            global_idx,
                            item_height,
                            content_width,
                            offset,
                        );
                        let resp = item.event(&WidgetEvent::Mouse(*mouse), screen);
                        if resp.status.stops_propagation() {
                            return resp;
                        }
                    }
                }
                EventResponse::default()
            }

            MouseEvent::MouseClickEvent(click) => {
                // Release scrollbar drag
                if click.state == MouseState::Released && self.scrollbar_dragging {
                    self.scrollbar_dragging = false;
                    return EventResponse {
                        status: EventStatus::Consumed,
                        ..Default::default()
                    };
                }

                // Start scrollbar drag or jump
                if click.state == MouseState::Pressed && self.has_overflow() {
                    let thumb = self.thumb_rect(&rect);
                    if thumb.contains(&click.position) {
                        self.scrollbar_dragging = true;
                        self.scrollbar_drag_anchor = click.position.y - thumb.y1;
                        return EventResponse {
                            status: EventStatus::Consumed,
                            ..Default::default()
                        };
                    }
                    let track = self.track_rect(&rect);
                    if track.contains(&click.position) {
                        let (thumb_h, _) = self.thumb_geometry();
                        let center = click.position.y - track.y1 - thumb_h / 2.0;
                        let scrollable = vp_h - thumb_h;
                        if scrollable > 0.0 {
                            let ratio = (center / scrollable) as f64;
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

                // Forward to visible children first
                if viewport.contains(&click.position) {
                    for (local_idx, item) in self.visible_items.iter_mut().enumerate() {
                        let global_idx = range_start + local_idx;
                        let screen = Self::item_screen_rect(
                            &rect,
                            global_idx,
                            item_height,
                            content_width,
                            offset,
                        );
                        let resp = item.event(&WidgetEvent::Mouse(*mouse), screen);
                        if resp.status.stops_propagation() {
                            return resp;
                        }
                    }

                    // If no child consumed, handle selection on release
                    if click.state == MouseState::Released {
                        let content_y = (click.position.y - rect.y1) as f64 + offset;
                        let idx = (content_y / self.item_height as f64) as usize;
                        if idx < self.item_count {
                            self.selected = Some(idx);
                            if let Some(ref mut cb) = self.on_select {
                                cb(idx);
                            }
                            return EventResponse {
                                status: EventStatus::Consumed,
                                cursor: Some(CursorIcon::Pointer),
                                ..Default::default()
                            };
                        }
                    }
                }
                EventResponse::default()
            }

            MouseEvent::MouseScrollEvent(delta) => {
                if viewport.contains(&self.last_mouse_pos) {
                    // Forward to children first (nested scrollables)
                    for (local_idx, item) in self.visible_items.iter_mut().enumerate() {
                        let global_idx = range_start + local_idx;
                        let screen = Self::item_screen_rect(
                            &rect,
                            global_idx,
                            item_height,
                            content_width,
                            offset,
                        );
                        let resp = item.event(&WidgetEvent::Mouse(*mouse), screen);
                        if resp.status.stops_propagation() {
                            return resp;
                        }
                    }

                    let old = self.scroll_offset;
                    self.scroll_offset = (self.scroll_offset
                        - (*delta as f64) * SCROLL_SPEED as f64)
                        .clamp(0.0, self.max_scroll);
                    if let Some(ref state) = self.state {
                        state.set(self.scroll_offset);
                    }
                    if (self.scroll_offset - old).abs() > 0.001 {
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

    fn event_overlay(&mut self, event: &WidgetEvent, rect: Rect) -> EventResponse {
        let offset = self.clamped_offset();
        let item_height = self.item_height;
        let content_width = self.content_width;
        let range_start = self.visible_range.0;
        for (local_idx, item) in self.visible_items.iter_mut().enumerate() {
            let global_idx = range_start + local_idx;
            let screen =
                Self::item_screen_rect(&rect, global_idx, item_height, content_width, offset);
            let resp = item.event_overlay(event, screen);
            if resp.status.stops_propagation() {
                return resp;
            }
        }
        EventResponse::default()
    }

    fn needs_animation(&self) -> bool {
        self.scrollbar_dragging || self.visible_items.iter().any(|c| c.needs_animation())
    }

    #[cfg(feature = "a11y")]
    fn access_info(&self) -> aurora_a11y::NodeInfo {
        aurora_a11y::NodeInfo::new(aurora_a11y::accesskit::Role::List)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::box_widget::BoxWidget;

    fn make_list(count: usize, item_height: f32) -> VirtualList {
        VirtualList::new(count, item_height, |_| Box::new(BoxWidget::new()))
    }

    #[test]
    fn empty_list() {
        let mut list = make_list(0, 36.0);
        list.viewport_height = 400.0;
        list.max_scroll = 0.0;
        assert_eq!(list.compute_visible_range(), (0, 0));
    }

    #[test]
    fn visible_range_at_top() {
        let mut list = make_list(1000, 36.0).buffer_count(3);
        list.viewport_height = 200.0;
        list.max_scroll = 1000.0 * 36.0 - 200.0;
        list.scroll_offset = 0.0;
        let (start, end) = list.compute_visible_range();
        assert_eq!(start, 0);
        // ceil(200/36) + 1 = 7, + buffer 3 = 10
        assert!(end <= 10);
        assert!(end >= 7);
    }

    #[test]
    fn visible_range_scrolled() {
        let mut list = make_list(1000, 36.0).buffer_count(2);
        list.viewport_height = 200.0;
        list.max_scroll = 1000.0 * 36.0 - 200.0;
        list.scroll_offset = 360.0; // item 10 at top
        let (start, end) = list.compute_visible_range();
        // first_visible = 10, buffer back 2 = 8
        assert_eq!(start, 8);
        // last_visible ~= 10 + 7 = 17, + buffer 2 = 19
        assert!(end >= 17);
        assert!(end <= 19);
    }

    #[test]
    fn visible_range_at_bottom() {
        let mut list = make_list(100, 36.0).buffer_count(5);
        list.viewport_height = 200.0;
        list.max_scroll = 100.0 * 36.0 - 200.0;
        list.scroll_offset = list.max_scroll; // scrolled to bottom
        let (_, end) = list.compute_visible_range();
        assert_eq!(end, 100); // clamped to item_count
    }

    #[test]
    fn clamped_offset_bounds() {
        let mut list = make_list(10, 36.0);
        list.max_scroll = 100.0;
        list.scroll_offset = -50.0;
        assert_eq!(list.clamped_offset(), 0.0);
        list.scroll_offset = 200.0;
        assert_eq!(list.clamped_offset(), 100.0);
        list.scroll_offset = 50.0;
        assert_eq!(list.clamped_offset(), 50.0);
    }

    #[test]
    fn no_overflow_no_scrollbar() {
        let mut list = make_list(3, 36.0);
        list.viewport_height = 200.0;
        list.max_scroll = 0.0;
        assert!(!list.has_overflow());
        assert_eq!(list.effective_scrollbar_width(), 0.0);
    }

    #[test]
    fn overflow_shows_scrollbar() {
        let mut list = make_list(100, 36.0);
        list.viewport_height = 200.0;
        list.max_scroll = 100.0 * 36.0 - 200.0;
        assert!(list.has_overflow());
        assert_eq!(list.effective_scrollbar_width(), DEFAULT_SCROLLBAR_WIDTH);
    }
}
