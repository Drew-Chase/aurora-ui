use crate::widgets::{EventResponse, EventStatus, LayoutCtx, Widget};
use aurora_core::color::Color;
use aurora_core::geometry::corners::Corners;
use aurora_core::geometry::point::Point;
use aurora_core::geometry::rect::Rect;
use aurora_core::geometry::size::Size;
use aurora_core::kmi::WidgetEvent;
use aurora_core::kmi::cursor_icon::CursorIcon;
use aurora_core::kmi::mouse::{MouseEvent, MouseState};
use aurora_render::canvas::Canvas;
use aurora_text::font_options::FontWeight;
use aurora_text::text_layout::TextLayout;

use super::colors;

/// Orientation for the stepper layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StepperOrientation {
    #[default]
    Horizontal,
    Vertical,
}

/// A step-by-step wizard / progress indicator.
///
/// Displays a sequence of labeled steps with connecting lines. Each step
/// is rendered as a circle (with a number or checkmark) plus a text label.
/// Completed steps show a filled circle with a checkmark, the current step
/// shows a filled primary circle with a white number, and upcoming steps
/// show a muted outline with a muted number.
///
/// # Example
/// ```ignore
/// Stepper::new()
///     .step("Account")
///     .step("Profile")
///     .step("Review")
///     .current(1)
///     .orientation(StepperOrientation::Horizontal)
/// ```
pub struct Stepper {
    steps: Vec<String>,
    current: usize,
    orientation: StepperOrientation,
    step_size: f32,
    spacing: f32,
    completed_color: Color,
    current_color: Color,
    upcoming_color: Color,
    text_color: Color,
    line_color: Color,
    width: Option<f32>,
    clickable: bool,
    on_step_click: Option<Box<dyn FnMut(usize)>>,
    step_layouts: Vec<Option<TextLayout>>,
    number_layouts: Vec<Option<TextLayout>>,
}

impl Stepper {
    pub fn new() -> Self {
        Self {
            steps: Vec::new(),
            current: 0,
            orientation: StepperOrientation::default(),
            step_size: 32.0,
            spacing: 8.0,
            completed_color: colors::primary(),
            current_color: colors::primary(),
            upcoming_color: colors::muted(),
            text_color: colors::foreground(),
            line_color: colors::border(),
            width: None,
            clickable: false,
            on_step_click: None,
            step_layouts: Vec::new(),
            number_layouts: Vec::new(),
        }
    }

    /// Add a step with the given label.
    pub fn step(mut self, label: impl Into<String>) -> Self {
        self.steps.push(label.into());
        self
    }

    /// Set the current (active) step index (0-based). Clamped to the number of steps.
    pub fn current(mut self, current: usize) -> Self {
        self.current = current;
        self
    }

    /// Set horizontal or vertical orientation.
    pub fn orientation(mut self, orientation: StepperOrientation) -> Self {
        self.orientation = orientation;
        self
    }

    /// Set the step circle diameter.
    pub fn step_size(mut self, size: f32) -> Self {
        self.step_size = size;
        self
    }

    /// Set the spacing between the circle and the label.
    pub fn spacing(mut self, spacing: f32) -> Self {
        self.spacing = spacing;
        self
    }

    /// Set the color for completed steps.
    pub fn completed_color(mut self, color: Color) -> Self {
        self.completed_color = color;
        self
    }

    /// Set the color for the current step.
    pub fn current_color(mut self, color: Color) -> Self {
        self.current_color = color;
        self
    }

    /// Set the color for upcoming steps.
    pub fn upcoming_color(mut self, color: Color) -> Self {
        self.upcoming_color = color;
        self
    }

    /// Set the text label color.
    pub fn text_color(mut self, color: Color) -> Self {
        self.text_color = color;
        self
    }

    /// Set the connecting line color.
    pub fn line_color(mut self, color: Color) -> Self {
        self.line_color = color;
        self
    }

    /// Set a fixed width.
    pub fn width(mut self, width: impl Into<f32>) -> Self {
        self.width = Some(width.into());
        self
    }

    /// Enable clicking on step circles to navigate.
    pub fn clickable(mut self, clickable: bool) -> Self {
        self.clickable = clickable;
        self
    }

    /// Callback invoked when a step circle is clicked (requires `clickable(true)`).
    pub fn on_step_click(mut self, cb: impl FnMut(usize) + 'static) -> Self {
        self.on_step_click = Some(Box::new(cb));
        self
    }

    /// Clamp `current` to valid range.
    fn clamped_current(&self) -> usize {
        if self.steps.is_empty() {
            0
        } else {
            self.current.min(self.steps.len() - 1)
        }
    }

