use crate::widgets::{EventResponse, EventStatus, LayoutCtx, Widget};
use aurora_core::color::Color;
use aurora_core::geometry::corners::Corners;
use aurora_core::geometry::rect::Rect;
use aurora_core::geometry::size::Size;
use aurora_core::kmi::WidgetEvent;
use aurora_core::kmi::cursor_icon::CursorIcon;
use aurora_core::kmi::mouse::{MouseEvent, MouseState};
use aurora_render::canvas::Canvas;

use super::colors;

/// Width of each spinner column (hour, minute, second, AM/PM).
const FIELD_WIDTH: f32 = 40.0;
/// Width of the colon separator between fields.
const SEPARATOR_WIDTH: f32 = 12.0;
/// Height of the up/down arrow hit regions.
const ARROW_HEIGHT: f32 = 10.0;

/// Identifies which time field is being interacted with.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeField {
    Hour,
    Minute,
    Second,
    AmPm,
}

/// A time selection widget with hour/minute spinners.
///
/// Displays columnar spinners for hour, minute, and optionally second.
/// Each spinner has up/down arrow regions and a numeric display.
/// Supports both 12-hour (with AM/PM toggle) and 24-hour formats.
///
/// # Example
/// ```ignore
/// TimePicker::new()
///     .hour(14)
///     .minute(30)
///     .format_24h(false)
///     .on_change(|h, m, s| println!("{h:02}:{m:02}:{s:02}"))
/// ```
pub struct TimePicker {
    hour: u8,
    minute: u8,
    second: u8,
    format_24h: bool,
    show_seconds: bool,
    width: Option<f32>,
    height: f32,
    background: Color,
    border_color: Color,
    text_color: Color,
    arrow_color: Color,
    corners: Corners,
    disabled: bool,
    #[allow(clippy::type_complexity)]
    on_change: Option<Box<dyn FnMut(u8, u8, u8)>>,
    hover_field: Option<TimeField>,
    hour_layout: Option<aurora_text::text_layout::TextLayout>,
    minute_layout: Option<aurora_text::text_layout::TextLayout>,
    second_layout: Option<aurora_text::text_layout::TextLayout>,
    separator_layout: Option<aurora_text::text_layout::TextLayout>,
    ampm_layout: Option<aurora_text::text_layout::TextLayout>,
    error: bool,
}

impl TimePicker {
    pub fn new() -> Self {
        Self {
            hour: 0,
            minute: 0,
            second: 0,
            format_24h: true,
            show_seconds: false,
            width: None,
            height: 40.0,
            background: colors::background(),
            border_color: colors::input_border(),
            text_color: colors::foreground(),
            arrow_color: colors::muted_foreground(),
            corners: Corners::all(6.0),
            disabled: false,
            on_change: None,
            hover_field: None,
            hour_layout: None,
            minute_layout: None,
            second_layout: None,
            separator_layout: None,
            ampm_layout: None,
            error: false,
        }
    }

    pub fn hour(mut self, hour: u8) -> Self {
        self.hour = hour.min(23);
        self
    }

    pub fn minute(mut self, minute: u8) -> Self {
        self.minute = minute.min(59);
        self
    }

    pub fn second(mut self, second: u8) -> Self {
        self.second = second.min(59);
        self
    }

    pub fn format_24h(mut self, format_24h: bool) -> Self {
        self.format_24h = format_24h;
        self
    }

