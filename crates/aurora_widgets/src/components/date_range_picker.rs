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

use super::colors;

// ─── Animation helpers ──────────────────────────────────────────────────────

const ANIM_DURATION: f32 = 0.25;

fn ease_out_cubic(t: f32) -> f32 {
    1.0 - (1.0 - t).powi(3)
}

fn anim_t(from: f32, to: f32, start: Instant, duration: f32) -> f32 {
    let elapsed = start.elapsed().as_secs_f32();
    let raw = (elapsed / duration).min(1.0);
    let eased = ease_out_cubic(raw);
    from + (to - from) * eased
}

// ─── Date helpers ───────────────────────────────────────────────────────────

fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            let y = year as u32;
            if (y.is_multiple_of(4) && !y.is_multiple_of(100)) || y.is_multiple_of(400) {
                29
            } else {
                28
            }
        }
        _ => 30,
    }
}

/// Zeller-like algorithm: 0 = Sunday, 1 = Monday, ... 6 = Saturday.
fn day_of_week(year: i32, month: u32, day: u32) -> u32 {
    let t = [0i32, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];
    let y = if month < 3 { year - 1 } else { year };
    ((y + y / 4 - y / 100 + y / 400 + t[month as usize - 1] + day as i32) % 7).unsigned_abs()
}

/// First day of week for a given year/month (0 = Sunday).
fn first_weekday(year: i32, month: u32) -> u32 {
    day_of_week(year, month, 1)
}

fn month_name(month: u32) -> &'static str {
    match month {
        1 => "January",
        2 => "February",
        3 => "March",
        4 => "April",
        5 => "May",
        6 => "June",
        7 => "July",
        8 => "August",
        9 => "September",
        10 => "October",
        11 => "November",
        12 => "December",
        _ => "???",
    }
}

fn advance_month(year: i32, month: u32) -> (i32, u32) {
    if month == 12 {
        (year + 1, 1)
    } else {
        (year, month + 1)
    }
}

const CELL_SIZE: f32 = 36.0;
const CAL_WIDTH: f32 = CELL_SIZE * 7.0;
const HEADER_H: f32 = 36.0;
const WEEKDAY_H: f32 = CELL_SIZE;
const GAP: f32 = 16.0;
const DROPDOWN_PAD: f32 = 12.0;

/// A date range picker input that opens a dual-calendar dropdown.
///
/// Click once to set the range start, click again to set the range end.
/// The two calendars display consecutive months for easy range selection.
///
/// # Example
/// ```ignore
/// DateRangePicker::new()
///     .on_change(|start, end| {
///         println!("Range: {}-{}-{} to {}-{}-{}", start.0, start.1, start.2, end.0, end.1, end.2);
///     })
/// ```
pub struct DateRangePicker {
    start_year: i32,
    start_month: u32,
    start_day: u32,
    end_year: i32,
    end_month: u32,
    end_day: u32,
    selecting_end: bool,
    open: bool,
    width: Option<f32>,
    height: f32,
    background: Color,
    border_color: Color,
    corners: Corners,
    padding: Edges,
    #[allow(clippy::type_complexity)]
    on_change: Option<Box<dyn FnMut((i32, u32, u32), (i32, u32, u32))>>,
    display_layout: Option<aurora_text::text_layout::TextLayout>,
    anim_from: f32,
    anim_to: f32,
    anim_start: Instant,
    left_year: i32,
    left_month: u32,
    hover_day_left: Option<u32>,
    hover_day_right: Option<u32>,
    // Text layout caches
    left_month_label: Option<aurora_text::text_layout::TextLayout>,
    right_month_label: Option<aurora_text::text_layout::TextLayout>,
    left_day_layouts: Vec<Option<aurora_text::text_layout::TextLayout>>,
    right_day_layouts: Vec<Option<aurora_text::text_layout::TextLayout>>,
    weekday_layouts: Vec<Option<aurora_text::text_layout::TextLayout>>,
    prev_arrow_layout: Option<aurora_text::text_layout::TextLayout>,
    next_arrow_layout: Option<aurora_text::text_layout::TextLayout>,
    error: bool,
}

