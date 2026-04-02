use crate::widgets::{EventResponse, EventStatus, LayoutCtx, Widget};
use aurora_core::color::Color;
use aurora_core::geometry::point::Point;
use aurora_core::geometry::rect::Rect;
use aurora_core::geometry::size::Size;
use aurora_core::kmi::WidgetEvent;
use aurora_core::kmi::cursor_icon::CursorIcon;
use aurora_core::kmi::mouse::{MouseEvent, MouseState};
use aurora_render::canvas::Canvas;
use std::time::Instant;

use super::colors;

/// Duration of the handle hover color animation in seconds.
const HANDLE_ANIM_DURATION: f32 = 0.12;

/// Maximum interval in milliseconds between two clicks to count as a double-click.
const DOUBLE_CLICK_MS: u128 = 400;

/// Split direction for a [`SplitPane`].
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum SplitDirection {
    /// Left | Right layout (vertical divider).
    #[default]
    Horizontal,
    /// Top / Bottom layout (horizontal divider).
    Vertical,
}

/// A resizable split view with two children separated by a draggable divider.
///
/// The divider can be dragged to resize the two panels. Double-click the handle
/// to reset to the initial ratio. The handle shows a centered dot pattern and
/// animates its color on hover.
///
/// # Example
/// ```ignore
/// SplitPane::new()
///     .direction(SplitDirection::Horizontal)
///     .ratio(0.5)
///     .first(left_panel)
///     .second(right_panel)
/// ```
pub struct SplitPane {
    first: Option<Box<dyn Widget>>,
    second: Option<Box<dyn Widget>>,
    direction: SplitDirection,
    ratio: f32,
    initial_ratio: f32,
    min_first: f32,
    min_second: f32,
    handle_width: f32,
    handle_color: Color,
    handle_hover_color: Color,
    dragging: bool,
    drag_start: f32,
    drag_start_ratio: f32,
    hovered: bool,
    first_size: Size,
    second_size: Size,
    // Handle color animation
    anim_from: f32,
    anim_to: f32,
    anim_start: Instant,
    // Double-click detection
    last_click_time: Instant,
}

impl SplitPane {
    pub fn new() -> Self {
        let now = Instant::now();
        Self {
            first: None,
            second: None,
            direction: SplitDirection::Horizontal,
            ratio: 0.5,
            initial_ratio: 0.5,
            min_first: 50.0,
            min_second: 50.0,
            handle_width: 6.0,
            handle_color: colors::border(),
            handle_hover_color: colors::primary(),
            dragging: false,
            drag_start: 0.0,
            drag_start_ratio: 0.0,
            hovered: false,
            first_size: Size::default(),
            second_size: Size::default(),
            anim_from: 0.0,
            anim_to: 0.0,
            anim_start: now,
            last_click_time: now,
        }
    }

    /// Sets the first (left or top) child widget.
    pub fn first(mut self, widget: impl Widget + 'static) -> Self {
        self.first = Some(Box::new(widget));
        self
    }

    /// Sets the second (right or bottom) child widget.
    pub fn second(mut self, widget: impl Widget + 'static) -> Self {
        self.second = Some(Box::new(widget));
        self
    }

    /// Sets the split direction. Default: [`SplitDirection::Horizontal`].
    pub fn direction(mut self, direction: SplitDirection) -> Self {
        self.direction = direction;
        self
    }

    /// Sets the initial split ratio (0.0..1.0). Default: 0.5.
    pub fn ratio(mut self, ratio: f32) -> Self {
        let r = ratio.clamp(0.0, 1.0);
        self.ratio = r;
        self.initial_ratio = r;
        self
    }

    /// Sets the minimum size in pixels for the first panel. Default: 50.0.
    pub fn min_first(mut self, min: f32) -> Self {
        self.min_first = min;
        self
    }

    /// Sets the minimum size in pixels for the second panel. Default: 50.0.
    pub fn min_second(mut self, min: f32) -> Self {
        self.min_second = min;
        self
    }

    /// Sets the drag handle width in pixels. Default: 6.0.
    pub fn handle_width(mut self, width: f32) -> Self {
        self.handle_width = width;
        self
    }

