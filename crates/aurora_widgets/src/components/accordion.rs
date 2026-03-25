use crate::widgets::{EventResponse, LayoutCtx, Widget};
use aurora_core::color::Color;
use aurora_core::geometry::edges::Edges;
use aurora_core::geometry::point::Point;
use aurora_core::geometry::rect::Rect;
use aurora_core::geometry::size::Size;
use aurora_core::kmi::cursor_icon::CursorIcon;
use aurora_core::kmi::mouse::{MouseEvent, MouseState};
use aurora_core::kmi::WidgetEvent;
use aurora_render::canvas::Canvas;

use super::colors;

/// A single section in an accordion.
struct AccordionSection {
    title: String,
    content: Box<dyn Widget>,
    expanded: bool,
    title_layout: Option<aurora_text::text_layout::TextLayout>,
    content_size: Size,
}

/// Expandable sections with a title and content. Click a section title to toggle
/// its content visibility.
///
/// # Example
/// ```ignore
/// Accordion::new()
///     .section("Section 1", Label::new("Content 1"))
///     .section("Section 2", Label::new("Content 2"))
///     .section("Section 3", Label::new("Content 3"))
/// ```
pub struct Accordion {
    sections: Vec<AccordionSection>,
    allow_multiple: bool,
    header_height: f32,
    header_padding: Edges,
    content_padding: Edges,
    border_color: Color,
    header_bg: Color,
    spacing: f32,
    width: Option<f32>,
    on_change: Option<Box<dyn FnMut(usize, bool)>>,
}

impl Accordion {
    pub fn new() -> Self {
        Self {
            sections: Vec::new(),
            allow_multiple: false,
            header_height: 44.0,
            header_padding: Edges::new(0.0, 16.0, 0.0, 16.0),
            content_padding: Edges::new(0.0, 16.0, 16.0, 16.0),
            border_color: colors::BORDER,
            header_bg: Color::TRANSPARENT,
            spacing: 0.0,
            width: None,
            on_change: None,
        }
    }

    pub fn section(mut self, title: impl Into<String>, content: impl Widget + 'static) -> Self {
        self.sections.push(AccordionSection {
            title: title.into(),
            content: Box::new(content),
            expanded: false,
            title_layout: None,
            content_size: Size::default(),
        });
        self
    }

    pub fn expanded(mut self, index: usize) -> Self {
        if let Some(section) = self.sections.get_mut(index) {
            section.expanded = true;
        }
        self
    }

    pub fn allow_multiple(mut self, allow: bool) -> Self {
        self.allow_multiple = allow;
        self
    }

    pub fn header_height(mut self, height: f32) -> Self {
        self.header_height = height;
        self
    }

    pub fn border_color(mut self, color: Color) -> Self {
        self.border_color = color;
        self
    }

    pub fn width(mut self, width: impl Into<f32>) -> Self {
        self.width = Some(width.into());
        self
    }

    pub fn on_change(mut self, cb: impl FnMut(usize, bool) + 'static) -> Self {
        self.on_change = Some(Box::new(cb));
        self
    }

    fn section_height(&self, i: usize) -> f32 {
        let s = &self.sections[i];
        let mut h = self.header_height;
        if s.expanded {
            h += self.content_padding.top + s.content_size.height + self.content_padding.bottom;
        }
        h
    }
}

impl Default for Accordion {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for Accordion {
    fn layout(&mut self, available: Size, ctx: &mut LayoutCtx) -> Size {
        let w = self.width.unwrap_or(available.width);
        let inner_w = w - self.content_padding.left - self.content_padding.right;
        let mut total_h = 0.0;

        for (i, section) in self.sections.iter_mut().enumerate() {
            // Layout title
            let mut opts = ctx.font_options.clone();
            opts.size = Some(14.0);
            opts.weight = Some(aurora_text::font_options::FontWeight::Medium);
            let tl = aurora_text::text_layout::TextLayout::new(
                ctx.font_manager, &section.title, &opts, colors::FOREGROUND, None,
            );
            section.title_layout = Some(tl);

            // Layout content
            if section.expanded {
                let content_available = Size::new(inner_w.max(0.0), f32::MAX);
                section.content_size = section.content.layout(content_available, ctx);
            }

            total_h += self.header_height;
            if section.expanded {
                total_h += self.content_padding.top + section.content_size.height + self.content_padding.bottom;
            }
            if i > 0 {
                total_h += self.spacing;
            }
        }

        Size::new(w, total_h)
    }

