use crate::widgets::{EventResponse, LayoutCtx, Widget};
use aurora_core::color::Color;
use aurora_core::geometry::edges::Edges;
use aurora_core::geometry::point::Point;
use aurora_core::geometry::rect::Rect;
use aurora_core::geometry::size::Size;
use aurora_core::kmi::WidgetEvent;
use aurora_core::kmi::cursor_icon::CursorIcon;
use aurora_core::kmi::mouse::{MouseEvent, MouseState};
use aurora_render::canvas::Canvas;
use std::time::Instant;

use super::colors;

const ANIM_DURATION: f32 = 0.2; // seconds

/// A single collapsible section with a title and toggleable content.
/// Content animates open and closed.
///
/// # Example
/// ```ignore
/// Collapsible::new("Details")
///     .expanded(true)
///     .child(Label::new("Hidden content here"))
/// ```
pub struct Collapsible {
    title: String,
    expanded: bool,
    child: Option<Box<dyn Widget>>,
    header_height: f32,
    content_padding: Edges,
    border_color: Color,
    width: Option<f32>,
    on_toggle: Option<Box<dyn FnMut(bool)>>,
    title_layout: Option<aurora_text::text_layout::TextLayout>,
    child_size: Size,
    /// 0.0 = collapsed, 1.0 = expanded
    anim_from: f32,
    anim_to: f32,
    anim_start: Instant,
}

impl Collapsible {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            expanded: false,
            child: None,
            header_height: 40.0,
            content_padding: Edges::new(8.0, 0.0, 8.0, 0.0),
            border_color: colors::border(),
            width: None,
            on_toggle: None,
            title_layout: None,
            child_size: Size::default(),
            anim_from: 0.0,
            anim_to: 0.0,
            anim_start: Instant::now(),
        }
    }

    pub fn expanded(mut self, expanded: bool) -> Self {
        self.expanded = expanded;
        let t = if expanded { 1.0 } else { 0.0 };
        self.anim_from = t;
        self.anim_to = t;
        self
    }

    pub fn child(mut self, widget: impl Widget + 'static) -> Self {
        self.child = Some(Box::new(widget));
        self
    }

    pub fn header_height(mut self, height: f32) -> Self {
        self.header_height = height;
        self
    }

    pub fn content_padding(mut self, padding: Edges) -> Self {
        self.content_padding = padding;
        self
    }

    pub fn border_color(mut self, color: Color) -> Self {
        self.border_color = color;
        self
    }

    pub fn width(mut self, width: impl Into<f32>) -> Self {
        self.width = Some(width.into());
        self
    }

    pub fn on_toggle(mut self, cb: impl FnMut(bool) + 'static) -> Self {
        self.on_toggle = Some(Box::new(cb));
        self
    }

    fn current_t(&self) -> f32 {
        let elapsed = self.anim_start.elapsed().as_secs_f32();
        let t = (elapsed / ANIM_DURATION).min(1.0);
        let eased = 1.0 - (1.0 - t).powi(3);
        self.anim_from + (self.anim_to - self.anim_from) * eased
    }
}

impl Widget for Collapsible {
    fn layout(&mut self, available: Size, ctx: &mut LayoutCtx) -> Size {
        let w = self.width.unwrap_or(available.width);

        // Title layout
        let mut opts = ctx.font_options.clone();
        opts.size = Some(14.0);
        opts.weight = Some(aurora_text::font_options::FontWeight::Medium);
        let tl = aurora_text::text_layout::TextLayout::new(
            ctx.font_manager,
            &self.title,
            &opts,
            colors::foreground(),
            None,
        );
        self.title_layout = Some(tl);

        // Always layout child for animation height
        if let Some(ref mut child) = self.child {
            let content_w = w - self.content_padding.left - self.content_padding.right;
            let content_available = Size::new(content_w.max(0.0), f32::MAX);
            self.child_size = child.layout(content_available, ctx);
        }

        let t = self.current_t();
        let content_h =
            (self.content_padding.top + self.child_size.height + self.content_padding.bottom) * t;
        let total_h = self.header_height + content_h;

        Size::new(w, total_h)
    }

