use crate::widgets::{EventResponse, LayoutCtx, Widget};
use aurora_core::color::Color;
use aurora_core::geometry::corners::Corners;
use aurora_core::geometry::edges::Edges;
use aurora_core::geometry::rect::Rect;
use aurora_core::geometry::size::Size;
use aurora_core::kmi::WidgetEvent;
use aurora_core::kmi::mouse::{MouseEvent, MouseState};
use aurora_render::canvas::Canvas;

use super::colors;

/// A modal dialog overlay. When open, paints a semi-transparent backdrop
/// over the child and centers a content panel on top.
///
/// # Example
/// ```ignore
/// Dialog::new()
///     .open(true)
///     .title("Confirm Delete")
///     .content(Label::new("Are you sure you want to delete this item?"))
///     .on_close(|| println!("dialog closed"))
/// ```
pub struct Dialog {
    open: bool,
    title: Option<String>,
    content: Option<Box<dyn Widget>>,
    footer: Option<Box<dyn Widget>>,
    backdrop_color: Color,
    background: Color,
    border_color: Color,
    corners: Corners,
    padding: Edges,
    width: f32,
    on_close: Option<Box<dyn FnMut()>>,
    title_layout: Option<aurora_text::text_layout::TextLayout>,
    title_height: f32,
    content_size: Size,
    footer_size: Size,
}

impl Dialog {
    pub fn new() -> Self {
        Self {
            open: false,
            title: None,
            content: None,
            footer: None,
            backdrop_color: colors::overlay(),
            background: colors::background(),
            border_color: colors::border(),
            corners: Corners::all(12.0),
            padding: Edges::new(24.0, 24.0, 24.0, 24.0),
            width: 400.0,
            on_close: None,
            title_layout: None,
            title_height: 0.0,
            content_size: Size::default(),
            footer_size: Size::default(),
        }
    }

    pub fn open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn content(mut self, widget: impl Widget + 'static) -> Self {
        self.content = Some(Box::new(widget));
        self
    }

    pub fn footer(mut self, widget: impl Widget + 'static) -> Self {
        self.footer = Some(Box::new(widget));
        self
    }

    pub fn backdrop_color(mut self, color: Color) -> Self {
        self.backdrop_color = color;
        self
    }

    pub fn background(mut self, color: Color) -> Self {
        self.background = color;
        self
    }

    pub fn border_color(mut self, color: Color) -> Self {
        self.border_color = color;
        self
    }

    pub fn corners(mut self, corners: Corners) -> Self {
        self.corners = corners;
        self
    }

    pub fn padding(mut self, padding: Edges) -> Self {
        self.padding = padding;
        self
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    pub fn on_close(mut self, cb: impl FnMut() + 'static) -> Self {
        self.on_close = Some(Box::new(cb));
        self
    }

    pub fn set_open(&mut self, open: bool) {
        self.open = open;
    }

    fn dialog_rect(&self, viewport: &Rect) -> Rect {
        let _inner_w = self.width - self.padding.left - self.padding.right;
        let mut total_h = self.padding.top;
        if self.title.is_some() {
            total_h += self.title_height + 16.0;
        }
        total_h += self.content_size.height;
        if self.footer.is_some() {
            total_h += 16.0 + self.footer_size.height;
        }
        total_h += self.padding.bottom;

        let cx = viewport.x1 + viewport.width() / 2.0;
        let cy = viewport.y1 + viewport.height() / 2.0;

        Rect::new(
            cx - self.width / 2.0,
            cy - total_h / 2.0,
            cx + self.width / 2.0,
            cy + total_h / 2.0,
        )
    }
}

impl Default for Dialog {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for Dialog {
    fn layout(&mut self, available: Size, ctx: &mut LayoutCtx) -> Size {
        if !self.open {
            return Size::new(0.0, 0.0);
        }

        let inner_w = self.width - self.padding.left - self.padding.right;

        // Title
        if let Some(ref title) = self.title {
            let mut opts = ctx.font_options.clone();
            opts.size = Some(18.0);
            opts.weight = Some(aurora_text::font_options::FontWeight::Bold);
            let mut tl = aurora_text::text_layout::TextLayout::new(
                ctx.font_manager,
                title,
                &opts,
                colors::foreground(),
                None,
            );
            tl.set_max_width(ctx.font_manager, inner_w);
            let th = tl.size().height;
            self.title_height = th;
            self.title_layout = Some(tl);
        }

        // Content
        if let Some(ref mut content) = self.content {
            let content_available = Size::new(inner_w.max(0.0), f32::MAX);
            self.content_size = content.layout(content_available, ctx);
        }

        // Footer
        if let Some(ref mut footer) = self.footer {
            let footer_available = Size::new(inner_w.max(0.0), f32::MAX);
            self.footer_size = footer.layout(footer_available, ctx);
        }

        // The dialog is an overlay -- it doesn't consume layout space in the parent
        Size::new(available.width, available.height)
    }

