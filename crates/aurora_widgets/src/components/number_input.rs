use crate::widgets::{EventResponse, EventStatus, LayoutCtx, Widget};
use aurora_core::color::Color;
use aurora_core::geometry::corners::Corners;
use aurora_core::geometry::rect::Rect;
use aurora_core::geometry::size::Size;
use aurora_core::kmi::WidgetEvent;
use aurora_core::kmi::cursor_icon::CursorIcon;
use aurora_core::kmi::mouse::{MouseEvent, MouseState};
use aurora_render::canvas::Canvas;
use std::time::Instant;

use super::colors;

const ANIM_DURATION: f32 = 0.12; // seconds
const BUTTON_WIDTH: f32 = 36.0;

/// A numeric input with increment/decrement buttons, min/max clamping, and step value.
///
/// # Example
/// ```ignore
/// NumberInput::new()
///     .value(42.0)
///     .min(0.0)
///     .max(100.0)
///     .step(1.0)
///     .precision(0)
///     .width(200.0)
///     .on_change(|val| println!("{val}"))
/// ```
pub struct NumberInput {
    value: f64,
    min: f64,
    max: f64,
    step: f64,
    precision: u32,
    width: Option<f32>,
    height: f32,
    background: Color,
    border_color: Color,
    corners: Corners,
    button_bg: Color,
    button_hover_bg: Color,
    text_color: Color,
    disabled: bool,
    on_change: Option<Box<dyn FnMut(f64)>>,
    display_layout: Option<aurora_text::text_layout::TextLayout>,
    minus_layout: Option<aurora_text::text_layout::TextLayout>,
    plus_layout: Option<aurora_text::text_layout::TextLayout>,
    hover_minus: bool,
    hover_plus: bool,
    minus_anim_from: f32,
    minus_anim_to: f32,
    minus_anim_start: Instant,
    plus_anim_from: f32,
    plus_anim_to: f32,
    plus_anim_start: Instant,
    error: bool,
}

impl NumberInput {
    pub fn new() -> Self {
        Self {
            value: 0.0,
            min: -f64::MAX,
            max: f64::MAX,
            step: 1.0,
            precision: 2,
            width: None,
            height: 40.0,
            background: colors::background(),
            border_color: colors::input_border(),
            corners: Corners::all(6.0),
            button_bg: colors::muted(),
            button_hover_bg: colors::accent(),
            text_color: colors::foreground(),
            disabled: false,
            on_change: None,
            display_layout: None,
            minus_layout: None,
            plus_layout: None,
            hover_minus: false,
            hover_plus: false,
            minus_anim_from: 0.0,
            minus_anim_to: 0.0,
            minus_anim_start: Instant::now(),
            plus_anim_from: 0.0,
            plus_anim_to: 0.0,
            plus_anim_start: Instant::now(),
            error: false,
        }
    }

    pub fn value(mut self, value: f64) -> Self {
        self.value = value;
        self
    }

    pub fn min(mut self, min: f64) -> Self {
        self.min = min;
        self
    }

    pub fn max(mut self, max: f64) -> Self {
        self.max = max;
        self
    }

    pub fn step(mut self, step: f64) -> Self {
        self.step = step;
        self
    }

    pub fn precision(mut self, precision: u32) -> Self {
        self.precision = precision;
        self
    }

    pub fn width(mut self, width: impl Into<f32>) -> Self {
        self.width = Some(width.into());
        self
    }

