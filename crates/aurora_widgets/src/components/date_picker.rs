use crate::widgets::{EventResponse, LayoutCtx, Widget};
use aurora_core::color::Color;
use aurora_core::geometry::corners::Corners;
use aurora_core::geometry::edges::Edges;
use aurora_core::geometry::rect::Rect;
use aurora_core::geometry::size::Size;
use aurora_core::kmi::cursor_icon::CursorIcon;
use aurora_core::kmi::mouse::{MouseEvent, MouseState};
use aurora_core::kmi::WidgetEvent;
use aurora_render::canvas::Canvas;

use super::colors;
use super::calendar::Calendar;

/// A date picker input that opens a calendar dropdown.
///
/// # Example
/// ```ignore
/// DatePicker::new()
///     .placeholder("Pick a date")
///     .year(2026)
///     .month(3)
///     .on_select(|y, m, d| println!("date: {y}-{m}-{d}"))
/// ```
pub struct DatePicker {
    year: u32,
    month: u32,
    selected_day: Option<u32>,
    placeholder: String,
    open: bool,
    height: f32,
    background: Color,
    border_color: Color,
    corners: Corners,
    padding: Edges,
    width: Option<f32>,
    on_select: Option<Box<dyn FnMut(u32, u32, u32)>>,
    display_layout: Option<aurora_text::text_layout::TextLayout>,
    calendar: Calendar,
    calendar_size: Size,
}

impl DatePicker {
    pub fn new() -> Self {
        Self {
            year: 2026,
            month: 1,
            selected_day: None,
            placeholder: "Pick a date".to_string(),
            open: false,
            height: 40.0,
            background: colors::BACKGROUND,
            border_color: colors::INPUT_BORDER,
            corners: Corners::all(6.0),
            padding: Edges::new(0.0, 12.0, 0.0, 12.0),
            width: None,
            on_select: None,
            display_layout: None,
            calendar: Calendar::new(),
            calendar_size: Size::default(),
        }
    }

    pub fn year(mut self, year: u32) -> Self {
        self.year = year;
        self.calendar = self.calendar.year(year);
        self
    }

    pub fn month(mut self, month: u32) -> Self {
        self.month = month;
        self.calendar = self.calendar.month(month);
        self
    }

    pub fn selected_day(mut self, day: u32) -> Self {
        self.selected_day = Some(day);
        self.calendar = self.calendar.selected_day(day);
        self
    }

    pub fn placeholder(mut self, text: impl Into<String>) -> Self {
        self.placeholder = text.into();
        self
    }

    pub fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }

    pub fn width(mut self, width: impl Into<f32>) -> Self {
        self.width = Some(width.into());
        self
    }

    pub fn on_select(mut self, cb: impl FnMut(u32, u32, u32) + 'static) -> Self {
        self.on_select = Some(Box::new(cb));
        self
    }

    fn dropdown_rect(&self, trigger_rect: &Rect) -> Rect {
        Rect::new(
            trigger_rect.x1,
            trigger_rect.y2 + 4.0,
            trigger_rect.x1 + self.calendar_size.width,
            trigger_rect.y2 + 4.0 + self.calendar_size.height + 16.0,
        )
    }
}

impl Default for DatePicker {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for DatePicker {
    fn layout(&mut self, available: Size, ctx: &mut LayoutCtx) -> Size {
        let w = self.width.unwrap_or(available.width);

        // Display text
        let display_text = if let Some(day) = self.selected_day {
            format!("{:04}-{:02}-{:02}", self.year, self.month, day)
        } else {
            self.placeholder.clone()
        };

        let _text_color = if self.selected_day.is_some() {
            colors::FOREGROUND
        } else {
            colors::MUTED_FOREGROUND
        };

        let mut opts = ctx.font_options.clone();
        opts.size = Some(14.0);
        opts.weight = Some(aurora_text::font_options::FontWeight::Normal);
        let inner_w = w - self.padding.left - self.padding.right;
        let mut tl = aurora_text::text_layout::TextLayout::new(ctx.font_manager, &display_text, &opts, colors::FOREGROUND, None);
        tl.set_max_width(ctx.font_manager, inner_w.max(0.0));
        self.display_layout = Some(tl);

        // Layout calendar
        if self.open {
            self.calendar_size = self.calendar.layout(
                Size::new(40.0 * 7.0, f32::MAX),
                ctx,
            );
        }

        Size::new(w, self.height)
    }

