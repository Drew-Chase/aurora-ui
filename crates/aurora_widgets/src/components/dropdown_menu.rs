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

/// A dropdown menu with text items triggered by clicking.
///
/// # Example
/// ```ignore
/// DropdownMenu::new()
///     .trigger(Label::new("Options"))
///     .item("Edit")
///     .item("Duplicate")
///     .separator()
///     .item("Delete")
///     .on_select(|idx| println!("selected: {idx}"))
/// ```
pub struct DropdownMenu {
    trigger: Option<Box<dyn Widget>>,
    items: Vec<MenuItemKind>,
    open: bool,
    item_height: f32,
    width: f32,
    background: Color,
    border_color: Color,
    hover_bg: Color,
    corners: Corners,
    padding: Edges,
    on_select: Option<Box<dyn FnMut(usize)>>,
    trigger_size: Size,
    item_layouts: Vec<Option<aurora_text::text_layout::TextLayout>>,
    hover_index: Option<usize>,
}

enum MenuItemKind {
    Item(String),
    Separator,
}

impl DropdownMenu {
    pub fn new() -> Self {
        Self {
            trigger: None,
            items: Vec::new(),
            open: false,
            item_height: 36.0,
            width: 200.0,
            background: colors::POPOVER,
            border_color: colors::BORDER,
            hover_bg: colors::ACCENT,
            corners: Corners::all(8.0),
            padding: Edges::new(4.0, 4.0, 4.0, 4.0),
            on_select: None,
            trigger_size: Size::default(),
            item_layouts: Vec::new(),
            hover_index: None,
        }
    }

    pub fn trigger(mut self, widget: impl Widget + 'static) -> Self {
        self.trigger = Some(Box::new(widget));
        self
    }

    pub fn item(mut self, label: impl Into<String>) -> Self {
        self.items.push(MenuItemKind::Item(label.into()));
        self
    }

    pub fn separator(mut self) -> Self {
        self.items.push(MenuItemKind::Separator);
        self
    }

    pub fn item_height(mut self, height: f32) -> Self {
        self.item_height = height;
        self
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    pub fn background(mut self, color: Color) -> Self {
        self.background = color;
        self
    }

    pub fn on_select(mut self, cb: impl FnMut(usize) + 'static) -> Self {
        self.on_select = Some(Box::new(cb));
        self
    }

    fn menu_rect(&self, trigger_rect: &Rect) -> Rect {
        let mut total_h = self.padding.top + self.padding.bottom;
        for item in &self.items {
            match item {
                MenuItemKind::Item(_) => total_h += self.item_height,
                MenuItemKind::Separator => total_h += 9.0,
            }
        }
        Rect::new(
            trigger_rect.x1,
            trigger_rect.y2 + 4.0,
            trigger_rect.x1 + self.width,
            trigger_rect.y2 + 4.0 + total_h,
        )
    }

    fn item_index_at(&self, y: f32, menu_rect: &Rect) -> Option<usize> {
        let mut cy = menu_rect.y1 + self.padding.top;
        let mut logical_idx = 0;
        for item in &self.items {
            match item {
                MenuItemKind::Item(_) => {
                    if y >= cy && y < cy + self.item_height {
                        return Some(logical_idx);
                    }
                    cy += self.item_height;
                    logical_idx += 1;
                }
                MenuItemKind::Separator => {
                    cy += 9.0;
                }
            }
        }
        None
    }
}

impl Default for DropdownMenu {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for DropdownMenu {
    fn layout(&mut self, available: Size, ctx: &mut LayoutCtx) -> Size {
        if let Some(ref mut trigger) = self.trigger {
            self.trigger_size = trigger.layout(available, ctx);
        }

        // Layout item labels
        self.item_layouts.clear();
        let inner_w = self.width - self.padding.left - self.padding.right - 16.0;
        for item in &self.items {
            match item {
                MenuItemKind::Item(label) => {
                    let mut opts = ctx.font_options.clone();
                    opts.size = Some(14.0);
                    opts.weight = Some(aurora_text::font_options::FontWeight::Normal);
                    let mut tl = aurora_text::text_layout::TextLayout::new(ctx.font_manager, label, &opts, colors::FOREGROUND, None);
                    tl.set_max_width(ctx.font_manager, inner_w.max(0.0));
                    self.item_layouts.push(Some(tl));
                }
                MenuItemKind::Separator => {
                    self.item_layouts.push(None);
                }
            }
        }

        self.trigger_size
    }

    fn paint(&self, canvas: &mut Canvas, rect: Rect) {
        // Trigger
        if let Some(ref trigger) = self.trigger {
            trigger.paint(canvas, rect);
        }

        if !self.open {
            return;
        }

        // Menu panel
        let mr = self.menu_rect(&rect);
        canvas.fill_rounded_rect(mr, self.corners, self.background);
        canvas.stroke_rounded_rect(mr, self.corners, 1, self.border_color);

        let mut y = mr.y1 + self.padding.top;
        let mut layout_idx = 0;
        let mut logical_idx = 0;
        for item in &self.items {
            match item {
                MenuItemKind::Item(_) => {
                    let item_rect = Rect::new(
                        mr.x1 + self.padding.left,
                        y,
                        mr.x2 - self.padding.right,
                        y + self.item_height,
                    );

                    if self.hover_index == Some(logical_idx) {
                        let hover_corners = Corners::all(4.0);
                        canvas.fill_rounded_rect(item_rect, hover_corners, self.hover_bg);
                    }

                    if let Some(Some(ref tl)) = self.item_layouts.get(layout_idx) {
                        let th = tl.size().height;
                        let tx = item_rect.x1 + 8.0;
                        let ty = y + (self.item_height - th) / 2.0;
                        canvas.draw_text(tl, tx as i32, ty as i32);
                    }

                    y += self.item_height;
                    layout_idx += 1;
                    logical_idx += 1;
                }
                MenuItemKind::Separator => {
                    let sep_y = y + 4.0;
                    canvas.fill_rect(
                        Rect::new(mr.x1 + self.padding.left, sep_y, mr.x2 - self.padding.right, sep_y + 1.0),
                        colors::BORDER,
                    );
                    y += 9.0;
                    layout_idx += 1;
                }
            }
        }
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
                    self.hover_index = None;
                    return EventResponse {
                        handled: true,
                        cursor: Some(CursorIcon::Pointer),
                        ..Default::default()
                    };
                }
                if self.open {
                    let mr = self.menu_rect(&rect);
                    if mr.contains(&e.position) {
                        if let Some(idx) = self.item_index_at(e.position.y, &mr) {
                            self.open = false;
                            if let Some(ref mut cb) = self.on_select {
                                cb(idx);
                            }
                            return EventResponse {
                                handled: true,
                                ..Default::default()
                            };
                        }
                    }
                    // Click outside closes
                    self.open = false;
                    return EventResponse {
                        handled: true,
                        ..Default::default()
                    };
                }
                EventResponse::default()
            }
            WidgetEvent::Mouse(MouseEvent::MouseMoveEvent(pos)) => {
                if self.open {
                    let mr = self.menu_rect(&rect);
                    if mr.contains(pos) {
                        self.hover_index = self.item_index_at(pos.y, &mr);
                        return EventResponse {
                            cursor: Some(CursorIcon::Pointer),
                            ..Default::default()
                        };
                    } else {
                        self.hover_index = None;
                    }
                }
                if rect.contains(pos) {
                    return EventResponse {
                        cursor: Some(CursorIcon::Pointer),
                        ..Default::default()
                    };
                }
                EventResponse::default()
            }
            _ => EventResponse::default(),
        }
    }
}