impl DateRangePicker {
    pub fn new() -> Self {
        let now = Instant::now();
        Self {
            start_year: 0,
            start_month: 0,
            start_day: 0,
            end_year: 0,
            end_month: 0,
            end_day: 0,
            selecting_end: false,
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
            left_year: 2026,
            left_month: 4,
            hover_day_left: None,
            hover_day_right: None,
            left_month_label: None,
            right_month_label: None,
            left_day_layouts: Vec::new(),
            right_day_layouts: Vec::new(),
            weekday_layouts: Vec::new(),
            prev_arrow_layout: None,
            next_arrow_layout: None,
            error: false,
        }
    }

    // ── Builder methods ─────────────────────────────────────────────────

    pub fn start_date(mut self, year: i32, month: u32, day: u32) -> Self {
        self.start_year = year;
        self.start_month = month;
        self.start_day = day;
        self
    }

    pub fn end_date(mut self, year: i32, month: u32, day: u32) -> Self {
        self.end_year = year;
        self.end_month = month;
        self.end_day = day;
        self
    }

    pub fn left_month(mut self, year: i32, month: u32) -> Self {
        self.left_year = year;
        self.left_month = month.clamp(1, 12);
        self
    }

    pub fn width(mut self, w: impl Into<f32>) -> Self {
        self.width = Some(w.into());
        self
    }

    pub fn height(mut self, h: f32) -> Self {
        self.height = h;
        self
    }

    pub fn on_change(mut self, cb: impl FnMut((i32, u32, u32), (i32, u32, u32)) + 'static) -> Self {
        self.on_change = Some(Box::new(cb));
        self
    }

    pub fn error(mut self, error: bool) -> Self {
        self.error = error;
        self
    }

    // ── Internals ───────────────────────────────────────────────────────

    fn has_start(&self) -> bool {
        self.start_year != 0
    }

    fn has_end(&self) -> bool {
        self.end_year != 0
    }

    fn has_range(&self) -> bool {
        self.has_start() && self.has_end()
    }

    fn display_text(&self) -> String {
        if self.has_range() {
            format!(
                "{:04}-{:02}-{:02}  —  {:04}-{:02}-{:02}",
                self.start_year,
                self.start_month,
                self.start_day,
                self.end_year,
                self.end_month,
                self.end_day,
            )
        } else if self.has_start() {
            format!(
                "{:04}-{:02}-{:02}  —  ...",
                self.start_year, self.start_month, self.start_day,
            )
        } else {
            "Pick a date range".to_string()
        }
    }

    fn right_year_month(&self) -> (i32, u32) {
        advance_month(self.left_year, self.left_month)
    }

    fn dropdown_width(&self) -> f32 {
        CAL_WIDTH * 2.0 + GAP + DROPDOWN_PAD * 2.0
    }

    fn dropdown_height(&self) -> f32 {
        // Header + weekday row + max 6 rows of days + padding
        HEADER_H + WEEKDAY_H + CELL_SIZE * 6.0 + DROPDOWN_PAD * 2.0
    }

    fn dropdown_rect(&self, trigger_rect: &Rect) -> Rect {
        let w = self.dropdown_width();
        let h = self.dropdown_height();
        Rect::new(
            trigger_rect.x1,
            trigger_rect.y2 + 4.0,
            trigger_rect.x1 + w,
            trigger_rect.y2 + 4.0 + h,
        )
    }

    fn prev_month(&mut self) {
        if self.left_month == 1 {
            self.left_month = 12;
            self.left_year -= 1;
        } else {
            self.left_month -= 1;
        }
        self.anim_from = 1.0;
        self.anim_to = 0.0;
        self.anim_start = Instant::now();
    }