    fn paint(&self, canvas: &mut Canvas, rect: Rect) {
        let mut y = rect.y1;

        for (i, section) in self.sections.iter().enumerate() {
            if i > 0 {
                y += self.spacing;
            }

            let sec_h = self.section_height(i);

            // Header background
            if self.header_bg.alpha > 0 {
                let header_rect = Rect::new(rect.x1, y, rect.x2, y + self.header_height);
                canvas.fill_rect(header_rect, self.header_bg);
            }

            // Header text
            if let Some(ref tl) = section.title_layout {
                let s = tl.size();
                let tx = rect.x1 + self.header_padding.left;
                let ty = y + (self.header_height - s.height) / 2.0;
                canvas.draw_text(tl, tx as i32, ty as i32);
            }

            // Chevron indicator
            let chevron_x = rect.x2 - self.header_padding.right - 10.0;
            let chevron_cy = y + self.header_height / 2.0;
            if section.expanded {
                canvas.draw_line(
                    Point::new(chevron_x - 4.0, chevron_cy - 2.0),
                    Point::new(chevron_x, chevron_cy + 2.0),
                    1, colors::MUTED_FOREGROUND,
                );
                canvas.draw_line(
                    Point::new(chevron_x, chevron_cy + 2.0),
                    Point::new(chevron_x + 4.0, chevron_cy - 2.0),
                    1, colors::MUTED_FOREGROUND,
                );
            } else {
                canvas.draw_line(
                    Point::new(chevron_x - 2.0, chevron_cy - 4.0),
                    Point::new(chevron_x + 2.0, chevron_cy),
                    1, colors::MUTED_FOREGROUND,
                );
                canvas.draw_line(
                    Point::new(chevron_x + 2.0, chevron_cy),
                    Point::new(chevron_x - 2.0, chevron_cy + 4.0),
                    1, colors::MUTED_FOREGROUND,
                );
            }

            // Bottom border
            let border_y = y + self.header_height;
            canvas.fill_rect(
                Rect::new(rect.x1, border_y - 1.0, rect.x2, border_y),
                self.border_color,
            );

            // Content
            if section.expanded {
                let content_y = y + self.header_height + self.content_padding.top;
                let content_rect = Rect::new(
                    rect.x1 + self.content_padding.left,
                    content_y,
                    rect.x1 + self.content_padding.left + section.content_size.width,
                    content_y + section.content_size.height,
                );
                section.content.paint(canvas, content_rect);
            }

            y += sec_h;
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
                let mut y = rect.y1;
                for i in 0..self.sections.len() {
                    if i > 0 {
                        y += self.spacing;
                    }
                    let header_rect = Rect::new(rect.x1, y, rect.x2, y + self.header_height);
                    if header_rect.contains(&e.position) {
                        let new_state = !self.sections[i].expanded;
                        if !self.allow_multiple && new_state {
                            for j in 0..self.sections.len() {
                                if j != i {
                                    self.sections[j].expanded = false;
                                }
                            }
                        }
                        self.sections[i].expanded = new_state;
                        if let Some(ref mut cb) = self.on_change {
                            cb(i, self.sections[i].expanded);
                        }
                        return EventResponse {
                            handled: true,
                            cursor: Some(CursorIcon::Pointer),
                            ..Default::default()
                        };
                    }
                    y += self.section_height(i);
                }
                EventResponse::default()
            }
            WidgetEvent::Mouse(MouseEvent::MouseMoveEvent(pos)) if rect.contains(pos) => {
                let mut y = rect.y1;
                for i in 0..self.sections.len() {
                    if i > 0 {
                        y += self.spacing;
                    }
                    let header_rect = Rect::new(rect.x1, y, rect.x2, y + self.header_height);
                    if header_rect.contains(pos) {
                        return EventResponse {
                            cursor: Some(CursorIcon::Pointer),
                            ..Default::default()
                        };
                    }
                    y += self.section_height(i);
                }
                // Forward to expanded content
                y = rect.y1;
                for (i, section) in self.sections.iter_mut().enumerate() {
                    if i > 0 {
                        y += self.spacing;
                    }
                    if section.expanded {
                        let content_y = y + self.header_height + self.content_padding.top;
                        let content_rect = Rect::new(
                            rect.x1 + self.content_padding.left,
                            content_y,
                            rect.x1 + self.content_padding.left + section.content_size.width,
                            content_y + section.content_size.height,
                        );
                        let resp = section.content.event(event, content_rect);
                        if resp.handled {
                            return resp;
                        }
                    }
                    let sec_h = self.header_height
                        + if section.expanded { self.content_padding.top + section.content_size.height + self.content_padding.bottom } else { 0.0 };
                    y += sec_h;
                }
                EventResponse::default()
            }
            _ => {
                let mut y = rect.y1;
                for (i, section) in self.sections.iter_mut().enumerate() {
                    if i > 0 {
                        y += self.spacing;
                    }
                    if section.expanded {
                        let content_y = y + self.header_height + self.content_padding.top;
                        let content_rect = Rect::new(
                            rect.x1 + self.content_padding.left,
                            content_y,
                            rect.x1 + self.content_padding.left + section.content_size.width,
                            content_y + section.content_size.height,
                        );
                        let resp = section.content.event(event, content_rect);
                        if resp.handled {
                            return resp;
                        }
                    }
                    let sec_h = self.header_height
                        + if section.expanded { self.content_padding.top + section.content_size.height + self.content_padding.bottom } else { 0.0 };
                    y += sec_h;
                }
                EventResponse::default()
            }
        }
    }
}
