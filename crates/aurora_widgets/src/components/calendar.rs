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

// ─── Animation helpers ──────────────────────────────────────────────────────

const ANIM_DURATION: f32 = 0.30;
const SLIDE_DURATION: f32 = 0.25;
const SELECTOR_ITEM_H: f32 = 32.0;
const SELECTOR_YEAR_COUNT: i32 = 21;

fn ease_out_cubic(t: f32) -> f32 {
    1.0 - (1.0 - t).powi(3)
}

fn anim_t(from: f32, to: f32, start: Instant, duration: f32) -> f32 {
    let elapsed = start.elapsed().as_secs_f32();
    let raw = (elapsed / duration).min(1.0);
    let eased = ease_out_cubic(raw);
    from + (to - from) * eased
}

fn anim_active(start: Instant, duration: f32) -> bool {
    start.elapsed().as_secs_f32() < duration
}

fn anim_finished(start: Instant, duration: f32) -> bool {
    start.elapsed().as_secs_f32() >= duration
}

// ─── Public types ───────────────────────────────────────────────────────────

/// Controls how the month/year selector behaves when clicking the header.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MonthYearSelector {
    Separate,
    Combined,
}

/// Visual style for a cell indicator on a calendar day.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum IndicatorStyle {
    Dot,
    Ring,
}

/// A visual marker on a specific day cell with an optional tooltip.
#[derive(Clone)]
pub struct CellIndicator {
    pub day: u32,
    pub color: Color,
    pub style: IndicatorStyle,
    pub tooltip: Option<String>,
}

// ─── Internal enums ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq)]
enum SelectorView {
    None,
    MonthColumn,
    YearColumn,
    Combined,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum HeaderHit {
    None,
    Prev,
    Next,
    Month,
    Year,
}

// ─── Calendar widget ────────────────────────────────────────────────────────

pub struct Calendar {
    year: u32,
    month: u32,
    selected_day: Option<u32>,
    cell_size: f32,
    header_height: f32,
    corners: Corners,
    // Configurable colors
    selected_bg: Color,
    selected_fg: Color,
    today_bg: Color,
    hover_bg: Color,
    header_hover_bg: Color,
    selector_bg: Color,
    selector_border_color: Color,
    today_day: Option<u32>,
    on_select: Option<Box<dyn FnMut(u32)>>,
    on_month_change: Option<Box<dyn FnMut(u32, u32)>>,
    disabled_days: Vec<u32>,
    min_date: Option<(u32, u32, u32)>,
    max_date: Option<(u32, u32, u32)>,
    month_year_selector: Option<MonthYearSelector>,
    selector_view: SelectorView,
    indicators: Vec<CellIndicator>,
    indicator_tooltip_day: Option<u32>,
    indicator_tooltip_layout: Option<aurora_text::text_layout::TextLayout>,
    // Hover state
    hovered_day: Option<u32>,
    hovered_header: Option<HeaderHit>,
    hovered_selector_month: Option<u32>,
    hovered_selector_year: Option<u32>,
    // Animation: selector open/close
    selector_anim_from: f32,
    selector_anim_to: f32,
    selector_anim_start: Instant,
    // Animation: month slide (dual-month)
    slide_direction: f32,
    slide_start: Instant,
    slide_old_day_layouts: Vec<Option<aurora_text::text_layout::TextLayout>>,
    slide_old_prev_trailing: Vec<Option<aurora_text::text_layout::TextLayout>>,
    slide_old_next_trailing: Vec<Option<aurora_text::text_layout::TextLayout>>,
    slide_old_first_weekday: u32,
    slide_old_days_in_month: u32,
    // Selector scroll offsets
    month_scroll_offset: f32,
    year_scroll_offset: f32,
    // Precomputed layouts
    day_layouts: Vec<Option<aurora_text::text_layout::TextLayout>>,
    prev_trailing_layouts: Vec<Option<aurora_text::text_layout::TextLayout>>,
    next_trailing_layouts: Vec<Option<aurora_text::text_layout::TextLayout>>,
    weekday_layouts: Vec<Option<aurora_text::text_layout::TextLayout>>,
    month_label_layout: Option<aurora_text::text_layout::TextLayout>,
    month_name_layout: Option<aurora_text::text_layout::TextLayout>,
    year_label_layout: Option<aurora_text::text_layout::TextLayout>,
    prev_layout: Option<aurora_text::text_layout::TextLayout>,
    next_layout: Option<aurora_text::text_layout::TextLayout>,
    selector_month_layouts: Vec<Option<aurora_text::text_layout::TextLayout>>,
    selector_year_layouts: Vec<Option<aurora_text::text_layout::TextLayout>>,
    pub(crate) days_in_month: u32,
    pub(crate) first_weekday: u32,
    prev_month_days: u32,
}

impl Calendar {
    pub fn new() -> Self {
        let now = Instant::now();
        Self {
            year: 2026,
            month: 1,
            selected_day: None,
            cell_size: 40.0,
            header_height: 40.0,
            corners: Corners::all(9999.0),
            selected_bg: colors::primary(),
            selected_fg: colors::primary_foreground(),
            today_bg: colors::accent(),
            hover_bg: colors::secondary(),
            header_hover_bg: colors::secondary(),
            selector_bg: colors::card(),
            selector_border_color: colors::border(),
            today_day: None,
            on_select: None,
            on_month_change: None,
            disabled_days: Vec::new(),
            min_date: None,
            max_date: None,
            month_year_selector: None,
            selector_view: SelectorView::None,
            indicators: Vec::new(),
            indicator_tooltip_day: None,
            indicator_tooltip_layout: None,
            hovered_day: None,
            hovered_header: None,
            hovered_selector_month: None,
            hovered_selector_year: None,
            selector_anim_from: 0.0,
            selector_anim_to: 0.0,
            selector_anim_start: now,
            slide_direction: 0.0,
            slide_start: now,
            slide_old_day_layouts: Vec::new(),
            slide_old_prev_trailing: Vec::new(),
            slide_old_next_trailing: Vec::new(),
            slide_old_first_weekday: 0,
            slide_old_days_in_month: 0,
            month_scroll_offset: 0.0,
            year_scroll_offset: 0.0,
            day_layouts: Vec::new(),
            prev_trailing_layouts: Vec::new(),
            next_trailing_layouts: Vec::new(),
            weekday_layouts: Vec::new(),
            month_label_layout: None,
            month_name_layout: None,
            year_label_layout: None,
            prev_layout: None,
            next_layout: None,
            selector_month_layouts: Vec::new(),
            selector_year_layouts: Vec::new(),
            days_in_month: 31,
            first_weekday: 0,
            prev_month_days: 31,
        }
    }

