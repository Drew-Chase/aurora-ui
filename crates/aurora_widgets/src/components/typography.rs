use crate::widgets::{EventResponse, LayoutCtx, Widget};
use aurora_core::color::Color;
use aurora_core::geometry::corners::Corners;
use aurora_core::geometry::rect::Rect;
use aurora_core::geometry::size::Size;
use aurora_core::kmi::WidgetEvent;
use aurora_render::canvas::Canvas;
use aurora_text::font_options::FontWeight;
use aurora_text::text_layout::TextLayout;

use super::colors;

/// Typography level for heading hierarchy.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TypographyLevel {
    H1,
    H2,
    H3,
    H4,
    Paragraph,
    Lead,
    Large,
    Small,
    Muted,
    InlineCode,
}

/// A typography component for consistent text styling.
///
/// # Example
/// ```ignore
/// Typography::h1("Page Title")
/// Typography::paragraph("Body text content.")
/// Typography::muted("Secondary information.")
/// ```
pub struct Typography {
    text: String,
    level: TypographyLevel,
    color: Option<Color>,
    width: Option<f32>,
    layout: Option<TextLayout>,
}

impl Typography {
    pub fn new(text: impl Into<String>, level: TypographyLevel) -> Self {
        Self {
            text: text.into(),
            level,
            color: None,
            width: None,
            layout: None,
        }
    }

    pub fn h1(text: impl Into<String>) -> Self {
        Self::new(text, TypographyLevel::H1)
    }

    pub fn h2(text: impl Into<String>) -> Self {
        Self::new(text, TypographyLevel::H2)
    }

    pub fn h3(text: impl Into<String>) -> Self {
        Self::new(text, TypographyLevel::H3)
    }

    pub fn h4(text: impl Into<String>) -> Self {
        Self::new(text, TypographyLevel::H4)
    }

    pub fn paragraph(text: impl Into<String>) -> Self {
        Self::new(text, TypographyLevel::Paragraph)
    }

    pub fn lead(text: impl Into<String>) -> Self {
        Self::new(text, TypographyLevel::Lead)
    }

    pub fn large(text: impl Into<String>) -> Self {
        Self::new(text, TypographyLevel::Large)
    }

    pub fn small(text: impl Into<String>) -> Self {
        Self::new(text, TypographyLevel::Small)
    }

    pub fn muted(text: impl Into<String>) -> Self {
        Self::new(text, TypographyLevel::Muted)
    }

    pub fn inline_code(text: impl Into<String>) -> Self {
        Self::new(text, TypographyLevel::InlineCode)
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    pub fn width(mut self, width: impl Into<f32>) -> Self {
        self.width = Some(width.into());
        self
    }

    fn font_size(&self) -> f32 {
        match self.level {
            TypographyLevel::H1 => 36.0,
            TypographyLevel::H2 => 30.0,
            TypographyLevel::H3 => 24.0,
            TypographyLevel::H4 => 20.0,
            TypographyLevel::Paragraph => 16.0,
            TypographyLevel::Lead => 20.0,
            TypographyLevel::Large => 18.0,
            TypographyLevel::Small => 14.0,
            TypographyLevel::Muted => 14.0,
            TypographyLevel::InlineCode => 14.0,
        }
    }

    fn font_weight(&self) -> FontWeight {
        match self.level {
            TypographyLevel::H1 => FontWeight::ExtraBold,
            TypographyLevel::H2 => FontWeight::SemiBold,
            TypographyLevel::H3 => FontWeight::SemiBold,
            TypographyLevel::H4 => FontWeight::SemiBold,
            TypographyLevel::Large => FontWeight::SemiBold,
            _ => FontWeight::Normal,
        }
    }

    fn default_color(&self) -> Color {
        match self.level {
            TypographyLevel::Muted => colors::muted_foreground(),
            TypographyLevel::Lead => colors::muted_foreground(),
            _ => colors::foreground(),
        }
    }
}

impl Widget for Typography {
    fn layout(&mut self, available: Size, ctx: &mut LayoutCtx) -> Size {
        let mut opts = ctx.font_options.clone();
        opts.size = Some(self.font_size());
        opts.weight = Some(self.font_weight());

        let text_color = self.color.unwrap_or(self.default_color());
        let max_w = self.width.unwrap_or(available.width);
        let mut tl = TextLayout::new(ctx.font_manager, &self.text, &opts, text_color, None);
        tl.set_max_width(ctx.font_manager, max_w);
        let ts = tl.size();
        self.layout = Some(tl);

        let line_spacing = match self.level {
            TypographyLevel::H1 => 8.0,
            TypographyLevel::H2 => 6.0,
            TypographyLevel::H3 | TypographyLevel::H4 => 4.0,
            _ => 0.0,
        };

        Size::new(self.width.unwrap_or(ts.width), ts.height + line_spacing)
    }

    fn paint(&self, canvas: &mut Canvas, rect: Rect) {
        // InlineCode has a background
        if matches!(self.level, TypographyLevel::InlineCode) {
            let bg_rect = Rect::new(rect.x1 - 2.0, rect.y1, rect.x2 + 2.0, rect.y2);
            let corners = Corners::all(3.0);
            canvas.fill_rounded_rect(bg_rect, corners, colors::muted());
        }

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
}
