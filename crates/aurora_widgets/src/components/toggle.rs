use crate::widgets::{EventResponse, LayoutCtx, Widget};
use aurora_core::color::Color;
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

/// Toggle variant style.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum ToggleVariant {
    #[default]
    Default,
    Outline,
}

/// A pressable toggle button.
///
/// # Example
/// ```ignore
/// Toggle::new("Bold")
///     .pressed(true)
///     .on_change(|pressed| println!("pressed: {pressed}"))
/// ```
pub struct Toggle {
    label: Option<String>,
    pressed: bool,
    disabled: bool,
    variant: ToggleVariant,
    width: Option<f32>,
    height: Option<f32>,
    on_change: Option<Box<dyn FnMut(bool)>>,
    child: Option<Box<dyn Widget>>,
    child_size: Size,
    label_layout: Option<TextLayout>,
}

impl Toggle {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: Some(label.into()),
            pressed: false,
            disabled: false,
            variant: ToggleVariant::Default,
            width: None,
            height: Some(40.0),
            on_change: None,
            child: None,
            child_size: Size::default(),
            label_layout: None,
        }
    }

    pub fn with_child(widget: impl Widget + 'static) -> Self {
        Self {
            label: None,
            pressed: false,
            disabled: false,
            variant: ToggleVariant::Default,
            width: None,
            height: Some(40.0),
            on_change: None,
            child: Some(Box::new(widget)),
            child_size: Size::default(),
            label_layout: None,
        }
    }

    pub fn pressed(mut self, pressed: bool) -> Self {
        self.pressed = pressed;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn variant(mut self, variant: ToggleVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn width(mut self, width: impl Into<f32>) -> Self {
        self.width = Some(width.into());
        self
    }

    pub fn height(mut self, height: impl Into<f32>) -> Self {
        self.height = Some(height.into());
        self
    }

    pub fn on_change(mut self, cb: impl FnMut(bool) + 'static) -> Self {
        self.on_change = Some(Box::new(cb));
        self
    }
}

impl Widget for Toggle {
    fn layout(&mut self, available: Size, ctx: &mut LayoutCtx) -> Size {
        let h = self.height.unwrap_or(40.0);
        let mut content_w = 0.0f32;

        if let Some(ref label) = self.label {
            let mut opts = ctx.font_options.clone();
            opts.size = Some(14.0);
            opts.weight = Some(FontWeight::Medium);
            let mut tl = TextLayout::new(ctx.font_manager, label, &opts, colors::FOREGROUND, None);
            tl.set_max_width(ctx.font_manager, f32::MAX);
            let ts = tl.size();
            content_w = ts.width;
            self.label_layout = Some(tl);
        }

        if let Some(ref mut child) = self.child {
            self.child_size = child.layout(Size::new(available.width, h), ctx);
            content_w = self.child_size.width;
        }

        let pad = 20.0;
        let w = self.width.unwrap_or(content_w + pad * 2.0);
        Size::new(w, h)
    }

    fn paint(&self, canvas: &mut Canvas, rect: Rect) {
        let corners = Corners::all(6.0);

        let bg = if self.pressed {
            colors::ACCENT
        } else {
            Color::TRANSPARENT
        };

        if bg.alpha > 0 {
            canvas.fill_rounded_rect(rect, corners, bg);
        }
        if matches!(self.variant, ToggleVariant::Outline) || self.pressed {
            canvas.stroke_rounded_rect(rect, corners, 1, colors::BORDER);
        }

        if let Some(ref tl) = self.label_layout {
            let ts = tl.size();
            let x = rect.x1 + (rect.width() - ts.width) / 2.0;
            let y = rect.y1 + (rect.height() - ts.height) / 2.0;
            canvas.draw_text(tl, x as i32, y as i32);
        }
        if let Some(ref child) = self.child {
            let cx = rect.x1 + (rect.width() - self.child_size.width) / 2.0;
            let cy = rect.y1 + (rect.height() - self.child_size.height) / 2.0;
            let child_rect = Rect::new(cx, cy, cx + self.child_size.width, cy + self.child_size.height);
            child.paint(canvas, child_rect);
        }
    }

    fn children(&self) -> &[Box<dyn Widget>] {
        match &self.child {
            Some(c) => std::slice::from_ref(c),
            None => &[],
        }
    }

    fn event(&mut self, event: &WidgetEvent, rect: Rect) -> EventResponse {
        if self.disabled {
            return EventResponse::default();
        }
        match event {
            WidgetEvent::Mouse(MouseEvent::MouseClickEvent(e))
                if e.state == MouseState::Pressed && rect.contains(&e.position) =>
            {
                self.pressed = !self.pressed;
                if let Some(ref mut cb) = self.on_change {
                    cb(self.pressed);
                }
                EventResponse {
                    handled: true,
                    cursor: Some(CursorIcon::Pointer),
                    ..Default::default()
                }
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
