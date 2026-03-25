use crate::widgets::{EventResponse, LayoutCtx, Widget};
use aurora_core::geometry::rect::Rect;
use aurora_core::geometry::size::Size;
use aurora_core::kmi::WidgetEvent;
use aurora_render::canvas::Canvas;

use super::colors;

/// A form field with a label, input widget, and optional error/description message.
///
/// # Example
/// ```ignore
/// Field::new("Email")
///     .input(TextInput::new().placeholder("you@example.com"))
///     .description("We'll never share your email.")
///     .error("Invalid email address")
/// ```
pub struct Field {
    label: String,
    description: Option<String>,
    error: Option<String>,
    input: Option<Box<dyn Widget>>,
    required: bool,
    label_spacing: f32,
    input_spacing: f32,
    width: Option<f32>,
    label_layout: Option<aurora_text::text_layout::TextLayout>,
    desc_layout: Option<aurora_text::text_layout::TextLayout>,
    error_layout: Option<aurora_text::text_layout::TextLayout>,
    label_size: Size,
    input_size: Size,
    desc_size: Size,
    error_size: Size,
}

impl Field {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            description: None,
            error: None,
            input: None,
            required: false,
            label_spacing: 6.0,
            input_spacing: 6.0,
            width: None,
            label_layout: None,
            desc_layout: None,
            error_layout: None,
            label_size: Size::default(),
            input_size: Size::default(),
            desc_size: Size::default(),
            error_size: Size::default(),
        }
    }

    pub fn input(mut self, widget: impl Widget + 'static) -> Self {
        self.input = Some(Box::new(widget));
        self
    }

    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    pub fn error(mut self, error: impl Into<String>) -> Self {
        self.error = Some(error.into());
        self
    }

    pub fn required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }

    pub fn width(mut self, width: impl Into<f32>) -> Self {
        self.width = Some(width.into());
        self
    }
}

impl Widget for Field {
    fn layout(&mut self, available: Size, ctx: &mut LayoutCtx) -> Size {
        let w = self.width.unwrap_or(available.width);
        let mut total_h = 0.0;

        // Label
        let label_text = if self.required {
            format!("{} *", self.label)
        } else {
            self.label.clone()
        };
        let mut opts = ctx.font_options.clone();
        opts.size = Some(14.0);
        opts.weight = Some(aurora_text::font_options::FontWeight::Medium);
        let mut tl = aurora_text::text_layout::TextLayout::new(ctx.font_manager, &label_text, &opts, colors::FOREGROUND, None);
        tl.set_max_width(ctx.font_manager, w);
        let _s = tl.size(); let lw = _s.width; let lh = _s.height;
        self.label_size = Size::new(lw, lh);
        self.label_layout = Some(tl);
        total_h += lh + self.label_spacing;

        // Input
        if let Some(ref mut input) = self.input {
            let input_available = Size::new(w, f32::MAX);
            self.input_size = input.layout(input_available, ctx);
            total_h += self.input_size.height;
        }

        // Description
        if let Some(ref desc) = self.description {
            let mut opts = ctx.font_options.clone();
            opts.size = Some(12.0);
            opts.weight = Some(aurora_text::font_options::FontWeight::Normal);
            let mut tl = aurora_text::text_layout::TextLayout::new(ctx.font_manager, desc, &opts, colors::FOREGROUND, None);
            tl.set_max_width(ctx.font_manager, w);
            let _s = tl.size(); let dw = _s.width; let dh = _s.height;
            self.desc_size = Size::new(dw, dh);
            self.desc_layout = Some(tl);
            total_h += self.input_spacing + dh;
        }

        // Error
        if let Some(ref error) = self.error {
            let mut opts = ctx.font_options.clone();
            opts.size = Some(12.0);
            opts.weight = Some(aurora_text::font_options::FontWeight::Normal);
            let mut tl = aurora_text::text_layout::TextLayout::new(ctx.font_manager, error, &opts, colors::FOREGROUND, None);
            tl.set_max_width(ctx.font_manager, w);
            let _s = tl.size(); let ew = _s.width; let eh = _s.height;
            self.error_size = Size::new(ew, eh);
            self.error_layout = Some(tl);
            total_h += self.input_spacing + eh;
        }

        Size::new(w, total_h)
    }

    fn paint(&self, canvas: &mut Canvas, rect: Rect) {
        let mut y = rect.y1;

        // Label
        if let Some(ref tl) = self.label_layout {
            canvas.draw_text(tl, rect.x1 as i32, y as i32);
            y += self.label_size.height + self.label_spacing;
        }

        // Input
        if let Some(ref input) = self.input {
            let input_rect = Rect::new(
                rect.x1,
                y,
                rect.x1 + self.input_size.width,
                y + self.input_size.height,
            );
            input.paint(canvas, input_rect);
            y += self.input_size.height;
        }

        // Description
        if let Some(ref tl) = self.desc_layout {
            y += self.input_spacing;
            canvas.draw_text(tl, rect.x1 as i32, y as i32);
            y += self.desc_size.height;
        }

        // Error
        if let Some(ref tl) = self.error_layout {
            y += self.input_spacing;
            canvas.draw_text(tl, rect.x1 as i32, y as i32);
        }
    }

    fn children(&self) -> &[Box<dyn Widget>] {
        match &self.input {
            Some(i) => std::slice::from_ref(i),
            None => &[],
        }
    }

    fn event(&mut self, event: &WidgetEvent, rect: Rect) -> EventResponse {
        if let Some(ref mut input) = self.input {
            let y = rect.y1 + self.label_size.height + self.label_spacing;
            let input_rect = Rect::new(
                rect.x1,
                y,
                rect.x1 + self.input_size.width,
                y + self.input_size.height,
            );
            return input.event(event, input_rect);
        }
        EventResponse::default()
    }
}
