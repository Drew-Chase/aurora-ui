use crate::widgets::{EventResponse, LayoutCtx, Widget};
use aurora_core::color::Color;
use aurora_core::geometry::corners::Corners;
use aurora_core::geometry::rect::Rect;
use aurora_core::geometry::size::Size;
use aurora_core::kmi::WidgetEvent;
use aurora_core::kmi::cursor_icon::CursorIcon;
use aurora_core::kmi::mouse::{MouseEvent, MouseState};
use aurora_render::canvas::Canvas;

use super::colors;

/// A calendar date grid displaying a month view.
///
/// # Example
/// ```ignore
/// Calendar::new()
///     .year(2026)
///     .month(3)
///     .selected_day(25)
///     .on_select(|day| println!("selected day: {day}"))
/// ```
pub struct Calendar {
    year: u32,
    month: u32,
    selected_day: Option<u32>,
    cell_size: f32,
    header_height: f32,
    corners: Corners,
    selected_bg: Color,
    selected_fg: Color,
    _today_bg: Color,
    today_day: Option<u32>,
    on_select: Option<Box<dyn FnMut(u32)>>,
    on_month_change: Option<Box<dyn FnMut(u32, u32)>>,
    day_layouts: Vec<Option<aurora_text::text_layout::TextLayout>>,
    weekday_layouts: Vec<Option<aurora_text::text_layout::TextLayout>>,
    month_label_layout: Option<aurora_text::text_layout::TextLayout>,
    prev_layout: Option<aurora_text::text_layout::TextLayout>,
    next_layout: Option<aurora_text::text_layout::TextLayout>,
    days_in_month: u32,
    first_weekday: u32,
}

impl Calendar {
    pub fn new() -> Self {
        Self {
            year: 2026,
            month: 1,
            selected_day: None,
            cell_size: 40.0,
            header_height: 40.0,
            corners: Corners::all(9999.0),
            selected_bg: colors::primary(),
            selected_fg: colors::primary_foreground(),
            _today_bg: colors::accent(),
            today_day: None,
            on_select: None,
            on_month_change: None,
            day_layouts: Vec::new(),
            weekday_layouts: Vec::new(),
            month_label_layout: None,
            prev_layout: None,
            next_layout: None,
            days_in_month: 31,
            first_weekday: 0,
        }
    }

    pub fn year(mut self, year: u32) -> Self {
        self.year = year;
        self
    }

    pub fn month(mut self, month: u32) -> Self {
        self.month = month.clamp(1, 12);
        self
    }

    pub fn selected_day(mut self, day: u32) -> Self {
        self.selected_day = Some(day);
        self
    }

    pub fn today(mut self, day: u32) -> Self {
        self.today_day = Some(day);
        self
    }

    pub fn cell_size(mut self, size: f32) -> Self {
        self.cell_size = size;
        self
    }

    pub fn on_select(mut self, cb: impl FnMut(u32) + 'static) -> Self {
        self.on_select = Some(Box::new(cb));
        self
    }

    pub fn on_month_change(mut self, cb: impl FnMut(u32, u32) + 'static) -> Self {
        self.on_month_change = Some(Box::new(cb));
        self
    }

    fn compute_days_in_month(year: u32, month: u32) -> u32 {
        match month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 => {
                if (year.is_multiple_of(4) && !year.is_multiple_of(100)) || year.is_multiple_of(400)
                {
                    29
                } else {
                    28
                }
            }
            _ => 30,
        }
    }

    /// Computes the day of week (0=Sun, 6=Sat) for the first day of the month
    /// using Zeller-like formula (Tomohiko Sakamoto's algorithm).
    fn first_day_of_week(year: u32, month: u32) -> u32 {
        let t = [0u32, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];
        let y = if month < 3 { year - 1 } else { year };
        let m = month as usize;
        (y + y / 4 - y / 100 + y / 400 + t[m - 1] + 1) % 7
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
            _ => "",
        }
    }

    fn prev_month(&mut self) {
        if self.month == 1 {
            self.month = 12;
            self.year -= 1;
        } else {
            self.month -= 1;
        }
        if let Some(ref mut cb) = self.on_month_change {
            cb(self.year, self.month);
        }
    }

    fn next_month(&mut self) {
        if self.month == 12 {
            self.month = 1;
            self.year += 1;
        } else {
            self.month += 1;
        }
        if let Some(ref mut cb) = self.on_month_change {
            cb(self.year, self.month);
        }
    }

    fn total_rows(&self) -> u32 {
        let total_cells = self.first_weekday + self.days_in_month;
        total_cells.div_ceil(7)
    }
}