    /// Compute the bounding rect of step circle `i` given the full widget rect.
    fn circle_rect(&self, i: usize, rect: Rect) -> Rect {
        let n = self.steps.len();
        if n == 0 {
            return Rect::default();
        }
        let r = self.step_size;

        match self.orientation {
            StepperOrientation::Horizontal => {
                // Evenly space circles across the width
                let total_w = rect.width();
                let step_span = if n > 1 {
                    (total_w - r) / (n - 1) as f32
                } else {
                    0.0
                };
                let cx = rect.x1 + r / 2.0 + step_span * i as f32;
                let cy = rect.y1 + r / 2.0;
                Rect::new(cx - r / 2.0, cy - r / 2.0, cx + r / 2.0, cy + r / 2.0)
            }
            StepperOrientation::Vertical => {
                let label_height = self
                    .step_layouts
                    .get(i)
                    .and_then(|l| l.as_ref())
                    .map(|tl| tl.size().height)
                    .unwrap_or(20.0);
                let row_height = r.max(label_height) + self.spacing;
                let cy = rect.y1 + r / 2.0 + row_height * i as f32;
                let cx = rect.x1 + r / 2.0;
                Rect::new(cx - r / 2.0, cy - r / 2.0, cx + r / 2.0, cy + r / 2.0)
            }
        }
    }
}

impl Default for Stepper {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for Stepper {
    fn layout(&mut self, available: Size, ctx: &mut LayoutCtx) -> Size {
        let n = self.steps.len();
        self.step_layouts.clear();
        self.number_layouts.clear();

        if n == 0 {
            return Size::new(0.0, 0.0);
        }

        let total_w = self.width.unwrap_or(available.width);

        // Build text layouts for each step label and number
        for (i, label) in self.steps.iter().enumerate() {
            // Label layout
            let mut opts = ctx.font_options.clone();
            opts.size = Some(13.0);
            let current = self.clamped_current();
            opts.weight = Some(if i <= current {
                FontWeight::Medium
            } else {
                FontWeight::Normal
            });
            let label_color = if i <= current {
                self.text_color
            } else {
                colors::muted_foreground()
            };
            let tl = TextLayout::new(ctx.font_manager, label, &opts, label_color, None);
            self.step_layouts.push(Some(tl));

            // Number layout (1-based index as string)
            let num_str = format!("{}", i + 1);
            let mut num_opts = ctx.font_options.clone();
            num_opts.size = Some(13.0);
            num_opts.weight = Some(FontWeight::SemiBold);
            let num_color = if i <= current {
                colors::primary_foreground()
            } else {
                self.upcoming_color
            };
            let num_tl =
                TextLayout::new(ctx.font_manager, &num_str, &num_opts, num_color, None);
            self.number_layouts.push(Some(num_tl));
        }

        match self.orientation {
            StepperOrientation::Horizontal => {
                // Height: circle + spacing + label text height
                let label_h = self
                    .step_layouts
                    .iter()
                    .filter_map(|l| l.as_ref())
                    .map(|tl| tl.size().height)
                    .fold(0.0_f32, f32::max);
                let h = self.step_size + self.spacing + label_h;
                Size::new(total_w, h)
            }
            StepperOrientation::Vertical => {
                // Width: circle + spacing + max label width
                let label_w = self
                    .step_layouts
                    .iter()
                    .filter_map(|l| l.as_ref())
                    .map(|tl| tl.size().width)
                    .fold(0.0_f32, f32::max);
                let w = self.step_size + self.spacing + label_w;
                let w = if let Some(fixed) = self.width {
                    fixed
                } else {
                    w.min(available.width)
                };

                let mut total_h = 0.0;
                for i in 0..n {
                    let label_h = self
                        .step_layouts
                        .get(i)
                        .and_then(|l| l.as_ref())
                        .map(|tl| tl.size().height)
                        .unwrap_or(20.0);
                    let row_h = self.step_size.max(label_h);
                    total_h += row_h;
                    if i < n - 1 {
                        total_h += self.spacing;
                    }
                }
                Size::new(w, total_h)
            }
        }
    }

    fn paint(&self, canvas: &mut Canvas, rect: Rect) {
        let n = self.steps.len();
        if n == 0 {
            return;
        }
        let current = self.clamped_current();
        let r = self.step_size;
        let half = r / 2.0;

        match self.orientation {
            StepperOrientation::Horizontal => {
                self.paint_horizontal(canvas, rect, n, current, r, half);
            }
            StepperOrientation::Vertical => {
                self.paint_vertical(canvas, rect, n, current, r, half);
            }
        }
    }

    fn children(&self) -> &[Box<dyn Widget>] {
        &[]
    }

