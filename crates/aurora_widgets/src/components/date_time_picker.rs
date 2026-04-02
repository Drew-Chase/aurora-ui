use crate::widgets::{EventResponse, EventStatus, LayoutCtx, Widget};
use aurora_core::color::Color;
use aurora_core::geometry::corners::Corners;
use aurora_core::geometry::edges::Edges;
use aurora_core::geometry::rect::Rect;
use aurora_core::geometry::size::Size;
use aurora_core::kmi::WidgetEvent;
use aurora_core::kmi::cursor_icon::CursorIcon;
use aurora_core::kmi::mouse::{MouseEvent, MouseState};
use aurora_render::canvas::Canvas;
use std::time::Instant;

use super::calendar::Calendar;
use super::colors;

// ─── Animation helpers ──────────────────────────────────────────────────────

const ANIM_DURATION: f32 = 0.20;
const TIME_SPINNER_H: f32 = 80.0;
const TIME_SECTION_PADDING: f32 = 16.0;

fn ease_out_cubic(t: f32) -> f32 {
    1.0 - (1.0 - t).powi(3)
}

fn anim_t(from: f32, to: f32, start: Instant, duration: f32) -> f32 {
    let elapsed = start.elapsed().as_secs_f32();
    let raw = (elapsed / duration).min(1.0);
    let eased = ease_out_cubic(raw);
    from + (to - from) * eased
}

// ─── Date/time helpers ──────────────────────────────────────────────────────

/// Returns the number of days in a given month.
fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) {
                29
            } else {
                28
            }
        }
        _ => 30,
    }
}

/// Returns the day of the week for a given date.
/// 0 = Sunday, 1 = Monday, ..., 6 = Saturday.
/// Uses Tomohiko Sakamoto's algorithm.
#[cfg(test)]
fn day_of_week(year: i32, month: u32, day: u32) -> u32 {
    let t = [0i64, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];
    let y = if month <= 2 {
        year as i64 - 1
    } else {
        year as i64
    };
    let m = month as usize;
    let d = day as i64;
    ((d + t[m - 1] + y + y / 4 - y / 100 + y / 400) % 7) as u32
}

#[allow(clippy::too_many_arguments)]
fn format_display(
    year: i32,
    month: u32,
    day: u32,
    hour: u8,
    minute: u8,
    second: u8,
    show_seconds: bool,
    format_24h: bool,
) -> String {
    if format_24h {
        if show_seconds {
            format!(
                "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
                year, month, day, hour, minute, second
            )
        } else {
            format!(
                "{:04}-{:02}-{:02} {:02}:{:02}",
                year, month, day, hour, minute
            )
        }
    } else {
        let (h12, ampm) = if hour == 0 {
            (12, "AM")
        } else if hour < 12 {
            (hour, "AM")
        } else if hour == 12 {
            (12, "PM")
        } else {
            (hour - 12, "PM")
        };
        if show_seconds {
            format!(
                "{:04}-{:02}-{:02} {:02}:{:02}:{:02} {}",
                year, month, day, h12, minute, second, ampm
            )
        } else {
            format!(
                "{:04}-{:02}-{:02} {:02}:{:02} {}",
                year, month, day, h12, minute, ampm
            )
        }
    }
}

// ─── DateTimePicker widget ──────────────────────────────────────────────────

/// A combined date and time picker that shows a trigger button displaying
/// "YYYY-MM-DD HH:MM" text. When clicked, opens an overlay dropdown with a
/// calendar (month view) and time spinners below.
///
/// # Example
/// ```ignore
/// DateTimePicker::new()
///     .year(2026)
///     .month(4)
///     .day(1)
///     .hour(14)
///     .minute(30)
///     .on_change(|y, mo, d, h, mi, s| {
///         println!("DateTime: {y}-{mo}-{d} {h}:{mi}:{s}");
///     })
/// ```
pub struct DateTimePicker {
    year: i32,
    month: u32,
    day: u32,
    hour: u8,
    minute: u8,
    second: u8,
    format_24h: bool,
    show_seconds: bool,
    open: bool,
    width: Option<f32>,
    height: f32,
    background: Color,
    border_color: Color,
    corners: Corners,
    padding: Edges,
    #[allow(clippy::type_complexity)]
    on_change: Option<Box<dyn FnMut(i32, u32, u32, u8, u8, u8)>>,
    display_layout: Option<aurora_text::text_layout::TextLayout>,
    anim_from: f32,
    anim_to: f32,
    anim_start: Instant,
    // Calendar state
    view_year: i32,
    view_month: u32,
    hover_day: Option<u32>,
    // Time state
    hover_time_field: Option<u8>, // 0=hour, 1=min, 2=sec
    // Cached text layouts
    month_label_layout: Option<aurora_text::text_layout::TextLayout>,
    day_layouts: Vec<Option<aurora_text::text_layout::TextLayout>>,
    time_layouts: Vec<Option<aurora_text::text_layout::TextLayout>>,
    weekday_layouts: Vec<Option<aurora_text::text_layout::TextLayout>>,
    prev_layout: Option<aurora_text::text_layout::TextLayout>,
    next_layout: Option<aurora_text::text_layout::TextLayout>,
    // Time spinner arrow layouts
    hour_up_layout: Option<aurora_text::text_layout::TextLayout>,
    hour_down_layout: Option<aurora_text::text_layout::TextLayout>,
    min_up_layout: Option<aurora_text::text_layout::TextLayout>,
    min_down_layout: Option<aurora_text::text_layout::TextLayout>,
    sec_up_layout: Option<aurora_text::text_layout::TextLayout>,
    sec_down_layout: Option<aurora_text::text_layout::TextLayout>,
    colon_layout: Option<aurora_text::text_layout::TextLayout>,
    // Computed calendar data
    computed_days_in_month: u32,
    computed_first_weekday: u32,
    calendar_width: f32,
    error: bool,
}

