use crate::layout::{Align, Justify};
use crate::widgets::{EventResponse, LayoutCtx, Widget};
use aurora_core::geometry::edges::Edges;
use aurora_core::geometry::rect::Rect;
use aurora_core::geometry::size::Size;
use aurora_core::kmi::cursor_icon::CursorIcon;
use aurora_core::kmi::mouse::MouseEvent;
use aurora_core::kmi::WidgetEvent;
use aurora_render::canvas::Canvas;

/// A horizontal layout container that arranges children left-to-right.
///
/// Uses a two-pass flex algorithm: children with a fixed size are measured
/// first, then remaining space is divided equally among flexible children
/// (those that consume all available width).
///
/// # Example
///
/// ```no_run
/// use aurora_ui::prelude::*;
///
/// row!()
///     .spacing(10.0)
///     .height(100)
///     .child(BoxWidget::new().background_color(Color::RED))
///     .child(BoxWidget::new().width(80).background_color(Color::BLUE))
/// ```
pub struct Row {
    justify: Justify,
    align: Align,
    spacing: f32,
    padding: Edges,
    children: Vec<Box<dyn Widget>>,
    child_rects: Vec<Rect>,
    width: Option<u32>,
    height: Option<u32>,
}

impl Row {
    /// Creates a new row with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Appends a child widget to the row.
    pub fn child(mut self, widget: impl Widget + 'static) -> Self {
        self.children.push(Box::new(widget));
        self
    }

    /// Sets the gap between children in pixels.
    pub fn spacing(mut self, spacing: f32) -> Self {
        self.spacing = spacing;
        self
    }

    /// Sets the inner padding around the content area.
    pub fn padding(mut self, padding: impl Into<Edges>) -> Self {
        self.padding = padding.into();
        self
    }

    /// Sets how children are distributed along the horizontal (main) axis.
    pub fn justify(mut self, justify: Justify) -> Self {
        self.justify = justify;
        self
    }

    /// Sets how children are aligned along the vertical (cross) axis.
    pub fn align(mut self, align: Align) -> Self {
        self.align = align;
        self
    }

    /// Sets a fixed width for the row, overriding the available width.
    pub fn width(mut self, width: u32) -> Self {
        self.width = Some(width);
        self
    }

    /// Sets a fixed height for the row, overriding the available height.
    pub fn height(mut self, height: u32) -> Self {
        self.height = Some(height);
        self
    }
}

impl Default for Row {
    fn default() -> Self {
        Self {
            justify: Justify::Start,
            align: Align::Start,
            spacing: 0.0,
            padding: Edges::zero(),
            children: vec![],
            child_rects: vec![],
            width: None,
            height: None,
        }
    }
}