    fn next_month(&mut self) {
        if self.left_month == 12 {
            self.left_month = 1;
            self.left_year += 1;
        } else {
            self.left_month += 1;
        }
        self.anim_from = 1.0;
        self.anim_to = 0.0;
        self.anim_start = Instant::now();
    }

    /// Check if a date (year, month, day) falls within the selected range.
    fn is_in_range(&self, year: i32, month: u32, day: u32) -> bool {
        if !self.has_start() {
            return false;
        }
        let date = (year, month, day);
        let start = (self.start_year, self.start_month, self.start_day);
        if self.has_end() {
            let end = (self.end_year, self.end_month, self.end_day);
            date >= start && date <= end
        } else {
            date == start
        }
    }

    /// Check if a date is the start or end of the selected range.
    fn is_range_endpoint(&self, year: i32, month: u32, day: u32) -> bool {
        let date = (year, month, day);
        if self.has_start() && date == (self.start_year, self.start_month, self.start_day) {
            return true;
        }
        if self.has_end() && date == (self.end_year, self.end_month, self.end_day) {
            return true;
        }
        false
    }

    fn select_day(&mut self, year: i32, month: u32, day: u32) {
        if !self.selecting_end {
            // First click: set start
            self.start_year = year;
            self.start_month = month;
            self.start_day = day;
            self.end_year = 0;
            self.end_month = 0;
            self.end_day = 0;
            self.selecting_end = true;
        } else {
            // Second click: set end
            let start = (self.start_year, self.start_month, self.start_day);
            let end = (year, month, day);
            // Swap if end is before start
            if end < start {
                self.start_year = end.0;
                self.start_month = end.1;
                self.start_day = end.2;
                self.end_year = start.0;
                self.end_month = start.1;
                self.end_day = start.2;
            } else {
                self.end_year = year;
                self.end_month = month;
                self.end_day = day;
            }
            self.selecting_end = false;
            // Fire callback
            if let Some(ref mut cb) = self.on_change {
                cb(
                    (self.start_year, self.start_month, self.start_day),
                    (self.end_year, self.end_month, self.end_day),
                );
            }
        }
    }

    /// Returns the day at a position within a calendar grid.
    fn day_at_pos(
        &self,
        pos: &aurora_core::geometry::point::Point,
        cal_x: f32,
        cal_y: f32,
        year: i32,
        month: u32,
    ) -> Option<u32> {
        let grid_y = cal_y + HEADER_H + WEEKDAY_H;
        if pos.y < grid_y || pos.x < cal_x || pos.x > cal_x + CAL_WIDTH {
            return None;
        }
        let col = ((pos.x - cal_x) / CELL_SIZE) as u32;
        let row = ((pos.y - grid_y) / CELL_SIZE) as u32;
        if col >= 7 || row >= 6 {
            return None;
        }
        let fw = first_weekday(year, month);
        let cell_idx = row * 7 + col;
        if cell_idx < fw {
            return None;
        }
        let day = cell_idx - fw + 1;
        let dim = days_in_month(year, month);
        if day >= 1 && day <= dim {
            Some(day)
        } else {
            None
        }
    }

    fn build_day_layouts(
        &self,
        year: i32,
        month: u32,
        ctx: &mut LayoutCtx,
    ) -> Vec<Option<aurora_text::text_layout::TextLayout>> {
        let dim = days_in_month(year, month);
        let mut opts = ctx.font_options.clone();
        opts.size = Some(13.0);
        opts.weight = Some(aurora_text::font_options::FontWeight::Normal);
        let mut layouts = Vec::with_capacity(dim as usize);
        for day in 1..=dim {
            let fg = if self.is_range_endpoint(year, month, day) {
                colors::primary_foreground()
            } else {
                colors::foreground()
            };
            layouts.push(Some(aurora_text::text_layout::TextLayout::new(
                ctx.font_manager,
                &day.to_string(),
                &opts,
                fg,
                None,
            )));
        }
        layouts
    }

