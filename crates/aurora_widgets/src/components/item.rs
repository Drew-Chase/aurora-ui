use crate::widgets::{EventResponse, LayoutCtx, Widget};
use aurora_core::color::Color;
use aurora_core::geometry::corners::Corners;
use aurora_core::geometry::rect::Rect;
use aurora_core::geometry::size::Size;
use aurora_core::kmi::cursor_icon::CursorIcon;
use aurora_core::kmi::mouse::{MouseEvent, MouseState};
use aurora_core::kmi::WidgetEvent;
use aurora_render::canvas::Canvas;

use super::colors;

/// A generic list item with an optional icon, label, and description.
///
/// # Example
/// ```ignore
/// Item::new("Settings")
///     .description("Manage your preferences")
///     .on_click(|| println!("clicked"))
/// ```
pub struct Item {
    label: String,
    description: Option<String>,
    icon: Option<Box<dyn Widget>>,
    trailing: Option<Box<dyn Widget>>,
    height: Option<f32>,
    padding_x: f32,
    padding_y: f32,
    spacing: f32,
    hover_bg: Color,
    corners: Corners,
    hovered: bool,
    on_click: Option<Box<dyn FnMut()>>,
    label_layout: Option<aurora_text::text_layout::TextLayout>,
    desc_layout: Option<aurora_text::text_layout::TextLayout>,
    icon_size: Size,
    trailing_size: Size,
    label_height: f32,
    desc_height: f32,
}

impl Item {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            description: None,
            icon: None,
            trailing: None,
            height: None,
            padding_x: 12.0,
            padding_y: 8.0,
            spacing: 12.0,
            hover_bg: colors::accent(),
            corners: Corners::all(6.0),
            hovered: false,
            on_click: None,
            label_layout: None,
            desc_layout: None,
            icon_size: Size::default(),
            trailing_size: Size::default(),
            label_height: 0.0,
            desc_height: 0.0,
        }
    }

    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    pub fn icon(mut self, widget: impl Widget + 'static) -> Self {
        self.icon = Some(Box::new(widget));
        self
    }

    pub fn trailing(mut self, widget: impl Widget + 'static) -> Self {
        self.trailing = Some(Box::new(widget));
        self
    }

    pub fn height(mut self, height: impl Into<f32>) -> Self {
        self.height = Some(height.into());
        self
    }

    pub fn hover_bg(mut self, color: Color) -> Self {
        self.hover_bg = color;
        self
    }

    pub fn corners(mut self, corners: Corners) -> Self {
        self.corners = corners;
        self
    }

    pub fn on_click(mut self, cb: impl FnMut() + 'static) -> Self {
        self.on_click = Some(Box::new(cb));
        self
    }
}

impl Widget for Item {
    fn layout(&mut self, available: Size, ctx: &mut LayoutCtx) -> Size {
        let w = available.width;
        let mut used_w = self.padding_x * 2.0;
        let content_h_space = Size::new(w, f32::MAX);

        // Icon
        if let Some(ref mut icon) = self.icon {
            self.icon_size = icon.layout(Size::new(24.0, 24.0), ctx);
            used_w += self.icon_size.width + self.spacing;
        }

        // Trailing
        if let Some(ref mut trailing) = self.trailing {
            self.trailing_size = trailing.layout(content_h_space, ctx);
            used_w += self.trailing_size.width + self.spacing;
        }

        let text_w = (w - used_w).max(0.0);

        // Label
        let mut opts = ctx.font_options.clone();
        opts.size = Some(14.0);
        opts.weight = Some(aurora_text::font_options::FontWeight::Medium);
        let mut tl = aurora_text::text_layout::TextLayout::new(ctx.font_manager, &self.label, &opts, colors::foreground(), None);
        tl.set_max_width(ctx.font_manager, text_w);
        let lh = tl.size().height;
        self.label_height = lh;
        self.label_layout = Some(tl);

        // Description
        let mut text_total_h = lh;
        if let Some(ref desc) = self.description {
            let mut opts = ctx.font_options.clone();
            opts.size = Some(12.0);
            opts.weight = Some(aurora_text::font_options::FontWeight::Normal);
            let mut tl = aurora_text::text_layout::TextLayout::new(ctx.font_manager, desc, &opts, colors::foreground(), None);
            tl.set_max_width(ctx.font_manager, text_w);
            let dh = tl.size().height;
            self.desc_height = dh;
            text_total_h += 2.0 + dh;
            self.desc_layout = Some(tl);
        }

        let h = self.height.unwrap_or(text_total_h + self.padding_y * 2.0);
        Size::new(w, h)
    }

    fn paint(&self, canvas: &mut Canvas, rect: Rect) {
        // Hover background
        if self.hovered && self.hover_bg.alpha > 0 {
            canvas.fill_rounded_rect(rect, self.corners, self.hover_bg);
        }

        let mut x = rect.x1 + self.padding_x;
        let cy = rect.y1 + rect.height() / 2.0;

        // Icon
        if let Some(ref icon) = self.icon {
            let iy = cy - self.icon_size.height / 2.0;
            let icon_rect = Rect::new(x, iy, x + self.icon_size.width, iy + self.icon_size.height);
            icon.paint(canvas, icon_rect);
            x += self.icon_size.width + self.spacing;
        }

        // Text block
        let text_total_h = self.label_height + if self.description.is_some() { 2.0 + self.desc_height } else { 0.0 };
        let text_y = cy - text_total_h / 2.0;

        if let Some(ref tl) = self.label_layout {
            canvas.draw_text(tl, x as i32, text_y as i32);
        }
        if let Some(ref tl) = self.desc_layout {
            canvas.draw_text(tl, x as i32, (text_y + self.label_height + 2.0) as i32);
        }

        // Trailing widget
        if let Some(ref trailing) = self.trailing {
            let tx = rect.x2 - self.padding_x - self.trailing_size.width;
            let ty = cy - self.trailing_size.height / 2.0;
            let trailing_rect = Rect::new(tx, ty, tx + self.trailing_size.width, ty + self.trailing_size.height);
            trailing.paint(canvas, trailing_rect);
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
                if let Some(ref mut cb) = self.on_click {
                    cb();
                }
                EventResponse {
                    handled: true,
                    cursor: Some(CursorIcon::Pointer),
                    ..Default::default()
                }
            }
            WidgetEvent::Mouse(MouseEvent::MouseMoveEvent(pos)) => {
                let was_hovered = self.hovered;
                self.hovered = rect.contains(pos);
                if self.hovered {
                    EventResponse {
                        handled: was_hovered != self.hovered,
                        cursor: Some(CursorIcon::Pointer),
                        ..Default::default()
                    }
                } else {
                    EventResponse {
                        handled: was_hovered != self.hovered,
                        ..Default::default()
                    }
                }
            }
            _ => EventResponse::default(),
        }
    }
#[cfg(feature = "a11y")]    fn access_info(&self) -> aurora_a11y::NodeInfo {        aurora_a11y::NodeInfo::new(aurora_a11y::accesskit::Role::ListItem)    }
}
