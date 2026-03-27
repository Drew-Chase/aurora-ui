use crate::widgets::{EventResponse, LayoutCtx, Widget};
use aurora_core::color::Color;
use aurora_core::geometry::corners::Corners;
use aurora_core::geometry::edges::Edges;
use aurora_core::geometry::point::Point;
use aurora_core::geometry::rect::Rect;
use aurora_core::geometry::size::Size;
use aurora_core::kmi::cursor_icon::CursorIcon;
use aurora_core::kmi::mouse::{MouseEvent, MouseState};
use aurora_core::kmi::WidgetEvent;
use aurora_render::canvas::Canvas;

use super::colors;

/// Visual variant for a toast notification.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum ToastVariant {
    #[default]
    Default,
    Success,
    Error,
    Warning,
    Info,
}

/// A toast notification rendered at a fixed position.
///
/// # Example
/// ```ignore
/// Toast::new("Changes saved successfully")
///     .variant(ToastVariant::Success)
///     .visible(true)
///     .on_dismiss(|| println!("dismissed"))
/// ```
pub struct Toast {
    message: String,
    title: Option<String>,
    variant: ToastVariant,
    visible: bool,
    width: f32,
    corners: Corners,
    padding: Edges,
    on_dismiss: Option<Box<dyn FnMut()>>,
    title_layout: Option<aurora_text::text_layout::TextLayout>,
    message_layout: Option<aurora_text::text_layout::TextLayout>,
    title_height: f32,
    message_height: f32,
}

impl Toast {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            title: None,
            variant: ToastVariant::Default,
            visible: true,
            width: 360.0,
            corners: Corners::all(8.0),
            padding: Edges::new(16.0, 16.0, 16.0, 16.0),
            on_dismiss: None,
            title_layout: None,
            message_layout: None,
            title_height: 0.0,
            message_height: 0.0,
        }
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn variant(mut self, variant: ToastVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    pub fn on_dismiss(mut self, cb: impl FnMut() + 'static) -> Self {
        self.on_dismiss = Some(Box::new(cb));
        self
    }

    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    fn bg_color(&self) -> Color {
        match self.variant {
            ToastVariant::Default => colors::background(),
            ToastVariant::Success => Color::new(240, 253, 244, 255),
            ToastVariant::Error => Color::new(254, 242, 242, 255),
            ToastVariant::Warning => Color::new(254, 252, 232, 255),
            ToastVariant::Info => Color::new(239, 246, 255, 255),
        }
    }

    fn border_color(&self) -> Color {
        match self.variant {
            ToastVariant::Default => colors::border(),
            ToastVariant::Success => colors::success(),
            ToastVariant::Error => colors::destructive(),
            ToastVariant::Warning => colors::warning(),
            ToastVariant::Info => colors::info(),
        }
    }

    #[allow(dead_code)]
    fn fg_color(&self) -> Color {
        match self.variant {
            ToastVariant::Default => colors::foreground(),
            ToastVariant::Success => Color::new(22, 163, 74, 255),
            ToastVariant::Error => colors::destructive(),
            ToastVariant::Warning => Color::new(161, 98, 7, 255),
            ToastVariant::Info => Color::new(37, 99, 235, 255),
        }
    }
}

impl Widget for Toast {
    fn layout(&mut self, _available: Size, ctx: &mut LayoutCtx) -> Size {
        if !self.visible {
            return Size::new(0.0, 0.0);
        }

        let inner_w = self.width - self.padding.left - self.padding.right - 24.0; // space for dismiss X

        // Title
        if let Some(ref title) = self.title {
            let mut opts = ctx.font_options.clone();
            opts.size = Some(14.0);
            opts.weight = Some(aurora_text::font_options::FontWeight::Medium);
            let mut tl = aurora_text::text_layout::TextLayout::new(ctx.font_manager, title, &opts, colors::foreground(), None);
            tl.set_max_width(ctx.font_manager, inner_w.max(0.0));
            let th = tl.size().height;
            self.title_height = th;
            self.title_layout = Some(tl);
        }

        // Message
        let mut opts = ctx.font_options.clone();
        opts.size = Some(14.0);
        opts.weight = Some(aurora_text::font_options::FontWeight::Normal);
        let mut tl = aurora_text::text_layout::TextLayout::new(ctx.font_manager, &self.message, &opts, colors::foreground(), None);
        tl.set_max_width(ctx.font_manager, inner_w.max(0.0));
        let mh = tl.size().height;
        self.message_height = mh;
        self.message_layout = Some(tl);

        let mut total_h = self.padding.top + self.padding.bottom + self.message_height;
        if self.title.is_some() {
            total_h += self.title_height + 4.0;
        }

        Size::new(self.width, total_h)
    }

    fn paint(&self, canvas: &mut Canvas, rect: Rect) {
        if !self.visible {
            return;
        }

        canvas.fill_rounded_rect(rect, self.corners, self.bg_color());
        canvas.stroke_rounded_rect(rect, self.corners, 1, self.border_color());

        let mut y = rect.y1 + self.padding.top;
        let x = rect.x1 + self.padding.left;

        // Title
        if let Some(ref tl) = self.title_layout {
            canvas.draw_text(tl, x as i32, y as i32);
            y += self.title_height + 4.0;
        }

        // Message
        if let Some(ref tl) = self.message_layout {
            canvas.draw_text(tl, x as i32, y as i32);
        }

        // Dismiss X button
        let close_x = rect.x2 - self.padding.right - 8.0;
        let close_y = rect.y1 + self.padding.top;
        canvas.draw_line(
            Point::new(close_x - 4.0, close_y),
            Point::new(close_x + 4.0, close_y + 8.0),
            1.0,
            colors::muted_foreground(),
        );
        canvas.draw_line(
            Point::new(close_x + 4.0, close_y),
            Point::new(close_x - 4.0, close_y + 8.0),
            1.0,
            colors::muted_foreground(),
        );
    }

    fn children(&self) -> &[Box<dyn Widget>] {
        &[]
    }

    fn event(&mut self, event: &WidgetEvent, rect: Rect) -> EventResponse {
        if !self.visible {
            return EventResponse::default();
        }

        match event {
            WidgetEvent::Mouse(MouseEvent::MouseClickEvent(e))
                if e.state == MouseState::Pressed && rect.contains(&e.position) =>
            {
                // Check dismiss button area
                let close_x = rect.x2 - self.padding.right - 8.0;
                let close_y = rect.y1 + self.padding.top;
                let close_rect = Rect::new(close_x - 8.0, close_y - 4.0, close_x + 12.0, close_y + 12.0);
                if close_rect.contains(&e.position) {
                    self.visible = false;
                    if let Some(ref mut cb) = self.on_dismiss {
                        cb();
                    }
                    return EventResponse {
                        handled: true,
                        ..Default::default()
                    };
                }
                EventResponse {
                    handled: true,
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