    /// Sets the default handle color. Default: `colors::border()`.
    pub fn handle_color(mut self, color: Color) -> Self {
        self.handle_color = color;
        self
    }

    /// Sets the hover/active handle color. Default: `colors::primary()`.
    pub fn handle_hover_color(mut self, color: Color) -> Self {
        self.handle_hover_color = color;
        self
    }

    /// Returns the total available space along the split axis (excluding handle).
    fn available_axis(&self, available: Size) -> f32 {
        let total = match self.direction {
            SplitDirection::Horizontal => available.width,
            SplitDirection::Vertical => available.height,
        };
        (total - self.handle_width).max(0.0)
    }

    /// Clamps the ratio so that both panels meet their minimum size constraints.
    fn clamp_ratio(&self, ratio: f32, axis_space: f32) -> f32 {
        if axis_space <= 0.0 {
            return ratio.clamp(0.0, 1.0);
        }
        let min_first_ratio = self.min_first / axis_space;
        let max_first_ratio = 1.0 - (self.min_second / axis_space);
        ratio.clamp(min_first_ratio.min(1.0), max_first_ratio.max(0.0))
    }

    /// Returns the rectangle for the drag handle given the overall bounds.
    fn handle_rect(&self, rect: &Rect) -> Rect {
        let axis = self.available_axis(Size::new(rect.width(), rect.height()));
        let first_size = axis * self.ratio;
        match self.direction {
            SplitDirection::Horizontal => Rect::new(
                rect.x1 + first_size,
                rect.y1,
                rect.x1 + first_size + self.handle_width,
                rect.y2,
            ),
            SplitDirection::Vertical => Rect::new(
                rect.x1,
                rect.y1 + first_size,
                rect.x2,
                rect.y1 + first_size + self.handle_width,
            ),
        }
    }

    /// Returns the rectangle for the first child.
    fn first_rect(&self, rect: &Rect) -> Rect {
        let axis = self.available_axis(Size::new(rect.width(), rect.height()));
        let first_size = axis * self.ratio;
        match self.direction {
            SplitDirection::Horizontal => {
                Rect::new(rect.x1, rect.y1, rect.x1 + first_size, rect.y2)
            }
            SplitDirection::Vertical => Rect::new(rect.x1, rect.y1, rect.x2, rect.y1 + first_size),
        }
    }

    /// Returns the rectangle for the second child.
    fn second_rect(&self, rect: &Rect) -> Rect {
        let axis = self.available_axis(Size::new(rect.width(), rect.height()));
        let first_size = axis * self.ratio;
        match self.direction {
            SplitDirection::Horizontal => Rect::new(
                rect.x1 + first_size + self.handle_width,
                rect.y1,
                rect.x2,
                rect.y2,
            ),
            SplitDirection::Vertical => Rect::new(
                rect.x1,
                rect.y1 + first_size + self.handle_width,
                rect.x2,
                rect.y2,
            ),
        }
    }

    /// Returns the current animation t value (0.0 = normal, 1.0 = hovered/active).
    fn current_anim_t(&self) -> f32 {
        let elapsed = self.anim_start.elapsed().as_secs_f32();
        let t = (elapsed / HANDLE_ANIM_DURATION).min(1.0);
        // Ease-out cubic
        let eased = 1.0 - (1.0 - t).powi(3);
        self.anim_from + (self.anim_to - self.anim_from) * eased
    }

    /// Starts an animation towards the given target (0.0 or 1.0).
    fn animate_to(&mut self, target: f32) {
        self.anim_from = self.current_anim_t();
        self.anim_to = target;
        self.anim_start = Instant::now();
    }

    /// Returns the cursor icon appropriate for the current split direction.
    fn resize_cursor(&self) -> CursorIcon {
        match self.direction {
            SplitDirection::Horizontal => CursorIcon::Grab,
            SplitDirection::Vertical => CursorIcon::Grab,
        }
    }

