use crate::widgets::{EventResponse, LayoutCtx, Widget};
use aurora_core::geometry::edges::Edges;
use aurora_core::geometry::rect::Rect;
use aurora_core::geometry::size::Size;
use aurora_core::kmi::WidgetEvent;
use aurora_render::canvas::Canvas;

/// An overlay container that layers children on top of each other.
///
/// All children share the same bounds (the Stack's content area minus
/// padding). Children paint in insertion order — the last child added
/// renders on top.
///
/// Returns [`Size::zero()`] if all children are zero-sized (e.g. all
/// absolutely/fixed positioned) and no explicit width/height is set,
/// preventing empty stacks from consuming layout space.
///
/// # Example
///
/// ```ignore
/// use aurora_ui::prelude::*;
/// use aurora_ui::aurora_widgets::layout::stack::Stack;
/// use aurora_ui::aurora_widgets::layout::position::Positioned;
///
/// Stack::new()
///     .child(BoxWidget::new().background_color(Color::RED))
///     .child(
///         Positioned::fixed((20.0, 20.0))
///             .child(BoxWidget::new().width(50).height(50).background_color(Color::BLUE))
///     )
/// ```
#[derive(Default)]
pub struct Stack {
    children: Vec<Box<dyn Widget>>,
    child_rects: Vec<Rect>,
    padding: Edges,
    width: Option<f32>,
    height: Option<f32>,
}

impl Stack {
    /// Creates a new empty stack.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a child layer. The last child added paints on top.
    pub fn child(mut self, child: impl Widget + 'static) -> Self {
        self.children.push(Box::new(child));
        self
    }

    /// Sets the inner padding around the content area.
    pub fn padding(mut self, padding: Edges) -> Self {
        self.padding = padding;
        self
    }

    /// Sets a fixed width, overriding the available width.
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    /// Sets a fixed height, overriding the available height.
    pub fn height(mut self, height: f32) -> Self {
        self.height = Some(height);
        self
    }
}

impl Widget for Stack {
    fn layout(&mut self, available: Size, ctx: &mut LayoutCtx) -> Size {
        let width = self.width.unwrap_or(available.width);
        let height = self.height.unwrap_or(available.height);
        let content = Size::new(
            (width - self.padding.horizontal()).max(0.0),
            (height - self.padding.vertical()).max(0.0),
        );

        self.child_rects.clear();
        let mut has_flow_children = false;

        for child in &mut self.children {
            let child_size = child.layout(content, ctx);
            self.child_rects.push(Rect::new(
                self.padding.left,
                self.padding.top,
                self.padding.left + child_size.width,
                self.padding.top + child_size.height,
            ));
            if child_size.width > 0.0 || child_size.height > 0.0 {
                has_flow_children = true;
            }
        }

        if has_flow_children || self.width.is_some() || self.height.is_some() {
            Size::new(width, height)
        } else {
            Size::zero()
        }
    }

    fn paint(&self, canvas: &mut Canvas, rect: Rect) {
        for (child, child_rect) in self.children.iter().zip(self.child_rects.iter()) {
            let translated = child_rect.translate(&rect.origin());
            child.paint(canvas, translated);
        }
    }

    fn paint_overlay(&self, canvas: &mut Canvas, rect: Rect) {
        for (child, child_rect) in self.children.iter().zip(self.child_rects.iter()) {
            let translated = child_rect.translate(&rect.origin());
            child.paint_overlay(canvas, translated);
        }
    }

    fn children(&self) -> &[Box<dyn Widget>] {
        &self.children
    }

    fn event_overlay(&mut self, event: &WidgetEvent, rect: Rect) -> EventResponse {
        // Reverse order: last child = highest z-order = first to receive overlay events
        for (child, child_rect) in self.children.iter_mut().zip(self.child_rects.iter()).rev() {
            let translated = child_rect.translate(&rect.origin());
            let response = child.event_overlay(event, translated);
            if response.status.stops_propagation() {
                return response;
            }
        }
        EventResponse::default()
    }

    fn event(&mut self, _event: &WidgetEvent, _rect: Rect) -> EventResponse {
        for child in &mut self.children {
            let response = child.event(_event, _rect);
            if response.status.stops_propagation() {
                return response;
            }
        }
        EventResponse::default()
    }
}

#[cfg(all(test, not(feature = "text")))]
mod tests {
    use super::*;

    struct FixedWidget {
        size: Size,
    }

    impl FixedWidget {
        fn new(w: f32, h: f32) -> Self {
            Self {
                size: Size::new(w, h),
            }
        }
    }

    impl Widget for FixedWidget {
        fn layout(&mut self, _available: Size, _ctx: &mut LayoutCtx) -> Size {
            self.size
        }
        fn paint(&self, _canvas: &mut Canvas, _rect: Rect) {}
        fn children(&self) -> &[Box<dyn Widget>] {
            &[]
        }
    }

    fn available() -> Size {
        Size::new(400.0, 300.0)
    }

    #[test]
    fn empty_stack_zero_size() {
        let mut stack = Stack::new();
        let size = stack.layout(available(), &mut LayoutCtx);
        assert_eq!(size, Size::zero());
    }

    #[test]
    fn stack_uses_available_size() {
        let mut stack = Stack::new().child(FixedWidget::new(50.0, 30.0));
        let size = stack.layout(available(), &mut LayoutCtx);
        assert_eq!(size.width, 400.0);
        assert_eq!(size.height, 300.0);
    }

    #[test]
    fn children_share_same_origin() {
        let mut stack = Stack::new()
            .child(FixedWidget::new(50.0, 30.0))
            .child(FixedWidget::new(70.0, 40.0));
        stack.layout(available(), &mut LayoutCtx);
        // Both children start at (0, 0) (no padding)
        assert_eq!(stack.child_rects[0].x1, 0.0);
        assert_eq!(stack.child_rects[0].y1, 0.0);
        assert_eq!(stack.child_rects[1].x1, 0.0);
        assert_eq!(stack.child_rects[1].y1, 0.0);
    }

    #[test]
    fn padding_offsets_children() {
        let mut stack = Stack::new()
            .padding(Edges::new(5.0, 10.0, 15.0, 20.0))
            .child(FixedWidget::new(50.0, 30.0));
        stack.layout(available(), &mut LayoutCtx);
        assert_eq!(stack.child_rects[0].x1, 20.0); // left padding
        assert_eq!(stack.child_rects[0].y1, 5.0); // top padding
    }

    #[test]
    fn explicit_width_height() {
        let mut stack = Stack::new()
            .width(200.0)
            .height(100.0)
            .child(FixedWidget::new(50.0, 30.0));
        let size = stack.layout(available(), &mut LayoutCtx);
        assert_eq!(size.width, 200.0);
        assert_eq!(size.height, 100.0);
    }

    #[test]
    fn children_returns_slice() {
        let stack = Stack::new()
            .child(FixedWidget::new(10.0, 10.0))
            .child(FixedWidget::new(20.0, 20.0));
        assert_eq!(stack.children().len(), 2);
    }
}
