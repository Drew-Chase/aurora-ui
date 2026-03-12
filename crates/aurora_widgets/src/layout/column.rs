use crate::layout::{Align, Justify};
use crate::widgets::Widget;
use aurora_core::geometry::edges::Edges;
use aurora_core::geometry::rect::Rect;
use aurora_core::geometry::size::Size;
use aurora_render::canvas::Canvas;

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
    pub fn new() -> Self {
        Self::default()
    }

    pub fn child(mut self, widget: impl Widget + 'static) -> Self {
        self.children.push(Box::new(widget));
        self
    }

    pub fn spacing(mut self, spacing: f32) -> Self {
        self.spacing = spacing;
        self
    }

    pub fn padding(mut self, padding: impl Into<Edges>) -> Self {
        self.padding = padding.into();
        self
    }

    pub fn justify(mut self, justify: Justify) -> Self {
        self.justify = justify;
        self
    }

    pub fn align(mut self, align: Align) -> Self {
        self.align = align;
        self
    }
    pub fn width(mut self, width: u32) -> Self {
        self.width = Some(width);
        self
    }
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
    fn layout(&mut self, available: Size) -> Size {
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
            let size = child.layout(content_area);
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
                    let size = child.layout(flex_available);
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

        // Position each child
        self.child_rects.clear();
        for child_size in final_sizes.iter() {
            let x = self.padding.left
                + match self.align {
                    Align::Start => 0.0,
                    Align::Center => (content_width - child_size.width) / 2.0,
                    Align::End => content_width - child_size.width,
                    Align::Stretch => 0.0,
                };

            let w = if self.align == Align::Stretch {
                content_width
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
            None => available.height,
        };
        Size::new(final_width, final_height)
    }

    fn paint(&self, canvas: &mut Canvas, rect: Rect) {
        for (child, child_rect) in self.children.iter().zip(self.child_rects.iter()) {
            let translated = child_rect.translate(&rect.origin());
            child.paint(canvas, translated);
        }
    }

    fn children(&self) -> &[Box<dyn Widget>] {
        &self.children
    }
}

#[macro_export]
macro_rules! column {
    () => {{
        use aurora_ui::aurora_widgets::layout::column::Column;
        Column::new()
    }};
}