impl DateTimePicker {
    pub fn new() -> Self {
        let now = Instant::now();
        Self {
            year: 2026,
            month: 1,
            day: 1,
            hour: 0,
            minute: 0,
            second: 0,
            format_24h: true,
            show_seconds: false,
            open: false,
            width: None,
            height: 40.0,
            background: colors::background(),
            border_color: colors::input_border(),
            corners: Corners::all(6.0),
            padding: Edges::new(0.0, 12.0, 0.0, 12.0),
            on_change: None,
            display_layout: None,
            anim_from: 0.0,
            anim_to: 0.0,
            anim_start: now,
            view_year: 2026,
            view_month: 1,
            hover_day: None,
            hover_time_field: None,
            month_label_layout: None,
            day_layouts: Vec::new(),
            time_layouts: Vec::new(),
            weekday_layouts: Vec::new(),
            prev_layout: None,
            next_layout: None,
            hour_up_layout: None,
            hour_down_layout: None,
            min_up_layout: None,
            min_down_layout: None,
            sec_up_layout: None,
            sec_down_layout: None,
            colon_layout: None,
            computed_days_in_month: 31,
            computed_first_weekday: 0,
            calendar_width: 280.0,
            error: false,
        }
    }

    // ── Builder methods ─────────────────────────────────────────────────

    pub fn year(mut self, year: i32) -> Self {
        self.year = year;
        self.view_year = year;
        self
    }

    pub fn month(mut self, month: u32) -> Self {
        self.month = month.clamp(1, 12);
        self.view_month = self.month;
        self
    }