    // ── Builder methods ─────────────────────────────────────────────────

    pub fn year(mut self, year: u32) -> Self { self.year = year; self }
    pub fn month(mut self, month: u32) -> Self { self.month = month.clamp(1, 12); self }
    pub fn selected_day(mut self, day: u32) -> Self { self.selected_day = Some(day); self }
    pub fn today(mut self, day: u32) -> Self { self.today_day = Some(day); self }
    pub fn cell_size(mut self, size: f32) -> Self { self.cell_size = size; self }
    pub fn hover_bg(mut self, color: Color) -> Self { self.hover_bg = color; self }
    pub fn header_hover_bg(mut self, color: Color) -> Self { self.header_hover_bg = color; self }
    pub fn selector_bg(mut self, color: Color) -> Self { self.selector_bg = color; self }
    pub fn selector_border_color(mut self, color: Color) -> Self { self.selector_border_color = color; self }

    pub fn on_select(mut self, cb: impl FnMut(u32) + 'static) -> Self {
        self.on_select = Some(Box::new(cb)); self
    }
    pub fn on_month_change(mut self, cb: impl FnMut(u32, u32) + 'static) -> Self {
        self.on_month_change = Some(Box::new(cb)); self
    }
    pub fn disabled_day(mut self, day: u32) -> Self { self.disabled_days.push(day); self }
    pub fn disabled_days(mut self, days: Vec<u32>) -> Self { self.disabled_days = days; self }
    pub fn min_date(mut self, year: u32, month: u32, day: u32) -> Self { self.min_date = Some((year, month, day)); self }
    pub fn max_date(mut self, year: u32, month: u32, day: u32) -> Self { self.max_date = Some((year, month, day)); self }
    pub fn month_year_selector(mut self, selector: MonthYearSelector) -> Self { self.month_year_selector = Some(selector); self }
    pub fn indicator(mut self, indicator: CellIndicator) -> Self { self.indicators.push(indicator); self }
    pub fn indicators(mut self, indicators: Vec<CellIndicator>) -> Self { self.indicators = indicators; self }

    // ── Internal helpers ────────────────────────────────────────────────

    pub(crate) fn is_day_disabled(&self, day: u32) -> bool {
        if self.disabled_days.contains(&day) { return true; }
        let date = (self.year, self.month, day);
        if let Some(min) = self.min_date && date < min { return true; }
        if let Some(max) = self.max_date && date > max { return true; }
        false
    }

    fn can_go_prev(&self) -> bool {
        if let Some((min_y, min_m, _)) = self.min_date {
            let (py, pm) = if self.month == 1 { (self.year - 1, 12) } else { (self.year, self.month - 1) };
            (py, pm) >= (min_y, min_m)
        } else { true }
    }

    fn can_go_next(&self) -> bool {
        if let Some((max_y, max_m, _)) = self.max_date {
            let (ny, nm) = if self.month == 12 { (self.year + 1, 1) } else { (self.year, self.month + 1) };
            (ny, nm) <= (max_y, max_m)
        } else { true }
    }

    pub(crate) fn compute_days_in_month(year: u32, month: u32) -> u32 {
        match month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 => if (year.is_multiple_of(4) && !year.is_multiple_of(100)) || year.is_multiple_of(400) { 29 } else { 28 },
            _ => 30,
        }
    }

    pub(crate) fn first_day_of_week(year: u32, month: u32) -> u32 {
        let t = [0u32, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];
        let y = if month < 3 { year - 1 } else { year };
        (y + y / 4 - y / 100 + y / 400 + t[month as usize - 1] + 1) % 7
    }