    fn paint(&self, canvas: &mut Canvas, rect: Rect) {
        let t = self.current_t();

        // Chevron: rotates from right-pointing (t=0) to down-pointing (t=1)
        let chevron_x = rect.x1 + 8.0;
        let chevron_cy = rect.y1 + self.header_height / 2.0;
        let angle = t * std::f32::consts::FRAC_PI_2;
        let cos_a = angle.cos();
        let sin_a = angle.sin();
        let rotate =
            |dx: f32, dy: f32| -> (f32, f32) { (dx * cos_a - dy * sin_a, dx * sin_a + dy * cos_a) };
        let (dx1, dy1) = rotate(-2.0, -4.0);
        let (dx2, dy2) = rotate(2.0, 0.0);
        let (dx3, dy3) = rotate(-2.0, 4.0);
        canvas.draw_line(
            Point::new(chevron_x + dx1, chevron_cy + dy1),
            Point::new(chevron_x + dx2, chevron_cy + dy2),
            1.0,
            colors::muted_foreground(),
        );
        canvas.draw_line(
            Point::new(chevron_x + dx2, chevron_cy + dy2),
            Point::new(chevron_x + dx3, chevron_cy + dy3),
            1.0,
            colors::muted_foreground(),
        );

        // Title text
        if let Some(ref tl) = self.title_layout {
            let th = tl.size().height;
            let tx = rect.x1 + 24.0;
            let ty = rect.y1 + (self.header_height - th) / 2.0;
            canvas.draw_text(tl, tx as i32, ty as i32);
        }

        // Bottom border on header
        let border_y = rect.y1 + self.header_height;
        canvas.fill_rect(
            Rect::new(rect.x1, border_y - 1.0, rect.x2, border_y),
            self.border_color,
        );

        // Content (clipped for animation)
        if t > 0.0
            && let Some(ref child) = self.child
        {
            let content_area_top = rect.y1 + self.header_height;
            let visible_h =
                (self.content_padding.top + self.child_size.height + self.content_padding.bottom)
                    * t;
            let clip = Rect::new(
                rect.x1,
                content_area_top,
                rect.x2,
                content_area_top + visible_h,
            );
            canvas.push_clip(clip);

            let content_y = content_area_top + self.content_padding.top;
            let content_rect = Rect::new(
                rect.x1 + self.content_padding.left,
                content_y,
                rect.x1 + self.content_padding.left + self.child_size.width,
                content_y + self.child_size.height,
            );
            child.paint(canvas, content_rect);

            canvas.pop_clip();
        }
    }

    fn children(&self) -> &[Box<dyn Widget>] {
        match &self.child {
            Some(c) => std::slice::from_ref(c),
            None => &[],
        }
    }

    fn event(&mut self, event: &WidgetEvent, rect: Rect) -> EventResponse {
        let header_rect = Rect::new(rect.x1, rect.y1, rect.x2, rect.y1 + self.header_height);

        match event {
            WidgetEvent::Mouse(MouseEvent::MouseClickEvent(e))
                if e.state == MouseState::Pressed && header_rect.contains(&e.position) =>
            {
                self.expanded = !self.expanded;
                self.anim_from = self.current_t();
                self.anim_to = if self.expanded { 1.0 } else { 0.0 };
                self.anim_start = Instant::now();
                if let Some(ref mut cb) = self.on_toggle {
                    cb(self.expanded);
                }
                EventResponse {
                    handled: true,
                    cursor: Some(CursorIcon::Pointer),
                    ..Default::default()
                }
            }
            WidgetEvent::Mouse(MouseEvent::MouseMoveEvent(pos)) if header_rect.contains(pos) => {
                EventResponse {
                    handled: true,
                    cursor: Some(CursorIcon::Pointer),
                    ..Default::default()
                }
            }
            _ => {
                if self.expanded
                    && let Some(ref mut child) = self.child
                {
                    let content_y = rect.y1 + self.header_height + self.content_padding.top;
                    let content_rect = Rect::new(
                        rect.x1 + self.content_padding.left,
                        content_y,
                        rect.x1 + self.content_padding.left + self.child_size.width,
                        content_y + self.child_size.height,
                    );
                    return child.event(event, content_rect);
                }
                EventResponse::default()
            }
        }
    }

    fn needs_animation(&self) -> bool {
        self.anim_start.elapsed().as_secs_f32() < ANIM_DURATION
    }
    #[cfg(feature = "a11y")]
    fn access_info(&self) -> aurora_a11y::NodeInfo {
        aurora_a11y::NodeInfo::new(aurora_a11y::accesskit::Role::DisclosureTriangle)
            .with_label(self.title.clone())
            .with_expanded(self.expanded)
    }
}