    pub fn day(mut self, day: u32) -> Self {
        self.day = day.clamp(1, 31);
        self
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

    pub fn format_24h(mut self, v: bool) -> Self {
        self.format_24h = v;
        self
    }

    pub fn show_seconds(mut self, v: bool) -> Self {
        self.show_seconds = v;
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

    pub fn on_change(mut self, cb: impl FnMut(i32, u32, u32, u8, u8, u8) + 'static) -> Self {
        self.on_change = Some(Box::new(cb));
        self
    }

    pub fn error(mut self, v: bool) -> Self {
        self.error = v;
        self
    }

    // ── Internal helpers ────────────────────────────────────────────────

    fn anim_opacity(&self) -> f32 {
        anim_t(self.anim_from, self.anim_to, self.anim_start, ANIM_DURATION)
    }

    fn dropdown_rect(&self, trigger_rect: &Rect) -> Rect {
        let w = self.calendar_width + 16.0; // 8px padding on each side
        let time_h = TIME_SECTION_PADDING + TIME_SPINNER_H + TIME_SECTION_PADDING;
        let rows = self.total_rows();
        let cal_h = 40.0 + 32.0 + rows as f32 * 32.0; // header + weekday row + day grid
        let h = 8.0 + cal_h + time_h + 8.0;
        Rect::new(
            trigger_rect.x1,
            trigger_rect.y2 + 4.0,
            trigger_rect.x1 + w,
            trigger_rect.y2 + 4.0 + h,
        )
    }

    fn total_rows(&self) -> u32 {
        (self.computed_first_weekday + self.computed_days_in_month).div_ceil(7)
    }

    fn day_at_position(
        &self,
        pos: &aurora_core::geometry::point::Point,
        grid_rect: &Rect,
    ) -> Option<u32> {
        let cell_w = self.calendar_width / 7.0;
        let cell_h = 32.0;
        let lx = pos.x - grid_rect.x1;
        let ly = pos.y - grid_rect.y1;
        if lx < 0.0 || ly < 0.0 {
            return None;
        }
        let col = (lx / cell_w) as u32;
        let row = (ly / cell_h) as u32;
        if col >= 7 {
            return None;
        }
        let cell_idx = row * 7 + col;
        if cell_idx < self.computed_first_weekday {
            return None;
        }
        let day = cell_idx - self.computed_first_weekday + 1;
        if (1..=self.computed_days_in_month).contains(&day) {
            Some(day)
        } else {
            None
        }
    }

    fn fire_on_change(&mut self) {
        if let Some(ref mut cb) = self.on_change {
            cb(
                self.year,
                self.month,
                self.day,
                self.hour,
                self.minute,
                self.second,
            );
        }
    }

    fn clamp_day(&mut self) {
        let max_day = days_in_month(self.year, self.month);
        if self.day > max_day {
            self.day = max_day;
        }
    }

    /// Compute rects for time spinner buttons within the dropdown.
    /// Returns (hour_up, hour_down, min_up, min_down, sec_up, sec_down, hour_val_rect, min_val_rect, sec_val_rect)
    fn time_rects(&self, dr: &Rect) -> TimeRects {
        let rows = self.total_rows();
        let cal_h = 40.0 + 32.0 + rows as f32 * 32.0;
        let time_y = dr.y1 + 8.0 + cal_h + TIME_SECTION_PADDING;

        let spinner_w = 60.0;
        let arrow_h = 20.0;
        let val_h = TIME_SPINNER_H - arrow_h * 2.0;
        let colon_w = 12.0;

        // Center the time spinners horizontally
        let total_spinners = if self.show_seconds { 3 } else { 2 };
        let total_w = spinner_w * total_spinners as f32 + colon_w * (total_spinners - 1) as f32;
        let start_x = dr.x1 + (dr.width() - total_w) / 2.0;

        let hour_x = start_x;
        let min_x = hour_x + spinner_w + colon_w;
        let sec_x = min_x + spinner_w + colon_w;

        TimeRects {
            hour_up: Rect::new(hour_x, time_y, hour_x + spinner_w, time_y + arrow_h),
            hour_val: Rect::new(
                hour_x,
                time_y + arrow_h,
                hour_x + spinner_w,
                time_y + arrow_h + val_h,
            ),
            hour_down: Rect::new(
                hour_x,
                time_y + arrow_h + val_h,
                hour_x + spinner_w,
                time_y + TIME_SPINNER_H,
            ),
            min_up: Rect::new(min_x, time_y, min_x + spinner_w, time_y + arrow_h),
            min_val: Rect::new(
                min_x,
                time_y + arrow_h,
                min_x + spinner_w,
                time_y + arrow_h + val_h,
            ),
            min_down: Rect::new(
                min_x,
                time_y + arrow_h + val_h,
                min_x + spinner_w,
                time_y + TIME_SPINNER_H,
            ),
            sec_up: Rect::new(sec_x, time_y, sec_x + spinner_w, time_y + arrow_h),
            sec_val: Rect::new(
                sec_x,
                time_y + arrow_h,
                sec_x + spinner_w,
                time_y + arrow_h + val_h,
            ),
            sec_down: Rect::new(
                sec_x,
                time_y + arrow_h + val_h,
                sec_x + spinner_w,
                time_y + TIME_SPINNER_H,
            ),
            colon1_x: hour_x + spinner_w,
            colon2_x: min_x + spinner_w,
            colon_w,
        }
    }

    fn header_rect(&self, dr: &Rect) -> Rect {
        Rect::new(dr.x1 + 8.0, dr.y1 + 8.0, dr.x2 - 8.0, dr.y1 + 8.0 + 40.0)
    }

    fn prev_arrow_rect(&self, dr: &Rect) -> Rect {
        let hr = self.header_rect(dr);
        Rect::new(hr.x1, hr.y1, hr.x1 + 32.0, hr.y2)
    }

    fn next_arrow_rect(&self, dr: &Rect) -> Rect {
        let hr = self.header_rect(dr);
        Rect::new(hr.x2 - 32.0, hr.y1, hr.x2, hr.y2)
    }

    fn grid_rect(&self, dr: &Rect) -> Rect {
        let rows = self.total_rows();
        let gy = dr.y1 + 8.0 + 40.0 + 32.0;
        Rect::new(
            dr.x1 + 8.0,
            gy,
            dr.x1 + 8.0 + self.calendar_width,
            gy + rows as f32 * 32.0,
        )
    }
}

impl Default for DateTimePicker {
    fn default() -> Self {
        Self::new()
    }
}

struct TimeRects {
    hour_up: Rect,
    hour_val: Rect,
    hour_down: Rect,
    min_up: Rect,
    min_val: Rect,
    min_down: Rect,
    sec_up: Rect,
    sec_val: Rect,
    sec_down: Rect,
    colon1_x: f32,
    colon2_x: f32,
    colon_w: f32,
}

impl Widget for DateTimePicker {
    fn layout(&mut self, available: Size, ctx: &mut LayoutCtx) -> Size {
        let w = self.width.unwrap_or(available.width);

        // Compute calendar data for current view
        self.computed_days_in_month = days_in_month(self.view_year, self.view_month);
        self.computed_first_weekday =
            Calendar::first_day_of_week(self.view_year as u32, self.view_month);
        self.calendar_width = 32.0 * 7.0; // 7 columns x 32px

        // Display text
        let display_text = format_display(
            self.year,
            self.month,
            self.day,
            self.hour,
            self.minute,
            self.second,
            self.show_seconds,
            self.format_24h,
        );

        let mut opts = ctx.font_options.clone();
        opts.size = Some(14.0);
        opts.weight = Some(aurora_text::font_options::FontWeight::Normal);
        let inner_w = w - self.padding.left - self.padding.right;
        let mut tl = aurora_text::text_layout::TextLayout::new(
            ctx.font_manager,
            &display_text,
            &opts,
            colors::foreground(),
            None,
        );
        tl.set_max_width(ctx.font_manager, inner_w.max(0.0));
        self.display_layout = Some(tl);

        // Layout calendar and time section text if open
        if self.open {
            // Month label
            let month_name = Calendar::month_name(self.view_month);
            let label_text = format!("{} {}", month_name, self.view_year);
            let mut lo = ctx.font_options.clone();
            lo.size = Some(14.0);
            lo.weight = Some(aurora_text::font_options::FontWeight::SemiBold);
            self.month_label_layout = Some(aurora_text::text_layout::TextLayout::new(
                ctx.font_manager,
                &label_text,
                &lo,
                colors::foreground(),
                None,
            ));

            // Nav arrows
            let mut arrow_opts = ctx.font_options.clone();
            arrow_opts.size = Some(14.0);
            arrow_opts.weight = Some(aurora_text::font_options::FontWeight::Normal);
            self.prev_layout = Some(aurora_text::text_layout::TextLayout::new(
                ctx.font_manager,
                "<",
                &arrow_opts,
                colors::foreground(),
                None,
            ));
            self.next_layout = Some(aurora_text::text_layout::TextLayout::new(
                ctx.font_manager,
                ">",
                &arrow_opts,
                colors::foreground(),
                None,
            ));

            // Weekday headers
            self.weekday_layouts.clear();
            let mut wo = ctx.font_options.clone();
            wo.size = Some(12.0);
            wo.weight = Some(aurora_text::font_options::FontWeight::Normal);
            for name in &["Su", "Mo", "Tu", "We", "Th", "Fr", "Sa"] {
                self.weekday_layouts
                    .push(Some(aurora_text::text_layout::TextLayout::new(
                        ctx.font_manager,
                        name,
                        &wo,
                        colors::muted_foreground(),
                        None,
                    )));
            }

            // Day number layouts
            self.day_layouts.clear();
            let mut day_opts = ctx.font_options.clone();
            day_opts.size = Some(14.0);
            day_opts.weight = Some(aurora_text::font_options::FontWeight::Normal);
            for d in 1..=self.computed_days_in_month {
                let fg = if self.view_year == self.year
                    && self.view_month == self.month
                    && d == self.day
                {
                    colors::primary_foreground()
                } else {
                    colors::foreground()
                };
                self.day_layouts
                    .push(Some(aurora_text::text_layout::TextLayout::new(
                        ctx.font_manager,
                        &d.to_string(),
                        &day_opts,
                        fg,
                        None,
                    )));
            }

            // Time spinner layouts
            self.time_layouts.clear();
            let mut to = ctx.font_options.clone();
            to.size = Some(14.0);
            to.weight = Some(aurora_text::font_options::FontWeight::SemiBold);
            // hour display
            self.time_layouts
                .push(Some(aurora_text::text_layout::TextLayout::new(
                    ctx.font_manager,
                    &format!("{:02}", self.hour),
                    &to,
                    colors::foreground(),
                    None,
                )));
            // minute display
            self.time_layouts
                .push(Some(aurora_text::text_layout::TextLayout::new(
                    ctx.font_manager,
                    &format!("{:02}", self.minute),
                    &to,
                    colors::foreground(),
                    None,
                )));
            // second display (if shown)
            if self.show_seconds {
                self.time_layouts
                    .push(Some(aurora_text::text_layout::TextLayout::new(
                        ctx.font_manager,
                        &format!("{:02}", self.second),
                        &to,
                        colors::foreground(),
                        None,
                    )));
            }

            // Arrow layouts for time spinners
            let mut small_opts = ctx.font_options.clone();
            small_opts.size = Some(10.0);
            small_opts.weight = Some(aurora_text::font_options::FontWeight::Normal);
            let arrow_color = colors::muted_foreground();

            self.hour_up_layout = Some(aurora_text::text_layout::TextLayout::new(
                ctx.font_manager,
                "\u{25B2}",
                &small_opts,
                arrow_color,
                None,
            ));
            self.hour_down_layout = Some(aurora_text::text_layout::TextLayout::new(
                ctx.font_manager,
                "\u{25BC}",
                &small_opts,
                arrow_color,
                None,
            ));
            self.min_up_layout = Some(aurora_text::text_layout::TextLayout::new(
                ctx.font_manager,
                "\u{25B2}",
                &small_opts,
                arrow_color,
                None,
            ));
            self.min_down_layout = Some(aurora_text::text_layout::TextLayout::new(
                ctx.font_manager,
                "\u{25BC}",
                &small_opts,
                arrow_color,
                None,
            ));
            if self.show_seconds {
                self.sec_up_layout = Some(aurora_text::text_layout::TextLayout::new(
                    ctx.font_manager,
                    "\u{25B2}",
                    &small_opts,
                    arrow_color,
                    None,
                ));
                self.sec_down_layout = Some(aurora_text::text_layout::TextLayout::new(
                    ctx.font_manager,
                    "\u{25BC}",
                    &small_opts,
                    arrow_color,
                    None,
                ));
            }

            // Colon layout
            self.colon_layout = Some(aurora_text::text_layout::TextLayout::new(
                ctx.font_manager,
                ":",
                &to,
                colors::foreground(),
                None,
            ));
        }

        Size::new(w, self.height)
    }

    fn paint(&self, canvas: &mut Canvas, rect: Rect) {
        // Trigger input box
        let border_color = if self.error {
            colors::destructive()
        } else if self.open {
            colors::ring()
        } else {
            self.border_color
        };
        canvas.fill_rounded_rect(rect, self.corners, self.background);
        canvas.stroke_rounded_rect(rect, self.corners, 1, border_color);

        // Display text
        if let Some(ref tl) = self.display_layout {
            let th = tl.size().height;
            let tx = rect.x1 + self.padding.left;
            let ty = rect.y1 + (rect.height() - th) / 2.0;
            canvas.draw_text(tl, tx as i32, ty as i32);
        }

        // Calendar + clock icon
        let icon_x = rect.x2 - self.padding.right - 16.0;
        let icon_y = rect.y1 + (rect.height() - 16.0) / 2.0;
        canvas.stroke_rect(
            Rect::new(icon_x, icon_y + 2.0, icon_x + 14.0, icon_y + 14.0),
            1u32,
            colors::muted_foreground(),
        );
        canvas.fill_rect(
            Rect::new(icon_x, icon_y + 6.0, icon_x + 14.0, icon_y + 7.0),
            colors::muted_foreground(),
        );
    }

    fn paint_overlay(&self, canvas: &mut Canvas, rect: Rect) {
        if !self.open {
            return;
        }

        let opacity = self.anim_opacity();
        if opacity <= 0.0 {
            return;
        }

        let dr = self.dropdown_rect(&rect);
        let drop_corners = Corners::all(8.0);

        // Dropdown background
        canvas.fill_rounded_rect(dr, drop_corners, colors::popover());
        canvas.stroke_rounded_rect(dr, drop_corners, 1, colors::border());

        let inner_x = dr.x1 + 8.0;
        let mut cy = dr.y1 + 8.0;

        // ── Header: < Month Year > ──
        let header_h = 40.0;
        let header_rect = Rect::new(inner_x, cy, dr.x2 - 8.0, cy + header_h);

        // Prev arrow
        if let Some(ref tl) = self.prev_layout {
            let ax = header_rect.x1 + 8.0;
            let ay = header_rect.y1 + (header_h - tl.size().height) / 2.0;
            let hover_rect = Rect::new(
                header_rect.x1,
                header_rect.y1,
                header_rect.x1 + 32.0,
                header_rect.y2,
            );
            canvas.fill_rounded_rect(hover_rect, Corners::all(4.0), Color::TRANSPARENT);
            canvas.draw_text(tl, ax as i32, ay as i32);
        }

        // Next arrow
        if let Some(ref tl) = self.next_layout {
            let ax = header_rect.x2 - 24.0;
            let ay = header_rect.y1 + (header_h - tl.size().height) / 2.0;
            canvas.draw_text(tl, ax as i32, ay as i32);
        }

        // Month label centered
        if let Some(ref tl) = self.month_label_layout {
            let lw = tl.size().width;
            let lx = header_rect.x1 + (header_rect.width() - lw) / 2.0;
            let ly = header_rect.y1 + (header_h - tl.size().height) / 2.0;
            canvas.draw_text(tl, lx as i32, ly as i32);
        }

        cy += header_h;

        // ── Weekday headers ──
        let cell_w = self.calendar_width / 7.0;
        let weekday_h = 32.0;
        for (i, layout_opt) in self.weekday_layouts.iter().enumerate() {
            if let Some(tl) = layout_opt {
                let cx = inner_x + i as f32 * cell_w;
                let tx = cx + (cell_w - tl.size().width) / 2.0;
                let ty = cy + (weekday_h - tl.size().height) / 2.0;
                canvas.draw_text(tl, tx as i32, ty as i32);
            }
        }
        cy += weekday_h;

        // ── Day grid ──
        let day_cell_h = 32.0;
        for (idx, layout_opt) in self.day_layouts.iter().enumerate() {
            let day = idx as u32 + 1;
            let cell_idx = self.computed_first_weekday + day - 1;
            let row = cell_idx / 7;
            let col = cell_idx % 7;
            let cx = inner_x + col as f32 * cell_w;
            let cell_y = cy + row as f32 * day_cell_h;
            let cell_rect = Rect::new(cx, cell_y, cx + cell_w, cell_y + day_cell_h);

            let is_selected =
                self.view_year == self.year && self.view_month == self.month && day == self.day;
            let is_hovered = self.hover_day == Some(day);

            // Background
            if is_selected {
                canvas.fill_rounded_rect(
                    cell_rect.inset(&Edges::all(2.0)),
                    Corners::all(4.0),
                    colors::primary(),
                );
            } else if is_hovered {
                canvas.fill_rounded_rect(
                    cell_rect.inset(&Edges::all(2.0)),
                    Corners::all(4.0),
                    colors::secondary(),
                );
            }

            // Day number
            if let Some(tl) = layout_opt {
                let tx = cx + (cell_w - tl.size().width) / 2.0;
                let ty = cell_y + (day_cell_h - tl.size().height) / 2.0;
                canvas.draw_text(tl, tx as i32, ty as i32);
            }
        }

        // ── Separator line ──
        let sep_y = cy + self.total_rows() as f32 * day_cell_h + 4.0;
        canvas.fill_rect(
            Rect::new(dr.x1 + 8.0, sep_y, dr.x2 - 8.0, sep_y + 1.0),
            colors::border(),
        );

        // ── Time spinners ──
        let tr = self.time_rects(&dr);

        // Draw hour spinner
        self.paint_time_spinner(
            canvas,
            &tr.hour_up,
            &tr.hour_val,
            &tr.hour_down,
            self.hour_up_layout.as_ref(),
            self.hour_down_layout.as_ref(),
            self.time_layouts.first().and_then(|o| o.as_ref()),
            self.hover_time_field == Some(0),
        );

        // Draw colon between hour and min
        if let Some(ref tl) = self.colon_layout {
            let cx = tr.colon1_x + (tr.colon_w - tl.size().width) / 2.0;
            let cy_colon = tr.hour_val.y1 + (tr.hour_val.height() - tl.size().height) / 2.0;
            canvas.draw_text(tl, cx as i32, cy_colon as i32);
        }

        // Draw minute spinner
        self.paint_time_spinner(
            canvas,
            &tr.min_up,
            &tr.min_val,
            &tr.min_down,
            self.min_up_layout.as_ref(),
            self.min_down_layout.as_ref(),
            self.time_layouts.get(1).and_then(|o| o.as_ref()),
            self.hover_time_field == Some(1),
        );

        // Draw second spinner (if shown)
        if self.show_seconds {
            // Draw colon between min and sec
            if let Some(ref tl) = self.colon_layout {
                let cx = tr.colon2_x + (tr.colon_w - tl.size().width) / 2.0;
                let cy_colon = tr.min_val.y1 + (tr.min_val.height() - tl.size().height) / 2.0;
                canvas.draw_text(tl, cx as i32, cy_colon as i32);
            }

            self.paint_time_spinner(
                canvas,
                &tr.sec_up,
                &tr.sec_val,
                &tr.sec_down,
                self.sec_up_layout.as_ref(),
                self.sec_down_layout.as_ref(),
                self.time_layouts.get(2).and_then(|o| o.as_ref()),
                self.hover_time_field == Some(2),
            );
        }
    }

    fn children(&self) -> &[Box<dyn Widget>] {
        &[]
    }

    fn event(&mut self, event: &WidgetEvent, rect: Rect) -> EventResponse {
        match event {
            WidgetEvent::Mouse(MouseEvent::MouseClickEvent(e))
                if e.state == MouseState::Released =>
            {
                if rect.contains(&e.position) {
                    if self.open {
                        self.open = false;
                        self.anim_from = 1.0;
                        self.anim_to = 0.0;
                        self.anim_start = Instant::now();
                    } else {
                        self.open = true;
                        self.view_year = self.year;
                        self.view_month = self.month;
                        self.anim_from = 0.0;
                        self.anim_to = 1.0;
                        self.anim_start = Instant::now();
                    }
                    return EventResponse {
                        status: EventStatus::Consumed,
                        cursor: Some(CursorIcon::Pointer),
                        ..Default::default()
                    };
                }
                EventResponse::default()
            }
            WidgetEvent::Mouse(MouseEvent::MouseMoveEvent(pos)) => {
                if rect.contains(pos) {
                    return EventResponse {
                        status: EventStatus::Consumed,
                        cursor: Some(CursorIcon::Pointer),
                        ..Default::default()
                    };
                }
                EventResponse::default()
            }
            _ => EventResponse::default(),
        }
    }

    fn event_overlay(&mut self, event: &WidgetEvent, rect: Rect) -> EventResponse {
        if !self.open {
            return EventResponse::default();
        }

        match event {
            WidgetEvent::Mouse(MouseEvent::MouseClickEvent(e)) => {
                let dr = self.dropdown_rect(&rect);
                if !dr.contains(&e.position) {
                    // Click outside — only close on Release
                    if e.state == MouseState::Released {
                        self.open = false;
                        self.anim_from = 1.0;
                        self.anim_to = 0.0;
                        self.anim_start = Instant::now();
                    }
                    return EventResponse {
                        status: EventStatus::Canceled,
                        ..Default::default()
                    };
                }

                // Handle inside clicks on Pressed for responsiveness
                if e.state == MouseState::Pressed {
                    // Check nav arrows
                    let prev_rect = self.prev_arrow_rect(&dr);
                    let next_rect = self.next_arrow_rect(&dr);
                    if prev_rect.contains(&e.position) {
                        if self.view_month == 1 {
                            self.view_month = 12;
                            self.view_year -= 1;
                        } else {
                            self.view_month -= 1;
                        }
                        return EventResponse {
                            status: EventStatus::Consumed,
                            cursor: Some(CursorIcon::Pointer),
                            ..Default::default()
                        };
                    }
                    if next_rect.contains(&e.position) {
                        if self.view_month == 12 {
                            self.view_month = 1;
                            self.view_year += 1;
                        } else {
                            self.view_month += 1;
                        }
                        return EventResponse {
                            status: EventStatus::Consumed,
                            cursor: Some(CursorIcon::Pointer),
                            ..Default::default()
                        };
                    }

                    // Check day grid
                    let grid = self.grid_rect(&dr);
                    if grid.contains(&e.position)
                        && let Some(day) = self.day_at_position(&e.position, &grid)
                    {
                        self.day = day;
                        self.year = self.view_year;
                        self.month = self.view_month;
                        self.clamp_day();
                        self.fire_on_change();
                        return EventResponse {
                            status: EventStatus::Consumed,
                            cursor: Some(CursorIcon::Pointer),
                            ..Default::default()
                        };
                    }

                    // Check time spinners
                    let tr = self.time_rects(&dr);
                    if tr.hour_up.contains(&e.position) {
                        self.hour = if self.hour == 23 { 0 } else { self.hour + 1 };
                        self.fire_on_change();
                        return EventResponse {
                            status: EventStatus::Consumed,
                            cursor: Some(CursorIcon::Pointer),
                            ..Default::default()
                        };
                    }
                    if tr.hour_down.contains(&e.position) {
                        self.hour = if self.hour == 0 { 23 } else { self.hour - 1 };
                        self.fire_on_change();
                        return EventResponse {
                            status: EventStatus::Consumed,
                            cursor: Some(CursorIcon::Pointer),
                            ..Default::default()
                        };
                    }
                    if tr.min_up.contains(&e.position) {
                        self.minute = if self.minute == 59 {
                            0
                        } else {
                            self.minute + 1
                        };
                        self.fire_on_change();
                        return EventResponse {
                            status: EventStatus::Consumed,
                            cursor: Some(CursorIcon::Pointer),
                            ..Default::default()
                        };
                    }
                    if tr.min_down.contains(&e.position) {
                        self.minute = if self.minute == 0 {
                            59
                        } else {
                            self.minute - 1
                        };
                        self.fire_on_change();
                        return EventResponse {
                            status: EventStatus::Consumed,
                            cursor: Some(CursorIcon::Pointer),
                            ..Default::default()
                        };
                    }
                    if self.show_seconds {
                        if tr.sec_up.contains(&e.position) {
                            self.second = if self.second == 59 {
                                0
                            } else {
                                self.second + 1
                            };
                            self.fire_on_change();
                            return EventResponse {
                                status: EventStatus::Consumed,
                                cursor: Some(CursorIcon::Pointer),
                                ..Default::default()
                            };
                        }
                        if tr.sec_down.contains(&e.position) {
                            self.second = if self.second == 0 {
                                59
                            } else {
                                self.second - 1
                            };
                            self.fire_on_change();
                            return EventResponse {
                                status: EventStatus::Consumed,
                                cursor: Some(CursorIcon::Pointer),
                                ..Default::default()
                            };
                        }
                    }
                }

                EventResponse {
                    status: EventStatus::Consumed,
                    ..Default::default()
                }
            }
            WidgetEvent::Mouse(MouseEvent::MouseMoveEvent(pos)) => {
                let dr = self.dropdown_rect(&rect);
                if !dr.contains(pos) {
                    self.hover_day = None;
                    self.hover_time_field = None;
                    return EventResponse::default();
                }

                // Check day grid hover
                let grid = self.grid_rect(&dr);
                if grid.contains(pos) {
                    self.hover_day = self.day_at_position(pos, &grid);
                    self.hover_time_field = None;
                    return EventResponse {
                        status: EventStatus::Consumed,
                        cursor: Some(CursorIcon::Pointer),
                        ..Default::default()
                    };
                }
                self.hover_day = None;

                // Check time spinner hover
                let tr = self.time_rects(&dr);
                if tr.hour_up.contains(pos)
                    || tr.hour_down.contains(pos)
                    || tr.hour_val.contains(pos)
                {
                    self.hover_time_field = Some(0);
                    return EventResponse {
                        status: EventStatus::Consumed,
                        cursor: Some(CursorIcon::Pointer),
                        ..Default::default()
                    };
                }
                if tr.min_up.contains(pos) || tr.min_down.contains(pos) || tr.min_val.contains(pos)
                {
                    self.hover_time_field = Some(1);
                    return EventResponse {
                        status: EventStatus::Consumed,
                        cursor: Some(CursorIcon::Pointer),
                        ..Default::default()
                    };
                }
                if self.show_seconds
                    && (tr.sec_up.contains(pos)
                        || tr.sec_down.contains(pos)
                        || tr.sec_val.contains(pos))
                {
                    self.hover_time_field = Some(2);
                    return EventResponse {
                        status: EventStatus::Consumed,
                        cursor: Some(CursorIcon::Pointer),
                        ..Default::default()
                    };
                }
                self.hover_time_field = None;

                // Check nav arrows hover
                let prev_rect = self.prev_arrow_rect(&dr);
                let next_rect = self.next_arrow_rect(&dr);
                if prev_rect.contains(pos) || next_rect.contains(pos) {
                    return EventResponse {
                        status: EventStatus::Consumed,
                        cursor: Some(CursorIcon::Pointer),
                        ..Default::default()
                    };
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
        self.anim_start.elapsed().as_secs_f32() < ANIM_DURATION
    }

    #[cfg(feature = "a11y")]
    fn access_info(&self) -> aurora_a11y::NodeInfo {
        aurora_a11y::NodeInfo::new(aurora_a11y::accesskit::Role::Group)
            .with_label("Date time picker".to_string())
            .with_expanded(self.open)
    }
}

impl DateTimePicker {
    #[allow(clippy::too_many_arguments)]
    fn paint_time_spinner(
        &self,
        canvas: &mut Canvas,
        up_rect: &Rect,
        val_rect: &Rect,
        down_rect: &Rect,
        up_layout: Option<&aurora_text::text_layout::TextLayout>,
        down_layout: Option<&aurora_text::text_layout::TextLayout>,
        val_layout: Option<&aurora_text::text_layout::TextLayout>,
        hovered: bool,
    ) {
        // Background highlight on hover
        let full_rect = Rect::new(up_rect.x1, up_rect.y1, down_rect.x2, down_rect.y2);
        if hovered {
            canvas.fill_rounded_rect(full_rect, Corners::all(4.0), colors::secondary());
        }

        // Up arrow
        if let Some(tl) = up_layout {
            let tx = up_rect.x1 + (up_rect.width() - tl.size().width) / 2.0;
            let ty = up_rect.y1 + (up_rect.height() - tl.size().height) / 2.0;
            canvas.draw_text(tl, tx as i32, ty as i32);
        }

        // Value
        if let Some(tl) = val_layout {
            let tx = val_rect.x1 + (val_rect.width() - tl.size().width) / 2.0;
            let ty = val_rect.y1 + (val_rect.height() - tl.size().height) / 2.0;
            canvas.draw_text(tl, tx as i32, ty as i32);
        }

        // Down arrow
        if let Some(tl) = down_layout {
            let tx = down_rect.x1 + (down_rect.width() - tl.size().width) / 2.0;
            let ty = down_rect.y1 + (down_rect.height() - tl.size().height) / 2.0;
            canvas.draw_text(tl, tx as i32, ty as i32);
        }
    }
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn days_in_month_february_leap() {
        assert_eq!(days_in_month(2024, 2), 29);
        assert_eq!(days_in_month(2000, 2), 29);
        assert_eq!(days_in_month(2400, 2), 29);
    }

    #[test]
    fn days_in_month_february_non_leap() {
        assert_eq!(days_in_month(2023, 2), 28);
        assert_eq!(days_in_month(1900, 2), 28);
        assert_eq!(days_in_month(2100, 2), 28);
    }

    #[test]
    fn days_in_month_regular() {
        assert_eq!(days_in_month(2026, 1), 31);
        assert_eq!(days_in_month(2026, 4), 30);
        assert_eq!(days_in_month(2026, 7), 31);
        assert_eq!(days_in_month(2026, 9), 30);
        assert_eq!(days_in_month(2026, 12), 31);
    }

    #[test]
    fn format_display_24h() {
        let s = format_display(2026, 4, 1, 14, 30, 0, false, true);
        assert_eq!(s, "2026-04-01 14:30");
    }

    #[test]
    fn format_display_24h_with_seconds() {
        let s = format_display(2026, 4, 1, 14, 30, 45, true, true);
        assert_eq!(s, "2026-04-01 14:30:45");
    }

    #[test]
    fn format_display_12h() {
        let s = format_display(2026, 4, 1, 14, 30, 0, false, false);
        assert_eq!(s, "2026-04-01 02:30 PM");
    }

    #[test]
    fn format_display_12h_midnight() {
        let s = format_display(2026, 1, 1, 0, 0, 0, false, false);
        assert_eq!(s, "2026-01-01 12:00 AM");
    }

    #[test]
    fn format_display_12h_noon() {
        let s = format_display(2026, 6, 15, 12, 0, 0, false, false);
        assert_eq!(s, "2026-06-15 12:00 PM");
    }

    #[test]
    fn default_values() {
        let picker = DateTimePicker::new();
        assert_eq!(picker.year, 2026);
        assert_eq!(picker.month, 1);
        assert_eq!(picker.day, 1);
        assert_eq!(picker.hour, 0);
        assert_eq!(picker.minute, 0);
        assert_eq!(picker.second, 0);
        assert!(picker.format_24h);
        assert!(!picker.show_seconds);
        assert!(!picker.open);
        assert_eq!(picker.height, 40.0);
    }

    #[test]
    fn day_selection() {
        let picker = DateTimePicker::new()
            .year(2026)
            .month(4)
            .day(15)
            .hour(10)
            .minute(30);
        assert_eq!(picker.year, 2026);
        assert_eq!(picker.month, 4);
        assert_eq!(picker.day, 15);
        assert_eq!(picker.hour, 10);
        assert_eq!(picker.minute, 30);
    }

    #[test]
    fn day_of_week_known_dates() {
        // 2026-04-01 is a Wednesday (0=Sun, 3=Wed)
        assert_eq!(day_of_week(2026, 4, 1), 3);
        // 2024-01-01 is a Monday (0=Sun, 1=Mon)
        assert_eq!(day_of_week(2024, 1, 1), 1);
    }

    #[test]
    fn builder_chaining() {
        let picker = DateTimePicker::new()
            .year(2026)
            .month(12)
            .day(25)
            .hour(23)
            .minute(59)
            .second(59)
            .format_24h(false)
            .show_seconds(true)
            .width(300.0)
            .height(48.0)
            .error(true);
        assert_eq!(picker.year, 2026);
        assert_eq!(picker.month, 12);
        assert_eq!(picker.day, 25);
        assert_eq!(picker.hour, 23);
        assert_eq!(picker.minute, 59);
        assert_eq!(picker.second, 59);
        assert!(!picker.format_24h);
        assert!(picker.show_seconds);
        assert_eq!(picker.width, Some(300.0));
        assert_eq!(picker.height, 48.0);
        assert!(picker.error);
    }

    #[test]
    fn clamp_values() {
        let picker = DateTimePicker::new()
            .month(0)
            .day(0)
            .hour(255)
            .minute(255)
            .second(255);
        assert_eq!(picker.month, 1);
        assert_eq!(picker.day, 1);
        assert_eq!(picker.hour, 23);
        assert_eq!(picker.minute, 59);
        assert_eq!(picker.second, 59);
    }
}