    pub(crate) fn month_name(month: u32) -> &'static str {
        match month {
            1 => "January", 2 => "February", 3 => "March", 4 => "April",
            5 => "May", 6 => "June", 7 => "July", 8 => "August",
            9 => "September", 10 => "October", 11 => "November", 12 => "December",
            _ => "",
        }
    }

    fn month_name_short(month: u32) -> &'static str {
        match month {
            1 => "Jan", 2 => "Feb", 3 => "Mar", 4 => "Apr", 5 => "May", 6 => "Jun",
            7 => "Jul", 8 => "Aug", 9 => "Sep", 10 => "Oct", 11 => "Nov", 12 => "Dec",
            _ => "",
        }
    }

    fn prev_month(&mut self) {
        if !self.can_go_prev() { return; }
        // Save old grid for dual-slide
        self.slide_old_day_layouts = std::mem::take(&mut self.day_layouts);
        self.slide_old_prev_trailing = std::mem::take(&mut self.prev_trailing_layouts);
        self.slide_old_next_trailing = std::mem::take(&mut self.next_trailing_layouts);
        self.slide_old_first_weekday = self.first_weekday;
        self.slide_old_days_in_month = self.days_in_month;
        self.slide_direction = -1.0; // new enters from left
        self.slide_start = Instant::now();
        if self.month == 1 { self.month = 12; self.year -= 1; } else { self.month -= 1; }
        if let Some(ref mut cb) = self.on_month_change { cb(self.year, self.month); }
    }

    fn next_month(&mut self) {
        if !self.can_go_next() { return; }
        self.slide_old_day_layouts = std::mem::take(&mut self.day_layouts);
        self.slide_old_prev_trailing = std::mem::take(&mut self.prev_trailing_layouts);
        self.slide_old_next_trailing = std::mem::take(&mut self.next_trailing_layouts);
        self.slide_old_first_weekday = self.first_weekday;
        self.slide_old_days_in_month = self.days_in_month;
        self.slide_direction = 1.0; // new enters from right
        self.slide_start = Instant::now();
        if self.month == 12 { self.month = 1; self.year += 1; } else { self.month += 1; }
        if let Some(ref mut cb) = self.on_month_change { cb(self.year, self.month); }
    }

    fn total_rows(&self) -> u32 { (self.first_weekday + self.days_in_month).div_ceil(7) }

    fn total_rows_for(first_weekday: u32, days_in_month: u32) -> u32 {
        (first_weekday + days_in_month).div_ceil(7)
    }

    pub(crate) fn day_at_position(&self, pos: &aurora_core::geometry::point::Point, rect: &Rect) -> Option<u32> {
        let grid_y = rect.y1 + self.header_height + self.cell_size;
        if pos.y < grid_y || pos.x < rect.x1 || pos.x > rect.x1 + self.cell_size * 7.0 { return None; }
        let col = ((pos.x - rect.x1) / self.cell_size) as u32;
        let row = ((pos.y - grid_y) / self.cell_size) as u32;
        if col >= 7 || row >= self.total_rows() { return None; }
        let cell_idx = row * 7 + col;
        if cell_idx < self.first_weekday { return None; }
        let day = cell_idx - self.first_weekday + 1;
        if (1..=self.days_in_month).contains(&day) { Some(day) } else { None }
    }

    fn header_hit(&self, pos: &aurora_core::geometry::point::Point, rect: &Rect) -> HeaderHit {
        let hr = Rect::new(rect.x1, rect.y1, rect.x1 + self.cell_size * 7.0, rect.y1 + self.header_height);
        if !hr.contains(pos) { return HeaderHit::None; }
        let btn_w = self.header_height;
        if pos.x < hr.x1 + btn_w { return HeaderHit::Prev; }
        if pos.x > hr.x2 - btn_w { return HeaderHit::Next; }
        // Center area: split into Month/Year for Separate mode
        if self.month_year_selector == Some(MonthYearSelector::Separate) {
            let mid_start = hr.x1 + btn_w;
            let mid_end = hr.x2 - btn_w;
            let mid = mid_start + (mid_end - mid_start) / 2.0;
            if pos.x < mid { HeaderHit::Month } else { HeaderHit::Year }
        } else {
            HeaderHit::Month // Combined/None: whole center is one button
        }
    }

    fn open_selector(&mut self, view: SelectorView) {
        self.selector_anim_from = self.selector_anim_t();
        self.selector_anim_to = 1.0;
        self.selector_anim_start = Instant::now();
        self.selector_view = view;
        // Center scroll on current month/year
        let grid_h = self.cell_size * self.total_rows() as f32 + self.cell_size;
        self.month_scroll_offset = (self.month as f32 - 1.0) * SELECTOR_ITEM_H - grid_h / 2.0 + SELECTOR_ITEM_H / 2.0;
        let yc = SELECTOR_YEAR_COUNT as f32 / 2.0;
        self.year_scroll_offset = yc * SELECTOR_ITEM_H - grid_h / 2.0 + SELECTOR_ITEM_H / 2.0;
        self.clamp_scroll();
    }

    fn close_selector(&mut self) {
        self.selector_anim_from = self.selector_anim_t();
        self.selector_anim_to = 0.0;
        self.selector_anim_start = Instant::now();
        // selector_view is cleared when animation finishes (checked in event/paint)
    }

    fn check_selector_closed(&mut self) {
        if self.selector_view != SelectorView::None
            && self.selector_anim_to == 0.0
            && anim_finished(self.selector_anim_start, ANIM_DURATION)
        {
            self.selector_view = SelectorView::None;
        }
    }

    fn selector_anim_t(&self) -> f32 {
        anim_t(self.selector_anim_from, self.selector_anim_to, self.selector_anim_start, ANIM_DURATION)
    }

    fn slide_progress(&self) -> f32 {
        if self.slide_direction == 0.0 { return 1.0; }
        let elapsed = self.slide_start.elapsed().as_secs_f32();
        ease_out_cubic((elapsed / SLIDE_DURATION).min(1.0))
    }

    fn is_sliding(&self) -> bool {
        self.slide_direction != 0.0 && anim_active(self.slide_start, SLIDE_DURATION)
    }

    fn selector_grid_rect(&self, rect: &Rect) -> Rect {
        let gy = rect.y1 + self.header_height;
        let gw = self.cell_size * 7.0;
        let gh = self.cell_size * self.total_rows() as f32 + self.cell_size;
        Rect::new(rect.x1, gy, rect.x1 + gw, gy + gh)
    }

    fn month_scroll_max(&self) -> f32 {
        let viewport_h = self.selector_grid_rect(&Rect::new(0.0, 0.0, 0.0, 0.0)).height();
        // Use the real viewport: grid area height
        let grid_h = self.cell_size * self.total_rows() as f32 + self.cell_size;
        (12.0 * SELECTOR_ITEM_H - grid_h).max(0.0)
    }

    fn year_scroll_max(&self) -> f32 {
        let grid_h = self.cell_size * self.total_rows() as f32 + self.cell_size;
        (SELECTOR_YEAR_COUNT as f32 * SELECTOR_ITEM_H - grid_h).max(0.0)
    }

    fn clamp_scroll(&mut self) {
        let m_max = self.month_scroll_max();
        let y_max = self.year_scroll_max();
        self.month_scroll_offset = self.month_scroll_offset.clamp(0.0, m_max);
        self.year_scroll_offset = self.year_scroll_offset.clamp(0.0, y_max);
    }

    fn selector_month_at(&self, pos: &aurora_core::geometry::point::Point, col_rect: &Rect) -> Option<u32> {
        if !col_rect.contains(pos) { return None; }
        let idx = ((pos.y - col_rect.y1 + self.month_scroll_offset) / SELECTOR_ITEM_H) as u32;
        if (1..=12).contains(&(idx + 1)) { Some(idx + 1) } else { None }
    }

    fn selector_year_at(&self, pos: &aurora_core::geometry::point::Point, col_rect: &Rect) -> Option<u32> {
        if !col_rect.contains(pos) { return None; }
        let idx = ((pos.y - col_rect.y1 + self.year_scroll_offset) / SELECTOR_ITEM_H) as i32;
        if idx < 0 || idx >= SELECTOR_YEAR_COUNT { return None; }
        let yr = self.year as i32 - SELECTOR_YEAR_COUNT / 2 + idx;
        if yr > 0 { Some(yr as u32) } else { None }
    }

    // ── Paint a day grid at an x-offset (for slide animation) ───────────

    fn paint_day_grid(
        canvas: &mut Canvas,
        day_layouts: &[Option<aurora_text::text_layout::TextLayout>],
        prev_trailing: &[Option<aurora_text::text_layout::TextLayout>],
        next_trailing: &[Option<aurora_text::text_layout::TextLayout>],
        first_weekday: u32,
        days_in_month: u32,
        rect_x1: f32,
        grid_y: f32,
        cell_size: f32,
        offset_x: f32,
        hovered_day: Option<u32>,
        selected_day: Option<u32>,
        today_day: Option<u32>,
        selected_bg: Color,
        hover_bg: Color,
        today_bg: Color,
        corners: Corners,
        is_active: bool, // false for old grid during slide (no hover/select)
        indicators: &[CellIndicator],
        disabled_days: &[u32],
        min_date: Option<(u32, u32, u32)>,
        max_date: Option<(u32, u32, u32)>,
        year: u32,
        month: u32,
    ) {
        let total_rows = Self::total_rows_for(first_weekday, days_in_month);

        // Previous month trailing
        for (i, layout) in prev_trailing.iter().enumerate() {
            if let Some(tl) = layout {
                let cx = rect_x1 + i as f32 * cell_size + offset_x;
                let cy = grid_y;
                let s = tl.size();
                canvas.draw_text(tl, (cx + (cell_size - s.width) / 2.0) as i32, (cy + (cell_size - s.height) / 2.0) as i32);
            }
        }

        // Day cells
        for day in 1..=days_in_month {
            let cell_idx = first_weekday + day - 1;
            let row = cell_idx / 7;
            let col = cell_idx % 7;
            let cx = rect_x1 + col as f32 * cell_size + offset_x;
            let cy = grid_y + row as f32 * cell_size;
            let cell_rect = Rect::new(cx, cy, cx + cell_size, cy + cell_size);

            let is_disabled = disabled_days.contains(&day)
                || min_date.is_some_and(|min| (year, month, day) < min)
                || max_date.is_some_and(|max| (year, month, day) > max);

            if is_active {
                let is_selected = selected_day == Some(day);
                let is_hovered = hovered_day == Some(day) && !is_disabled;
                let is_today = today_day == Some(day);

                if is_selected {
                    canvas.fill_rounded_rect(cell_rect, corners, selected_bg);
                } else if is_hovered {
                    canvas.fill_rounded_rect(cell_rect, corners, hover_bg);
                }
                if is_today && !is_selected {
                    canvas.stroke_rounded_rect(cell_rect, corners, 1, today_bg);
                }

                // Indicators
                for ind in indicators {
                    if ind.day != day { continue; }
                    match ind.style {
                        IndicatorStyle::Dot => {
                            let dy = cy + cell_size - 8.0;
                            let dcx = cx + cell_size / 2.0;
                            canvas.fill_rounded_rect(Rect::new(dcx - 2.0, dy - 2.0, dcx + 2.0, dy + 2.0), Corners::all(9999.0), ind.color);
                        }
                        IndicatorStyle::Ring => {
                            let ins = 2.0;
                            canvas.stroke_rounded_rect(Rect::new(cx + ins, cy + ins, cx + cell_size - ins, cy + cell_size - ins), corners, 1, ind.color);
                        }
                    }
                }
            }

            if let Some(Some(tl)) = day_layouts.get((day - 1) as usize) {
                let s = tl.size();
                canvas.draw_text(tl, (cx + (cell_size - s.width) / 2.0) as i32, (cy + (cell_size - s.height) / 2.0) as i32);
            }
        }

        // Next month trailing
        let last_cell = first_weekday + days_in_month;
        let total_cells = total_rows * 7;
        for (i, layout) in next_trailing.iter().enumerate() {
            if i as u32 >= total_cells - last_cell { break; }
            if let Some(tl) = layout {
                let cell_idx = last_cell + i as u32;
                let row = cell_idx / 7;
                let col = cell_idx % 7;
                let cx = rect_x1 + col as f32 * cell_size + offset_x;
                let cy = grid_y + row as f32 * cell_size;
                let s = tl.size();
                canvas.draw_text(tl, (cx + (cell_size - s.width) / 2.0) as i32, (cy + (cell_size - s.height) / 2.0) as i32);
            }
        }
    }
}

