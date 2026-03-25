use crate::widgets::{EventResponse, LayoutCtx, Widget};
use aurora_core::geometry::rect::Rect;
use aurora_core::geometry::size::Size;
use aurora_core::kmi::cursor_icon::CursorIcon;
use aurora_core::kmi::mouse::{MouseEvent, MouseState};
use aurora_core::kmi::WidgetEvent;
use aurora_render::canvas::Canvas;

use super::colors;

/// A navigation breadcrumb trail.
///
/// # Example
/// ```ignore
/// Breadcrumb::new()
///     .item("Home")
///     .item("Products")
///     .item("Widget")
///     .on_click(|idx| println!("breadcrumb {idx} clicked"))
/// ```
pub struct Breadcrumb {
    items: Vec<String>,
    separator: String,
    height: f32,
    on_click: Option<Box<dyn FnMut(usize)>>,
    item_layouts: Vec<Option<aurora_text::text_layout::TextLayout>>,
    separator_layouts: Vec<Option<aurora_text::text_layout::TextLayout>>,
    item_widths: Vec<f32>,
    separator_width: f32,
    spacing: f32,
}

impl Breadcrumb {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            separator: "/".to_string(),
            height: 24.0,
            on_click: None,
            item_layouts: Vec::new(),
            separator_layouts: Vec::new(),
            item_widths: Vec::new(),
            separator_width: 0.0,
            spacing: 8.0,
        }
    }

    pub fn item(mut self, label: impl Into<String>) -> Self {
        self.items.push(label.into());
        self
    }

    pub fn separator(mut self, sep: impl Into<String>) -> Self {
        self.separator = sep.into();
        self
    }

    pub fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }

    pub fn on_click(mut self, cb: impl FnMut(usize) + 'static) -> Self {
        self.on_click = Some(Box::new(cb));
        self
    }
}

impl Default for Breadcrumb {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for Breadcrumb {
    fn layout(&mut self, _available: Size, ctx: &mut LayoutCtx) -> Size {
        self.item_layouts.clear();
        self.separator_layouts.clear();
        self.item_widths.clear();
        let mut total_w = 0.0;

        let last_idx = self.items.len().saturating_sub(1);

        for (i, label) in self.items.iter().enumerate() {
            let is_last = i == last_idx;
            let mut opts = ctx.font_options.clone();
            opts.size = Some(14.0);
            opts.weight = Some(if is_last {
                aurora_text::font_options::FontWeight::Medium
            } else {
                aurora_text::font_options::FontWeight::Normal
            });
            let fg = if is_last { colors::FOREGROUND } else { colors::MUTED_FOREGROUND };
            let mut tl = aurora_text::text_layout::TextLayout::new(ctx.font_manager, label, &opts, fg, None);
            tl.set_max_width(ctx.font_manager, f32::MAX);
            let tw = tl.size().width;
            self.item_widths.push(tw);
            total_w += tw;
            self.item_layouts.push(Some(tl));

            // Separator
            if !is_last {
                let mut sep_opts = ctx.font_options.clone();
                sep_opts.size = Some(14.0);
                sep_opts.weight = Some(aurora_text::font_options::FontWeight::Normal);
                let mut sep_tl = aurora_text::text_layout::TextLayout::new(ctx.font_manager, &self.separator, &sep_opts, colors::MUTED_FOREGROUND, None);
                sep_tl.set_max_width(ctx.font_manager, f32::MAX);
                let sw = sep_tl.size().width;
                self.separator_width = sw;
                total_w += self.spacing * 2.0 + sw;
                self.separator_layouts.push(Some(sep_tl));
            }
        }

        Size::new(total_w, self.height)
    }

    fn paint(&self, canvas: &mut Canvas, rect: Rect) {
        let mut x = rect.x1;
        let last_idx = self.items.len().saturating_sub(1);

        for (i, item_w) in self.item_widths.iter().enumerate() {
            if let Some(Some(tl)) = self.item_layouts.get(i) {
                let th = tl.size().height;
                let ty = rect.y1 + (rect.height() - th) / 2.0;
                canvas.draw_text(tl, x as i32, ty as i32);
            }
            x += item_w;

            if i < last_idx {
                x += self.spacing;
                if let Some(Some(sep_tl)) = self.separator_layouts.get(i) {
                    let sh = sep_tl.size().height;
                    let sy = rect.y1 + (rect.height() - sh) / 2.0;
                    canvas.draw_text(sep_tl, x as i32, sy as i32);
                    x += self.separator_width;
                }
                x += self.spacing;
            }
        }
    }

    fn children(&self) -> &[Box<dyn Widget>] {
        &[]
    }

    fn event(&mut self, event: &WidgetEvent, rect: Rect) -> EventResponse {
        let last_idx = self.items.len().saturating_sub(1);

        match event {
            WidgetEvent::Mouse(MouseEvent::MouseClickEvent(e))
                if e.state == MouseState::Pressed && rect.contains(&e.position) =>
            {
                let mut x = rect.x1;
                for (i, item_w) in self.item_widths.iter().enumerate() {
                    let item_rect = Rect::new(x, rect.y1, x + item_w, rect.y2);
                    if item_rect.contains(&e.position) && i < last_idx {
                        if let Some(ref mut cb) = self.on_click {
                            cb(i);
                        }
                        return EventResponse {
                            handled: true,
                            cursor: Some(CursorIcon::Pointer),
                            ..Default::default()
                        };
                    }
                    x += item_w;
                    if i < last_idx {
                        x += self.spacing + self.separator_width + self.spacing;
                    }
                }
                EventResponse::default()
            }
            WidgetEvent::Mouse(MouseEvent::MouseMoveEvent(pos)) if rect.contains(pos) => {
                let mut x = rect.x1;
                for (i, item_w) in self.item_widths.iter().enumerate() {
                    let item_rect = Rect::new(x, rect.y1, x + item_w, rect.y2);
                    if item_rect.contains(pos) && i < last_idx {
                        return EventResponse {
                            cursor: Some(CursorIcon::Pointer),
                            ..Default::default()
                        };
                    }
                    x += item_w;
                    if i < last_idx {
                        x += self.spacing + self.separator_width + self.spacing;
                    }
                }
                EventResponse::default()
            }
            _ => EventResponse::default(),
        }
    }
}
