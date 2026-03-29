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

/// A vertical navigation menu with selectable items.
///
/// # Example
/// ```ignore
/// NavigationMenu::new()
///     .item("Dashboard")
///     .item("Settings")
///     .item("Profile")
///     .item("Logout")
///     .selected(0)
///     .on_select(|idx| println!("nav: {idx}"))
/// ```
pub struct NavigationMenu {
    items: Vec<String>,
    selected: usize,
    item_height: f32,
    spacing: f32,
    padding_x: f32,
    indicator_width: f32,
    indicator_color: Color,
    hover_bg: Color,
    corners: Corners,
    width: Option<f32>,
    on_select: Option<Box<dyn FnMut(usize)>>,
    item_layouts: Vec<Option<aurora_text::text_layout::TextLayout>>,
    hover_index: Option<usize>,
}

impl NavigationMenu {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            selected: 0,
            item_height: 40.0,
            spacing: 2.0,
            padding_x: 12.0,
            indicator_width: 3.0,
            indicator_color: colors::primary(),
            hover_bg: colors::accent(),
            corners: Corners::all(6.0),
            width: None,
            on_select: None,
            item_layouts: Vec::new(),
            hover_index: None,
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

    pub fn item_height(mut self, height: f32) -> Self {
        self.item_height = height;
        self
    }

    pub fn width(mut self, width: impl Into<f32>) -> Self {
        self.width = Some(width.into());
        self
    }

    pub fn on_select(mut self, cb: impl FnMut(usize) + 'static) -> Self {
        self.on_select = Some(Box::new(cb));
        self
    }
}

impl Default for NavigationMenu {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for NavigationMenu {
    fn layout(&mut self, available: Size, ctx: &mut LayoutCtx) -> Size {
        let w = self.width.unwrap_or(available.width);
        self.item_layouts.clear();

        for (i, label) in self.items.iter().enumerate() {
            let is_selected = i == self.selected;
            let mut opts = ctx.font_options.clone();
            opts.size = Some(14.0);
            opts.weight = Some(if is_selected {
                aurora_text::font_options::FontWeight::Medium
            } else {
                aurora_text::font_options::FontWeight::Normal
            });
            let fg = if is_selected {
                colors::foreground()
            } else {
                colors::muted_foreground()
            };
            let mut tl =
                aurora_text::text_layout::TextLayout::new(ctx.font_manager, label, &opts, fg, None);
            tl.set_max_width(
                ctx.font_manager,
                w - self.padding_x * 2.0 - self.indicator_width - 8.0,
            );
            self.item_layouts.push(Some(tl));
        }

        let total_h = self.item_height * self.items.len() as f32
            + self.spacing * self.items.len().saturating_sub(1) as f32;
        Size::new(w, total_h)
    }

    fn paint(&self, canvas: &mut Canvas, rect: Rect) {
        let mut y = rect.y1;

        for (i, _) in self.items.iter().enumerate() {
            let item_rect = Rect::new(rect.x1, y, rect.x2, y + self.item_height);
            let is_selected = i == self.selected;
            let is_hovered = self.hover_index == Some(i) && !is_selected;

            if is_hovered {
                canvas.fill_rounded_rect(item_rect, self.corners, self.hover_bg);
            }

            // Active indicator
            if is_selected {
                canvas.fill_rounded_rect(item_rect, self.corners, self.hover_bg);
                let indicator_rect = Rect::new(
                    rect.x1,
                    y + 4.0,
                    rect.x1 + self.indicator_width,
                    y + self.item_height - 4.0,
                );
                let ind_corners = Corners::all(self.indicator_width / 2.0);
                canvas.fill_rounded_rect(indicator_rect, ind_corners, self.indicator_color);
            }

            // Label
            if let Some(Some(tl)) = self.item_layouts.get(i) {
                let th = tl.size().height;
                let tx = rect.x1 + self.indicator_width + 8.0 + self.padding_x;
                let ty = y + (self.item_height - th) / 2.0;
                canvas.draw_text(tl, tx as i32, ty as i32);
            }

            y += self.item_height + self.spacing;
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
                for i in 0..self.items.len() {
                    let item_rect = Rect::new(rect.x1, y, rect.x2, y + self.item_height);
                    if item_rect.contains(&e.position) {
                        self.selected = i;
                        if let Some(ref mut cb) = self.on_select {
                            cb(i);
                        }
                        return EventResponse {
                            handled: true,
                            cursor: Some(CursorIcon::Pointer),
                            ..Default::default()
                        };
                    }
                    y += self.item_height + self.spacing;
                }
                EventResponse::default()
            }
            WidgetEvent::Mouse(MouseEvent::MouseMoveEvent(pos)) if rect.contains(pos) => {
                let mut y = rect.y1;
                self.hover_index = None;
                for i in 0..self.items.len() {
                    let item_rect = Rect::new(rect.x1, y, rect.x2, y + self.item_height);
                    if item_rect.contains(pos) {
                        self.hover_index = Some(i);
                        return EventResponse {
                            cursor: Some(CursorIcon::Pointer),
                            ..Default::default()
                        };
                    }
                    y += self.item_height + self.spacing;
                }
                EventResponse::default()
            }
            WidgetEvent::Mouse(MouseEvent::MouseMoveEvent(_)) => {
                self.hover_index = None;
                EventResponse::default()
            }
            _ => EventResponse::default(),
        }
    }
    #[cfg(feature = "a11y")]
    fn access_info(&self) -> aurora_a11y::NodeInfo {
        aurora_a11y::NodeInfo::new(aurora_a11y::accesskit::Role::Navigation)
    }
}
