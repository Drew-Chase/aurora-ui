use crate::widgets::{EventResponse, EventStatus, LayoutCtx, Widget};
use aurora_core::color::Color;
use aurora_core::geometry::corners::Corners;
use aurora_core::geometry::rect::Rect;
use aurora_core::geometry::size::Size;
use aurora_core::kmi::WidgetEvent;
use aurora_core::kmi::cursor_icon::CursorIcon;
use aurora_core::kmi::mouse::{MouseEvent, MouseState};
use aurora_render::canvas::Canvas;

use super::colors;

/// A menu within the menubar.
struct MenuBarMenu {
    label: String,
    items: Vec<String>,
}

/// A horizontal menu bar with dropdown menus.
///
/// # Example
/// ```ignore
/// Menubar::new()
///     .menu("File", vec!["New", "Open", "Save", "Exit"])
///     .menu("Edit", vec!["Undo", "Redo", "Cut", "Copy", "Paste"])
///     .menu("Help", vec!["About"])
///     .on_select(|menu, item| println!("menu {menu} item {item}"))
/// ```
pub struct Menubar {
    menus: Vec<MenuBarMenu>,
    height: f32,
    item_height: f32,
    padding: f32,
    background: Color,
    border_color: Color,
    hover_bg: Color,
    active_menu: Option<usize>,
    hover_item: Option<usize>,
    on_select: Option<Box<dyn FnMut(usize, usize)>>,
    menu_layouts: Vec<Option<aurora_text::text_layout::TextLayout>>,
    menu_widths: Vec<f32>,
    item_layouts: Vec<Vec<Option<aurora_text::text_layout::TextLayout>>>,
    dropdown_width: f32,
}

impl Menubar {
    pub fn new() -> Self {
        Self {
            menus: Vec::new(),
            height: 36.0,
            item_height: 32.0,
            padding: 12.0,
            background: colors::background(),
            border_color: colors::border(),
            hover_bg: colors::accent(),
            active_menu: None,
            hover_item: None,
            on_select: None,
            menu_layouts: Vec::new(),
            menu_widths: Vec::new(),
            item_layouts: Vec::new(),
            dropdown_width: 200.0,
        }
    }

    pub fn menu(mut self, label: impl Into<String>, items: Vec<impl Into<String>>) -> Self {
        self.menus.push(MenuBarMenu {
            label: label.into(),
            items: items.into_iter().map(|i| i.into()).collect(),
        });
        self
    }

    pub fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }

    pub fn background(mut self, color: Color) -> Self {
        self.background = color;
        self
    }

    pub fn on_select(mut self, cb: impl FnMut(usize, usize) + 'static) -> Self {
        self.on_select = Some(Box::new(cb));
        self
    }

    fn menu_x_offset(&self, idx: usize) -> f32 {
        self.menu_widths[..idx].iter().sum()
    }

    fn dropdown_rect(&self, menu_idx: usize, bar_rect: &Rect) -> Rect {
        let menu = &self.menus[menu_idx];
        let x = bar_rect.x1 + self.menu_x_offset(menu_idx);
        let h = 8.0 + self.item_height * menu.items.len() as f32 + 8.0;
        Rect::new(x, bar_rect.y2, x + self.dropdown_width, bar_rect.y2 + h)
    }
}

impl Default for Menubar {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for Menubar {
    fn layout(&mut self, available: Size, ctx: &mut LayoutCtx) -> Size {
        self.menu_layouts.clear();
        self.menu_widths.clear();
        self.item_layouts.clear();

        for menu in &self.menus {
            let mut opts = ctx.font_options.clone();
            opts.size = Some(14.0);
            opts.weight = Some(aurora_text::font_options::FontWeight::Medium);
            let mut tl = aurora_text::text_layout::TextLayout::new(
                ctx.font_manager,
                &menu.label,
                &opts,
                colors::foreground(),
                None,
            );
            tl.set_max_width(ctx.font_manager, f32::MAX);
            let tw = tl.size().width;
            self.menu_widths.push(tw + self.padding * 2.0);
            self.menu_layouts.push(Some(tl));

            // Item layouts
            let mut layouts = Vec::new();
            for item in &menu.items {
                let mut opts = ctx.font_options.clone();
                opts.size = Some(14.0);
                opts.weight = Some(aurora_text::font_options::FontWeight::Normal);
                let mut tl = aurora_text::text_layout::TextLayout::new(
                    ctx.font_manager,
                    item,
                    &opts,
                    colors::foreground(),
                    None,
                );
                tl.set_max_width(ctx.font_manager, self.dropdown_width - 24.0);
                layouts.push(Some(tl));
            }
            self.item_layouts.push(layouts);
        }

        Size::new(available.width, self.height)
    }

    fn paint(&self, canvas: &mut Canvas, rect: Rect) {
        // Bar background
        canvas.fill_rect(rect, self.background);
        canvas.fill_rect(
            Rect::new(rect.x1, rect.y2 - 1.0, rect.x2, rect.y2),
            self.border_color,
        );

        // Menu labels
        let mut x = rect.x1;
        for (i, menu_w) in self.menu_widths.iter().enumerate() {
            let menu_rect = Rect::new(x, rect.y1, x + menu_w, rect.y2);
            let is_active = self.active_menu == Some(i);

            if is_active {
                canvas.fill_rect(menu_rect, self.hover_bg);
            }

            if let Some(Some(tl)) = self.menu_layouts.get(i) {
                let _s = tl.size();
                let tw = _s.width;
                let th = _s.height;
                let tx = menu_rect.x1 + (menu_rect.width() - tw) / 2.0;
                let ty = menu_rect.y1 + (menu_rect.height() - th) / 2.0;
                canvas.draw_text(tl, tx as i32, ty as i32);
            }

            x += menu_w;
        }
    }

