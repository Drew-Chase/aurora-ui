use crate::widgets::{EventResponse, EventStatus, LayoutCtx, Widget};
use aurora_core::color::Color;
use aurora_core::geometry::corners::Corners;
use aurora_core::geometry::rect::Rect;
use aurora_core::geometry::size::Size;
use aurora_core::kmi::WidgetEvent;
use aurora_core::kmi::cursor_icon::CursorIcon;
use aurora_core::kmi::mouse::{MouseEvent, MouseState};
use aurora_render::canvas::Canvas;

use super::calendar::{Calendar, CellIndicator, MonthYearSelector};
use super::colors;

/// A calendar widget that supports selecting a date range.
///
/// Click once to set the range start, click again to set the range end.
/// Days between start and end are highlighted with a translucent band.
/// A third click resets the range and starts a new selection.
///
/// # Example
/// ```ignore
/// CalendarRange::new()
///     .year(2026)
///     .month(3)
///     .on_range_select(|start, end| println!("range: {start}–{end}"))
/// ```
pub struct CalendarRange {
    calendar: Calendar,
    range_start: Option<u32>,
    range_end: Option<u32>,
    range_bg: Color,
    on_range_select: Option<Box<dyn FnMut(u32, u32)>>,
}

impl CalendarRange {
    pub fn new() -> Self {
        Self {
            calendar: Calendar::new(),
            range_start: None,
            range_end: None,
            range_bg: colors::primary().opacity(0.2),
            on_range_select: None,
        }
    }

    pub fn year(mut self, year: u32) -> Self {
        self.calendar = self.calendar.year(year);
        self
    }

    pub fn month(mut self, month: u32) -> Self {
        self.calendar = self.calendar.month(month);
        self
    }

    pub fn today(mut self, day: u32) -> Self {
        self.calendar = self.calendar.today(day);
        self
    }

    pub fn cell_size(mut self, size: f32) -> Self {
        self.calendar = self.calendar.cell_size(size);
        self
    }

    pub fn range_bg(mut self, color: Color) -> Self {
        self.range_bg = color;
        self
    }

    pub fn on_range_select(mut self, cb: impl FnMut(u32, u32) + 'static) -> Self {
        self.on_range_select = Some(Box::new(cb));
        self
    }

    pub fn on_month_change(mut self, cb: impl FnMut(u32, u32) + 'static) -> Self {
        self.calendar = self.calendar.on_month_change(cb);
        self
    }

    pub fn disabled_day(mut self, day: u32) -> Self {
        self.calendar = self.calendar.disabled_day(day);
        self
    }

    pub fn disabled_days(mut self, days: Vec<u32>) -> Self {
        self.calendar = self.calendar.disabled_days(days);
        self
    }

    pub fn min_date(mut self, year: u32, month: u32, day: u32) -> Self {
        self.calendar = self.calendar.min_date(year, month, day);
        self
    }

    pub fn max_date(mut self, year: u32, month: u32, day: u32) -> Self {
        self.calendar = self.calendar.max_date(year, month, day);
        self
    }

    pub fn month_year_selector(mut self, selector: MonthYearSelector) -> Self {
        self.calendar = self.calendar.month_year_selector(selector);
        self
    }

    pub fn indicator(mut self, indicator: CellIndicator) -> Self {
        self.calendar = self.calendar.indicator(indicator);
        self
    }

    pub fn indicators(mut self, indicators: Vec<CellIndicator>) -> Self {
        self.calendar = self.calendar.indicators(indicators);
        self
    }

    fn range_ordered(&self) -> Option<(u32, u32)> {
        match (self.range_start, self.range_end) {
            (Some(s), Some(e)) => {
                if s <= e {
                    Some((s, e))
                } else {
                    Some((e, s))
                }
            }
            _ => None,
        }
    }
}

impl Default for CalendarRange {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for CalendarRange {
    fn layout(&mut self, available: Size, ctx: &mut LayoutCtx) -> Size {
        self.calendar.layout(available, ctx)
    }

