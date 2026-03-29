use crate::widgets::{EventResponse, LayoutCtx, Widget};
use aurora_core::geometry::corners::Corners;
use aurora_core::geometry::rect::Rect;
use aurora_core::geometry::size::Size;
use aurora_core::kmi::WidgetEvent;
use aurora_render::canvas::Canvas;
use aurora_text::font_options::FontWeight;
use aurora_text::text_layout::TextLayout;

use super::colors;

/// Visual style variant for a badge.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum BadgeVariant {
    #[default]
    Default,
    Secondary,
    Destructive,
    Outline,
    Success,
    Warning,
    Info,
}

/// A small status indicator or label.
///
/// # Example
/// ```ignore
/// Badge::new("New")
///     .variant(BadgeVariant::Success)
/// ```
pub struct Badge {
    text: String,
    variant: BadgeVariant,
    font_size: f32,
    corners: Corners,
    layout: Option<TextLayout>,
}

impl Badge {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            variant: BadgeVariant::Default,
            font_size: 12.0,
            corners: Corners::all(9999.0), // pill shape
            layout: None,
        }
    }

    pub fn variant(mut self, variant: BadgeVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn font_size(mut self, size: f32) -> Self {
        self.font_size = size;
        self
    }

    pub fn corners(mut self, corners: Corners) -> Self {
        self.corners = corners;
        self
    }

    fn bg_color(&self) -> aurora_core::color::Color {
        match self.variant {
            BadgeVariant::Default => colors::primary(),
            BadgeVariant::Secondary => colors::secondary(),
            BadgeVariant::Destructive => colors::destructive(),
            BadgeVariant::Outline => aurora_core::color::Color::TRANSPARENT,
            BadgeVariant::Success => colors::success(),
            BadgeVariant::Warning => colors::warning(),
            BadgeVariant::Info => colors::info(),
        }
    }

    fn fg_color(&self) -> aurora_core::color::Color {
        match self.variant {
            BadgeVariant::Default => colors::primary_foreground(),
            BadgeVariant::Secondary => colors::secondary_foreground(),
            BadgeVariant::Destructive => colors::destructive_foreground(),
            BadgeVariant::Outline => colors::foreground(),
            BadgeVariant::Success => colors::success_foreground(),
            BadgeVariant::Warning => colors::warning_foreground(),
            BadgeVariant::Info => colors::info_foreground(),
        }
    }
}

impl Widget for Badge {
    fn layout(&mut self, _available: Size, ctx: &mut LayoutCtx) -> Size {
        let mut opts = ctx.font_options.clone();
        opts.size = Some(self.font_size);
        opts.weight = Some(FontWeight::Medium);

        let mut tl = TextLayout::new(ctx.font_manager, &self.text, &opts, self.fg_color(), None);
        tl.set_max_width(ctx.font_manager, f32::MAX);
        let ts = tl.size();
        self.layout = Some(tl);

        let pad_x = 10.0;
        let pad_y = 2.0;
        Size::new(ts.width + pad_x * 2.0, ts.height + pad_y * 2.0)
    }

    fn paint(&self, canvas: &mut Canvas, rect: Rect) {
        let bg = self.bg_color();
        if bg.alpha > 0 {
            canvas.fill_rounded_rect(rect, self.corners, bg);
        }
        if matches!(self.variant, BadgeVariant::Outline) {
            canvas.stroke_rounded_rect(rect, self.corners, 1, colors::border());
        }
        if let Some(ref tl) = self.layout {
            let ts = tl.size();
            let x = rect.x1 + (rect.width() - ts.width) / 2.0;
            let y = rect.y1 + (rect.height() - ts.height) / 2.0;
            canvas.draw_text(tl, x as i32, y as i32);
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