    pub fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }

    pub fn background(mut self, color: Color) -> Self {
        self.background = color;
        self
    }

    pub fn border_color(mut self, color: Color) -> Self {
        self.border_color = color;
        self
    }

    pub fn corners(mut self, corners: Corners) -> Self {
        self.corners = corners;
        self
    }

    pub fn button_bg(mut self, color: Color) -> Self {
        self.button_bg = color;
        self
    }

    pub fn button_hover_bg(mut self, color: Color) -> Self {
        self.button_hover_bg = color;
        self
    }

    pub fn text_color(mut self, color: Color) -> Self {
        self.text_color = color;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn error(mut self, error: bool) -> Self {
        self.error = error;
        self
    }

    pub fn on_change(mut self, cb: impl FnMut(f64) + 'static) -> Self {
        self.on_change = Some(Box::new(cb));
        self
    }

    fn format_value(&self) -> String {
        format!("{:.prec$}", self.value, prec = self.precision as usize)
    }

    fn clamp_value(&self, val: f64) -> f64 {
        let effective_min = self.min.min(self.max);
        let effective_max = self.min.max(self.max);
        val.clamp(effective_min, effective_max)
    }

    fn increment(&mut self) {
        self.value = self.clamp_value(self.value + self.step);
        if let Some(ref mut cb) = self.on_change {
            cb(self.value);
        }
    }

    fn decrement(&mut self) {
        self.value = self.clamp_value(self.value - self.step);
        if let Some(ref mut cb) = self.on_change {
            cb(self.value);
        }
    }

    /// Compute the eased animation progress for the minus button hover.
    fn minus_anim_t(&self) -> f32 {
        let elapsed = self.minus_anim_start.elapsed().as_secs_f32();
        let t = (elapsed / ANIM_DURATION).min(1.0);
        let eased = 1.0 - (1.0 - t).powi(3);
        self.minus_anim_from + (self.minus_anim_to - self.minus_anim_from) * eased
    }

    /// Compute the eased animation progress for the plus button hover.
    fn plus_anim_t(&self) -> f32 {
        let elapsed = self.plus_anim_start.elapsed().as_secs_f32();
        let t = (elapsed / ANIM_DURATION).min(1.0);
        let eased = 1.0 - (1.0 - t).powi(3);
        self.plus_anim_from + (self.plus_anim_to - self.plus_anim_from) * eased
    }

    /// Returns the rect for the minus button area given the overall widget rect.
    fn minus_rect(&self, rect: &Rect) -> Rect {
        Rect::new(rect.x1, rect.y1, rect.x1 + BUTTON_WIDTH, rect.y2)
    }

    /// Returns the rect for the plus button area given the overall widget rect.
    fn plus_rect(&self, rect: &Rect) -> Rect {
        Rect::new(rect.x2 - BUTTON_WIDTH, rect.y1, rect.x2, rect.y2)
    }
}

impl Default for NumberInput {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for NumberInput {
    fn layout(&mut self, available: Size, ctx: &mut LayoutCtx) -> Size {
        let w = self.width.unwrap_or(available.width);

        // Shape the value display text
        let display_text = self.format_value();
        let mut opts = ctx.font_options.clone();
        opts.size = Some(14.0);
        opts.weight = Some(aurora_text::font_options::FontWeight::Normal);
        self.display_layout = Some(aurora_text::text_layout::TextLayout::new(
            ctx.font_manager,
            &display_text,
            &opts,
            self.text_color,
            None,
        ));

        // Shape the minus symbol
        let mut symbol_opts = ctx.font_options.clone();
        symbol_opts.size = Some(16.0);
        symbol_opts.weight = Some(aurora_text::font_options::FontWeight::Medium);
        self.minus_layout = Some(aurora_text::text_layout::TextLayout::new(
            ctx.font_manager,
            "\u{2212}",
            &symbol_opts,
            self.text_color,
            None,
        ));

        // Shape the plus symbol
        self.plus_layout = Some(aurora_text::text_layout::TextLayout::new(
            ctx.font_manager,
            "+",
            &symbol_opts,
            self.text_color,
            None,
        ));

        Size::new(w, self.height)
    }