    #[allow(clippy::too_many_arguments)]
    fn paint_calendar(
        &self,
        canvas: &mut Canvas,
        cal_x: f32,
        cal_y: f32,
        year: i32,
        month: u32,
        day_layouts: &[Option<aurora_text::text_layout::TextLayout>],
        month_label: &Option<aurora_text::text_layout::TextLayout>,
        hover_day: Option<u32>,
    ) {
        let dim = days_in_month(year, month);
        let fw = first_weekday(year, month);

        // Month label (centered)
        if let Some(tl) = month_label {
            let s = tl.size();
            let tx = cal_x + (CAL_WIDTH - s.width) / 2.0;
            let ty = cal_y + (HEADER_H - s.height) / 2.0;
            canvas.draw_text(tl, tx as i32, ty as i32);
        }

        // Weekday headers
        let wd_y = cal_y + HEADER_H;
        for (i, wl) in self.weekday_layouts.iter().enumerate() {
            if let Some(tl) = wl {
                let s = tl.size();
                let tx = cal_x + i as f32 * CELL_SIZE + (CELL_SIZE - s.width) / 2.0;
                let ty = wd_y + (WEEKDAY_H - s.height) / 2.0;
                canvas.draw_text(tl, tx as i32, ty as i32);
            }
        }

        // Day grid
        let grid_y = wd_y + WEEKDAY_H;
        let primary = colors::primary();
        let range_bg = colors::primary().opacity(0.15);
        let hover_bg = colors::secondary();

        for day in 1..=dim {
            let cell_idx = fw + day - 1;
            let row = cell_idx / 7;
            let col = cell_idx % 7;
            let cx = cal_x + col as f32 * CELL_SIZE;
            let cy = grid_y + row as f32 * CELL_SIZE;
            let cell_rect = Rect::new(cx, cy, cx + CELL_SIZE, cy + CELL_SIZE);

            let is_endpoint = self.is_range_endpoint(year, month, day);
            let in_range = self.is_in_range(year, month, day);
            let is_hovered = hover_day == Some(day);

            // Background
            if is_endpoint {
                canvas.fill_rounded_rect(cell_rect, Corners::all(9999.0), primary);
            } else if in_range {
                canvas.fill_rect(cell_rect, range_bg);
            } else if is_hovered {
                canvas.fill_rounded_rect(cell_rect, Corners::all(9999.0), hover_bg);
            }

            // Day label
            if let Some(Some(tl)) = day_layouts.get((day - 1) as usize) {
                let s = tl.size();
                let tx = cx + (CELL_SIZE - s.width) / 2.0;
                let ty = cy + (CELL_SIZE - s.height) / 2.0;
                canvas.draw_text(tl, tx as i32, ty as i32);
            }
        }
    }
}