impl Default for Calendar {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for Calendar {
    fn layout(&mut self, _available: Size, ctx: &mut LayoutCtx) -> Size {
        self.days_in_month = Self::compute_days_in_month(self.year, self.month);
        self.first_weekday = Self::first_day_of_week(self.year, self.month);

        let grid_w = self.cell_size * 7.0;

        // Month label
        let month_str = format!("{} {}", Self::month_name(self.month), self.year);
        let mut opts = ctx.font_options.clone();
        opts.size = Some(14.0);
        opts.weight = Some(aurora_text::font_options::FontWeight::Medium);
        let tl = aurora_text::text_layout::TextLayout::new(
            ctx.font_manager,
            &month_str,
            &opts,
            colors::foreground(),
            None,
        );
        self.month_label_layout = Some(tl);

        // Prev/Next arrows
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
        let weekdays = ["Su", "Mo", "Tu", "We", "Th", "Fr", "Sa"];
        for wd in &weekdays {
            let mut opts = ctx.font_options.clone();
            opts.size = Some(12.0);
            opts.weight = Some(aurora_text::font_options::FontWeight::Medium);
            let tl = aurora_text::text_layout::TextLayout::new(
                ctx.font_manager,
                wd,
                &opts,
                colors::foreground(),
                None,
            );
            self.weekday_layouts.push(Some(tl));
        }

        // Day number layouts
        self.day_layouts.clear();
        for day in 1..=self.days_in_month {
            let is_selected = self.selected_day == Some(day);
            let mut opts = ctx.font_options.clone();
            opts.size = Some(14.0);
            opts.weight = Some(aurora_text::font_options::FontWeight::Normal);
            let fg = if is_selected {
                self.selected_fg
            } else {
                colors::foreground()
            };
            let text = day.to_string();
            let tl =
                aurora_text::text_layout::TextLayout::new(ctx.font_manager, &text, &opts, fg, None);
            self.day_layouts.push(Some(tl));
        }

        let rows = self.total_rows();
        let total_h = self.header_height + self.cell_size + self.cell_size * rows as f32;
        Size::new(grid_w, total_h)
    }

