use crate::widgets::{EventResponse, LayoutCtx, Widget};
use aurora_core::color::Color;
use aurora_core::geometry::corners::Corners;
use aurora_core::geometry::edges::Edges;
use aurora_core::geometry::rect::Rect;
use aurora_core::geometry::size::Size;
use aurora_core::kmi::WidgetEvent;
use aurora_render::canvas::Canvas;

use super::colors;

/// An input with optional prefix and suffix elements (wraps a child input widget).
///
/// # Example
/// ```ignore
/// InputGroup::new(TextInput::new().placeholder("Search..."))
///     .prefix(Label::new("$"))
///     .suffix(Label::new(".00"))
/// ```
pub struct InputGroup {
    input: Box<dyn Widget>,
    prefix: Option<Box<dyn Widget>>,
    suffix: Option<Box<dyn Widget>>,
    border_color: Color,
    background: Color,
    corners: Corners,
    height: Option<f32>,
    width: Option<f32>,
    spacing: f32,
    padding: Edges,
    input_size: Size,
    prefix_size: Size,
    suffix_size: Size,
}

impl InputGroup {
    pub fn new(input: impl Widget + 'static) -> Self {
        Self {
            input: Box::new(input),
            prefix: None,
            suffix: None,
            border_color: colors::input_border(),
            background: colors::muted(),
            corners: Corners::all(6.0),
            height: Some(40.0),
            width: None,
            spacing: 0.0,
            padding: Edges::new(0.0, 12.0, 0.0, 12.0),
            input_size: Size::default(),
            prefix_size: Size::default(),
            suffix_size: Size::default(),
        }
    }

    pub fn prefix(mut self, widget: impl Widget + 'static) -> Self {
        self.prefix = Some(Box::new(widget));
        self
    }

    pub fn suffix(mut self, widget: impl Widget + 'static) -> Self {
        self.suffix = Some(Box::new(widget));
        self
    }

    pub fn border_color(mut self, color: Color) -> Self {
        self.border_color = color;
        self
    }

    pub fn background(mut self, color: Color) -> Self {
        self.background = color;
        self
    }

    pub fn corners(mut self, corners: Corners) -> Self {
        self.corners = corners;
        self
    }

    pub fn height(mut self, height: impl Into<f32>) -> Self {
        self.height = Some(height.into());
        self
    }

    pub fn width(mut self, width: impl Into<f32>) -> Self {
        self.width = Some(width.into());
        self
    }

    pub fn spacing(mut self, spacing: f32) -> Self {
        self.spacing = spacing;
        self
    }
}

impl Widget for InputGroup {
    fn layout(&mut self, available: Size, ctx: &mut LayoutCtx) -> Size {
        let w = self.width.unwrap_or(available.width);
        let h = self.height.unwrap_or(40.0);
        let inner_h = h;
        let mut used_w = self.padding.left + self.padding.right;

        if let Some(ref mut prefix) = self.prefix {
            let prefix_available = Size::new(w * 0.3, inner_h);
            self.prefix_size = prefix.layout(prefix_available, ctx);
            used_w += self.prefix_size.width + self.spacing;
        }

        if let Some(ref mut suffix) = self.suffix {
            let suffix_available = Size::new(w * 0.3, inner_h);
            self.suffix_size = suffix.layout(suffix_available, ctx);
            used_w += self.suffix_size.width + self.spacing;
        }

        let input_w = (w - used_w).max(0.0);
        let input_available = Size::new(input_w, inner_h);
        self.input_size = self.input.layout(input_available, ctx);

        Size::new(w, h)
    }

    fn paint(&self, canvas: &mut Canvas, rect: Rect) {
        // Background and border
        canvas.fill_rounded_rect(rect, self.corners, self.background);
        canvas.stroke_rounded_rect(rect, self.corners, 1, self.border_color);

        let mut x = rect.x1 + self.padding.left;
        let cy = rect.y1 + rect.height() / 2.0;

        // Prefix
        if let Some(ref prefix) = self.prefix {
            let py = cy - self.prefix_size.height / 2.0;
            let prefix_rect = Rect::new(
                x,
                py,
                x + self.prefix_size.width,
                py + self.prefix_size.height,
            );
            prefix.paint(canvas, prefix_rect);
            x += self.prefix_size.width + self.spacing;

            // Separator
            canvas.fill_rect(
                Rect::new(
                    x - self.spacing / 2.0 - 0.5,
                    rect.y1 + 4.0,
                    x - self.spacing / 2.0 + 0.5,
                    rect.y2 - 4.0,
                ),
                colors::border(),
            );
        }

        // Input
        let iy = cy - self.input_size.height / 2.0;
        let input_rect = Rect::new(
            x,
            iy,
            x + self.input_size.width,
            iy + self.input_size.height,
        );
        self.input.paint(canvas, input_rect);
        x += self.input_size.width + self.spacing;

        // Suffix
        if let Some(ref suffix) = self.suffix {
            // Separator
            canvas.fill_rect(
                Rect::new(
                    x - self.spacing / 2.0 - 0.5,
                    rect.y1 + 4.0,
                    x - self.spacing / 2.0 + 0.5,
                    rect.y2 - 4.0,
                ),
                colors::border(),
            );

            let sy = cy - self.suffix_size.height / 2.0;
            let suffix_rect = Rect::new(
                x,
                sy,
                x + self.suffix_size.width,
                sy + self.suffix_size.height,
            );
            suffix.paint(canvas, suffix_rect);
        }
    }

    fn children(&self) -> &[Box<dyn Widget>] {
        std::slice::from_ref(&self.input)
    }

    fn event(&mut self, event: &WidgetEvent, rect: Rect) -> EventResponse {
        let mut x = rect.x1 + self.padding.left;
        let cy = rect.y1 + rect.height() / 2.0;

        if let Some(ref mut prefix) = self.prefix {
            let py = cy - self.prefix_size.height / 2.0;
            let prefix_rect = Rect::new(
                x,
                py,
                x + self.prefix_size.width,
                py + self.prefix_size.height,
            );
            let resp = prefix.event(event, prefix_rect);
            if resp.status.stops_propagation() {
                return resp;
            }
            x += self.prefix_size.width + self.spacing;
        }

        let iy = cy - self.input_size.height / 2.0;
        let input_rect = Rect::new(
            x,
            iy,
            x + self.input_size.width,
            iy + self.input_size.height,
        );
        let resp = self.input.event(event, input_rect);
        if resp.status.stops_propagation() {
            return resp;
        }
        x += self.input_size.width + self.spacing;

        if let Some(ref mut suffix) = self.suffix {
            let sy = cy - self.suffix_size.height / 2.0;
            let suffix_rect = Rect::new(
                x,
                sy,
                x + self.suffix_size.width,
                sy + self.suffix_size.height,
            );
            let resp = suffix.event(event, suffix_rect);
            if resp.status.stops_propagation() {
                return resp;
            }
        }

        EventResponse::default()
    }
    #[cfg(feature = "a11y")]
    fn access_info(&self) -> aurora_a11y::NodeInfo {
        aurora_a11y::NodeInfo::new(aurora_a11y::accesskit::Role::Group)
    }
}
