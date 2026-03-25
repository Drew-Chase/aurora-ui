use crate::widgets::{EventResponse, LayoutCtx, Widget};
use aurora_core::geometry::corners::Corners;
use aurora_core::geometry::rect::Rect;
use aurora_core::geometry::size::Size;
use aurora_core::kmi::cursor_icon::CursorIcon;
use aurora_core::kmi::mouse::{MouseEvent, MouseState};
use aurora_core::kmi::WidgetEvent;
use aurora_render::canvas::Canvas;

use super::colors;

/// Page navigation with prev/next and numbered page buttons.
///
/// # Example
/// ```ignore
/// Pagination::new()
///     .total_pages(10)
///     .current_page(1)
///     .on_change(|page| println!("page: {page}"))
/// ```
pub struct Pagination {
    current_page: usize,
    total_pages: usize,
    visible_pages: usize,
    button_size: f32,
    spacing: f32,
    corners: Corners,
    on_change: Option<Box<dyn FnMut(usize)>>,
    page_layouts: Vec<Option<aurora_text::text_layout::TextLayout>>,
    prev_layout: Option<aurora_text::text_layout::TextLayout>,
    next_layout: Option<aurora_text::text_layout::TextLayout>,
    displayed_pages: Vec<usize>,
}

impl Pagination {
    pub fn new() -> Self {
        Self {
            current_page: 1,
            total_pages: 1,
            visible_pages: 5,
            button_size: 36.0,
            spacing: 4.0,
            corners: Corners::all(6.0),
            on_change: None,
            page_layouts: Vec::new(),
            prev_layout: None,
            next_layout: None,
            displayed_pages: Vec::new(),
        }
    }

    pub fn current_page(mut self, page: usize) -> Self {
        self.current_page = page.max(1);
        self
    }

    pub fn total_pages(mut self, total: usize) -> Self {
        self.total_pages = total.max(1);
        self
    }

    pub fn visible_pages(mut self, count: usize) -> Self {
        self.visible_pages = count.max(1);
        self
    }

    pub fn button_size(mut self, size: f32) -> Self {
        self.button_size = size;
        self
    }

    pub fn corners(mut self, corners: Corners) -> Self {
        self.corners = corners;
        self
    }

    pub fn on_change(mut self, cb: impl FnMut(usize) + 'static) -> Self {
        self.on_change = Some(Box::new(cb));
        self
    }

    fn compute_displayed_pages(&self) -> Vec<usize> {
        let half = self.visible_pages / 2;
        let start = if self.current_page <= half + 1 {
            1
        } else if self.current_page + half > self.total_pages {
            self.total_pages.saturating_sub(self.visible_pages - 1)
        } else {
            self.current_page - half
        };
        let end = (start + self.visible_pages - 1).min(self.total_pages);
        (start..=end).collect()
    }
}

impl Default for Pagination {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for Pagination {
    fn layout(&mut self, _available: Size, ctx: &mut LayoutCtx) -> Size {
        self.displayed_pages = self.compute_displayed_pages();
        self.page_layouts.clear();

        // Prev button
        let mut opts = ctx.font_options.clone();
        opts.size = Some(14.0);
        opts.weight = Some(aurora_text::font_options::FontWeight::Normal);
        let prev_tl = aurora_text::text_layout::TextLayout::new(ctx.font_manager, "<", &opts, colors::foreground(), None);
        self.prev_layout = Some(prev_tl);

        // Page number buttons
        for &page in &self.displayed_pages {
            let mut opts = ctx.font_options.clone();
            opts.size = Some(14.0);
            opts.weight = Some(if page == self.current_page {
                aurora_text::font_options::FontWeight::Medium
            } else {
                aurora_text::font_options::FontWeight::Normal
            });
            let fg = if page == self.current_page { colors::primary_foreground() } else { colors::foreground() };
            let text = page.to_string();
            let tl = aurora_text::text_layout::TextLayout::new(ctx.font_manager, &text, &opts, fg, None);
            self.page_layouts.push(Some(tl));
        }

        // Next button
        let next_tl = aurora_text::text_layout::TextLayout::new(ctx.font_manager, ">", &opts, colors::foreground(), None);
        self.next_layout = Some(next_tl);

        // Total width: prev + pages + next + spacing
        let button_count = 2 + self.displayed_pages.len();
        let total_w = self.button_size * button_count as f32 + self.spacing * (button_count - 1) as f32;

        Size::new(total_w, self.button_size)
    }