    fn paint(&self, canvas: &mut Canvas, rect: Rect) {
        // Input box
        let border_color = if self.open { colors::RING } else { self.border_color };
        canvas.fill_rounded_rect(rect, self.corners, self.background);
        canvas.stroke_rounded_rect(rect, self.corners, 1, border_color);

        // Display text
        if let Some(ref tl) = self.display_layout {
            let th = tl.size().height;
            let tx = rect.x1 + self.padding.left;
            let ty = rect.y1 + (rect.height() - th) / 2.0;
            canvas.draw_text(tl, tx as i32, ty as i32);
        }

        // Calendar icon (simple lines)
        let icon_x = rect.x2 - self.padding.right - 16.0;
        let icon_y = rect.y1 + (rect.height() - 16.0) / 2.0;
        canvas.stroke_rect(
            Rect::new(icon_x, icon_y + 2.0, icon_x + 14.0, icon_y + 14.0),
            1u32,
            colors::MUTED_FOREGROUND,
        );
        canvas.fill_rect(
            Rect::new(icon_x, icon_y + 6.0, icon_x + 14.0, icon_y + 7.0),
            colors::MUTED_FOREGROUND,
        );

        if !self.open {
            return;
        }

        // Calendar dropdown
        let dr = self.dropdown_rect(&rect);
        let drop_corners = Corners::all(8.0);
        canvas.fill_rounded_rect(dr, drop_corners, colors::POPOVER);
        canvas.stroke_rounded_rect(dr, drop_corners, 1, colors::BORDER);

        let cal_rect = Rect::new(
            dr.x1 + 8.0,
            dr.y1 + 8.0,
            dr.x1 + 8.0 + self.calendar_size.width,
            dr.y1 + 8.0 + self.calendar_size.height,
        );
        self.calendar.paint(canvas, cal_rect);
    }

    fn children(&self) -> &[Box<dyn Widget>] {
        &[]
    }

    fn event(&mut self, event: &WidgetEvent, rect: Rect) -> EventResponse {
        match event {
            WidgetEvent::Mouse(MouseEvent::MouseClickEvent(e))
                if e.state == MouseState::Pressed =>
            {
                if rect.contains(&e.position) {
                    self.open = !self.open;
                    return EventResponse {
                        handled: true,
                        cursor: Some(CursorIcon::Pointer),
                        ..Default::default()
                    };
                }
                if self.open {
                    let dr = self.dropdown_rect(&rect);
                    if dr.contains(&e.position) {
                        // Forward to calendar
                        let cal_rect = Rect::new(
                            dr.x1 + 8.0,
                            dr.y1 + 8.0,
                            dr.x1 + 8.0 + self.calendar_size.width,
                            dr.y1 + 8.0 + self.calendar_size.height,
                        );
                        let resp = self.calendar.event(event, cal_rect);
                        // Check if a day was selected in the calendar
                        // Since Calendar manages its own selected_day,
                        // we sync afterwards
                        if resp.handled {
                            // Check if calendar updated its selection
                            // We need to read it back (the calendar stores it internally)
                            return resp;
                        }
                    }
                    self.open = false;
                    return EventResponse {
                        handled: true,
                        ..Default::default()
                    };
                }
                EventResponse::default()
            }
            WidgetEvent::Mouse(MouseEvent::MouseMoveEvent(pos)) => {
                if rect.contains(pos) {
                    return EventResponse {
                        cursor: Some(CursorIcon::Pointer),
                        ..Default::default()
                    };
                }
                if self.open {
                    let dr = self.dropdown_rect(&rect);
                    if dr.contains(pos) {
                        let cal_rect = Rect::new(
                            dr.x1 + 8.0,
                            dr.y1 + 8.0,
                            dr.x1 + 8.0 + self.calendar_size.width,
                            dr.y1 + 8.0 + self.calendar_size.height,
                        );
                        return self.calendar.event(event, cal_rect);
                    }
                }
                EventResponse::default()
            }
            _ => EventResponse::default(),
        }
    }
}