    fn paint(&self, canvas: &mut Canvas, rect: Rect) {
        if !self.open {
            return;
        }

        // Backdrop
        canvas.fill_rect(rect, self.backdrop_color);

        // Dialog panel
        let dr = self.dialog_rect(&rect);
        canvas.fill_rounded_rect(dr, self.corners, self.background);
        canvas.stroke_rounded_rect(dr, self.corners, 1, self.border_color);

        let mut y = dr.y1 + self.padding.top;
        let x = dr.x1 + self.padding.left;

        // Title
        if let Some(ref tl) = self.title_layout {
            canvas.draw_text(tl, x as i32, y as i32);
            y += self.title_height + 16.0;
        }

        // Content
        if let Some(ref content) = self.content {
            let content_rect = Rect::new(
                x,
                y,
                x + self.content_size.width,
                y + self.content_size.height,
            );
            content.paint(canvas, content_rect);
            y += self.content_size.height;
        }

        // Footer
        if let Some(ref footer) = self.footer {
            y += 16.0;
            let footer_rect = Rect::new(
                x,
                y,
                x + self.footer_size.width,
                y + self.footer_size.height,
            );
            footer.paint(canvas, footer_rect);
        }
    }

    fn children(&self) -> &[Box<dyn Widget>] {
        &[]
    }

    fn event(&mut self, event: &WidgetEvent, rect: Rect) -> EventResponse {
        if !self.open {
            return EventResponse::default();
        }

        let dr = self.dialog_rect(&rect);

        match event {
            WidgetEvent::Mouse(MouseEvent::MouseClickEvent(e))
                if e.state == MouseState::Pressed =>
            {
                // Click outside dialog closes it
                if !dr.contains(&e.position) {
                    self.open = false;
                    if let Some(ref mut cb) = self.on_close {
                        cb();
                    }
                    return EventResponse {
                        handled: true,
                        ..Default::default()
                    };
                }
                // Forward to content/footer
                let mut y = dr.y1 + self.padding.top;
                let x = dr.x1 + self.padding.left;

                if self.title.is_some() {
                    y += self.title_height + 16.0;
                }

                if let Some(ref mut content) = self.content {
                    let content_rect = Rect::new(
                        x,
                        y,
                        x + self.content_size.width,
                        y + self.content_size.height,
                    );
                    let resp = content.event(event, content_rect);
                    if resp.handled {
                        return resp;
                    }
                    y += self.content_size.height + 16.0;
                }

                if let Some(ref mut footer) = self.footer {
                    let footer_rect = Rect::new(
                        x,
                        y,
                        x + self.footer_size.width,
                        y + self.footer_size.height,
                    );
                    let resp = footer.event(event, footer_rect);
                    if resp.handled {
                        return resp;
                    }
                }

                EventResponse {
                    handled: true,
                    ..Default::default()
                }
            }
            _ => {
                // Forward keyboard/other events to content
                if let Some(ref mut content) = self.content {
                    let y = dr.y1
                        + self.padding.top
                        + if self.title.is_some() {
                            self.title_height + 16.0
                        } else {
                            0.0
                        };
                    let x = dr.x1 + self.padding.left;
                    let content_rect = Rect::new(
                        x,
                        y,
                        x + self.content_size.width,
                        y + self.content_size.height,
                    );
                    let resp = content.event(event, content_rect);
                    if resp.handled {
                        return resp;
                    }
                }
                // Consume all events when dialog is open
                EventResponse {
                    handled: true,
                    ..Default::default()
                }
            }
        }
    }

    #[cfg(feature = "a11y")]
    fn access_info(&self) -> aurora_a11y::NodeInfo {
        let mut info =
            aurora_a11y::NodeInfo::new(aurora_a11y::accesskit::Role::Dialog).with_modal();
        if !self.open {
            info = info.with_hidden();
        }
        if let Some(ref title) = self.title {
            info = info.with_label(title.clone());
        }
        info
    }
}
