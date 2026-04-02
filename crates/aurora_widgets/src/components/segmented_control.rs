use crate::widgets::{EventResponse, EventStatus, LayoutCtx, Widget};
use aurora_core::color::Color;
use aurora_core::geometry::corners::Corners;
use aurora_core::geometry::rect::Rect;
use aurora_core::geometry::size::Size;
use aurora_core::kmi::WidgetEvent;
use aurora_core::kmi::cursor_icon::CursorIcon;
use aurora_core::kmi::mouse::{MouseEvent, MouseState};
use aurora_render::canvas::Canvas;
use aurora_text::font_options::FontWeight;
use aurora_text::text_layout::TextLayout;
use std::time::Instant;

use super::colors;

const ANIM_DURATION: f32 = 0.2; // seconds

/// An iOS-style segmented toggle bar with a sliding pill indicator.
///
/// Displays a horizontal bar divided into equal-width segments. The active
/// segment is highlighted by a rounded indicator pill that animates smoothly
/// when the selection changes.
///
/// # Example
/// ```ignore
/// SegmentedControl::new()
///     .option("Day")
///     .option("Week")
///     .option("Month")
///     .selected(0)
///     .on_change(|idx| println!("selected: {idx}"))
/// ```
pub struct SegmentedControl {
    options: Vec<String>,
    selected: usize,
    width: Option<f32>,
    height: f32,
    background: Color,
    indicator_color: Color,
    text_color: Color,
    selected_text_color: Color,
    corners: Corners,
    padding: f32,
    on_change: Option<Box<dyn FnMut(usize)>>,
    option_layouts: Vec<Option<TextLayout>>,
    // Animated indicator
    indicator_from_x: f32,
    indicator_to_x: f32,
    indicator_from_w: f32,
    indicator_to_w: f32,
    anim_start: Instant,
}

impl SegmentedControl {
    pub fn new() -> Self {
        Self {
            options: Vec::new(),
            selected: 0,
            width: None,
            height: 36.0,
            background: colors::muted(),
            indicator_color: colors::background(),
            text_color: colors::foreground(),
            selected_text_color: colors::foreground(),
            corners: Corners::all(6.0),
            padding: 3.0,
            on_change: None,
            option_layouts: Vec::new(),
            indicator_from_x: 0.0,
            indicator_to_x: 0.0,
            indicator_from_w: 0.0,
            indicator_to_w: 0.0,
            anim_start: Instant::now(),
        }
    }

    /// Add an option segment with the given label.
    pub fn option(mut self, label: impl Into<String>) -> Self {
        self.options.push(label.into());
        self
    }

    /// Set the initially selected segment index (0-based).
    pub fn selected(mut self, index: usize) -> Self {
        self.selected = index;
        self
    }

    /// Set a fixed width. If not set, the control uses the available width.
    pub fn width(mut self, width: impl Into<f32>) -> Self {
        self.width = Some(width.into());
        self
    }

    /// Set the overall height of the control.
    pub fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }

    /// Set the background track color.
    pub fn background(mut self, color: Color) -> Self {
        self.background = color;
        self
    }

    /// Set the indicator pill color.
    pub fn indicator_color(mut self, color: Color) -> Self {
        self.indicator_color = color;
        self
    }

    /// Set the text color for unselected options.
    pub fn text_color(mut self, color: Color) -> Self {
        self.text_color = color;
        self
    }

    /// Set the text color for the selected option.
    pub fn selected_text_color(mut self, color: Color) -> Self {
        self.selected_text_color = color;
        self
    }

    /// Set the corner radii for the track and indicator.
    pub fn corners(mut self, corners: Corners) -> Self {
        self.corners = corners;
        self
    }

    /// Set the inner padding between the track edge and the indicator.
    pub fn padding(mut self, padding: f32) -> Self {
        self.padding = padding;
        self
    }

    /// Set a callback fired when the selection changes.
    pub fn on_change(mut self, cb: impl FnMut(usize) + 'static) -> Self {
        self.on_change = Some(Box::new(cb));
        self
    }

    /// Compute the x-position and width of the segment at `index` within
    /// a track that starts at `base_x` and has the given `total_width`.
    fn segment_rect(&self, index: usize, base_x: f32, total_width: f32) -> (f32, f32) {
        if self.options.is_empty() {
            return (base_x, 0.0);
        }
        let seg_w = total_width / self.options.len() as f32;
        let x = base_x + seg_w * index as f32;
        (x, seg_w)
    }

    /// Returns the current animated indicator (x, width) relative to the
    /// inner content area (i.e. after padding is applied).
    fn current_indicator(&self) -> (f32, f32) {
        let elapsed = self.anim_start.elapsed().as_secs_f32();
        let t = (elapsed / ANIM_DURATION).min(1.0);
        // CubicOut easing: 1 - (1 - t)^3
        let eased = 1.0 - (1.0 - t).powi(3);
        let ix = self.indicator_from_x + (self.indicator_to_x - self.indicator_from_x) * eased;
        let iw = self.indicator_from_w + (self.indicator_to_w - self.indicator_from_w) * eased;
        (ix, iw)
    }
}

