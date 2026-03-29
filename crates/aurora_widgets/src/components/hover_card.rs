use crate::widgets::{EventResponse, LayoutCtx, Widget};
use aurora_core::color::Color;
use aurora_core::geometry::corners::Corners;
use aurora_core::geometry::edges::Edges;
use aurora_core::geometry::rect::Rect;
use aurora_core::geometry::size::Size;
use aurora_core::kmi::WidgetEvent;
use aurora_core::kmi::mouse::MouseEvent;
use aurora_render::canvas::Canvas;

use super::colors;

/// A hover-triggered card popup. Shows a card when the user hovers over
/// the trigger element.
///
/// # Example
/// ```ignore
/// HoverCard::new()
///     .trigger(Label::new("@username"))
///     .content(
///         Card::new()
///             .child(Label::new("User profile info"))
///     )
/// ```
pub struct HoverCard {
    trigger: Option<Box<dyn Widget>>,
    content: Option<Box<dyn Widget>>,
    hovered: bool,
    offset: f32,
    background: Color,
    border_color: Color,
    corners: Corners,
    padding: Edges,
    trigger_size: Size,
    content_size: Size,
}

impl HoverCard {
    pub fn new() -> Self {
        Self {
            trigger: None,
            content: None,
            hovered: false,
            offset: 4.0,
            background: colors::popover(),
            border_color: colors::border(),
            corners: Corners::all(8.0),
            padding: Edges::new(16.0, 16.0, 16.0, 16.0),
            trigger_size: Size::default(),
            content_size: Size::default(),
        }
    }

    pub fn trigger(mut self, widget: impl Widget + 'static) -> Self {
        self.trigger = Some(Box::new(widget));
        self
    }

    pub fn content(mut self, widget: impl Widget + 'static) -> Self {
        self.content = Some(Box::new(widget));
        self
    }

    pub fn offset(mut self, offset: f32) -> Self {
        self.offset = offset;
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

    fn card_rect(&self, trigger_rect: &Rect) -> Rect {
        let cw = self.content_size.width + self.padding.left + self.padding.right;
        let ch = self.content_size.height + self.padding.top + self.padding.bottom;
        Rect::new(
            trigger_rect.x1,
            trigger_rect.y2 + self.offset,
            trigger_rect.x1 + cw,
            trigger_rect.y2 + self.offset + ch,
        )
    }
}

impl Default for HoverCard {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for HoverCard {
    fn layout(&mut self, available: Size, ctx: &mut LayoutCtx) -> Size {
        if let Some(ref mut trigger) = self.trigger {
            self.trigger_size = trigger.layout(available, ctx);
        }

        if let Some(ref mut content) = self.content {
            let content_available = Size::new(
                (available.width - self.padding.left - self.padding.right).max(0.0),
                f32::MAX,
            );
            self.content_size = content.layout(content_available, ctx);
        }

        self.trigger_size
    }

    fn paint(&self, canvas: &mut Canvas, rect: Rect) {
        // Paint trigger
        if let Some(ref trigger) = self.trigger {
            trigger.paint(canvas, rect);
        }
    }

    fn paint_overlay(&self, canvas: &mut Canvas, rect: Rect) {
        if !self.hovered {
            return;
        }

        let cr = self.card_rect(&rect);
        canvas.fill_rounded_rect(cr, self.corners, self.background);
        canvas.stroke_rounded_rect(cr, self.corners, 1, self.border_color);

        if let Some(ref content) = self.content {
            let content_rect = Rect::new(
                cr.x1 + self.padding.left,
                cr.y1 + self.padding.top,
                cr.x1 + self.padding.left + self.content_size.width,
                cr.y1 + self.padding.top + self.content_size.height,
            );
            content.paint(canvas, content_rect);
        }
    }

    fn children(&self) -> &[Box<dyn Widget>] {
        &[]
    }

    fn event(&mut self, event: &WidgetEvent, rect: Rect) -> EventResponse {
        match event {
            WidgetEvent::Mouse(MouseEvent::MouseMoveEvent(pos)) => {
                let card_r = self.card_rect(&rect);
                self.hovered = rect.contains(pos) || (self.hovered && card_r.contains(pos));

                if let Some(ref mut trigger) = self.trigger {
                    let resp = trigger.event(event, rect);
                    if resp.handled {
                        return resp;
                    }
                }
                if self.hovered
                    && let Some(ref mut content) = self.content
                {
                    let content_rect = Rect::new(
                        card_r.x1 + self.padding.left,
                        card_r.y1 + self.padding.top,
                        card_r.x1 + self.padding.left + self.content_size.width,
                        card_r.y1 + self.padding.top + self.content_size.height,
                    );
                    return content.event(event, content_rect);
                }
                EventResponse::default()
            }
            _ => {
                if let Some(ref mut trigger) = self.trigger {
                    return trigger.event(event, rect);
                }
                EventResponse::default()
            }
        }
    }

    #[cfg(feature = "a11y")]
    fn access_info(&self) -> aurora_a11y::NodeInfo {
        let mut info = aurora_a11y::NodeInfo::new(aurora_a11y::accesskit::Role::Tooltip);
        if !self.hovered {
            info = info.with_hidden();
        }
        info
    }
}