impl Widget for Row {
    fn layout(&mut self, available: Size, ctx: &mut LayoutCtx) -> Size {
        let total_width = match self.width {
            Some(w) => w as f32,
            None => available.width,
        };
        let total_height = match self.height {
            Some(h) => h as f32,
            None => available.height,
        };
        let content_width = total_width - self.padding.horizontal();
        let content_height = total_height - self.padding.vertical();
        let content_area = Size::new(content_width.max(0.0), content_height.max(0.0));

        // First pass: measure fixed children, count flexible ones
        let mut fixed_total = 0.0;
        let mut flex_count = 0;
        let mut child_sizes: Vec<Option<Size>> = Vec::with_capacity(self.children.len());

        for child in &mut self.children {
            let size = child.layout(content_area, ctx);
            // If the child used the full available width, it's flexible
            if size.width >= content_area.width {
                child_sizes.push(None);
                flex_count += 1;
            } else {
                fixed_total += size.width;
                child_sizes.push(Some(size));
            }
        }

        // Calculate remaining space for flexible children
        let total_spacing = if self.children.len() > 1 {
            self.spacing * (self.children.len() - 1) as f32
        } else {
            0.0
        };
        let remaining = (content_width - fixed_total - total_spacing).max(0.0);
        let flex_width = if flex_count > 0 {
            remaining / flex_count as f32
        } else {
            0.0
        };

        // Second pass: fill in flexible children with their share
        let mut final_sizes: Vec<Size> = Vec::with_capacity(self.children.len());
        for (i, child) in self.children.iter_mut().enumerate() {
            match child_sizes[i] {
                Some(size) => final_sizes.push(size),
                None => {
                    let flex_available = Size::new(flex_width, content_area.height);
                    let size = child.layout(flex_available, ctx);
                    final_sizes.push(size);
                }
            }
        }

        // Total width of all children + spacing
        let total_child_width: f32 = final_sizes.iter().map(|s| s.width).sum();
        let total_width = total_child_width + total_spacing;
        let leftover = (content_width - total_width).max(0.0);

        // Starting x offset based on justify
        let mut x = self.padding.left
            + match self.justify {
                Justify::Start => 0.0,
                Justify::Center => leftover / 2.0,
                Justify::End => leftover,
                Justify::SpaceBetween => 0.0,
            };

        // Spacing override for SpaceBetween
        let actual_spacing = if self.justify == Justify::SpaceBetween && self.children.len() > 1 {
            (content_width - total_child_width).max(0.0) / (self.children.len() - 1) as f32
        } else {
            self.spacing
        };

        // For alignment, use actual max child height when no explicit height is set
        let align_height = if self.height.is_some() {
            content_height
        } else {
            final_sizes.iter().map(|s| s.height).fold(0.0f32, f32::max)
        };

        // Position each child
        self.child_rects.clear();
        for child_size in final_sizes.iter() {
            let y = self.padding.top
                + match self.align {
                    Align::Start => 0.0,
                    Align::Center => (align_height - child_size.height).max(0.0) / 2.0,
                    Align::End => (align_height - child_size.height).max(0.0),
                    Align::Stretch => 0.0,
                };

            let h = if self.align == Align::Stretch {
                align_height
            } else {
                child_size.height
            };

            self.child_rects
                .push(Rect::new(x, y, x + child_size.width, y + h));
            x += child_size.width + actual_spacing;
        }

        // Return the row's total size
        let final_width = match self.width {
            Some(w) => w as f32,
            None => total_child_width + total_spacing + self.padding.horizontal(),
        };
        let max_child_height = final_sizes.iter().map(|s| s.height).fold(0.0f32, f32::max);
        let final_height = match self.height {
            Some(h) => h as f32,
            None => max_child_height + self.padding.vertical(),
        };
        Size::new(final_width, final_height)
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

    fn event(&mut self, event: &WidgetEvent, rect: Rect) -> EventResponse {
        let mouse = match event {
            WidgetEvent::Mouse(e) => e,
            _ => {
                let mut result = EventResponse::default();
                for (child, child_rect) in self.children.iter_mut().zip(self.child_rects.iter()) {
                    let translated = child_rect.translate(&rect.origin());
                    let response = child.event(event, translated);
                    if response.handled {
                        result.handled = true;
                    }
                    if response.focus_next {
                        result.focus_next = true;
                    }
                    if response.focus_prev {
                        result.focus_prev = true;
                    }
                    if response.request_focus.is_some() {
                        result.request_focus = response.request_focus;
                    }
                }
                return result;
            }
        };
        let pos = match mouse {
            MouseEvent::MouseMoveEvent(p) => *p,
            MouseEvent::MouseClickEvent(c) => c.position,
            MouseEvent::MouseScrollEvent(_) => {
                // Forward scroll to all children so ScrollViews can handle it
                for (child, child_rect) in self.children.iter_mut().zip(self.child_rects.iter()) {
                    let translated = child_rect.translate(&rect.origin());
                    let response = child.event(event, translated);
                    if response.handled {
                        return response;
                    }
                }
                return EventResponse::default();
            }
        };

        let is_move = matches!(mouse, MouseEvent::MouseMoveEvent(_));
        let mut handled = false;
        let mut cursor: Option<CursorIcon> = None;

        for (child, child_rect) in self.children.iter_mut().zip(self.child_rects.iter()) {
            let translated = child_rect.translate(&rect.origin());
            let response = child.event(event, translated);
            if is_move {
                if response.handled && !translated.contains(&pos) {
                    return EventResponse {
                        handled: true,
                        cursor: response.cursor,
                        request_focus: response.request_focus,
                        focus_next: response.focus_next,
                        focus_prev: response.focus_prev,
                    };
                }
                if response.handled {
                    handled = true;
                }
                if translated.contains(&pos) {
                    cursor = response.cursor;
                }
            } else if response.handled {
                return EventResponse {
                    handled: true,
                    cursor: response.cursor,
                    request_focus: response.request_focus,
                    focus_next: response.focus_next,
                    focus_prev: response.focus_prev,
                };
            }
        }
        EventResponse {
            handled,
            cursor,
            ..Default::default()
        }
    }
}
/// Shorthand for [`Row::new()`].
///
/// ```no_run
/// # use aurora_ui::prelude::*;
/// row!()
///     .spacing(10.0)
///     .height(100)
///     .child(BoxWidget::new().background_color(Color::RED))
/// ```
#[macro_export]
macro_rules! row {
    () => {{
        Row::new()
    }};
}

#[cfg(all(test, not(feature = "text")))]
mod tests {
    use super::*;
    use crate::layout::{Align, Justify};

    /// A minimal widget that returns a fixed size from layout.
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
    fn empty_row() {
        let mut row = Row::new();
        let size = row.layout(available(), &mut LayoutCtx);
        assert_eq!(size.width, 0.0);
        assert_eq!(size.height, 0.0);
    }

    #[test]
    fn single_fixed_child() {
        let mut row = Row::new().child(FixedWidget::new(50.0, 30.0));
        let size = row.layout(available(), &mut LayoutCtx);
        assert_eq!(size.width, 50.0);
        assert_eq!(size.height, 30.0);
    }