    fn paint(&self, canvas: &mut Canvas, rect: Rect) {
        // Background fill
        canvas.fill_rounded_rect(rect, self.corners, self.background);

        // Border
        canvas.stroke_rounded_rect(rect, self.corners, 1, self.border_color);

        let minus_rect = self.minus_rect(&rect);
        let plus_rect = self.plus_rect(&rect);

        // Left button area (minus) with animated background
        let minus_t = self.minus_anim_t();
        let minus_bg = self.button_bg.lerp(&self.button_hover_bg, minus_t);
        let minus_inner = Rect::new(
            minus_rect.x1 + 1.0,
            minus_rect.y1 + 1.0,
            minus_rect.x2,
            minus_rect.y2 - 1.0,
        );
        let left_corners = Corners {
            top_left: self.corners.top_left.max(1.0) - 1.0,
            top_right: 0.0,
            bottom_right: 0.0,
            bottom_left: self.corners.bottom_left.max(1.0) - 1.0,
        };
        canvas.fill_rounded_rect(minus_inner, left_corners, minus_bg);

        // Draw minus symbol centered in left button
        if let Some(ref tl) = self.minus_layout {
            let ts = tl.size();
            let tx = minus_rect.x1 + (BUTTON_WIDTH - ts.width) / 2.0;
            let ty = minus_rect.y1 + (self.height - ts.height) / 2.0;
            canvas.draw_text(tl, tx as i32, ty as i32);
        }

        // Left vertical separator
        let sep_x = minus_rect.x2;
        canvas.draw_line(
            (sep_x, rect.y1 + 1.0),
            (sep_x, rect.y2 - 1.0),
            1.0,
            self.border_color,
        );

        // Value text centered in the middle region
        if let Some(ref tl) = self.display_layout {
            let ts = tl.size();
            let mid_x1 = minus_rect.x2;
            let mid_x2 = plus_rect.x1;
            let mid_w = mid_x2 - mid_x1;
            let tx = mid_x1 + (mid_w - ts.width) / 2.0;
            let ty = rect.y1 + (self.height - ts.height) / 2.0;
            canvas.draw_text(tl, tx as i32, ty as i32);
        }

        // Right vertical separator
        let sep_x = plus_rect.x1;
        canvas.draw_line(
            (sep_x, rect.y1 + 1.0),
            (sep_x, rect.y2 - 1.0),
            1.0,
            self.border_color,
        );

        // Right button area (plus) with animated background
        let plus_t = self.plus_anim_t();
        let plus_bg = self.button_bg.lerp(&self.button_hover_bg, plus_t);
        let plus_inner = Rect::new(
            plus_rect.x1,
            plus_rect.y1 + 1.0,
            plus_rect.x2 - 1.0,
            plus_rect.y2 - 1.0,
        );
        let right_corners = Corners {
            top_left: 0.0,
            top_right: self.corners.top_right.max(1.0) - 1.0,
            bottom_right: self.corners.bottom_right.max(1.0) - 1.0,
            bottom_left: 0.0,
        };
        canvas.fill_rounded_rect(plus_inner, right_corners, plus_bg);

        // Draw plus symbol centered in right button
        if let Some(ref tl) = self.plus_layout {
            let ts = tl.size();
            let tx = plus_rect.x1 + (BUTTON_WIDTH - ts.width) / 2.0;
            let ty = plus_rect.y1 + (self.height - ts.height) / 2.0;
            canvas.draw_text(tl, tx as i32, ty as i32);
        }

        // Disabled overlay
        if self.disabled {
            canvas.fill_rounded_rect(rect, self.corners, Color::new(255, 255, 255, 80));
        }
    }

    fn children(&self) -> &[Box<dyn Widget>] {
        &[]
    }