    fn paint(&self, canvas: &mut Canvas, rect: Rect) {
        // Paint the base calendar (header, weekdays, day cells)
        self.calendar.paint(canvas, rect);

        // Overlay the range band on top of the day grid
        if let Some((start, end)) = self.range_ordered() {
            let weekday_y = rect.y1 + self.calendar.header_height_val() + self.calendar.cell_size_val();
            let cs = self.calendar.cell_size_val();
            let selected_bg = colors::primary();

            for day in start..=end {
                if day < 1 || day > self.calendar.days_in_month {
                    continue;
                }
                let cell_idx = self.calendar.first_weekday + day - 1;
                let row = cell_idx / 7;
                let col = cell_idx % 7;
                let cx = rect.x1 + col as f32 * cs;
                let cy = weekday_y + row as f32 * cs;
                let cell_rect = Rect::new(cx, cy, cx + cs, cy + cs);

                let is_endpoint = day == start || day == end;
                if is_endpoint {
                    // Full-opacity selected background on endpoints
                    let corners = if start == end {
                        Corners::all(9999.0)
                    } else if day == start {
                        Corners::new(9999.0, 0.0, 0.0, 9999.0)
                    } else {
                        Corners::new(0.0, 9999.0, 9999.0, 0.0)
                    };
                    canvas.fill_rounded_rect(cell_rect, corners, selected_bg);
                } else {
                    // Translucent range background on in-between days
                    canvas.fill_rect(cell_rect, self.range_bg);
                }
            }

            // Repaint day labels on top of the range band so text stays visible
            for day in start..=end {
                if day < 1 || day > self.calendar.days_in_month {
                    continue;
                }
                let cell_idx = self.calendar.first_weekday + day - 1;
                let row = cell_idx / 7;
                let col = cell_idx % 7;
                let cx = rect.x1 + col as f32 * cs;
                let cy = weekday_y + row as f32 * cs;

                if let Some(Some(tl)) = self.calendar.day_layouts().get((day - 1) as usize) {
                    let s = tl.size();
                    let tx = cx + (cs - s.width) / 2.0;
                    let ty = cy + (cs - s.height) / 2.0;
                    canvas.draw_text(tl, tx as i32, ty as i32);
                }
            }
        }
    }

    fn paint_overlay(&self, canvas: &mut Canvas, rect: Rect) {
        self.calendar.paint_overlay(canvas, rect);
    }

    fn children(&self) -> &[Box<dyn Widget>] {
        &[]
    }

    fn event(&mut self, event: &WidgetEvent, rect: Rect) -> EventResponse {
        // Intercept day clicks for range selection
        if let WidgetEvent::Mouse(MouseEvent::MouseClickEvent(e)) = event {
            if e.state == MouseState::Pressed && rect.contains(&e.position) {
                if let Some(day) = self.calendar.day_at_position(&e.position, &rect) {
                    if !self.calendar.is_day_disabled(day) {
                        if self.range_start.is_some() && self.range_end.is_none() {
                            // Second click: set end
                            self.range_end = Some(day);
                            if let Some((s, e)) = self.range_ordered() {
                                if let Some(ref mut cb) = self.on_range_select {
                                    cb(s, e);
                                }
                            }
                        } else {
                            // First click (or third+ click resets)
                            self.range_start = Some(day);
                            self.range_end = None;
                        }
                        return EventResponse {
                            status: EventStatus::Consumed,
                            cursor: Some(CursorIcon::Pointer),
                            ..Default::default()
                        };
                    }
                }
            }
        }

        // Delegate everything else to the inner calendar
        self.calendar.event(event, rect)
    }

    #[cfg(feature = "a11y")]
    fn access_info(&self) -> aurora_a11y::NodeInfo {
        aurora_a11y::NodeInfo::new(aurora_a11y::accesskit::Role::Group)
            .with_label("Calendar range".to_string())
    }
}