    #[test]
    fn two_children_no_spacing() {
        let mut row = Row::new()
            .child(FixedWidget::new(50.0, 30.0))
            .child(FixedWidget::new(70.0, 40.0));
        let size = row.layout(available(), &mut LayoutCtx);
        assert_eq!(size.width, 120.0); // 50 + 70
        assert_eq!(size.height, 40.0); // max height
    }

    #[test]
    fn children_with_spacing() {
        let mut row = Row::new()
            .spacing(10.0)
            .child(FixedWidget::new(50.0, 30.0))
            .child(FixedWidget::new(70.0, 30.0));
        let size = row.layout(available(), &mut LayoutCtx);
        assert_eq!(size.width, 130.0); // 50 + 10 + 70
    }

    #[test]
    fn padding_adds_to_size() {
        let mut row = Row::new()
            .padding(Edges::all(10.0))
            .child(FixedWidget::new(50.0, 30.0));
        let size = row.layout(available(), &mut LayoutCtx);
        assert_eq!(size.width, 70.0); // 50 + 10 + 10
        assert_eq!(size.height, 50.0); // 30 + 10 + 10
    }

    #[test]
    fn padding_offsets_child_rects() {
        let mut row = Row::new()
            .padding(Edges::new(5.0, 10.0, 15.0, 20.0))
            .child(FixedWidget::new(50.0, 30.0));
        row.layout(available(), &mut LayoutCtx);
        let r = &row.child_rects[0];
        assert_eq!(r.x1, 20.0); // left padding
        assert_eq!(r.y1, 5.0); // top padding
    }

    #[test]
    fn justify_start() {
        let mut row = Row::new()
            .width(200)
            .justify(Justify::Start)
            .child(FixedWidget::new(50.0, 30.0));
        row.layout(available(), &mut LayoutCtx);
        assert_eq!(row.child_rects[0].x1, 0.0);
    }

    #[test]
    fn justify_center() {
        let mut row = Row::new()
            .width(200)
            .justify(Justify::Center)
            .child(FixedWidget::new(50.0, 30.0));
        row.layout(available(), &mut LayoutCtx);
        // leftover = 200 - 50 = 150, offset = 75
        assert_eq!(row.child_rects[0].x1, 75.0);
    }

    #[test]
    fn justify_end() {
        let mut row = Row::new()
            .width(200)
            .justify(Justify::End)
            .child(FixedWidget::new(50.0, 30.0));
        row.layout(available(), &mut LayoutCtx);
        assert_eq!(row.child_rects[0].x1, 150.0);
    }

    #[test]
    fn justify_space_between() {
        let mut row = Row::new()
            .width(200)
            .justify(Justify::SpaceBetween)
            .child(FixedWidget::new(50.0, 30.0))
            .child(FixedWidget::new(50.0, 30.0));
        row.layout(available(), &mut LayoutCtx);
        assert_eq!(row.child_rects[0].x1, 0.0);
        // Second child: spacing = (200 - 100) / 1 = 100, so x = 50 + 100 = 150
        assert_eq!(row.child_rects[1].x1, 150.0);
    }

    #[test]
    fn align_start() {
        let mut row = Row::new()
            .height(100)
            .align(Align::Start)
            .child(FixedWidget::new(50.0, 30.0));
        row.layout(available(), &mut LayoutCtx);
        assert_eq!(row.child_rects[0].y1, 0.0);
    }

    #[test]
    fn align_center() {
        let mut row = Row::new()
            .height(100)
            .align(Align::Center)
            .child(FixedWidget::new(50.0, 30.0));
        row.layout(available(), &mut LayoutCtx);
        // (100 - 30) / 2 = 35
        assert_eq!(row.child_rects[0].y1, 35.0);
    }

    #[test]
    fn align_end() {
        let mut row = Row::new()
            .height(100)
            .align(Align::End)
            .child(FixedWidget::new(50.0, 30.0));
        row.layout(available(), &mut LayoutCtx);
        assert_eq!(row.child_rects[0].y1, 70.0);
    }

    #[test]
    fn align_stretch() {
        let mut row = Row::new()
            .height(100)
            .align(Align::Stretch)
            .child(FixedWidget::new(50.0, 30.0));
        row.layout(available(), &mut LayoutCtx);
        // Stretch: child rect height should be full content height
        let r = &row.child_rects[0];
        assert_eq!(r.y2 - r.y1, 100.0);
    }

    #[test]
    fn explicit_width_height() {
        let mut row = Row::new()
            .width(300)
            .height(200)
            .child(FixedWidget::new(50.0, 30.0));
        let size = row.layout(available(), &mut LayoutCtx);
        assert_eq!(size.width, 300.0);
        assert_eq!(size.height, 200.0);
    }

    #[test]
    fn children_returns_slice() {
        let row = Row::new()
            .child(FixedWidget::new(10.0, 10.0))
            .child(FixedWidget::new(20.0, 20.0));
        assert_eq!(row.children().len(), 2);
    }
}