    /// Paints the centered dot pattern on the handle.
    fn paint_handle_dots(&self, canvas: &mut Canvas, hr: &Rect, color: Color) {
        let dot_radius = 1.5;
        let dot_size = dot_radius * 2.0;
        let cx = hr.x1 + hr.width() / 2.0;
        let cy = hr.y1 + hr.height() / 2.0;

        let dot_color = color.opacity(0.7);

        match self.direction {
            SplitDirection::Horizontal => {
                // Three dots stacked vertically
                let spacing = 6.0;
                for i in -1..=1 {
                    let y = cy + i as f32 * spacing - dot_radius;
                    let x = cx - dot_radius;
                    canvas.circle(Point::new(x, y), dot_size, dot_radius, dot_color);
                }
            }
            SplitDirection::Vertical => {
                // Three dots side by side horizontally
                let spacing = 6.0;
                for i in -1..=1 {
                    let x = cx + i as f32 * spacing - dot_radius;
                    let y = cy - dot_radius;
                    canvas.circle(Point::new(x, y), dot_size, dot_radius, dot_color);
                }
            }
        }
    }
}

impl Default for SplitPane {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for SplitPane {
    fn layout(&mut self, available: Size, ctx: &mut LayoutCtx) -> Size {
        let axis = self.available_axis(available);
        self.ratio = self.clamp_ratio(self.ratio, axis);

        let first_axis = axis * self.ratio;
        let second_axis = axis - first_axis;

        // Layout first child
        if let Some(ref mut child) = self.first {
            let avail = match self.direction {
                SplitDirection::Horizontal => Size::new(first_axis, available.height),
                SplitDirection::Vertical => Size::new(available.width, first_axis),
            };
            self.first_size = child.layout(avail, ctx);
        }

        // Layout second child
        if let Some(ref mut child) = self.second {
            let avail = match self.direction {
                SplitDirection::Horizontal => Size::new(second_axis, available.height),
                SplitDirection::Vertical => Size::new(available.width, second_axis),
            };
            self.second_size = child.layout(avail, ctx);
        }

        available
    }

    fn paint(&self, canvas: &mut Canvas, rect: Rect) {
        let fr = self.first_rect(&rect);
        let hr = self.handle_rect(&rect);
        let sr = self.second_rect(&rect);

        // Paint first child
        if let Some(ref child) = self.first {
            canvas.push_clip(fr);
            let child_rect = Rect::new(
                fr.x1,
                fr.y1,
                fr.x1 + self.first_size.width,
                fr.y1 + self.first_size.height,
            );
            child.paint(canvas, child_rect);
            canvas.pop_clip();
        }

        // Paint handle with animated color
        let t = self.current_anim_t();
        let handle_color = self.handle_color.lerp(&self.handle_hover_color, t);
        canvas.fill_rect(hr, handle_color);
        self.paint_handle_dots(canvas, &hr, handle_color);

        // Paint second child
        if let Some(ref child) = self.second {
            canvas.push_clip(sr);
            let child_rect = Rect::new(
                sr.x1,
                sr.y1,
                sr.x1 + self.second_size.width,
                sr.y1 + self.second_size.height,
            );
            child.paint(canvas, child_rect);
            canvas.pop_clip();
        }
    }

    fn children(&self) -> &[Box<dyn Widget>] {
        // SplitPane owns two optional children stored separately, not in a slice.
        &[]
    }