    fn paint(&self, canvas: &mut Canvas, rect: Rect) {
        let mut x = rect.x1;

        // Prev button
        let prev_rect = Rect::new(x, rect.y1, x + self.button_size, rect.y2);
        canvas.stroke_rounded_rect(prev_rect, self.corners, 1, colors::border());
        if let Some(ref tl) = self.prev_layout {
            let _s = tl.size(); let tw = _s.width; let th = _s.height;
            let tx = prev_rect.x1 + (self.button_size - tw) / 2.0;
            let ty = prev_rect.y1 + (self.button_size - th) / 2.0;
            canvas.draw_text(tl, tx as i32, ty as i32);
        }
        x += self.button_size + self.spacing;

        // Page buttons
        for (i, &page) in self.displayed_pages.iter().enumerate() {
            let btn_rect = Rect::new(x, rect.y1, x + self.button_size, rect.y2);
            let is_current = page == self.current_page;

            if is_current {
                canvas.fill_rounded_rect(btn_rect, self.corners, colors::primary());
            } else {
                canvas.stroke_rounded_rect(btn_rect, self.corners, 1, colors::border());
            }

            if let Some(Some(tl)) = self.page_layouts.get(i) {
                let _s = tl.size(); let tw = _s.width; let th = _s.height;
                let tx = btn_rect.x1 + (self.button_size - tw) / 2.0;
                let ty = btn_rect.y1 + (self.button_size - th) / 2.0;
                canvas.draw_text(tl, tx as i32, ty as i32);
            }

            x += self.button_size + self.spacing;
        }

        // Next button
        let next_rect = Rect::new(x, rect.y1, x + self.button_size, rect.y2);
        canvas.stroke_rounded_rect(next_rect, self.corners, 1, colors::border());
        if let Some(ref tl) = self.next_layout {
            let _s = tl.size(); let tw = _s.width; let th = _s.height;
            let tx = next_rect.x1 + (self.button_size - tw) / 2.0;
            let ty = next_rect.y1 + (self.button_size - th) / 2.0;
            canvas.draw_text(tl, tx as i32, ty as i32);
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
                let mut x = rect.x1;

                // Prev button
                let prev_rect = Rect::new(x, rect.y1, x + self.button_size, rect.y2);
                if prev_rect.contains(&e.position) && self.current_page > 1 {
                    self.current_page -= 1;
                    if let Some(ref mut cb) = self.on_change {
                        cb(self.current_page);
                    }
                    return EventResponse {
                        handled: true,
                        cursor: Some(CursorIcon::Pointer),
                        ..Default::default()
                    };
                }
                x += self.button_size + self.spacing;

                // Page buttons
                for &page in &self.displayed_pages {
                    let btn_rect = Rect::new(x, rect.y1, x + self.button_size, rect.y2);
                    if btn_rect.contains(&e.position) {
                        self.current_page = page;
                        if let Some(ref mut cb) = self.on_change {
                            cb(self.current_page);
                        }
                        return EventResponse {
                            handled: true,
                            cursor: Some(CursorIcon::Pointer),
                            ..Default::default()
                        };
                    }
                    x += self.button_size + self.spacing;
                }

                // Next button
                let next_rect = Rect::new(x, rect.y1, x + self.button_size, rect.y2);
                if next_rect.contains(&e.position) && self.current_page < self.total_pages {
                    self.current_page += 1;
                    if let Some(ref mut cb) = self.on_change {
                        cb(self.current_page);
                    }
                    return EventResponse {
                        handled: true,
                        cursor: Some(CursorIcon::Pointer),
                        ..Default::default()
                    };
                }

                EventResponse::default()
            }
            WidgetEvent::Mouse(MouseEvent::MouseMoveEvent(pos)) if rect.contains(pos) => {
                EventResponse {
                    cursor: Some(CursorIcon::Pointer),
                    ..Default::default()
                }
            }
            _ => EventResponse::default(),
        }
    }
}