    pub fn show_seconds(mut self, show: bool) -> Self {
        self.show_seconds = show;
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

    pub fn text_color(mut self, color: Color) -> Self {
        self.text_color = color;
        self
    }

    pub fn arrow_color(mut self, color: Color) -> Self {
        self.arrow_color = color;
        self
    }

    pub fn corners(mut self, corners: Corners) -> Self {
        self.corners = corners;
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

    pub fn on_change(mut self, cb: impl FnMut(u8, u8, u8) + 'static) -> Self {
        self.on_change = Some(Box::new(cb));
        self
    }

    /// Compute the total width needed based on visible fields.
    fn computed_width(&self) -> f32 {
        // [HH] : [MM]
        let mut w = FIELD_WIDTH * 2.0 + SEPARATOR_WIDTH;
        // (: [SS])
        if self.show_seconds {
            w += SEPARATOR_WIDTH + FIELD_WIDTH;
        }
        // (AM/PM)
        if !self.format_24h {
            w += FIELD_WIDTH + 4.0; // 4px gap before AM/PM
        }
        w
    }

    /// Returns the display string for the hour value.
    fn hour_display(&self) -> String {
        if self.format_24h {
            format!("{:02}", self.hour)
        } else {
            let h = self.display_hour_12h();
            format!("{:02}", h)
        }
    }

    /// Returns the 12-hour display value (1-12) from the internal 24h hour.
    fn display_hour_12h(&self) -> u8 {
        match self.hour {
            0 => 12,
            h if h > 12 => h - 12,
            h => h,
        }
    }

    /// Returns "AM" or "PM" based on the internal 24h hour.
    fn ampm_str(&self) -> &'static str {
        if self.hour < 12 { "AM" } else { "PM" }
    }

    /// Increment the hour, wrapping around (0-23 internally for both formats).
    fn increment_hour(&mut self) {
        self.hour = (self.hour + 1) % 24;
    }

    /// Decrement the hour, wrapping around (0-23 internally for both formats).
    fn decrement_hour(&mut self) {
        self.hour = if self.hour == 0 { 23 } else { self.hour - 1 };
    }

    /// Increment the minute, wrapping around.
    fn increment_minute(&mut self) {
        self.minute = (self.minute + 1) % 60;
    }

    /// Decrement the minute, wrapping around.
    fn decrement_minute(&mut self) {
        self.minute = if self.minute == 0 {
            59
        } else {
            self.minute - 1
        };
    }

    /// Increment the second, wrapping around.
    fn increment_second(&mut self) {
        self.second = (self.second + 1) % 60;
    }

    /// Decrement the second, wrapping around.
    fn decrement_second(&mut self) {
        self.second = if self.second == 0 {
            59
        } else {
            self.second - 1
        };
    }

    /// Toggle AM/PM by flipping 12 hours.
    fn toggle_ampm(&mut self) {
        self.hour = (self.hour + 12) % 24;
    }

    fn notify_change(&mut self) {
        if let Some(ref mut cb) = self.on_change {
            cb(self.hour, self.minute, self.second);
        }
    }

    /// Returns the rect for a given field column within the overall widget rect.
    fn field_rect(&self, field: TimeField, rect: &Rect) -> Rect {
        let x = rect.x1 + self.field_x_offset(field);
        Rect::new(x, rect.y1, x + FIELD_WIDTH, rect.y2)
    }

    /// Returns the x-offset from the widget's left edge for a given field.
    fn field_x_offset(&self, field: TimeField) -> f32 {
        match field {
            TimeField::Hour => 0.0,
            TimeField::Minute => FIELD_WIDTH + SEPARATOR_WIDTH,
            TimeField::Second => FIELD_WIDTH * 2.0 + SEPARATOR_WIDTH * 2.0,
            TimeField::AmPm => {
                let mut x = FIELD_WIDTH * 2.0 + SEPARATOR_WIDTH;
                if self.show_seconds {
                    x += SEPARATOR_WIDTH + FIELD_WIDTH;
                }
                x + 4.0 // 4px gap
            }
        }
    }

    /// Returns the rect for the up-arrow hit region of a field.
    fn up_arrow_rect(&self, field: TimeField, rect: &Rect) -> Rect {
        let fr = self.field_rect(field, rect);
        Rect::new(fr.x1, fr.y1, fr.x2, fr.y1 + ARROW_HEIGHT)
    }

    /// Returns the rect for the down-arrow hit region of a field.
    fn down_arrow_rect(&self, field: TimeField, rect: &Rect) -> Rect {
        let fr = self.field_rect(field, rect);
        Rect::new(fr.x1, fr.y2 - ARROW_HEIGHT, fr.x2, fr.y2)
    }

    /// Determines which field, if any, the given x coordinate falls in.
    fn field_at_x(&self, x: f32, rect: &Rect) -> Option<TimeField> {
        let fields = self.visible_fields();
        for field in fields {
            let fr = self.field_rect(field, rect);
            if x >= fr.x1 && x < fr.x2 {
                return Some(field);
            }
        }
        None
    }

    /// Returns the list of visible fields based on configuration.
    fn visible_fields(&self) -> Vec<TimeField> {
        let mut fields = vec![TimeField::Hour, TimeField::Minute];
        if self.show_seconds {
            fields.push(TimeField::Second);
        }
        if !self.format_24h {
            fields.push(TimeField::AmPm);
        }
        fields
    }

    /// Increment the given field.
    fn increment_field(&mut self, field: TimeField) {
        match field {
            TimeField::Hour => self.increment_hour(),
            TimeField::Minute => self.increment_minute(),
            TimeField::Second => self.increment_second(),
            TimeField::AmPm => self.toggle_ampm(),
        }
        self.notify_change();
    }

    /// Decrement the given field.
    fn decrement_field(&mut self, field: TimeField) {
        match field {
            TimeField::Hour => self.decrement_hour(),
            TimeField::Minute => self.decrement_minute(),
            TimeField::Second => self.decrement_second(),
            TimeField::AmPm => self.toggle_ampm(),
        }
        self.notify_change();
    }

    /// Paint a small up-arrow triangle centered in the given rect.
    fn paint_up_arrow(canvas: &mut Canvas, rect: Rect, color: Color) {
        let cx = rect.x1 + rect.width() / 2.0;
        let top_y = rect.y1 + 2.0;
        let bot_y = rect.y2 - 1.0;
        let half_w = 4.0;
        // Draw as three horizontal lines to approximate a triangle
        let steps = ((bot_y - top_y) as i32).max(1);
        for i in 0..steps {
            let t = i as f32 / steps as f32;
            let hw = half_w * (1.0 - t);
            let y = bot_y - i as f32;
            if hw > 0.3 {
                canvas.fill_rect(Rect::new(cx - hw, y, cx + hw, y + 1.0), color);
            }
        }
    }

    /// Paint a small down-arrow triangle centered in the given rect.
    fn paint_down_arrow(canvas: &mut Canvas, rect: Rect, color: Color) {
        let cx = rect.x1 + rect.width() / 2.0;
        let top_y = rect.y1 + 1.0;
        let bot_y = rect.y2 - 2.0;
        let half_w = 4.0;
        let steps = ((bot_y - top_y) as i32).max(1);
        for i in 0..steps {
            let t = i as f32 / steps as f32;
            let hw = half_w * (1.0 - t);
            let y = top_y + i as f32;
            if hw > 0.3 {
                canvas.fill_rect(Rect::new(cx - hw, y, cx + hw, y + 1.0), color);
            }
        }
    }
}

impl Default for TimePicker {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for TimePicker {
    fn layout(&mut self, available: Size, ctx: &mut LayoutCtx) -> Size {
        let w = self.width.unwrap_or_else(|| self.computed_width());

        let mut opts = ctx.font_options.clone();
        opts.size = Some(14.0);
        opts.weight = Some(aurora_text::font_options::FontWeight::Normal);

        // Layout hour text
        let hour_text = self.hour_display();
        self.hour_layout = Some(aurora_text::text_layout::TextLayout::new(
            ctx.font_manager,
            &hour_text,
            &opts,
            self.text_color,
            None,
        ));

        // Layout minute text
        let minute_text = format!("{:02}", self.minute);
        self.minute_layout = Some(aurora_text::text_layout::TextLayout::new(
            ctx.font_manager,
            &minute_text,
            &opts,
            self.text_color,
            None,
        ));

        // Layout second text
        if self.show_seconds {
            let second_text = format!("{:02}", self.second);
            self.second_layout = Some(aurora_text::text_layout::TextLayout::new(
                ctx.font_manager,
                &second_text,
                &opts,
                self.text_color,
                None,
            ));
        } else {
            self.second_layout = None;
        }

        // Layout separator ":"
        self.separator_layout = Some(aurora_text::text_layout::TextLayout::new(
            ctx.font_manager,
            ":",
            &opts,
            self.text_color,
            None,
        ));

        // Layout AM/PM text
        if !self.format_24h {
            let ampm_text = self.ampm_str();
            let mut ampm_opts = opts.clone();
            ampm_opts.size = Some(12.0);
            ampm_opts.weight = Some(aurora_text::font_options::FontWeight::Medium);
            self.ampm_layout = Some(aurora_text::text_layout::TextLayout::new(
                ctx.font_manager,
                ampm_text,
                &ampm_opts,
                self.text_color,
                None,
            ));
        } else {
            self.ampm_layout = None;
        }

        Size::new(w.max(available.width.min(w)), self.height)
    }

