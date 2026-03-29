use crate::widgets::{EventResponse, LayoutCtx, Widget};
use aurora_core::geometry::corners::Corners;
use aurora_core::geometry::rect::Rect;
use aurora_core::geometry::size::Size;
use aurora_core::kmi::WidgetEvent;
use aurora_core::kmi::cursor_icon::CursorIcon;
use aurora_core::kmi::mouse::{MouseEvent, MouseState};
use aurora_render::canvas::Canvas;
use aurora_text::font_options::FontWeight;
use aurora_text::text_layout::TextLayout;

use super::colors;

/// A group of toggle buttons where one (or multiple) can be selected.
///
/// # Example
/// ```ignore
/// ToggleGroup::new()
///     .item("Left")
///     .item("Center")
///     .item("Right")
///     .selected(1)
///     .on_change(|idx| println!("selected: {idx}"))
/// ```
pub struct ToggleGroup {
    items: Vec<String>,
    selected: Option<usize>,
    height: f32,
    on_change: Option<Box<dyn FnMut(usize)>>,
    item_layouts: Vec<Option<TextLayout>>,
    item_widths: Vec<f32>,
}

impl ToggleGroup {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            selected: None,
            height: 40.0,
            on_change: None,
            item_layouts: Vec::new(),
            item_widths: Vec::new(),
        }
    }

    pub fn item(mut self, label: impl Into<String>) -> Self {
        self.items.push(label.into());
        self
    }

    pub fn selected(mut self, index: usize) -> Self {
        self.selected = Some(index);
        self
    }

    pub fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }

    pub fn on_change(mut self, cb: impl FnMut(usize) + 'static) -> Self {
        self.on_change = Some(Box::new(cb));
        self
    }
}

impl Default for ToggleGroup {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for ToggleGroup {
    fn layout(&mut self, _available: Size, ctx: &mut LayoutCtx) -> Size {
        self.item_layouts.clear();
        self.item_widths.clear();
        let mut total_w = 0.0;
        let pad = 16.0;

        for label in &self.items {
            let mut opts = ctx.font_options.clone();
            opts.size = Some(14.0);
            opts.weight = Some(FontWeight::Medium);
            let mut tl =
                TextLayout::new(ctx.font_manager, label, &opts, colors::foreground(), None);
            tl.set_max_width(ctx.font_manager, f32::MAX);
            let ts = tl.size();
            let item_w = ts.width + pad * 2.0;
            self.item_widths.push(item_w);
            total_w += item_w;
            self.item_layouts.push(Some(tl));
        }

        Size::new(total_w, self.height)
    }

    fn paint(&self, canvas: &mut Canvas, rect: Rect) {
        let outer_corners = Corners::all(6.0);
        canvas.stroke_rounded_rect(rect, outer_corners, 1, colors::border());

        let mut x = rect.x1;
        for (i, item_w) in self.item_widths.iter().enumerate() {
            let item_rect = Rect::new(x, rect.y1, x + item_w, rect.y2);
            let is_selected = self.selected == Some(i);

            if is_selected {
                canvas.fill_rounded_rect(item_rect, outer_corners, colors::accent());
            }

            if let Some(Some(tl)) = self.item_layouts.get(i) {
                let ts = tl.size();
                let tx = item_rect.x1 + (item_rect.width() - ts.width) / 2.0;
                let ty = item_rect.y1 + (item_rect.height() - ts.height) / 2.0;
                canvas.draw_text(tl, tx as i32, ty as i32);
            }

            // Vertical separator (except last)
            if i < self.items.len() - 1 && !is_selected {
                let sep_x = x + item_w;
                canvas.fill_rect(
                    Rect::new(sep_x - 0.5, rect.y1 + 4.0, sep_x + 0.5, rect.y2 - 4.0),
                    colors::border(),
                );
            }

            x += item_w;
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
                for (i, item_w) in self.item_widths.iter().enumerate() {
                    let item_rect = Rect::new(x, rect.y1, x + item_w, rect.y2);
                    if item_rect.contains(&e.position) {
                        self.selected = Some(i);
                        if let Some(ref mut cb) = self.on_change {
                            cb(i);
                        }
                        return EventResponse {
                            handled: true,
                            cursor: Some(CursorIcon::Pointer),
                            ..Default::default()
                        };
                    }
                    x += item_w;
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
    #[cfg(feature = "a11y")]
    fn access_info(&self) -> aurora_a11y::NodeInfo {
        aurora_a11y::NodeInfo::new(aurora_a11y::accesskit::Role::Group)
    }
}