    fn event(&mut self, event: &WidgetEvent, rect: Rect) -> EventResponse {
        let hr = self.handle_rect(&rect);
        let resize_cursor = self.resize_cursor();

        match event {
            WidgetEvent::Mouse(MouseEvent::MouseClickEvent(e)) => {
                if e.state == MouseState::Pressed && hr.contains(&e.position) {
                    // Double-click detection: reset to initial ratio
                    let now = Instant::now();
                    let elapsed = now.duration_since(self.last_click_time).as_millis();
                    self.last_click_time = now;

                    if elapsed < DOUBLE_CLICK_MS {
                        // Double-click: reset ratio
                        let axis = self.available_axis(Size::new(rect.width(), rect.height()));
                        self.ratio = self.clamp_ratio(self.initial_ratio, axis);
                        return EventResponse {
                            status: EventStatus::Consumed,
                            cursor: Some(resize_cursor),
                            ..Default::default()
                        };
                    }

                    // Start drag
                    self.dragging = true;
                    self.drag_start = match self.direction {
                        SplitDirection::Horizontal => e.position.x,
                        SplitDirection::Vertical => e.position.y,
                    };
                    self.drag_start_ratio = self.ratio;
                    return EventResponse {
                        status: EventStatus::Consumed,
                        cursor: Some(resize_cursor),
                        ..Default::default()
                    };
                }
                if e.state == MouseState::Released && self.dragging {
                    self.dragging = false;
                    return EventResponse {
                        status: EventStatus::Consumed,
                        ..Default::default()
                    };
                }
                // Forward to children
                let fr = self.first_rect(&rect);
                let sr = self.second_rect(&rect);
                if let Some(ref mut child) = self.first
                    && fr.contains(&e.position)
                {
                    let resp = child.event(event, fr);
                    if resp.status.is_handled() {
                        return resp;
                    }
                }
                if let Some(ref mut child) = self.second
                    && sr.contains(&e.position)
                {
                    return child.event(event, sr);
                }
                EventResponse::default()
            }
            WidgetEvent::Mouse(MouseEvent::MouseMoveEvent(pos)) => {
                if self.dragging {
                    let axis = self.available_axis(Size::new(rect.width(), rect.height()));
                    if axis > 0.0 {
                        let current = match self.direction {
                            SplitDirection::Horizontal => pos.x,
                            SplitDirection::Vertical => pos.y,
                        };
                        let delta = current - self.drag_start;
                        let delta_ratio = delta / axis;
                        self.ratio = self.clamp_ratio(self.drag_start_ratio + delta_ratio, axis);
                    }
                    return EventResponse {
                        status: EventStatus::Consumed,
                        cursor: Some(resize_cursor),
                        ..Default::default()
                    };
                }

                let was_hovered = self.hovered;
                self.hovered = hr.contains(pos);

                if self.hovered && !was_hovered {
                    self.animate_to(1.0);
                } else if !self.hovered && was_hovered {
                    self.animate_to(0.0);
                }

                if self.hovered {
                    return EventResponse {
                        cursor: Some(resize_cursor),
                        ..Default::default()
                    };
                }

                // Forward to children
                let fr = self.first_rect(&rect);
                let sr = self.second_rect(&rect);
                if let Some(ref mut child) = self.first
                    && fr.contains(pos)
                {
                    return child.event(event, fr);
                }
                if let Some(ref mut child) = self.second
                    && sr.contains(pos)
                {
                    return child.event(event, sr);
                }
                EventResponse::default()
            }
            _ => {
                // Forward to both children
                let fr = self.first_rect(&rect);
                let sr = self.second_rect(&rect);
                if let Some(ref mut child) = self.first {
                    let resp = child.event(event, fr);
                    if resp.status.is_handled() {
                        return resp;
                    }
                }
                if let Some(ref mut child) = self.second {
                    return child.event(event, sr);
                }
                EventResponse::default()
            }
        }
    }

    fn paint_overlay(&self, canvas: &mut Canvas, rect: Rect) {
        let fr = self.first_rect(&rect);
        let sr = self.second_rect(&rect);
        if let Some(ref child) = self.first {
            child.paint_overlay(canvas, fr);
        }
        if let Some(ref child) = self.second {
            child.paint_overlay(canvas, sr);
        }
    }

    fn needs_animation(&self) -> bool {
        // Handle color animation
        if self.anim_start.elapsed().as_secs_f32() < HANDLE_ANIM_DURATION {
            return true;
        }
        // Check children
        if let Some(ref child) = self.first
            && child.needs_animation()
        {
            return true;
        }
        if let Some(ref child) = self.second
            && child.needs_animation()
        {
            return true;
        }
        false
    }