impl Default for Calendar {
    fn default() -> Self { Self::new() }
}

// ─── Widget implementation ──────────────────────────────────────────────────

impl Widget for Calendar {
    fn layout(&mut self, _available: Size, ctx: &mut LayoutCtx) -> Size {
        // Check if close animation finished
        self.check_selector_closed();

        self.days_in_month = Self::compute_days_in_month(self.year, self.month);
        self.first_weekday = Self::first_day_of_week(self.year, self.month);
        let (py, pm) = if self.month == 1 { (self.year - 1, 12) } else { (self.year, self.month - 1) };
        self.prev_month_days = Self::compute_days_in_month(py, pm);

        let mut opts = ctx.font_options.clone();
        opts.size = Some(14.0);
        opts.weight = Some(aurora_text::font_options::FontWeight::Medium);

        // Month/year labels
        let month_str = format!("{} {}", Self::month_name(self.month), self.year);
        self.month_label_layout = Some(aurora_text::text_layout::TextLayout::new(
            ctx.font_manager, &month_str, &opts, colors::foreground(), None,
        ));
        // Separate month and year labels for Separate selector mode
        self.month_name_layout = Some(aurora_text::text_layout::TextLayout::new(
            ctx.font_manager, Self::month_name(self.month), &opts, colors::foreground(), None,
        ));
        self.year_label_layout = Some(aurora_text::text_layout::TextLayout::new(
            ctx.font_manager, &self.year.to_string(), &opts, colors::foreground(), None,
        ));

        // Prev/Next arrows
        let mut ao = ctx.font_options.clone();
        ao.size = Some(14.0);
        ao.weight = Some(aurora_text::font_options::FontWeight::Normal);
        let pc = if self.can_go_prev() { colors::foreground() } else { colors::muted_foreground() };
        let nc = if self.can_go_next() { colors::foreground() } else { colors::muted_foreground() };
        self.prev_layout = Some(aurora_text::text_layout::TextLayout::new(ctx.font_manager, "<", &ao, pc, None));
        self.next_layout = Some(aurora_text::text_layout::TextLayout::new(ctx.font_manager, ">", &ao, nc, None));

        // Weekday headers
        self.weekday_layouts.clear();
        let weekdays = ["Su", "Mo", "Tu", "We", "Th", "Fr", "Sa"];
        for wd in &weekdays {
            let mut o = ctx.font_options.clone();
            o.size = Some(12.0);
            o.weight = Some(aurora_text::font_options::FontWeight::Medium);
            self.weekday_layouts.push(Some(aurora_text::text_layout::TextLayout::new(
                ctx.font_manager, wd, &o, colors::muted_foreground(), None,
            )));
        }

        // Day layouts
        self.day_layouts.clear();
        let mut do_ = ctx.font_options.clone();
        do_.size = Some(14.0);
        do_.weight = Some(aurora_text::font_options::FontWeight::Normal);
        for day in 1..=self.days_in_month {
            let fg = if self.selected_day == Some(day) { self.selected_fg }
                else if self.is_day_disabled(day) { colors::muted_foreground() }
                else { colors::foreground() };
            self.day_layouts.push(Some(aurora_text::text_layout::TextLayout::new(
                ctx.font_manager, &day.to_string(), &do_, fg, None,
            )));
        }

        // Trailing days
        self.prev_trailing_layouts.clear();
        let mut to_ = ctx.font_options.clone();
        to_.size = Some(14.0);
        to_.weight = Some(aurora_text::font_options::FontWeight::Normal);
        for i in 0..self.first_weekday {
            let d = self.prev_month_days - self.first_weekday + i + 1;
            self.prev_trailing_layouts.push(Some(aurora_text::text_layout::TextLayout::new(
                ctx.font_manager, &d.to_string(), &to_, colors::muted_foreground(), None,
            )));
        }
        self.next_trailing_layouts.clear();
        let last = self.first_weekday + self.days_in_month;
        let total = self.total_rows() * 7;
        for i in 0..(total - last) {
            self.next_trailing_layouts.push(Some(aurora_text::text_layout::TextLayout::new(
                ctx.font_manager, &(i + 1).to_string(), &to_, colors::muted_foreground(), None,
            )));
        }

        // Selector layouts
        if self.selector_view != SelectorView::None {
            self.selector_month_layouts.clear();
            let mut so = ctx.font_options.clone();
            so.size = Some(13.0);
            so.weight = Some(aurora_text::font_options::FontWeight::Normal);
            for m in 1..=12u32 {
                let fg = if m == self.month { self.selected_fg } else { colors::foreground() };
                self.selector_month_layouts.push(Some(aurora_text::text_layout::TextLayout::new(
                    ctx.font_manager, Self::month_name_short(m), &so, fg, None,
                )));
            }
            self.selector_year_layouts.clear();
            let base = self.year as i32 - SELECTOR_YEAR_COUNT / 2;
            for i in 0..SELECTOR_YEAR_COUNT {
                let yr = base + i;
                let fg = if yr as u32 == self.year { self.selected_fg } else { colors::foreground() };
                self.selector_year_layouts.push(Some(aurora_text::text_layout::TextLayout::new(
                    ctx.font_manager, &yr.to_string(), &so, fg, None,
                )));
            }
        }

        // Indicator tooltip
        self.indicator_tooltip_layout = None;
        self.indicator_tooltip_day = None;
        if let Some(hovered) = self.hovered_day
            && let Some(ind) = self.indicators.iter().find(|i| i.day == hovered)
            && let Some(ref tip) = ind.tooltip
        {
            let mut o = ctx.font_options.clone();
            o.size = Some(12.0);
            o.weight = Some(aurora_text::font_options::FontWeight::Normal);
            self.indicator_tooltip_layout = Some(aurora_text::text_layout::TextLayout::new(
                ctx.font_manager, tip, &o, colors::popover_foreground(), None,
            ));
            self.indicator_tooltip_day = Some(hovered);
        }

        let rows = self.total_rows();
        Size::new(self.cell_size * 7.0, self.header_height + self.cell_size + self.cell_size * rows as f32)
    }