    fn paint(&self, canvas: &mut Canvas, rect: Rect) {
        // Background
        canvas.fill_rounded_rect(rect, self.corners, self.background);

        // Border (error > hover > default)
        let border_color = if self.error {
            colors::destructive()
        } else {
            self.border_color
        };
        canvas.stroke_rounded_rect(rect, self.corners, 1, border_color);

        let arrow_color = self.arrow_color;
        let center_y = rect.y1 + (rect.height() - 14.0) / 2.0;

        // Paint hour field
        let hour_fr = self.field_rect(TimeField::Hour, &rect);
        Self::paint_up_arrow(
            canvas,
            self.up_arrow_rect(TimeField::Hour, &rect),
            arrow_color,
        );
        Self::paint_down_arrow(
            canvas,
            self.down_arrow_rect(TimeField::Hour, &rect),
            arrow_color,
        );
        if let Some(ref tl) = self.hour_layout {
            let ts = tl.size();
            let tx = hour_fr.x1 + (FIELD_WIDTH - ts.width) / 2.0;
            canvas.draw_text(tl, tx as i32, center_y as i32);
        }

        // Paint first separator ":"
        if let Some(ref tl) = self.separator_layout {
            let ts = tl.size();
            let sep_x = hour_fr.x2 + (SEPARATOR_WIDTH - ts.width) / 2.0;
            canvas.draw_text(tl, sep_x as i32, center_y as i32);
        }

        // Paint minute field
        let minute_fr = self.field_rect(TimeField::Minute, &rect);
        Self::paint_up_arrow(
            canvas,
            self.up_arrow_rect(TimeField::Minute, &rect),
            arrow_color,
        );
        Self::paint_down_arrow(
            canvas,
            self.down_arrow_rect(TimeField::Minute, &rect),
            arrow_color,
        );
        if let Some(ref tl) = self.minute_layout {
            let ts = tl.size();
            let tx = minute_fr.x1 + (FIELD_WIDTH - ts.width) / 2.0;
            canvas.draw_text(tl, tx as i32, center_y as i32);
        }

        // Paint second separator and second field
        if self.show_seconds {
            if let Some(ref tl) = self.separator_layout {
                let ts = tl.size();
                let sep_x = minute_fr.x2 + (SEPARATOR_WIDTH - ts.width) / 2.0;
                canvas.draw_text(tl, sep_x as i32, center_y as i32);
            }

            let second_fr = self.field_rect(TimeField::Second, &rect);
            Self::paint_up_arrow(
                canvas,
                self.up_arrow_rect(TimeField::Second, &rect),
                arrow_color,
            );
            Self::paint_down_arrow(
                canvas,
                self.down_arrow_rect(TimeField::Second, &rect),
                arrow_color,
            );
            if let Some(ref tl) = self.second_layout {
                let ts = tl.size();
                let tx = second_fr.x1 + (FIELD_WIDTH - ts.width) / 2.0;
                canvas.draw_text(tl, tx as i32, center_y as i32);
            }
        }

        // Paint AM/PM toggle
        if !self.format_24h {
            let ampm_fr = self.field_rect(TimeField::AmPm, &rect);
            if let Some(ref tl) = self.ampm_layout {
                let ts = tl.size();
                let tx = ampm_fr.x1 + (FIELD_WIDTH - ts.width) / 2.0;
                let ty = rect.y1 + (rect.height() - ts.height) / 2.0;
                canvas.draw_text(tl, tx as i32, ty as i32);
            }
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

        match event {
            WidgetEvent::Mouse(MouseEvent::MouseClickEvent(e))
                if e.state == MouseState::Pressed =>
            {
                if !rect.contains(&e.position) {
                    return EventResponse::default();
                }

                // Check each visible field for up/down arrow clicks
                for field in self.visible_fields() {
                    if field == TimeField::AmPm {
                        // AM/PM: any click toggles
                        let fr = self.field_rect(field, &rect);
                        if fr.contains(&e.position) {
                            self.toggle_ampm();
                            self.notify_change();
                            return EventResponse {
                                status: EventStatus::Consumed,
                                cursor: Some(CursorIcon::Pointer),
                                ..Default::default()
                            };
                        }
                        continue;
                    }

                    let up_rect = self.up_arrow_rect(field, &rect);
                    if up_rect.contains(&e.position) {
                        self.increment_field(field);
                        return EventResponse {
                            status: EventStatus::Consumed,
                            cursor: Some(CursorIcon::Pointer),
                            ..Default::default()
                        };
                    }

                    let down_rect = self.down_arrow_rect(field, &rect);
                    if down_rect.contains(&e.position) {
                        self.decrement_field(field);
                        return EventResponse {
                            status: EventStatus::Consumed,
                            cursor: Some(CursorIcon::Pointer),
                            ..Default::default()
                        };
                    }
                }

                EventResponse {
                    status: EventStatus::Consumed,
                    cursor: Some(CursorIcon::Default),
                    ..Default::default()
                }
            }
            WidgetEvent::Mouse(MouseEvent::MouseMoveEvent(pos)) => {
                if rect.contains(pos) {
                    // Track which field is hovered
                    self.hover_field = self.field_at_x(pos.x, &rect);
                    return EventResponse {
                        cursor: Some(CursorIcon::Pointer),
                        ..Default::default()
                    };
                }
                self.hover_field = None;
                EventResponse::default()
            }
            WidgetEvent::Mouse(MouseEvent::MouseScrollEvent(delta)) => {
                // Scroll wheel changes the hovered field
                if let Some(field) = self.hover_field {
                    if *delta > 0.0 {
                        self.increment_field(field);
                    } else if *delta < 0.0 {
                        self.decrement_field(field);
                    }
                    return EventResponse {
                        status: EventStatus::Consumed,
                        ..Default::default()
                    };
                }
                EventResponse::default()
            }
            _ => EventResponse::default(),
        }
    }

    #[cfg(feature = "a11y")]
    fn access_info(&self) -> aurora_a11y::NodeInfo {
        aurora_a11y::NodeInfo::new(aurora_a11y::accesskit::Role::Group)
            .with_label("Time picker".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_values() {
        let tp = TimePicker::new();
        assert_eq!(tp.hour, 0);
        assert_eq!(tp.minute, 0);
        assert_eq!(tp.second, 0);
        assert!(tp.format_24h);
        assert!(!tp.show_seconds);
        assert_eq!(tp.height, 40.0);
        assert!(!tp.disabled);
        assert!(!tp.error);
        assert!(tp.width.is_none());
        assert!(tp.on_change.is_none());
        assert!(tp.hover_field.is_none());
    }

    #[test]
    fn wrap_hour_24h() {
        let mut tp = TimePicker::new().hour(23);
        assert_eq!(tp.hour, 23);
        tp.increment_hour();
        assert_eq!(tp.hour, 0);
        tp.decrement_hour();
        assert_eq!(tp.hour, 23);
    }

    #[test]
    fn wrap_hour_12h() {
        // In 12h mode, internal storage is still 0-23.
        // Starting at hour=0 (12 AM), incrementing goes to 1 (1 AM).
        let mut tp = TimePicker::new().format_24h(false);
        // format_24h(false) converts hour 0 to 12 for initial display,
        // but internal remains 0 if not explicitly set.
        // After format_24h(false), hour=0 is kept internally (displays as 12 AM).
        assert_eq!(tp.hour, 0);
        assert_eq!(tp.display_hour_12h(), 12);
        assert_eq!(tp.ampm_str(), "AM");

        // Wrap from 23 -> 0
        tp.hour = 23;
        tp.increment_hour();
        assert_eq!(tp.hour, 0);

        // Wrap from 0 -> 23
        tp.decrement_hour();
        assert_eq!(tp.hour, 23);
    }

    #[test]
    fn wrap_minute() {
        let mut tp = TimePicker::new().minute(59);
        assert_eq!(tp.minute, 59);
        tp.increment_minute();
        assert_eq!(tp.minute, 0);
        tp.decrement_minute();
        assert_eq!(tp.minute, 59);
    }

    #[test]
    fn am_pm_toggle() {
        let mut tp = TimePicker::new().format_24h(false).hour(9);
        assert_eq!(tp.hour, 9);
        assert_eq!(tp.ampm_str(), "AM");
        assert_eq!(tp.display_hour_12h(), 9);

        tp.toggle_ampm();
        assert_eq!(tp.hour, 21);
        assert_eq!(tp.ampm_str(), "PM");
        assert_eq!(tp.display_hour_12h(), 9);

        tp.toggle_ampm();
        assert_eq!(tp.hour, 9);
        assert_eq!(tp.ampm_str(), "AM");
    }

    #[test]
    fn format_display() {
        let tp = TimePicker::new().hour(14).minute(5);
        assert_eq!(tp.hour_display(), "14");
        assert_eq!(format!("{:02}", tp.minute), "05");

        let tp = TimePicker::new().format_24h(false).hour(14).minute(30);
        // 14 in 12h mode displays as 02 (PM)
        assert_eq!(tp.display_hour_12h(), 2);
        assert_eq!(tp.hour_display(), "02");
        assert_eq!(tp.ampm_str(), "PM");

        let tp = TimePicker::new().format_24h(false).hour(0);
        assert_eq!(tp.display_hour_12h(), 12);
        assert_eq!(tp.hour_display(), "12");
        assert_eq!(tp.ampm_str(), "AM");
    }

    #[test]
    fn wrap_second() {
        let mut tp = TimePicker::new().show_seconds(true).second(59);
        assert_eq!(tp.second, 59);
        tp.increment_second();
        assert_eq!(tp.second, 0);
        tp.decrement_second();
        assert_eq!(tp.second, 59);
    }

    #[test]
    fn computed_width_varies_by_config() {
        let tp24 = TimePicker::new();
        let tp12 = TimePicker::new().format_24h(false);
        let tp_sec = TimePicker::new().show_seconds(true);
        let tp_full = TimePicker::new().format_24h(false).show_seconds(true);

        // 24h: 2 fields + 1 separator
        assert!(tp24.computed_width() < tp12.computed_width());
        // with seconds adds another field + separator
        assert!(tp_sec.computed_width() > tp24.computed_width());
        // full has all fields
        assert!(tp_full.computed_width() > tp_sec.computed_width());
        assert!(tp_full.computed_width() > tp12.computed_width());
    }
}