impl Default for DateRangePicker {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for DateRangePicker {
    fn layout(&mut self, available: Size, ctx: &mut LayoutCtx) -> Size {
        let w = self.width.unwrap_or(available.width);

        // Display text layout
        let display_text = self.display_text();
        let text_color = if self.has_start() {
            colors::foreground()
        } else {
            colors::muted_foreground()
        };
        let mut opts = ctx.font_options.clone();
        opts.size = Some(14.0);
        opts.weight = Some(aurora_text::font_options::FontWeight::Normal);
        let inner_w = w - self.padding.left - self.padding.right;
        let mut tl = aurora_text::text_layout::TextLayout::new(
            ctx.font_manager,
            &display_text,
            &opts,
            text_color,
            None,
        );
        tl.set_max_width(ctx.font_manager, inner_w.max(0.0));
        self.display_layout = Some(tl);

        if self.open {
            // Month labels
            let mut label_opts = ctx.font_options.clone();
            label_opts.size = Some(14.0);
            label_opts.weight = Some(aurora_text::font_options::FontWeight::SemiBold);

            let left_str = format!("{} {}", month_name(self.left_month), self.left_year);
            self.left_month_label = Some(aurora_text::text_layout::TextLayout::new(
                ctx.font_manager,
                &left_str,
                &label_opts,
                colors::foreground(),
                None,
            ));

            let (ry, rm) = self.right_year_month();
            let right_str = format!("{} {}", month_name(rm), ry);
            self.right_month_label = Some(aurora_text::text_layout::TextLayout::new(
                ctx.font_manager,
                &right_str,
                &label_opts,
                colors::foreground(),
                None,
            ));

            // Day layouts for both calendars
            self.left_day_layouts = self.build_day_layouts(self.left_year, self.left_month, ctx);
            let (ry, rm) = self.right_year_month();
            self.right_day_layouts = self.build_day_layouts(ry, rm, ctx);

            // Weekday headers (shared)
            if self.weekday_layouts.is_empty() {
                let mut wd_opts = ctx.font_options.clone();
                wd_opts.size = Some(12.0);
                wd_opts.weight = Some(aurora_text::font_options::FontWeight::Medium);
                let names = ["Su", "Mo", "Tu", "We", "Th", "Fr", "Sa"];
                self.weekday_layouts = names
                    .iter()
                    .map(|n| {
                        Some(aurora_text::text_layout::TextLayout::new(
                            ctx.font_manager,
                            n,
                            &wd_opts,
                            colors::muted_foreground(),
                            None,
                        ))
                    })
                    .collect();
            }

            // Arrow layouts
            let mut arrow_opts = ctx.font_options.clone();
            arrow_opts.size = Some(16.0);
            arrow_opts.weight = Some(aurora_text::font_options::FontWeight::Normal);
            self.prev_arrow_layout = Some(aurora_text::text_layout::TextLayout::new(
                ctx.font_manager,
                "\u{276E}",
                &arrow_opts,
                colors::foreground(),
                None,
            ));
            self.next_arrow_layout = Some(aurora_text::text_layout::TextLayout::new(
                ctx.font_manager,
                "\u{276F}",
                &arrow_opts,
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

        // Calendar icon
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

        let dr = self.dropdown_rect(&rect);
        let drop_corners = Corners::all(8.0);

        // Animate opacity
        let opacity = anim_t(self.anim_from, self.anim_to, self.anim_start, ANIM_DURATION);
        let _ = opacity; // opacity applied via bg alpha below

        // Dropdown background
        canvas.fill_rounded_rect(dr, drop_corners, colors::popover());
        canvas.stroke_rounded_rect(dr, drop_corners, 1, colors::border());

        let inner_x = dr.x1 + DROPDOWN_PAD;
        let inner_y = dr.y1 + DROPDOWN_PAD;

        // Left calendar
        self.paint_calendar(
            canvas,
            inner_x,
            inner_y,
            self.left_year,
            self.left_month,
            &self.left_day_layouts,
            &self.left_month_label,
            self.hover_day_left,
        );

        // Right calendar
        let (ry, rm) = self.right_year_month();
        let right_x = inner_x + CAL_WIDTH + GAP;
        self.paint_calendar(
            canvas,
            right_x,
            inner_y,
            ry,
            rm,
            &self.right_day_layouts,
            &self.right_month_label,
            self.hover_day_right,
        );

        // Navigation arrows
        // Prev arrow on far left
        if let Some(ref tl) = self.prev_arrow_layout {
            let s = tl.size();
            let ax = inner_x + 4.0;
            let ay = inner_y + (HEADER_H - s.height) / 2.0;
            canvas.draw_text(tl, ax as i32, ay as i32);
        }

        // Next arrow on far right
        if let Some(ref tl) = self.next_arrow_layout {
            let s = tl.size();
            let ax = right_x + CAL_WIDTH - s.width - 4.0;
            let ay = inner_y + (HEADER_H - s.height) / 2.0;
            canvas.draw_text(tl, ax as i32, ay as i32);
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
                    self.open = !self.open;
                    if self.open {
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

        let dr = self.dropdown_rect(&rect);

        match event {
            WidgetEvent::Mouse(MouseEvent::MouseClickEvent(e)) => {
                if !dr.contains(&e.position) {
                    // Click outside — only close on Release
                    if e.state == MouseState::Released {
                        self.open = false;
                        self.hover_day_left = None;
                        self.hover_day_right = None;
                    }
                    return EventResponse {
                        status: EventStatus::Canceled,
                        ..Default::default()
                    };
                }

                // Handle inside clicks on Pressed for responsiveness
                if e.state == MouseState::Pressed {
                    let inner_x = dr.x1 + DROPDOWN_PAD;
                    let inner_y = dr.y1 + DROPDOWN_PAD;
                    let right_x = inner_x + CAL_WIDTH + GAP;

                    // Check prev arrow (far left header area)
                    let prev_rect = Rect::new(inner_x, inner_y, inner_x + 28.0, inner_y + HEADER_H);
                    if prev_rect.contains(&e.position) {
                        self.prev_month();
                        return EventResponse {
                            status: EventStatus::Consumed,
                            cursor: Some(CursorIcon::Pointer),
                            ..Default::default()
                        };
                    }

                    // Check next arrow (far right header area)
                    let next_rect = Rect::new(
                        right_x + CAL_WIDTH - 28.0,
                        inner_y,
                        right_x + CAL_WIDTH,
                        inner_y + HEADER_H,
                    );
                    if next_rect.contains(&e.position) {
                        self.next_month();
                        return EventResponse {
                            status: EventStatus::Consumed,
                            cursor: Some(CursorIcon::Pointer),
                            ..Default::default()
                        };
                    }

                    // Check left calendar day click
                    if let Some(day) = self.day_at_pos(
                        &e.position,
                        inner_x,
                        inner_y,
                        self.left_year,
                        self.left_month,
                    ) {
                        self.select_day(self.left_year, self.left_month, day);
                        return EventResponse {
                            status: EventStatus::Consumed,
                            cursor: Some(CursorIcon::Pointer),
                            ..Default::default()
                        };
                    }

                    // Check right calendar day click
                    let (ry, rm) = self.right_year_month();
                    if let Some(day) = self.day_at_pos(&e.position, right_x, inner_y, ry, rm) {
                        self.select_day(ry, rm, day);
                        return EventResponse {
                            status: EventStatus::Consumed,
                            cursor: Some(CursorIcon::Pointer),
                            ..Default::default()
                        };
                    }
                }

                EventResponse {
                    status: EventStatus::Consumed,
                    ..Default::default()
                }
            }
            WidgetEvent::Mouse(MouseEvent::MouseMoveEvent(pos)) => {
                if !dr.contains(pos) {
                    self.hover_day_left = None;
                    self.hover_day_right = None;
                    return EventResponse::default();
                }

                let inner_x = dr.x1 + DROPDOWN_PAD;
                let inner_y = dr.y1 + DROPDOWN_PAD;
                let right_x = inner_x + CAL_WIDTH + GAP;

                self.hover_day_left =
                    self.day_at_pos(pos, inner_x, inner_y, self.left_year, self.left_month);

                let (ry, rm) = self.right_year_month();
                self.hover_day_right = self.day_at_pos(pos, right_x, inner_y, ry, rm);

                EventResponse {
                    status: EventStatus::Consumed,
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
        aurora_a11y::NodeInfo::new(aurora_a11y::accesskit::Role::Group)
            .with_label("Date range picker".to_string())
            .with_expanded(self.open)
    }
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn range_selection() {
        let mut picker = DateRangePicker::new();
        assert!(!picker.has_start());
        assert!(!picker.has_end());

        // First click sets start
        picker.select_day(2026, 4, 10);
        assert_eq!(picker.start_year, 2026);
        assert_eq!(picker.start_month, 4);
        assert_eq!(picker.start_day, 10);
        assert!(picker.selecting_end);
        assert!(!picker.has_end());

        // Second click sets end
        picker.select_day(2026, 4, 20);
        assert_eq!(picker.end_year, 2026);
        assert_eq!(picker.end_month, 4);
        assert_eq!(picker.end_day, 20);
        assert!(!picker.selecting_end);
        assert!(picker.has_range());
    }

    #[test]
    fn start_after_end_swaps() {
        let mut picker = DateRangePicker::new();

        // First click: set start to a later date
        picker.select_day(2026, 5, 20);
        assert!(picker.selecting_end);

        // Second click: set end to an earlier date => should swap
        picker.select_day(2026, 3, 5);
        assert_eq!(picker.start_year, 2026);
        assert_eq!(picker.start_month, 3);
        assert_eq!(picker.start_day, 5);
        assert_eq!(picker.end_year, 2026);
        assert_eq!(picker.end_month, 5);
        assert_eq!(picker.end_day, 20);
    }

    #[test]
    fn days_in_month_leap() {
        // 2024 is a leap year
        assert_eq!(days_in_month(2024, 2), 29);
        // 2023 is not
        assert_eq!(days_in_month(2023, 2), 28);
        // 1900 is not (divisible by 100 but not 400)
        assert_eq!(days_in_month(1900, 2), 28);
        // 2000 is a leap year (divisible by 400)
        assert_eq!(days_in_month(2000, 2), 29);
        // Regular months
        assert_eq!(days_in_month(2026, 1), 31);
        assert_eq!(days_in_month(2026, 4), 30);
    }

    #[test]
    fn default_values() {
        let picker = DateRangePicker::new();
        assert_eq!(picker.start_year, 0);
        assert_eq!(picker.start_month, 0);
        assert_eq!(picker.start_day, 0);
        assert_eq!(picker.end_year, 0);
        assert_eq!(picker.end_month, 0);
        assert_eq!(picker.end_day, 0);
        assert!(!picker.selecting_end);
        assert!(!picker.open);
        assert_eq!(picker.height, 40.0);
        assert!(!picker.error);
    }

    #[test]
    fn month_navigation() {
        let mut picker = DateRangePicker::new();
        picker.left_year = 2026;
        picker.left_month = 1;

        // Navigate forward
        picker.next_month();
        assert_eq!(picker.left_year, 2026);
        assert_eq!(picker.left_month, 2);

        // Navigate to next year
        picker.left_month = 12;
        picker.next_month();
        assert_eq!(picker.left_year, 2027);
        assert_eq!(picker.left_month, 1);

        // Navigate backward
        picker.prev_month();
        assert_eq!(picker.left_year, 2026);
        assert_eq!(picker.left_month, 12);

        // Navigate backward across year boundary
        picker.left_month = 1;
        picker.prev_month();
        assert_eq!(picker.left_year, 2025);
        assert_eq!(picker.left_month, 12);
    }

    #[test]
    fn display_format() {
        let picker = DateRangePicker::new();
        assert_eq!(picker.display_text(), "Pick a date range");

        let mut picker = DateRangePicker::new();
        picker.select_day(2026, 4, 1);
        assert_eq!(picker.display_text(), "2026-04-01  —  ...");

        picker.select_day(2026, 4, 15);
        assert_eq!(picker.display_text(), "2026-04-01  —  2026-04-15");
    }

    #[test]
    fn right_calendar_wraps() {
        let picker = DateRangePicker::new().left_month(2026, 12);
        let (ry, rm) = picker.right_year_month();
        assert_eq!(ry, 2027);
        assert_eq!(rm, 1);
    }

    #[test]
    fn is_in_range_works() {
        let mut picker = DateRangePicker::new();
        picker.select_day(2026, 3, 10);
        picker.select_day(2026, 3, 20);

        assert!(picker.is_in_range(2026, 3, 10));
        assert!(picker.is_in_range(2026, 3, 15));
        assert!(picker.is_in_range(2026, 3, 20));
        assert!(!picker.is_in_range(2026, 3, 9));
        assert!(!picker.is_in_range(2026, 3, 21));
    }
}