    fn paint(&self, canvas: &mut Canvas, rect: Rect) {
        let grid_w = self.cell_size * 7.0;
        let header_rect = Rect::new(rect.x1, rect.y1, rect.x1 + grid_w, rect.y1 + self.header_height);

        // ── Header button rects (1:1 prev/next, center for label) ───
        let btn_w = self.header_height;
        let prev_btn = Rect::new(header_rect.x1, header_rect.y1, header_rect.x1 + btn_w, header_rect.y2);
        let next_btn = Rect::new(header_rect.x2 - btn_w, header_rect.y1, header_rect.x2, header_rect.y2);
        let mid_start = prev_btn.x2;
        let mid_end = next_btn.x1;
        let is_separate = self.month_year_selector == Some(MonthYearSelector::Separate);
        let gap = 2.0;
        let mid_center = mid_start + (mid_end - mid_start) / 2.0;

        // Header hover backgrounds (same corner style as day cells)
        if self.hovered_header == Some(HeaderHit::Prev) && self.can_go_prev() {
            canvas.fill_rounded_rect(prev_btn, self.corners, self.header_hover_bg);
        }
        if self.hovered_header == Some(HeaderHit::Next) && self.can_go_next() {
            canvas.fill_rounded_rect(next_btn, self.corners, self.header_hover_bg);
        }

        if is_separate {
            // Two separate buttons: month (left half) and year (right half)
            let month_btn = Rect::new(mid_start, header_rect.y1, mid_center - gap, header_rect.y2);
            let year_btn = Rect::new(mid_center + gap, header_rect.y1, mid_end, header_rect.y2);
            if self.hovered_header == Some(HeaderHit::Month) {
                canvas.fill_rounded_rect(month_btn, self.corners, self.header_hover_bg);
            }
            if self.hovered_header == Some(HeaderHit::Year) {
                canvas.fill_rounded_rect(year_btn, self.corners, self.header_hover_bg);
            }
            // Month name centered in left button
            if let Some(ref tl) = self.month_name_layout {
                let s = tl.size();
                canvas.draw_text(tl, (month_btn.x1 + (month_btn.width() - s.width) / 2.0) as i32, (month_btn.y1 + (self.header_height - s.height) / 2.0) as i32);
            }
            // Year centered in right button
            if let Some(ref tl) = self.year_label_layout {
                let s = tl.size();
                canvas.draw_text(tl, (year_btn.x1 + (year_btn.width() - s.width) / 2.0) as i32, (year_btn.y1 + (self.header_height - s.height) / 2.0) as i32);
            }
        } else {
            // Single combined "March 2026" button
            let mid_rect = Rect::new(mid_start, header_rect.y1, mid_end, header_rect.y2);
            if self.hovered_header == Some(HeaderHit::Month) && self.month_year_selector.is_some() {
                canvas.fill_rounded_rect(mid_rect, self.corners, self.header_hover_bg);
            }
            if let Some(ref tl) = self.month_label_layout {
                let s = tl.size();
                canvas.draw_text(tl, (mid_rect.x1 + (mid_rect.width() - s.width) / 2.0) as i32, (mid_rect.y1 + (self.header_height - s.height) / 2.0) as i32);
            }
        }

        // ── Prev/Next arrow text ────────────────────────────────────
        if let Some(ref tl) = self.prev_layout {
            let s = tl.size();
            canvas.draw_text(tl, (prev_btn.x1 + (btn_w - s.width) / 2.0) as i32, (prev_btn.y1 + (btn_w - s.height) / 2.0) as i32);
        }
        if let Some(ref tl) = self.next_layout {
            let s = tl.size();
            canvas.draw_text(tl, (next_btn.x1 + (btn_w - s.width) / 2.0) as i32, (next_btn.y1 + (btn_w - s.height) / 2.0) as i32);
        }

        // ── Selector animation state ────────────────────────────────
        let sel_t = self.selector_anim_t();
        let selector_active = self.selector_view != SelectorView::None || sel_t > 0.01;
        let selector_fully_open = sel_t > 0.99;

        // If selector is fully open, skip drawing the day grid entirely
        if selector_fully_open {
            self.paint_selector_columns(canvas, rect, sel_t);
            return;
        }

        // ── Day grid (with dual-month slide) ────────────────────────
        let weekday_y = rect.y1 + self.header_height;
        let grid_y = weekday_y + self.cell_size;
        let grid_clip = Rect::new(rect.x1, weekday_y, rect.x1 + grid_w, rect.y2);
        canvas.push_clip(grid_clip);

        let sliding = self.is_sliding();
        let t = self.slide_progress();

        if sliding {
            let dir = self.slide_direction;
            let old_x = -t * grid_w * dir;
            let new_x = (1.0 - t) * grid_w * dir;

            // Weekday headers for new grid
            for (col, layout) in self.weekday_layouts.iter().enumerate() {
                if let Some(tl) = layout {
                    let s = tl.size();
                    let cx = rect.x1 + col as f32 * self.cell_size + self.cell_size / 2.0 + new_x;
                    canvas.draw_text(tl, (cx - s.width / 2.0) as i32, (weekday_y + (self.cell_size - s.height) / 2.0) as i32);
                }
            }
            // Weekday headers for old grid (same labels)
            for (col, layout) in self.weekday_layouts.iter().enumerate() {
                if let Some(tl) = layout {
                    let s = tl.size();
                    let cx = rect.x1 + col as f32 * self.cell_size + self.cell_size / 2.0 + old_x;
                    canvas.draw_text(tl, (cx - s.width / 2.0) as i32, (weekday_y + (self.cell_size - s.height) / 2.0) as i32);
                }
            }

            // Old grid (no hover/select/indicators)
            Self::paint_day_grid(
                canvas, &self.slide_old_day_layouts, &self.slide_old_prev_trailing, &self.slide_old_next_trailing,
                self.slide_old_first_weekday, self.slide_old_days_in_month,
                rect.x1, grid_y, self.cell_size, old_x,
                None, None, None,
                self.selected_bg, self.hover_bg, self.today_bg, self.corners,
                false, &[], &[], None, None, self.year, self.month,
            );

            // New grid (active)
            Self::paint_day_grid(
                canvas, &self.day_layouts, &self.prev_trailing_layouts, &self.next_trailing_layouts,
                self.first_weekday, self.days_in_month,
                rect.x1, grid_y, self.cell_size, new_x,
                self.hovered_day, self.selected_day, self.today_day,
                self.selected_bg, self.hover_bg, self.today_bg, self.corners,
                true, &self.indicators, &self.disabled_days, self.min_date, self.max_date,
                self.year, self.month,
            );
        } else {
            // Static render (no slide)
            for (col, layout) in self.weekday_layouts.iter().enumerate() {
                if let Some(tl) = layout {
                    let s = tl.size();
                    let cx = rect.x1 + col as f32 * self.cell_size + self.cell_size / 2.0;
                    canvas.draw_text(tl, (cx - s.width / 2.0) as i32, (weekday_y + (self.cell_size - s.height) / 2.0) as i32);
                }
            }

            Self::paint_day_grid(
                canvas, &self.day_layouts, &self.prev_trailing_layouts, &self.next_trailing_layouts,
                self.first_weekday, self.days_in_month,
                rect.x1, grid_y, self.cell_size, 0.0,
                self.hovered_day, self.selected_day, self.today_day,
                self.selected_bg, self.hover_bg, self.today_bg, self.corners,
                true, &self.indicators, &self.disabled_days, self.min_date, self.max_date,
                self.year, self.month,
            );
        }

        canvas.pop_clip();

        // ── Selector slides down ON TOP of the day grid ─────────────
        if selector_active {
            // Fade out the day grid by drawing a translucent overlay
            let grid_area = self.selector_grid_rect(&rect);
            let fade_alpha = (sel_t * 255.0) as u8;
            let fade_color = Color::from_rgba(
                self.selector_bg.red,
                self.selector_bg.green,
                self.selector_bg.blue,
                fade_alpha,
            );
            canvas.fill_rect(grid_area, fade_color);

            // Paint selector (bg + content animate down together)
            self.paint_selector_columns(canvas, rect, sel_t);
        }
    }