    #[cfg(feature = "a11y")]
    fn access_info(&self) -> aurora_a11y::NodeInfo {
        aurora_a11y::NodeInfo::new(aurora_a11y::accesskit::Role::Splitter)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_values() {
        let sp = SplitPane::new();
        assert_eq!(sp.ratio, 0.5);
        assert_eq!(sp.initial_ratio, 0.5);
        assert_eq!(sp.min_first, 50.0);
        assert_eq!(sp.min_second, 50.0);
        assert_eq!(sp.handle_width, 6.0);
        assert_eq!(sp.direction, SplitDirection::Horizontal);
        assert!(!sp.dragging);
        assert!(!sp.hovered);
        assert!(sp.first.is_none());
        assert!(sp.second.is_none());
    }

    #[test]
    fn ratio_clamping() {
        let sp = SplitPane::new().min_first(100.0).min_second(100.0);
        // With 206 total (200 axis + 6 handle), min_first_ratio = 100/200 = 0.5
        let axis = 200.0;
        // Ratio 0.3 would give first=60px which is below min_first=100, so clamp up
        let clamped = sp.clamp_ratio(0.3, axis);
        assert!((clamped - 0.5).abs() < f32::EPSILON);

        // Ratio 0.8 would give second=40px which is below min_second=100, so clamp down
        let clamped = sp.clamp_ratio(0.8, axis);
        assert!((clamped - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn min_size_enforcement() {
        let sp = SplitPane::new().min_first(80.0).min_second(80.0);
        // axis=400, min_first_ratio = 80/400 = 0.2, max = 1 - 80/400 = 0.8
        let clamped_low = sp.clamp_ratio(0.1, 400.0);
        assert!((clamped_low - 0.2).abs() < f32::EPSILON);

        let clamped_high = sp.clamp_ratio(0.9, 400.0);
        assert!((clamped_high - 0.8).abs() < f32::EPSILON);

        // Within range stays unchanged
        let clamped_mid = sp.clamp_ratio(0.5, 400.0);
        assert!((clamped_mid - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn double_click_reset() {
        let mut sp = SplitPane::new().ratio(0.3);
        assert!((sp.initial_ratio - 0.3).abs() < f32::EPSILON);

        // Simulate dragging to change ratio
        sp.ratio = 0.7;
        assert!((sp.ratio - 0.7).abs() < f32::EPSILON);

        // Simulate double-click reset
        let axis = 400.0;
        sp.ratio = sp.clamp_ratio(sp.initial_ratio, axis);
        assert!((sp.ratio - 0.3).abs() < f32::EPSILON);
    }

    #[test]
    fn direction_layout() {
        let sp_h = SplitPane::new().direction(SplitDirection::Horizontal);
        let sp_v = SplitPane::new().direction(SplitDirection::Vertical);

        let bounds = Rect::new(0.0, 0.0, 400.0, 300.0);

        // Horizontal: handle is vertical bar
        let hr_h = sp_h.handle_rect(&bounds);
        assert!(hr_h.height() > hr_h.width()); // Tall, narrow handle

        // Vertical: handle is horizontal bar
        let hr_v = sp_v.handle_rect(&bounds);
        assert!(hr_v.width() > hr_v.height()); // Wide, short handle
    }

    #[test]
    fn handle_rect_position() {
        let sp = SplitPane::new(); // ratio 0.5, handle_width 6.0
        let bounds = Rect::new(0.0, 0.0, 206.0, 100.0);
        // axis = 206 - 6 = 200, first_size = 100
        let hr = sp.handle_rect(&bounds);
        assert!((hr.x1 - 100.0).abs() < f32::EPSILON);
        assert!((hr.x2 - 106.0).abs() < f32::EPSILON);
    }

    #[test]
    fn first_and_second_rects() {
        let sp = SplitPane::new(); // ratio 0.5, handle_width 6.0
        let bounds = Rect::new(0.0, 0.0, 206.0, 100.0);
        // axis = 200, first=100, handle=6, second=100

        let fr = sp.first_rect(&bounds);
        assert!((fr.x1 - 0.0).abs() < f32::EPSILON);
        assert!((fr.x2 - 100.0).abs() < f32::EPSILON);

        let sr = sp.second_rect(&bounds);
        assert!((sr.x1 - 106.0).abs() < f32::EPSILON);
        assert!((sr.x2 - 206.0).abs() < f32::EPSILON);
    }

    #[test]
    fn builder_chains() {
        let sp = SplitPane::new()
            .direction(SplitDirection::Vertical)
            .ratio(0.3)
            .min_first(100.0)
            .min_second(80.0)
            .handle_width(8.0);

        assert_eq!(sp.direction, SplitDirection::Vertical);
        assert!((sp.ratio - 0.3).abs() < f32::EPSILON);
        assert!((sp.min_first - 100.0).abs() < f32::EPSILON);
        assert!((sp.min_second - 80.0).abs() < f32::EPSILON);
        assert!((sp.handle_width - 8.0).abs() < f32::EPSILON);
    }
}
