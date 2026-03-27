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
/// ```no_run
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

    fn event(&mut self, _event: &WidgetEvent, _rect: Rect) -> EventResponse {
        for child in &mut self.children {
            let response = child.event(_event, _rect);
            if response.handled {
                return response;
            }
        }
        EventResponse::default()
    }
}