    fn paint_overlay(&self, canvas: &mut Canvas, rect: Rect) {
        let Some(menu_idx) = self.active_menu else {
            return;
        };

        let dr = self.dropdown_rect(menu_idx, &rect);
        let drop_corners = Corners::all(6.0);
        canvas.fill_rounded_rect(dr, drop_corners, colors::popover());
        canvas.stroke_rounded_rect(dr, drop_corners, 1, self.border_color);

        let mut iy = dr.y1 + 4.0;
        if let Some(item_layouts) = self.item_layouts.get(menu_idx) {
            for (j, layout) in item_layouts.iter().enumerate() {
                let item_rect = Rect::new(dr.x1 + 4.0, iy, dr.x2 - 4.0, iy + self.item_height);

                if self.hover_item == Some(j) {
                    let item_corners = Corners::all(4.0);
                    canvas.fill_rounded_rect(item_rect, item_corners, self.hover_bg);
                }

                if let Some(tl) = layout {
                    let th = tl.size().height;
                    let tx = item_rect.x1 + 8.0;
                    let ty = iy + (self.item_height - th) / 2.0;
                    canvas.draw_text(tl, tx as i32, ty as i32);
                }

                iy += self.item_height;
            }
        }
    }

    fn children(&self) -> &[Box<dyn Widget>] {
        &[]
    }

    fn event(&mut self, event: &WidgetEvent, rect: Rect) -> EventResponse {
        match event {
            WidgetEvent::Mouse(MouseEvent::MouseClickEvent(e))
                if e.state == MouseState::Released =>
            {
                // Check bar items
                let mut x = rect.x1;
                for (i, menu_w) in self.menu_widths.iter().enumerate() {
                    let menu_rect = Rect::new(x, rect.y1, x + menu_w, rect.y2);
                    if menu_rect.contains(&e.position) {
                        if self.active_menu == Some(i) {
                            self.active_menu = None;
                        } else {
                            self.active_menu = Some(i);
                        }
                        self.hover_item = None;
                        return EventResponse {
                            status: EventStatus::Consumed,
                            cursor: Some(CursorIcon::Pointer),
                            ..Default::default()
                        };
                    }
                    x += menu_w;
                }
                EventResponse::default()
            }
            WidgetEvent::Mouse(MouseEvent::MouseMoveEvent(pos)) => {
                // Highlight bar items on hover when a menu is open
                if self.active_menu.is_some() {
                    let mut x = rect.x1;
                    for (i, menu_w) in self.menu_widths.iter().enumerate() {
                        let menu_rect = Rect::new(x, rect.y1, x + menu_w, rect.y2);
                        if menu_rect.contains(pos) {
                            self.active_menu = Some(i);
                            self.hover_item = None;
                            return EventResponse {
                                status: EventStatus::Consumed,
                                cursor: Some(CursorIcon::Pointer),
                                ..Default::default()
                            };
                        }
                        x += menu_w;
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

    fn event_overlay(&mut self, event: &WidgetEvent, rect: Rect) -> EventResponse {
        if self.active_menu.is_none() {
            return EventResponse::default();
        }

        let menu_idx = self.active_menu.unwrap();

        match event {
            WidgetEvent::Mouse(MouseEvent::MouseClickEvent(e)) => {
                if e.state == MouseState::Released {
                    let dr = self.dropdown_rect(menu_idx, &rect);
                    if dr.contains(&e.position) {
                        let relative_y = e.position.y - dr.y1 - 4.0;
                        let idx = (relative_y / self.item_height) as usize;
                        if idx < self.menus[menu_idx].items.len() {
                            self.active_menu = None;
                            if let Some(ref mut cb) = self.on_select {
                                cb(menu_idx, idx);
                            }
                            return EventResponse {
                                status: EventStatus::Consumed,
                                ..Default::default()
                            };
                        }
                    }
                    self.active_menu = None;
                }
                EventResponse {
                    status: EventStatus::Canceled,
                    ..Default::default()
                }
            }
            WidgetEvent::Mouse(MouseEvent::MouseMoveEvent(pos)) => {
                let dr = self.dropdown_rect(menu_idx, &rect);
                if dr.contains(pos) {
                    let relative_y = pos.y - dr.y1 - 4.0;
                    let idx = (relative_y / self.item_height) as usize;
                    self.hover_item = if idx < self.menus[menu_idx].items.len() {
                        Some(idx)
                    } else {
                        None
                    };
                    return EventResponse {
                        status: EventStatus::Consumed,
                        cursor: Some(CursorIcon::Pointer),
                        ..Default::default()
                    };
                }
                self.hover_item = None;
                EventResponse {
                    status: EventStatus::Canceled,
                    ..Default::default()
                }
            }
            _ => EventResponse {
                status: EventStatus::Canceled,
                ..Default::default()
            },
        }
    }
    #[cfg(feature = "a11y")]
    fn access_info(&self) -> aurora_a11y::NodeInfo {
        aurora_a11y::NodeInfo::new(aurora_a11y::accesskit::Role::MenuBar)
    }
}