    fn paint_overlay(&self, canvas: &mut Canvas, rect: Rect) {
        if let (Some(day), Some(tl)) = (self.indicator_tooltip_day, &self.indicator_tooltip_layout) {
            let cell_idx = self.first_weekday + day - 1;
            let grid_y = rect.y1 + self.header_height + self.cell_size;
            let cx = rect.x1 + (cell_idx % 7) as f32 * self.cell_size;
            let cy = grid_y + (cell_idx / 7) as f32 * self.cell_size;
            let s = tl.size();
            let pad = 6.0;
            let tw = s.width + pad * 2.0;
            let th = s.height + pad * 2.0;
            let tx = cx + (self.cell_size - tw) / 2.0;
            let ty = cy - th - 4.0;
            let tr = Rect::new(tx, ty, tx + tw, ty + th);
            canvas.fill_rounded_rect(tr, Corners::all(4.0), colors::popover());
            canvas.stroke_rounded_rect(tr, Corners::all(4.0), 1, colors::border());
            canvas.draw_text(tl, (tx + pad) as i32, (ty + pad) as i32);
        }
    }

    fn children(&self) -> &[Box<dyn Widget>] { &[] }

    fn needs_animation(&self) -> bool {
        anim_active(self.selector_anim_start, ANIM_DURATION)
            || self.is_sliding()
    }