    fn event(&mut self, event: &WidgetEvent, rect: Rect) -> EventResponse {
        if !self.clickable {
            return EventResponse::default();
        }

        let n = self.steps.len();
        if n == 0 {
            return EventResponse::default();
        }

        match event {
            WidgetEvent::Mouse(MouseEvent::MouseClickEvent(e))
                if e.state == MouseState::Pressed && rect.contains(&e.position) =>
            {
                for i in 0..n {
                    let cr = self.circle_rect(i, rect);
                    if cr.contains(&e.position) {
                        if let Some(ref mut cb) = self.on_step_click {
                            cb(i);
                        }
                        return EventResponse {
                            status: EventStatus::Consumed,
                            cursor: Some(CursorIcon::Pointer),
                            ..Default::default()
                        };
                    }
                }
                EventResponse::default()
            }
            WidgetEvent::Mouse(MouseEvent::MouseMoveEvent(pos)) if rect.contains(pos) => {
                for i in 0..n {
                    let cr = self.circle_rect(i, rect);
                    if cr.contains(pos) {
                        return EventResponse {
                            cursor: Some(CursorIcon::Pointer),
                            ..Default::default()
                        };
                    }
                }
                EventResponse::default()
            }
            _ => EventResponse::default(),
        }
    }

    #[cfg(feature = "a11y")]
    fn access_info(&self) -> aurora_a11y::NodeInfo {
        aurora_a11y::NodeInfo::new(aurora_a11y::accesskit::Role::List)
            .with_label("Stepper".to_string())
    }
}

// Private paint helpers
impl Stepper {
    fn paint_horizontal(
        &self,
        canvas: &mut Canvas,
        rect: Rect,
        n: usize,
        current: usize,
        r: f32,
        half: f32,
    ) {
        let total_w = rect.width();
        let step_span = if n > 1 {
            (total_w - r) / (n - 1) as f32
        } else {
            0.0
        };

        // Draw connecting lines first (behind circles)
        for i in 0..n.saturating_sub(1) {
            let x1 = rect.x1 + half + step_span * i as f32 + half;
            let x2 = rect.x1 + half + step_span * (i + 1) as f32 - half;
            let cy = rect.y1 + half;
            let line_color = if i < current {
                self.completed_color
            } else {
                self.line_color
            };
            canvas.draw_line(
                Point::new(x1, cy),
                Point::new(x2, cy),
                2.0,
                line_color,
            );
        }

        // Draw circles and labels
        for i in 0..n {
            let cx = rect.x1 + half + step_span * i as f32;
            let cy = rect.y1 + half;
            let circle_rect = Rect::new(cx - half, cy - half, cx + half, cy + half);
            let corners = Corners::all(half);

            if i < current {
                // Completed: filled circle with checkmark
                canvas.fill_rounded_rect(circle_rect, corners, self.completed_color);
                self.draw_checkmark(canvas, cx, cy, r, colors::primary_foreground());
            } else if i == current {
                // Current: filled circle with number
                canvas.fill_rounded_rect(circle_rect, corners, self.current_color);
                if let Some(Some(num_tl)) = self.number_layouts.get(i) {
                    let ns = num_tl.size();
                    let nx = cx - ns.width / 2.0;
                    let ny = cy - ns.height / 2.0;
                    canvas.draw_text(num_tl, nx as i32, ny as i32);
                }
            } else {
                // Upcoming: outlined circle with muted number
                canvas.stroke_rounded_rect(circle_rect, corners, 2, self.upcoming_color);
                if let Some(Some(num_tl)) = self.number_layouts.get(i) {
                    let ns = num_tl.size();
                    let nx = cx - ns.width / 2.0;
                    let ny = cy - ns.height / 2.0;
                    canvas.draw_text(num_tl, nx as i32, ny as i32);
                }
            }

            // Draw label below the circle
            if let Some(Some(tl)) = self.step_layouts.get(i) {
                let ts = tl.size();
                let lx = cx - ts.width / 2.0;
                let ly = rect.y1 + r + self.spacing;
                canvas.draw_text(tl, lx as i32, ly as i32);
            }
        }
    }

