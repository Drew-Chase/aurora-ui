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

/// A single collapsible section with a title and toggleable content.
///
/// # Example
/// ```ignore
/// Collapsible::new("Details")
///     .expanded(true)
///     .child(Label::new("Hidden content here"))
/// ```
pub struct Collapsible {
    title: String,
    expanded: bool,
    child: Option<Box<dyn Widget>>,
    header_height: f32,
    content_padding: Edges,
    border_color: Color,
    width: Option<f32>,
    on_toggle: Option<Box<dyn FnMut(bool)>>,
    title_layout: Option<aurora_text::text_layout::TextLayout>,
    child_size: Size,
}

impl Collapsible {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            expanded: false,
            child: None,
            header_height: 40.0,
            content_padding: Edges::new(8.0, 0.0, 8.0, 0.0),
            border_color: colors::BORDER,
            width: None,
            on_toggle: None,
            title_layout: None,
            child_size: Size::default(),
        }
    }

    pub fn expanded(mut self, expanded: bool) -> Self {
        self.expanded = expanded;
        self
    }

    pub fn child(mut self, widget: impl Widget + 'static) -> Self {
        self.child = Some(Box::new(widget));
        self
    }

    pub fn header_height(mut self, height: f32) -> Self {
        self.header_height = height;
        self
    }

    pub fn content_padding(mut self, padding: Edges) -> Self {
        self.content_padding = padding;
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

    pub fn on_toggle(mut self, cb: impl FnMut(bool) + 'static) -> Self {
        self.on_toggle = Some(Box::new(cb));
        self
    }
}

impl Widget for Collapsible {
    fn layout(&mut self, available: Size, ctx: &mut LayoutCtx) -> Size {
        let w = self.width.unwrap_or(available.width);

        // Title layout
        let mut opts = ctx.font_options.clone();
        opts.size = Some(14.0);
        opts.weight = Some(aurora_text::font_options::FontWeight::Medium);
        let tl = aurora_text::text_layout::TextLayout::new(ctx.font_manager, &self.title, &opts, colors::FOREGROUND, None);
        self.title_layout = Some(tl);

        let mut total_h = self.header_height;

        if self.expanded {
            if let Some(ref mut child) = self.child {
                let content_w = w - self.content_padding.left - self.content_padding.right;
                let content_available = Size::new(content_w.max(0.0), f32::MAX);
                self.child_size = child.layout(content_available, ctx);
                total_h += self.content_padding.top + self.child_size.height + self.content_padding.bottom;
            }
        }

        Size::new(w, total_h)
    }

    fn paint(&self, canvas: &mut Canvas, rect: Rect) {
        // Header
        let header_rect = Rect::new(rect.x1, rect.y1, rect.x2, rect.y1 + self.header_height);

        // Chevron
        let chevron_x = rect.x1 + 8.0;
        let chevron_cy = rect.y1 + self.header_height / 2.0;
        if self.expanded {
            canvas.draw_line(
                Point::new(chevron_x - 4.0, chevron_cy - 2.0),
                Point::new(chevron_x, chevron_cy + 2.0),
                1,
                colors::MUTED_FOREGROUND,
            );
            canvas.draw_line(
                Point::new(chevron_x, chevron_cy + 2.0),
                Point::new(chevron_x + 4.0, chevron_cy - 2.0),
                1,
                colors::MUTED_FOREGROUND,
            );
        } else {
            canvas.draw_line(
                Point::new(chevron_x - 2.0, chevron_cy - 4.0),
                Point::new(chevron_x + 2.0, chevron_cy),
                1,
                colors::MUTED_FOREGROUND,
            );
            canvas.draw_line(
                Point::new(chevron_x + 2.0, chevron_cy),
                Point::new(chevron_x - 2.0, chevron_cy + 4.0),
                1,
                colors::MUTED_FOREGROUND,
            );
        }

        // Title text
        if let Some(ref tl) = self.title_layout {
            let th = tl.size().height;
            let tx = rect.x1 + 24.0;
            let ty = rect.y1 + (self.header_height - th) / 2.0;
            canvas.draw_text(tl, tx as i32, ty as i32);
        }

        // Bottom border on header
        canvas.fill_rect(
            Rect::new(rect.x1, header_rect.y2 - 1.0, rect.x2, header_rect.y2),
            self.border_color,
        );

        // Content
        if self.expanded {
            if let Some(ref child) = self.child {
                let content_y = rect.y1 + self.header_height + self.content_padding.top;
                let content_rect = Rect::new(
                    rect.x1 + self.content_padding.left,
                    content_y,
                    rect.x1 + self.content_padding.left + self.child_size.width,
                    content_y + self.child_size.height,
                );
                child.paint(canvas, content_rect);
            }
        }
    }

    fn children(&self) -> &[Box<dyn Widget>] {
        match &self.child {
            Some(c) => std::slice::from_ref(c),
            None => &[],
        }
    }

    fn event(&mut self, event: &WidgetEvent, rect: Rect) -> EventResponse {
        let header_rect = Rect::new(rect.x1, rect.y1, rect.x2, rect.y1 + self.header_height);

        match event {
            WidgetEvent::Mouse(MouseEvent::MouseClickEvent(e))
                if e.state == MouseState::Pressed && header_rect.contains(&e.position) =>
            {
                self.expanded = !self.expanded;
                if let Some(ref mut cb) = self.on_toggle {
                    cb(self.expanded);
                }
                EventResponse {
                    handled: true,
                    cursor: Some(CursorIcon::Pointer),
                    ..Default::default()
                }
            }
            WidgetEvent::Mouse(MouseEvent::MouseMoveEvent(pos)) if header_rect.contains(pos) => {
                EventResponse {
                    cursor: Some(CursorIcon::Pointer),
                    ..Default::default()
                }
            }
            _ => {
                if self.expanded {
                    if let Some(ref mut child) = self.child {
                        let content_y = rect.y1 + self.header_height + self.content_padding.top;
                        let content_rect = Rect::new(
                            rect.x1 + self.content_padding.left,
                            content_y,
                            rect.x1 + self.content_padding.left + self.child_size.width,
                            content_y + self.child_size.height,
                        );
                        return child.event(event, content_rect);
                    }
                }
                EventResponse::default()
            }
        }
    }
}