    fn event(&mut self, event: &WidgetEvent, rect: Rect) -> EventResponse {
        self.check_selector_closed();

        match event {
            WidgetEvent::Mouse(MouseEvent::MouseScrollEvent(delta)) => {
                if self.selector_view != SelectorView::None {
                    let show_m = matches!(self.selector_view, SelectorView::MonthColumn | SelectorView::Combined);
                    let show_y = matches!(self.selector_view, SelectorView::YearColumn | SelectorView::Combined);
                    if show_m && show_y {
                        if self.hovered_selector_month.is_some() {
                            self.month_scroll_offset -= delta * SELECTOR_ITEM_H;
                        } else if self.hovered_selector_year.is_some() {
                            self.year_scroll_offset -= delta * SELECTOR_ITEM_H;
                        }
                    } else if show_m {
                        self.month_scroll_offset -= delta * SELECTOR_ITEM_H;
                    } else if show_y {
                        self.year_scroll_offset -= delta * SELECTOR_ITEM_H;
                    }
                    self.clamp_scroll();
                    return EventResponse { status: EventStatus::Consumed, ..Default::default() };
                }
                EventResponse::default()
            }
            WidgetEvent::Mouse(MouseEvent::MouseClickEvent(e))
                if e.state == MouseState::Pressed && rect.contains(&e.position) =>
            {
                if self.selector_view != SelectorView::None && self.selector_anim_t() > 0.5 {
                    return self.handle_selector_click(e, &rect);
                }

                match self.header_hit(&e.position, &rect) {
                    HeaderHit::Prev if self.can_go_prev() => {
                        self.prev_month();
                        return EventResponse { status: EventStatus::Consumed, cursor: Some(CursorIcon::Pointer), ..Default::default() };
                    }
                    HeaderHit::Next if self.can_go_next() => {
                        self.next_month();
                        return EventResponse { status: EventStatus::Consumed, cursor: Some(CursorIcon::Pointer), ..Default::default() };
                    }
                    HeaderHit::Month if self.month_year_selector.is_some() => {
                        if self.selector_view != SelectorView::None {
                            self.close_selector();
                        } else {
                            let view = match self.month_year_selector.unwrap() {
                                MonthYearSelector::Separate => SelectorView::MonthColumn,
                                MonthYearSelector::Combined => SelectorView::Combined,
                            };
                            self.open_selector(view);
                        }
                        return EventResponse { status: EventStatus::Consumed, cursor: Some(CursorIcon::Pointer), ..Default::default() };
                    }
                    HeaderHit::Year if self.month_year_selector == Some(MonthYearSelector::Separate) => {
                        if self.selector_view != SelectorView::None {
                            self.close_selector();
                        } else {
                            self.open_selector(SelectorView::YearColumn);
                        }
                        return EventResponse { status: EventStatus::Consumed, cursor: Some(CursorIcon::Pointer), ..Default::default() };
                    }
                    _ => {}
                }

                if let Some(day) = self.day_at_position(&e.position, &rect)
                    && !self.is_day_disabled(day)
                {
                    self.selected_day = Some(day);
                    if let Some(ref mut cb) = self.on_select { cb(day); }
                    return EventResponse { status: EventStatus::Consumed, cursor: Some(CursorIcon::Pointer), ..Default::default() };
                }
                EventResponse::default()
            }
            WidgetEvent::Mouse(MouseEvent::MouseMoveEvent(pos)) => {
                if !rect.contains(pos) {
                    self.hovered_day = None;
                    self.hovered_header = None;
                    self.hovered_selector_month = None;
                    self.hovered_selector_year = None;
                    return EventResponse::default();
                }

                if self.selector_view != SelectorView::None && self.selector_anim_t() > 0.5 {
                    self.hovered_header = None;
                    return self.handle_selector_hover(pos, &rect);
                }

                // Header hover
                let hit = self.header_hit(pos, &rect);
                self.hovered_header = if matches!(hit, HeaderHit::None) { None } else { Some(hit) };

                match hit {
                    HeaderHit::Prev if self.can_go_prev() => { self.hovered_day = None; return EventResponse { status: EventStatus::Consumed, cursor: Some(CursorIcon::Pointer), ..Default::default() }; }
                    HeaderHit::Next if self.can_go_next() => { self.hovered_day = None; return EventResponse { status: EventStatus::Consumed, cursor: Some(CursorIcon::Pointer), ..Default::default() }; }
                    HeaderHit::Month if self.month_year_selector.is_some() => { self.hovered_day = None; return EventResponse { status: EventStatus::Consumed, cursor: Some(CursorIcon::Pointer), ..Default::default() }; }
                    HeaderHit::Year if self.month_year_selector == Some(MonthYearSelector::Separate) => { self.hovered_day = None; return EventResponse { status: EventStatus::Consumed, cursor: Some(CursorIcon::Pointer), ..Default::default() }; }
                    HeaderHit::Prev | HeaderHit::Next | HeaderHit::Month | HeaderHit::Year => { self.hovered_day = None; return EventResponse { status: EventStatus::Consumed, cursor: Some(CursorIcon::Default), ..Default::default() }; }
                    HeaderHit::None => {}
                }

                // Weekday row
                let wy = rect.y1 + self.header_height;
                if pos.y >= wy && pos.y < wy + self.cell_size {
                    self.hovered_day = None;
                    return EventResponse { status: EventStatus::Consumed, cursor: Some(CursorIcon::Default), ..Default::default() };
                }

                // Day grid
                if let Some(day) = self.day_at_position(pos, &rect) {
                    let dis = self.is_day_disabled(day);
                    self.hovered_day = if dis { None } else { Some(day) };
                    return EventResponse { status: EventStatus::Consumed, cursor: if dis { Some(CursorIcon::Default) } else { Some(CursorIcon::Pointer) }, ..Default::default() };
                }

                self.hovered_day = None;
                EventResponse { status: EventStatus::Consumed, cursor: Some(CursorIcon::Default), ..Default::default() }
            }
            _ => EventResponse::default(),
        }
    }

    #[cfg(feature = "a11y")]
    fn access_info(&self) -> aurora_a11y::NodeInfo {
        aurora_a11y::NodeInfo::new(aurora_a11y::accesskit::Role::Group).with_label("Calendar".to_string())
    }
}

// ─── Selector paint / event ─────────────────────────────────────────────────

impl Calendar {
    fn paint_selector_columns(&self, canvas: &mut Canvas, rect: Rect, t: f32) {
        let gr = self.selector_grid_rect(&rect);
        let anim_h = gr.height() * t;

        // Background + border animate down together with content
        let visible = Rect::new(gr.x1, gr.y1, gr.x2, gr.y1 + anim_h);
        let corners = Corners::new(0.0, 0.0, 6.0, 6.0);
        canvas.fill_rounded_rect(visible, corners, self.selector_bg);
        canvas.stroke_rounded_rect(visible, corners, 1, self.selector_border_color);

        // Clip content to the same animated height
        canvas.push_clip(visible);

        let show_m = matches!(self.selector_view, SelectorView::MonthColumn | SelectorView::Combined);
        let show_y = matches!(self.selector_view, SelectorView::YearColumn | SelectorView::Combined);

        if show_m && show_y {
            let hw = gr.width() / 2.0;
            let mc = Rect::new(gr.x1, gr.y1, gr.x1 + hw, gr.y2);
            let yc = Rect::new(gr.x1 + hw, gr.y1, gr.x2, gr.y2);
            self.paint_month_column(canvas, &mc);
            canvas.fill_rect(Rect::new(mc.x2 - 0.5, gr.y1 + 4.0, mc.x2 + 0.5, gr.y1 + anim_h - 4.0), self.selector_border_color);
            self.paint_year_column(canvas, &yc);
        } else if show_m {
            self.paint_month_column(canvas, &gr);
        } else if show_y {
            self.paint_year_column(canvas, &gr);
        }

        canvas.pop_clip();
    }