impl Default for SegmentedControl {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for SegmentedControl {
    fn layout(&mut self, available: Size, ctx: &mut LayoutCtx) -> Size {
        let w = self.width.unwrap_or(available.width);
        self.option_layouts.clear();

        // Shape text for each option
        for label in &self.options {
            let mut opts = ctx.font_options.clone();
            opts.size = Some(13.0);
            opts.weight = Some(FontWeight::Medium);
            let mut tl = TextLayout::new(ctx.font_manager, label, &opts, self.text_color, None);
            tl.set_max_width(ctx.font_manager, f32::MAX);
            self.option_layouts.push(Some(tl));
        }

        // Initialize indicator position when animation has finished
        let inner_w = w - self.padding * 2.0;
        if self.anim_start.elapsed().as_secs_f32() >= ANIM_DURATION {
            let (sx, sw) = self.segment_rect(self.selected, 0.0, inner_w);
            self.indicator_to_x = sx;
            self.indicator_to_w = sw;
            self.indicator_from_x = sx;
            self.indicator_from_w = sw;
        }

        Size::new(w, self.height)
    }

    fn paint(&self, canvas: &mut Canvas, rect: Rect) {
        let w = rect.width();

        // Inner corners for indicator (slightly smaller than outer)
        let inner_corners = Corners {
            top_left: (self.corners.top_left - 1.0).max(2.0),
            top_right: (self.corners.top_right - 1.0).max(2.0),
            bottom_right: (self.corners.bottom_right - 1.0).max(2.0),
            bottom_left: (self.corners.bottom_left - 1.0).max(2.0),
        };

        // 1. Draw background track
        canvas.fill_rounded_rect(rect, self.corners, self.background);

        // 2. Draw sliding indicator pill
        if !self.options.is_empty() {
            let inner_x = rect.x1 + self.padding;
            let inner_y = rect.y1 + self.padding;
            let inner_h = self.height - self.padding * 2.0;
            let inner_w = w - self.padding * 2.0;

            let (ix, iw) = self.current_indicator();
            let pill_rect = Rect::new(inner_x + ix, inner_y, inner_x + ix + iw, inner_y + inner_h);
            canvas.fill_rounded_rect(pill_rect, inner_corners, self.indicator_color);

            // 3. Draw text for each option
            let seg_w = inner_w / self.options.len() as f32;
            for (i, _label) in self.options.iter().enumerate() {
                if let Some(Some(tl)) = self.option_layouts.get(i) {
                    let ts = tl.size();
                    let seg_x = inner_x + seg_w * i as f32;
                    let tx = seg_x + (seg_w - ts.width) / 2.0;
                    let ty = rect.y1 + (self.height - ts.height) / 2.0;
                    canvas.draw_text(tl, tx as i32, ty as i32);
                }
            }
        }
    }

    fn children(&self) -> &[Box<dyn Widget>] {
        &[]
    }

    fn event(&mut self, event: &WidgetEvent, rect: Rect) -> EventResponse {
        match event {
            WidgetEvent::Mouse(MouseEvent::MouseClickEvent(e))
                if e.state == MouseState::Pressed && rect.contains(&e.position) =>
            {
                let w = rect.width();
                let inner_x = rect.x1 + self.padding;
                let inner_w = w - self.padding * 2.0;
                let inner_y = rect.y1 + self.padding;
                let inner_h = self.height - self.padding * 2.0;

                let seg_w = if self.options.is_empty() {
                    0.0
                } else {
                    inner_w / self.options.len() as f32
                };

                for i in 0..self.options.len() {
                    let seg_x = inner_x + seg_w * i as f32;
                    let seg_rect = Rect::new(seg_x, inner_y, seg_x + seg_w, inner_y + inner_h);
                    if seg_rect.contains(&e.position) && i != self.selected {
                        // Start animation from current position
                        let (cur_x, cur_w) = self.current_indicator();
                        self.indicator_from_x = cur_x;
                        self.indicator_from_w = cur_w;
                        let (to_x, to_w) = self.segment_rect(i, 0.0, inner_w);
                        self.indicator_to_x = to_x;
                        self.indicator_to_w = to_w;
                        self.anim_start = Instant::now();

                        self.selected = i;
                        if let Some(ref mut cb) = self.on_change {
                            cb(i);
                        }
                        return EventResponse {
                            status: EventStatus::Consumed,
                            cursor: Some(CursorIcon::Pointer),
                            ..Default::default()
                        };
                    }
                }
                EventResponse {
                    status: EventStatus::Consumed,
                    cursor: Some(CursorIcon::Pointer),
                    ..Default::default()
                }
            }
            WidgetEvent::Mouse(MouseEvent::MouseMoveEvent(pos)) if rect.contains(pos) => {
                EventResponse {
                    cursor: Some(CursorIcon::Pointer),
                    ..Default::default()
                }
            }
            _ => EventResponse::default(),
        }
    }