    fn paint(&self, canvas: &mut Canvas, rect: Rect) {
        let grid_w = self.cell_size * 7.0;

        // Month header with prev/next
        let header_rect = Rect::new(
            rect.x1,
            rect.y1,
            rect.x1 + grid_w,
            rect.y1 + self.header_height,
        );

        // Prev button
        if let Some(ref tl) = self.prev_layout {
            let _s = tl.size();
            let _tw = _s.width;
            let th = _s.height;
            let tx = header_rect.x1 + 8.0;
            let ty = header_rect.y1 + (self.header_height - th) / 2.0;
            canvas.draw_text(tl, tx as i32, ty as i32);
        }

        // Month label centered
        if let Some(ref tl) = self.month_label_layout {
            let _s = tl.size();
            let tw = _s.width;
            let th = _s.height;
            let tx = header_rect.x1 + (grid_w - tw) / 2.0;
            let ty = header_rect.y1 + (self.header_height - th) / 2.0;
            canvas.draw_text(tl, tx as i32, ty as i32);
        }

        // Next button
        if let Some(ref tl) = self.next_layout {
            let _s = tl.size();
            let tw = _s.width;
            let th = _s.height;
            let tx = header_rect.x2 - tw - 8.0;
            let ty = header_rect.y1 + (self.header_height - th) / 2.0;
            canvas.draw_text(tl, tx as i32, ty as i32);
        }

        // Weekday headers
        let weekday_y = rect.y1 + self.header_height;
        for (col, layout) in self.weekday_layouts.iter().enumerate() {
            if let Some(tl) = layout {
                let _s = tl.size();
                let tw = _s.width;
                let th = _s.height;
                let cx = rect.x1 + col as f32 * self.cell_size + self.cell_size / 2.0;
                let tx = cx - tw / 2.0;
                let ty = weekday_y + (self.cell_size - th) / 2.0;
                canvas.draw_text(tl, tx as i32, ty as i32);
            }
        }

        // Day grid
        let grid_y = weekday_y + self.cell_size;
        for day in 1..=self.days_in_month {
            let cell_idx = self.first_weekday + day - 1;
            let row = cell_idx / 7;
            let col = cell_idx % 7;

            let cx = rect.x1 + col as f32 * self.cell_size;
            let cy = grid_y + row as f32 * self.cell_size;
            let cell_rect = Rect::new(cx, cy, cx + self.cell_size, cy + self.cell_size);

            let is_selected = self.selected_day == Some(day);
            let is_today = self.today_day == Some(day);

            if is_selected {
                canvas.fill_rounded_rect(cell_rect, self.corners, self.selected_bg);
            } else if is_today {
                canvas.stroke_rounded_rect(cell_rect, self.corners, 1, colors::primary());
            }

            if let Some(Some(tl)) = self.day_layouts.get((day - 1) as usize) {
                let _s = tl.size();
                let tw = _s.width;
                let th = _s.height;
                let tx = cx + (self.cell_size - tw) / 2.0;
                let ty = cy + (self.cell_size - th) / 2.0;
                canvas.draw_text(tl, tx as i32, ty as i32);
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
                let header_rect =
                    Rect::new(rect.x1, rect.y1, rect.x2, rect.y1 + self.header_height);

                // Prev/next navigation
                if header_rect.contains(&e.position) {
                    let third = header_rect.width() / 3.0;
                    if e.position.x < header_rect.x1 + third {
                        self.prev_month();
                        return EventResponse {
                            handled: true,
                            cursor: Some(CursorIcon::Pointer),
                            ..Default::default()
                        };
                    } else if e.position.x > header_rect.x2 - third {
                        self.next_month();
                        return EventResponse {
                            handled: true,
                            cursor: Some(CursorIcon::Pointer),
                            ..Default::default()
                        };
                    }
                    return EventResponse::default();
                }

                // Day grid click
                let grid_y = rect.y1 + self.header_height + self.cell_size;
                if e.position.y >= grid_y {
                    let col = ((e.position.x - rect.x1) / self.cell_size) as u32;
                    let row = ((e.position.y - grid_y) / self.cell_size) as u32;
                    if col < 7 {
                        let cell_idx = row * 7 + col;
                        if cell_idx >= self.first_weekday {
                            let day = cell_idx - self.first_weekday + 1;
                            if day >= 1 && day <= self.days_in_month {
                                self.selected_day = Some(day);
                                if let Some(ref mut cb) = self.on_select {
                                    cb(day);
                                }
                                return EventResponse {
                                    handled: true,
                                    cursor: Some(CursorIcon::Pointer),
                                    ..Default::default()
                                };
                            }
                        }
                    }
                }
                EventResponse::default()
            }
            WidgetEvent::Mouse(MouseEvent::MouseMoveEvent(pos)) if rect.contains(pos) => {
                EventResponse {
                    handled: true,
                    cursor: Some(CursorIcon::Pointer),
                    ..Default::default()
                }
            }
            _ => EventResponse::default(),
        }
    }
    #[cfg(feature = "a11y")]
    fn access_info(&self) -> aurora_a11y::NodeInfo {
        aurora_a11y::NodeInfo::new(aurora_a11y::accesskit::Role::Group)
            .with_label("Calendar".to_string())
    }
}