    fn paint_vertical(
        &self,
        canvas: &mut Canvas,
        rect: Rect,
        n: usize,
        current: usize,
        r: f32,
        half: f32,
    ) {
        // Compute row positions
        let mut row_tops: Vec<f32> = Vec::with_capacity(n);
        let mut y = rect.y1;
        for i in 0..n {
            row_tops.push(y);
            let label_h = self
                .step_layouts
                .get(i)
                .and_then(|l| l.as_ref())
                .map(|tl| tl.size().height)
                .unwrap_or(20.0);
            let row_h = r.max(label_h);
            y += row_h;
            if i < n - 1 {
                y += self.spacing;
            }
        }

        // Draw connecting lines
        for i in 0..n.saturating_sub(1) {
            let cx = rect.x1 + half;
            let y1 = row_tops[i] + r;
            let y2 = row_tops[i + 1];
            let line_color = if i < current {
                self.completed_color
            } else {
                self.line_color
            };
            canvas.draw_line(
                Point::new(cx, y1),
                Point::new(cx, y2),
                2.0,
                line_color,
            );
        }

        // Draw circles and labels
        for (i, &row_top) in row_tops.iter().enumerate().take(n) {
            let cx = rect.x1 + half;
            let cy = row_top + half;
            let circle_rect = Rect::new(cx - half, cy - half, cx + half, cy + half);
            let corners = Corners::all(half);

            if i < current {
                canvas.fill_rounded_rect(circle_rect, corners, self.completed_color);
                self.draw_checkmark(canvas, cx, cy, r, colors::primary_foreground());
            } else if i == current {
                canvas.fill_rounded_rect(circle_rect, corners, self.current_color);
                if let Some(Some(num_tl)) = self.number_layouts.get(i) {
                    let ns = num_tl.size();
                    let nx = cx - ns.width / 2.0;
                    let ny = cy - ns.height / 2.0;
                    canvas.draw_text(num_tl, nx as i32, ny as i32);
                }
            } else {
                canvas.stroke_rounded_rect(circle_rect, corners, 2, self.upcoming_color);
                if let Some(Some(num_tl)) = self.number_layouts.get(i) {
                    let ns = num_tl.size();
                    let nx = cx - ns.width / 2.0;
                    let ny = cy - ns.height / 2.0;
                    canvas.draw_text(num_tl, nx as i32, ny as i32);
                }
            }

            // Draw label beside the circle
            if let Some(Some(tl)) = self.step_layouts.get(i) {
                let ts = tl.size();
                let lx = rect.x1 + r + self.spacing;
                let ly = cy - ts.height / 2.0;
                canvas.draw_text(tl, lx as i32, ly as i32);
            }
        }
    }

    /// Draw a checkmark inside a step circle at (cx, cy) with diameter `d`.
    fn draw_checkmark(&self, canvas: &mut Canvas, cx: f32, cy: f32, d: f32, color: Color) {
        // Checkmark: two line segments forming a check shape
        let s = d * 0.3; // scale factor
        let p1 = Point::new(cx - s * 0.6, cy);
        let p2 = Point::new(cx - s * 0.1, cy + s * 0.5);
        let p3 = Point::new(cx + s * 0.6, cy - s * 0.4);
        canvas.draw_line(p1, p2, 2.0, color);
        canvas.draw_line(p2, p3, 2.0, color);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_values() {
        let s = Stepper::new();
        assert_eq!(s.current, 0);
        assert_eq!(s.step_size, 32.0);
        assert_eq!(s.spacing, 8.0);
        assert_eq!(s.orientation, StepperOrientation::Horizontal);
        assert!(!s.clickable);
        assert!(s.steps.is_empty());
        assert!(s.on_step_click.is_none());
    }

    #[test]
    fn step_states() {
        let s = Stepper::new()
            .step("One")
            .step("Two")
            .step("Three")
            .current(1);

        assert_eq!(s.steps.len(), 3);
        assert_eq!(s.clamped_current(), 1);
        // Step 0 is completed (< current), step 1 is current, step 2 is upcoming
        assert_eq!(s.steps[0], "One");
        assert_eq!(s.steps[1], "Two");
        assert_eq!(s.steps[2], "Three");
    }

    #[test]
    fn current_clamping() {
        // current beyond steps count is clamped
        let s = Stepper::new()
            .step("A")
            .step("B")
            .current(99);
        assert_eq!(s.clamped_current(), 1); // 0-indexed, max = len-1

        // No steps: clamped to 0
        let s2 = Stepper::new().current(5);
        assert_eq!(s2.clamped_current(), 0);
    }

    #[test]
    fn clickable_interaction() {
        let s = Stepper::new()
            .step("Step 1")
            .step("Step 2")
            .clickable(true)
            .on_step_click(move |_idx| {});

        assert!(s.clickable);
        assert!(s.on_step_click.is_some());
    }

    #[test]
    fn orientation_difference() {
        let h = Stepper::new()
            .step("A")
            .step("B")
            .orientation(StepperOrientation::Horizontal);
        assert_eq!(h.orientation, StepperOrientation::Horizontal);

        let v = Stepper::new()
            .step("A")
            .step("B")
            .orientation(StepperOrientation::Vertical);
        assert_eq!(v.orientation, StepperOrientation::Vertical);
    }
}