    fn paint_month_column(&self, canvas: &mut Canvas, col_rect: &Rect) {
        canvas.push_clip(*col_rect);
        for (i, _) in self.selector_month_layouts.iter().enumerate() {
            let m = i as u32 + 1;
            let cy = col_rect.y1 + i as f32 * SELECTOR_ITEM_H - self.month_scroll_offset;
            let ir = Rect::new(col_rect.x1, cy, col_rect.x2, cy + SELECTOR_ITEM_H);
            if m == self.month {
                canvas.fill_rounded_rect(ir, Corners::all(4.0), self.selected_bg);
            } else if self.hovered_selector_month == Some(m) {
                canvas.fill_rounded_rect(ir, Corners::all(4.0), self.hover_bg);
            }
            if let Some(Some(tl)) = self.selector_month_layouts.get(i) {
                let s = tl.size();
                canvas.draw_text(tl, (col_rect.x1 + (col_rect.width() - s.width) / 2.0) as i32, (cy + (SELECTOR_ITEM_H - s.height) / 2.0) as i32);
            }
        }
        canvas.pop_clip();
    }

    fn paint_year_column(&self, canvas: &mut Canvas, col_rect: &Rect) {
        canvas.push_clip(*col_rect);
        let base = self.year as i32 - SELECTOR_YEAR_COUNT / 2;
        for (i, _) in self.selector_year_layouts.iter().enumerate() {
            let yr = (base + i as i32) as u32;
            let cy = col_rect.y1 + i as f32 * SELECTOR_ITEM_H - self.year_scroll_offset;
            let ir = Rect::new(col_rect.x1, cy, col_rect.x2, cy + SELECTOR_ITEM_H);
            if yr == self.year {
                canvas.fill_rounded_rect(ir, Corners::all(4.0), self.selected_bg);
            } else if self.hovered_selector_year == Some(yr) {
                canvas.fill_rounded_rect(ir, Corners::all(4.0), self.hover_bg);
            }
            if let Some(Some(tl)) = self.selector_year_layouts.get(i) {
                let s = tl.size();
                canvas.draw_text(tl, (col_rect.x1 + (col_rect.width() - s.width) / 2.0) as i32, (cy + (SELECTOR_ITEM_H - s.height) / 2.0) as i32);
            }
        }
        canvas.pop_clip();
    }

    fn handle_selector_click(&mut self, e: &aurora_core::kmi::mouse::MouseClickEvent, rect: &Rect) -> EventResponse {
        let gr = self.selector_grid_rect(rect);
        if e.position.y < gr.y1 {
            self.close_selector();
            return EventResponse { status: EventStatus::Consumed, ..Default::default() };
        }
        let show_m = matches!(self.selector_view, SelectorView::MonthColumn | SelectorView::Combined);
        let show_y = matches!(self.selector_view, SelectorView::YearColumn | SelectorView::Combined);

        if show_m && show_y {
            let hw = gr.width() / 2.0;
            let mc = Rect::new(gr.x1, gr.y1, gr.x1 + hw, gr.y2);
            let yc = Rect::new(gr.x1 + hw, gr.y1, gr.x2, gr.y2);
            if let Some(m) = self.selector_month_at(&e.position, &mc) {
                self.month = m; self.close_selector();
                if let Some(ref mut cb) = self.on_month_change { cb(self.year, self.month); }
                return EventResponse { status: EventStatus::Consumed, ..Default::default() };
            }
            if let Some(y) = self.selector_year_at(&e.position, &yc) {
                self.year = y; self.close_selector();
                if let Some(ref mut cb) = self.on_month_change { cb(self.year, self.month); }
                return EventResponse { status: EventStatus::Consumed, ..Default::default() };
            }
        } else if show_m {
            if let Some(m) = self.selector_month_at(&e.position, &gr) {
                self.month = m; self.close_selector();
                if let Some(ref mut cb) = self.on_month_change { cb(self.year, self.month); }
                return EventResponse { status: EventStatus::Consumed, ..Default::default() };
            }
        } else if show_y
            && let Some(y) = self.selector_year_at(&e.position, &gr) {
                self.year = y; self.close_selector();
                if let Some(ref mut cb) = self.on_month_change { cb(self.year, self.month); }
                return EventResponse { status: EventStatus::Consumed, ..Default::default() };
            }
        EventResponse::default()
    }

    fn handle_selector_hover(&mut self, pos: &aurora_core::geometry::point::Point, rect: &Rect) -> EventResponse {
        let gr = self.selector_grid_rect(rect);
        self.hovered_selector_month = None;
        self.hovered_selector_year = None;

        // Header hover even when selector is open
        let hit = self.header_hit(pos, rect);
        self.hovered_header = if matches!(hit, HeaderHit::None) { None } else { Some(hit) };

        let show_m = matches!(self.selector_view, SelectorView::MonthColumn | SelectorView::Combined);
        let show_y = matches!(self.selector_view, SelectorView::YearColumn | SelectorView::Combined);
        if show_m && show_y {
            let hw = gr.width() / 2.0;
            let mc = Rect::new(gr.x1, gr.y1, gr.x1 + hw, gr.y2);
            let yc = Rect::new(gr.x1 + hw, gr.y1, gr.x2, gr.y2);
            self.hovered_selector_month = self.selector_month_at(pos, &mc);
            self.hovered_selector_year = self.selector_year_at(pos, &yc);
        } else if show_m {
            self.hovered_selector_month = self.selector_month_at(pos, &gr);
        } else if show_y {
            self.hovered_selector_year = self.selector_year_at(pos, &gr);
        }

        let hovering = self.hovered_selector_month.is_some() || self.hovered_selector_year.is_some();
        EventResponse {
            status: EventStatus::Consumed,
            cursor: Some(if hovering || self.hovered_header.is_some() { CursorIcon::Pointer } else { CursorIcon::Default }),
            ..Default::default()
        }
    }

    // ── Accessors for CalendarRange ─────────────────────────────────────

    pub(crate) fn header_height_val(&self) -> f32 { self.header_height }
    pub(crate) fn cell_size_val(&self) -> f32 { self.cell_size }
    pub(crate) fn day_layouts(&self) -> &[Option<aurora_text::text_layout::TextLayout>] { &self.day_layouts }
    pub(crate) fn selected_fg_val(&self) -> Color { self.selected_fg }
}
