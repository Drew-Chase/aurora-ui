use crate::widgets::{EventResponse, LayoutCtx, Widget};
use aurora_core::color::Color;
use aurora_core::geometry::corners::Corners;
use aurora_core::geometry::edges::Edges;
use aurora_core::geometry::point::Point;
use aurora_core::geometry::rect::Rect;
use aurora_core::geometry::size::Size;
use aurora_core::kmi::mouse::MouseEvent;
use aurora_core::kmi::WidgetEvent;
use aurora_render::canvas::Canvas;

use super::colors;

/// Tooltip placement relative to the child widget.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum TooltipSide {
    #[default]
    Top,
    Bottom,
    Left,
    Right,
}

/// A hover tooltip that renders text near the child widget.
///
/// When the mouse hovers over the child, a tooltip box is painted
/// above (or on the configured side of) the child.
///
/// # Example
/// ```ignore
/// Tooltip::new("Helpful tip")
///     .child(Label::new("Hover me"))
///     .side(TooltipSide::Top)
/// ```
pub struct Tooltip {
    text: String,
    child: Option<Box<dyn Widget>>,
    side: TooltipSide,
    offset: f32,
    background: Color,
    foreground: Color,
    corners: Corners,
    padding: Edges,
    hovered: bool,
    mouse_pos: Point,
    child_size: Size,
    tooltip_layout: Option<aurora_text::text_layout::TextLayout>,
    tooltip_size: Size,
}

impl Tooltip {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            child: None,
            side: TooltipSide::Top,
            offset: 6.0,
            background: colors::primary(),
            foreground: colors::primary_foreground(),
            corners: Corners::all(4.0),
            padding: Edges::new(4.0, 8.0, 4.0, 8.0),
            hovered: false,
            mouse_pos: Point::new(0.0, 0.0),
            child_size: Size::default(),
            tooltip_layout: None,
            tooltip_size: Size::default(),
        }
    }

    pub fn child(mut self, widget: impl Widget + 'static) -> Self {
        self.child = Some(Box::new(widget));
        self
    }

    pub fn side(mut self, side: TooltipSide) -> Self {
        self.side = side;
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

    pub fn foreground(mut self, color: Color) -> Self {
        self.foreground = color;
        self
    }

    pub fn corners(mut self, corners: Corners) -> Self {
        self.corners = corners;
        self
    }

    fn tooltip_rect(&self, child_rect: &Rect) -> Rect {
        let tw = self.tooltip_size.width;
        let th = self.tooltip_size.height;
        let cx = child_rect.x1 + child_rect.width() / 2.0;
        let cy = child_rect.y1 + child_rect.height() / 2.0;

        match self.side {
            TooltipSide::Top => Rect::new(
                cx - tw / 2.0,
                child_rect.y1 - th - self.offset,
                cx + tw / 2.0,
                child_rect.y1 - self.offset,
            ),
            TooltipSide::Bottom => Rect::new(
                cx - tw / 2.0,
                child_rect.y2 + self.offset,
                cx + tw / 2.0,
                child_rect.y2 + self.offset + th,
            ),
            TooltipSide::Left => Rect::new(
                child_rect.x1 - tw - self.offset,
                cy - th / 2.0,
                child_rect.x1 - self.offset,
                cy + th / 2.0,
            ),
            TooltipSide::Right => Rect::new(
                child_rect.x2 + self.offset,
                cy - th / 2.0,
                child_rect.x2 + self.offset + tw,
                cy + th / 2.0,
            ),
        }
    }
}

impl Widget for Tooltip {
    fn layout(&mut self, available: Size, ctx: &mut LayoutCtx) -> Size {
        if let Some(ref mut child) = self.child {
            self.child_size = child.layout(available, ctx);
        }

        // Layout tooltip text
        let mut opts = ctx.font_options.clone();
        opts.size = Some(12.0);
        opts.weight = Some(aurora_text::font_options::FontWeight::Normal);
        let mut tl = aurora_text::text_layout::TextLayout::new(ctx.font_manager, &self.text, &opts, colors::foreground(), None);
        tl.set_max_width(ctx.font_manager, f32::MAX);
        let _s = tl.size(); let tw = _s.width; let th = _s.height;
        self.tooltip_size = Size::new(
            tw + self.padding.left + self.padding.right,
            th + self.padding.top + self.padding.bottom,
        );
        self.tooltip_layout = Some(tl);

        self.child_size
    }

    fn paint(&self, canvas: &mut Canvas, rect: Rect) {
        // Paint child
        if let Some(ref child) = self.child {
            child.paint(canvas, rect);
        }

        // Paint tooltip when hovered
        if self.hovered {
            let tr = self.tooltip_rect(&rect);
            canvas.fill_rounded_rect(tr, self.corners, self.background);
            if let Some(ref tl) = self.tooltip_layout {
                let tx = tr.x1 + self.padding.left;
                let ty = tr.y1 + self.padding.top;
                canvas.draw_text(tl, tx as i32, ty as i32);
            }
        }
    }

    fn children(&self) -> &[Box<dyn Widget>] {
        match &self.child {
            Some(c) => std::slice::from_ref(c),
            None => &[],
        }
    }

    fn event(&mut self, event: &WidgetEvent, rect: Rect) -> EventResponse {
        match event {
            WidgetEvent::Mouse(MouseEvent::MouseMoveEvent(pos)) => {
                self.hovered = rect.contains(pos);
                self.mouse_pos = *pos;
                if let Some(ref mut child) = self.child {
                    return child.event(event, rect);
                }
                EventResponse::default()
            }
            _ => {
                if let Some(ref mut child) = self.child {
                    return child.event(event, rect);
                }
                EventResponse::default()
            }
        }
    }
#[cfg(feature = "a11y")]    fn access_info(&self) -> aurora_a11y::NodeInfo {        aurora_a11y::NodeInfo::new(aurora_a11y::accesskit::Role::Tooltip).with_label(self.text.clone())    }
}
