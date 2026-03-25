use crate::widgets::{EventResponse, LayoutCtx, Widget};
use aurora_core::geometry::corners::Corners;
use aurora_core::geometry::rect::Rect;
use aurora_core::geometry::size::Size;
use aurora_core::kmi::cursor_icon::CursorIcon;
use aurora_core::kmi::mouse::{MouseEvent, MouseState};
use aurora_core::kmi::WidgetEvent;
use aurora_render::canvas::Canvas;
use aurora_text::font_options::FontWeight;
use aurora_text::text_layout::TextLayout;

use super::colors;

/// A group of radio buttons where exactly one can be selected.
///
/// # Example
/// ```ignore
/// RadioGroup::new()
///     .item("Option A")
///     .item("Option B")
///     .item("Option C")
///     .selected(0)
///     .on_change(|idx| println!("selected: {idx}"))
/// ```
pub struct RadioGroup {
    items: Vec<String>,
    selected: usize,
    radio_size: f32,
    spacing: f32,
    on_change: Option<Box<dyn FnMut(usize)>>,
    item_layouts: Vec<Option<TextLayout>>,
    item_heights: Vec<f32>,
}

impl RadioGroup {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            selected: 0,
            radio_size: 18.0,
            spacing: 8.0,
            on_change: None,
            item_layouts: Vec::new(),
            item_heights: Vec::new(),
        }
    }

    pub fn item(mut self, label: impl Into<String>) -> Self {
        self.items.push(label.into());
        self
    }

    pub fn selected(mut self, index: usize) -> Self {
        self.selected = index;
        self
    }

    pub fn spacing(mut self, spacing: f32) -> Self {
        self.spacing = spacing;
        self
    }

    pub fn on_change(mut self, cb: impl FnMut(usize) + 'static) -> Self {
        self.on_change = Some(Box::new(cb));
        self
    }
}

impl Default for RadioGroup {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for RadioGroup {
    fn layout(&mut self, available: Size, ctx: &mut LayoutCtx) -> Size {
        self.item_layouts.clear();
        self.item_heights.clear();
        let mut total_h = 0.0;

        for (i, label) in self.items.iter().enumerate() {
            let mut opts = ctx.font_options.clone();
            opts.size = Some(14.0);
            opts.weight = Some(FontWeight::Normal);
            let mut tl = TextLayout::new(ctx.font_manager, label, &opts, colors::FOREGROUND, None);
            tl.set_max_width(ctx.font_manager, available.width - self.radio_size - 8.0);
            let ts = tl.size();
            let item_h = ts.height.max(self.radio_size);
            self.item_heights.push(item_h);
            self.item_layouts.push(Some(tl));
            total_h += item_h;
            if i > 0 {
                total_h += self.spacing;
            }
        }

        Size::new(available.width, total_h)
    }

    fn paint(&self, canvas: &mut Canvas, rect: Rect) {
        let mut y = rect.y1;
        for (i, item_h) in self.item_heights.iter().enumerate() {
            let is_selected = self.selected == i;

            // Radio circle
            let circle_y = y + (item_h - self.radio_size) / 2.0;
            let circle_rect = Rect::new(
                rect.x1,
                circle_y,
                rect.x1 + self.radio_size,
                circle_y + self.radio_size,
            );
            let circle_corners = Corners::all(self.radio_size / 2.0);

            if is_selected {
                canvas.fill_rounded_rect(circle_rect, circle_corners, colors::PRIMARY);
                // Inner dot
                let dot_size = self.radio_size * 0.4;
                let dot_rect = Rect::new(
                    circle_rect.x1 + (self.radio_size - dot_size) / 2.0,
                    circle_rect.y1 + (self.radio_size - dot_size) / 2.0,
                    circle_rect.x1 + (self.radio_size + dot_size) / 2.0,
                    circle_rect.y1 + (self.radio_size + dot_size) / 2.0,
                );
                let dot_corners = Corners::all(dot_size / 2.0);
                canvas.fill_rounded_rect(dot_rect, dot_corners, colors::PRIMARY_FOREGROUND);
            } else {
                canvas.stroke_rounded_rect(circle_rect, circle_corners, 2, colors::INPUT_BORDER);
            }

            // Label
            if let Some(Some(tl)) = self.item_layouts.get(i) {
                let ts = tl.size();
                let label_x = rect.x1 + self.radio_size + 8.0;
                let label_y = y + (item_h - ts.height) / 2.0;
                canvas.draw_text(tl, label_x as i32, label_y as i32);
            }

            y += item_h + self.spacing;
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
                for (i, item_h) in self.item_heights.iter().enumerate() {
                    let item_rect = Rect::new(rect.x1, y, rect.x2, y + item_h);
                    if item_rect.contains(&e.position) {
                        self.selected = i;
                        if let Some(ref mut cb) = self.on_change {
                            cb(i);
                        }
                        return EventResponse {
                            handled: true,
                            cursor: Some(CursorIcon::Pointer),
                            ..Default::default()
                        };
                    }
                    y += item_h + self.spacing;
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