    fn needs_animation(&self) -> bool {
        self.anim_start.elapsed().as_secs_f32() < ANIM_DURATION
    }

    #[cfg(feature = "a11y")]
    fn access_info(&self) -> aurora_a11y::NodeInfo {
        aurora_a11y::NodeInfo::new(aurora_a11y::accesskit::Role::TabList)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_control() -> SegmentedControl {
        SegmentedControl::new()
            .option("Day")
            .option("Week")
            .option("Month")
    }

    #[test]
    fn default_values() {
        let ctrl = SegmentedControl::new();
        assert_eq!(ctrl.selected, 0);
        assert_eq!(ctrl.height, 36.0);
        assert_eq!(ctrl.padding, 3.0);
        assert_eq!(ctrl.corners, Corners::all(6.0));
        assert!(ctrl.options.is_empty());
        assert!(ctrl.on_change.is_none());
        assert!(ctrl.width.is_none());
    }

    #[test]
    fn selection() {
        let ctrl = make_control().selected(2);
        assert_eq!(ctrl.selected, 2);
        assert_eq!(ctrl.options.len(), 3);
        assert_eq!(ctrl.options[0], "Day");
        assert_eq!(ctrl.options[1], "Week");
        assert_eq!(ctrl.options[2], "Month");
    }

    #[test]
    fn equal_segment_sizing() {
        let ctrl = make_control();
        let total_w = 300.0;
        // Each segment should be total_w / 3
        let (x0, w0) = ctrl.segment_rect(0, 0.0, total_w);
        let (x1, w1) = ctrl.segment_rect(1, 0.0, total_w);
        let (x2, w2) = ctrl.segment_rect(2, 0.0, total_w);

        assert!((w0 - 100.0).abs() < f32::EPSILON);
        assert!((w1 - 100.0).abs() < f32::EPSILON);
        assert!((w2 - 100.0).abs() < f32::EPSILON);
        assert!((x0 - 0.0).abs() < f32::EPSILON);
        assert!((x1 - 100.0).abs() < f32::EPSILON);
        assert!((x2 - 200.0).abs() < f32::EPSILON);
    }

    #[test]
    fn animated_position() {
        let mut ctrl = make_control().selected(0);
        // Simulate that the indicator is at segment 0
        ctrl.indicator_from_x = 0.0;
        ctrl.indicator_from_w = 100.0;
        ctrl.indicator_to_x = 0.0;
        ctrl.indicator_to_w = 100.0;

        // Start animation to segment 2
        ctrl.indicator_from_x = 0.0;
        ctrl.indicator_from_w = 100.0;
        ctrl.indicator_to_x = 200.0;
        ctrl.indicator_to_w = 100.0;
        ctrl.anim_start = Instant::now();

        // At t=0, indicator should be at origin
        let (ix, iw) = ctrl.current_indicator();
        assert!((ix - 0.0).abs() < 1.0, "ix at t=0 should be ~0, got {ix}");
        assert!(
            (iw - 100.0).abs() < 1.0,
            "iw at t=0 should be ~100, got {iw}"
        );

        // After animation completes, set a past start time
        ctrl.anim_start = Instant::now() - std::time::Duration::from_secs(1);
        let (ix, iw) = ctrl.current_indicator();
        assert!(
            (ix - 200.0).abs() < f32::EPSILON,
            "ix after anim should be 200, got {ix}"
        );
        assert!(
            (iw - 100.0).abs() < f32::EPSILON,
            "iw after anim should be 100, got {iw}"
        );
    }

    #[test]
    fn single_option() {
        let ctrl = SegmentedControl::new().option("Only");
        assert_eq!(ctrl.options.len(), 1);
        let (x, w) = ctrl.segment_rect(0, 0.0, 200.0);
        assert!((x - 0.0).abs() < f32::EPSILON);
        assert!((w - 200.0).abs() < f32::EPSILON);
    }
}