    fn event(&mut self, event: &WidgetEvent, rect: Rect) -> EventResponse {
        if self.disabled {
            return EventResponse::default();
        }

        let minus_rect = self.minus_rect(&rect);
        let plus_rect = self.plus_rect(&rect);

        match event {
            WidgetEvent::Mouse(MouseEvent::MouseClickEvent(e))
                if e.state == MouseState::Pressed =>
            {
                if minus_rect.contains(&e.position) {
                    self.decrement();
                    return EventResponse {
                        status: EventStatus::Consumed,
                        cursor: Some(CursorIcon::Pointer),
                        ..Default::default()
                    };
                }
                if plus_rect.contains(&e.position) {
                    self.increment();
                    return EventResponse {
                        status: EventStatus::Consumed,
                        cursor: Some(CursorIcon::Pointer),
                        ..Default::default()
                    };
                }
                EventResponse::default()
            }
            WidgetEvent::Mouse(MouseEvent::MouseMoveEvent(pos)) => {
                let in_minus = minus_rect.contains(pos);
                let in_plus = plus_rect.contains(pos);

                // Animate minus button hover
                if in_minus != self.hover_minus {
                    self.hover_minus = in_minus;
                    self.minus_anim_from = self.minus_anim_t();
                    self.minus_anim_to = if in_minus { 1.0 } else { 0.0 };
                    self.minus_anim_start = Instant::now();
                }

                // Animate plus button hover
                if in_plus != self.hover_plus {
                    self.hover_plus = in_plus;
                    self.plus_anim_from = self.plus_anim_t();
                    self.plus_anim_to = if in_plus { 1.0 } else { 0.0 };
                    self.plus_anim_start = Instant::now();
                }

                if in_minus || in_plus {
                    return EventResponse {
                        cursor: Some(CursorIcon::Pointer),
                        ..Default::default()
                    };
                }

                if rect.contains(pos) {
                    return EventResponse {
                        cursor: Some(CursorIcon::Default),
                        ..Default::default()
                    };
                }

                EventResponse::default()
            }
            WidgetEvent::Mouse(MouseEvent::MouseScrollEvent(delta)) => {
                // Only handle scroll when the cursor is over the widget.
                // We don't have position info in scroll events, so always handle
                // if we received this event (framework dispatches to hovered widget).
                if *delta > 0.0 {
                    self.increment();
                } else if *delta < 0.0 {
                    self.decrement();
                }
                EventResponse {
                    status: EventStatus::Consumed,
                    ..Default::default()
                }
            }
            _ => EventResponse::default(),
        }
    }

    fn needs_animation(&self) -> bool {
        self.minus_anim_start.elapsed().as_secs_f32() < ANIM_DURATION
            || self.plus_anim_start.elapsed().as_secs_f32() < ANIM_DURATION
    }

    #[cfg(feature = "a11y")]
    fn access_info(&self) -> aurora_a11y::NodeInfo {
        aurora_a11y::NodeInfo::new(aurora_a11y::accesskit::Role::SpinButton)
            .with_numeric_value(self.value)
            .with_numeric_range(self.min, self.max)
            .with_numeric_step(self.step)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clamp_to_min_max() {
        let input = NumberInput::new().min(0.0).max(10.0);
        assert_eq!(input.clamp_value(-5.0), 0.0);
        assert_eq!(input.clamp_value(15.0), 10.0);
        assert_eq!(input.clamp_value(5.0), 5.0);
    }

    #[test]
    fn step_snapping() {
        let mut input = NumberInput::new().value(5.0).min(0.0).max(10.0).step(2.0);
        input.increment();
        assert!((input.value - 7.0).abs() < f64::EPSILON);
        input.decrement();
        assert!((input.value - 5.0).abs() < f64::EPSILON);
    }

    #[test]
    fn precision_formatting() {
        let input = NumberInput::new().value(7.456).precision(2);
        assert_eq!(input.format_value(), "7.46");

        let input = NumberInput::new().value(7.4567).precision(4);
        assert_eq!(input.format_value(), "7.4567");

        let input = NumberInput::new().value(42.0).precision(0);
        assert_eq!(input.format_value(), "42");
    }

    #[test]
    fn default_values() {
        let input = NumberInput::new();
        assert!((input.value - 0.0).abs() < f64::EPSILON);
        assert_eq!(input.min, -f64::MAX);
        assert_eq!(input.max, f64::MAX);
        assert!((input.step - 1.0).abs() < f64::EPSILON);
        assert_eq!(input.precision, 2);
        assert_eq!(input.height, 40.0);
        assert!(!input.disabled);
        assert!(!input.error);
        assert!(input.width.is_none());
        assert!(input.on_change.is_none());
    }

    #[test]
    fn disabled_state() {
        let input = NumberInput::new().disabled(true);
        assert!(input.disabled);

        let input = NumberInput::new().disabled(false);
        assert!(!input.disabled);
    }

    #[test]
    fn min_greater_than_max_handled() {
        // When min > max, clamp_value should still work sensibly
        let input = NumberInput::new().min(10.0).max(0.0);
        // effective_min = 0.0, effective_max = 10.0
        assert_eq!(input.clamp_value(5.0), 5.0);
        assert_eq!(input.clamp_value(-1.0), 0.0);
        assert_eq!(input.clamp_value(15.0), 10.0);
    }
}
