use crate::layout::{Align, Justify};
use crate::widgets::{EventResponse, LayoutCtx, Widget};
use aurora_core::geometry::edges::Edges;
use aurora_core::geometry::rect::Rect;
use aurora_core::geometry::size::Size;
use aurora_core::kmi::cursor_icon::CursorIcon;
use aurora_core::kmi::mouse::MouseEvent;
use aurora_core::kmi::WidgetEvent;
use aurora_render::canvas::Canvas;

/// A vertical layout container that arranges children top-to-bottom.
///
/// Uses a two-pass flex algorithm: children with a fixed size are measured
/// first, then remaining space is divided equally among flexible children
/// (those that consume all available height).
///
/// # Example
///
/// ```no_run
/// use aurora_ui::prelude::*;
///
/// col!()
///     .spacing(10.0)
///     .padding(Edges::all(20.0))
///     .justify(Justify::Center)
///     .align(Align::Stretch)
///     .child(BoxWidget::new().height(50).background_color(Color::RED))
///     .child(BoxWidget::new().background_color(Color::BLUE))
/// ```
pub struct Column {
    justify: Justify,
    align: Align,
    spacing: f32,
    padding: Edges,
    children: Vec<Box<dyn Widget>>,
    child_rects: Vec<Rect>,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

impl Column {
    /// Creates a new column with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Appends a child widget to the column.
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

    /// Sets how children are distributed along the vertical (main) axis.
    pub fn justify(mut self, justify: Justify) -> Self {
        self.justify = justify;
        self
    }

    /// Sets how children are aligned along the horizontal (cross) axis.
    pub fn align(mut self, align: Align) -> Self {
        self.align = align;
        self
    }

    /// Sets a fixed width for the column, overriding the available width.
    pub fn width(mut self, width: u32) -> Self {
        self.width = Some(width);
        self
    }

    /// Sets a fixed height for the column, overriding the available height.
    pub fn height(mut self, height: u32) -> Self {
        self.height = Some(height);
        self
    }
}

impl Default for Column {
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

impl Widget for Column {
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
            // If the child used the full available height, it's flexible
            if size.height >= content_area.height {
                child_sizes.push(None);
                flex_count += 1;
            } else {
                fixed_total += size.height;
                child_sizes.push(Some(size));
            }
        }

        // Calculate remaining space for flexible children
        let total_spacing = if self.children.len() > 1 {
            self.spacing * (self.children.len() - 1) as f32
        } else {
            0.0
        };
        let remaining = (content_height - fixed_total - total_spacing).max(0.0);
        let flex_height = if flex_count > 0 {
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
                    let flex_available = Size::new(content_area.width, flex_height);
                    let size = child.layout(flex_available, ctx);
                    final_sizes.push(size);
                }
            }
        }

        // Total height of all children + spacing
        let total_child_height: f32 = final_sizes.iter().map(|s| s.height).sum();
        let total_height = total_child_height + total_spacing;
        let leftover = (content_height - total_height).max(0.0);

        // Starting y offset based on justify
        let mut y = self.padding.top
            + match self.justify {
                Justify::Start => 0.0,
                Justify::Center => leftover / 2.0,
                Justify::End => leftover,
                Justify::SpaceBetween => 0.0,
            };

        // Spacing override for SpaceBetween
        let actual_spacing = if self.justify == Justify::SpaceBetween && self.children.len() > 1 {
            leftover / (self.children.len() - 1) as f32
        } else {
            self.spacing
        };

        // For alignment, use actual max child width when no explicit width is set
        let align_width = if self.width.is_some() {
            content_width
        } else {
            final_sizes.iter().map(|s| s.width).fold(0.0f32, f32::max)
        };

        // Position each child
        self.child_rects.clear();
        for child_size in final_sizes.iter() {
            let x = self.padding.left
                + match self.align {
                    Align::Start => 0.0,
                    Align::Center => (align_width - child_size.width).max(0.0) / 2.0,
                    Align::End => (align_width - child_size.width).max(0.0),
                    Align::Stretch => 0.0,
                };

            let w = if self.align == Align::Stretch {
                align_width
            } else {
                child_size.width
            };

            self.child_rects
                .push(Rect::new(x, y, x + w, y + child_size.height));
            y += child_size.height + actual_spacing;
        }

        // Return the column's total size
        let final_width = match self.width {
            Some(w) => w as f32,
            None => available.width,
        };
        let final_height = match self.height {
            Some(h) => h as f32,
            None => total_child_height + total_spacing + self.padding.vertical(),
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
                // Forward non-mouse events to all children, propagate focus responses
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
                // Overlay handling: a child handled a mouse move outside its
                // layout rect (e.g. a dropdown). Stop propagation so widgets
                // behind the overlay don't receive hover events.
                if response.handled && !translated.contains(&pos) {
                    return EventResponse {
                        handled: true,
                        cursor: response.cursor,
                        ..Default::default()
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
                    ..Default::default()
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

/// Shorthand for [`Column::new()`].
///
/// ```no_run
/// # use aurora_ui::prelude::*;
/// col!()
///     .spacing(10.0)
///     .child(BoxWidget::new().height(50).background_color(Color::RED))
/// ```
#[macro_export]
macro_rules! col {
    () => {{
        Column::new()
    }};
}
