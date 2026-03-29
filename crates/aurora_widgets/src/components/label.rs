use crate::widgets::{EventResponse, LayoutCtx, Widget};
use aurora_core::color::Color;
use aurora_core::geometry::rect::Rect;
use aurora_core::geometry::size::Size;
use aurora_core::kmi::WidgetEvent;
use aurora_render::canvas::Canvas;
use aurora_text::font_options::FontWeight;
use aurora_text::text_layout::TextLayout;

use super::colors;

/// A text label for form fields and UI elements.
pub struct Label {
    text: String,
    font_size: f32,
    color: Color,
    font_weight: FontWeight,
    width: Option<f32>,
    height: Option<f32>,
    layout: Option<TextLayout>,
}

impl Label {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            font_size: 14.0,
            color: colors::foreground(),
            font_weight: FontWeight::Medium,
            width: None,
            height: None,
            layout: None,
        }
    }

    pub fn font_size(mut self, size: f32) -> Self {
        self.font_size = size;
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn font_weight(mut self, weight: FontWeight) -> Self {
        self.font_weight = weight;
        self
    }

    pub fn width(mut self, width: impl Into<f32>) -> Self {
        self.width = Some(width.into());
        self
    }

    pub fn height(mut self, height: impl Into<f32>) -> Self {
        self.height = Some(height.into());
        self
    }
}

impl Widget for Label {
    fn layout(&mut self, available: Size, ctx: &mut LayoutCtx) -> Size {
        let mut opts = ctx.font_options.clone();
        opts.size = Some(self.font_size);
        opts.weight = Some(self.font_weight);

        let max_w = self.width.unwrap_or(available.width);
        let mut tl = TextLayout::new(ctx.font_manager, &self.text, &opts, self.color, None);
        tl.set_max_width(ctx.font_manager, max_w);
        let ts = tl.size();
        self.layout = Some(tl);

        Size::new(
            self.width.unwrap_or(ts.width),
            self.height.unwrap_or(ts.height),
        )
    }

    fn paint(&self, canvas: &mut Canvas, rect: Rect) {
        if let Some(ref tl) = self.layout {
            canvas.draw_text(tl, rect.x1 as i32, rect.y1 as i32);
        }
    }

    fn children(&self) -> &[Box<dyn Widget>] {
        &[]
    }

    fn event(&mut self, _event: &WidgetEvent, _rect: Rect) -> EventResponse {
        EventResponse::default()
    }
    #[cfg(feature = "a11y")]
    fn access_info(&self) -> aurora_a11y::NodeInfo {
        aurora_a11y::NodeInfo::new(aurora_a11y::accesskit::Role::Label)
            .with_label(self.text.clone())
    }
}
