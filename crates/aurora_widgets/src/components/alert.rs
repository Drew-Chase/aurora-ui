use crate::widgets::{EventResponse, LayoutCtx, Widget};
use aurora_core::geometry::corners::Corners;
use aurora_core::geometry::edges::Edges;
use aurora_core::geometry::rect::Rect;
use aurora_core::geometry::size::Size;
use aurora_core::kmi::WidgetEvent;
use aurora_render::canvas::Canvas;
use aurora_text::font_options::FontWeight;
use aurora_text::text_layout::TextLayout;

use super::colors;

/// Visual variant for an alert.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum AlertVariant {
    #[default]
    Default,
    Destructive,
    Success,
    Warning,
    Info,
}

/// A callout box for important messages.
///
/// # Example
/// ```ignore
/// Alert::new()
///     .variant(AlertVariant::Warning)
///     .title("Warning")
///     .description("This action cannot be undone.")
/// ```
pub struct Alert {
    title: Option<String>,
    description: Option<String>,
    variant: AlertVariant,
    corners: Corners,
    padding: Edges,
    width: Option<f32>,
    title_layout: Option<TextLayout>,
    desc_layout: Option<TextLayout>,
}

impl Alert {
    pub fn new() -> Self {
        Self {
            title: None,
            description: None,
            variant: AlertVariant::Default,
            corners: Corners::all(8.0),
            padding: Edges::new(16.0, 16.0, 16.0, 16.0),
            width: None,
            title_layout: None,
            desc_layout: None,
        }
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    pub fn variant(mut self, variant: AlertVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn corners(mut self, corners: Corners) -> Self {
        self.corners = corners;
        self
    }

    pub fn padding(mut self, padding: Edges) -> Self {
        self.padding = padding;
        self
    }

    pub fn width(mut self, width: impl Into<f32>) -> Self {
        self.width = Some(width.into());
        self
    }

    fn border_color(&self) -> aurora_core::color::Color {
        match self.variant {
            AlertVariant::Default => colors::border(),
            AlertVariant::Destructive => colors::destructive(),
            AlertVariant::Success => colors::success(),
            AlertVariant::Warning => colors::warning(),
            AlertVariant::Info => colors::info(),
        }
    }

    fn bg_color(&self) -> aurora_core::color::Color {
        match self.variant {
            AlertVariant::Default => colors::background(),
            AlertVariant::Destructive => aurora_core::color::Color::new(254, 242, 242, 255),
            AlertVariant::Success => aurora_core::color::Color::new(240, 253, 244, 255),
            AlertVariant::Warning => aurora_core::color::Color::new(254, 252, 232, 255),
            AlertVariant::Info => aurora_core::color::Color::new(239, 246, 255, 255),
        }
    }

    fn fg_color(&self) -> aurora_core::color::Color {
        match self.variant {
            AlertVariant::Default => colors::foreground(),
            AlertVariant::Destructive => colors::destructive(),
            AlertVariant::Success => aurora_core::color::Color::new(22, 163, 74, 255),
            AlertVariant::Warning => aurora_core::color::Color::new(161, 98, 7, 255),
            AlertVariant::Info => aurora_core::color::Color::new(37, 99, 235, 255),
        }
    }
}

impl Default for Alert {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for Alert {
    fn layout(&mut self, available: Size, ctx: &mut LayoutCtx) -> Size {
        let w = self.width.unwrap_or(available.width);
        let inner_w = w - self.padding.left - self.padding.right;
        let mut total_h = 0.0;

        if let Some(ref title) = self.title {
            let mut opts = ctx.font_options.clone();
            opts.size = Some(14.0);
            opts.weight = Some(FontWeight::Medium);
            let mut tl = TextLayout::new(ctx.font_manager, title, &opts, self.fg_color(), None);
            tl.set_max_width(ctx.font_manager, inner_w);
            let ts = tl.size();
            total_h += ts.height;
            self.title_layout = Some(tl);
        }

        if let Some(ref desc) = self.description {
            if self.title.is_some() {
                total_h += 4.0;
            }
            let mut opts = ctx.font_options.clone();
            opts.size = Some(14.0);
            opts.weight = Some(FontWeight::Normal);
            let mut tl = TextLayout::new(ctx.font_manager, desc, &opts, colors::muted_foreground(), None);
            tl.set_max_width(ctx.font_manager, inner_w);
            let ts = tl.size();
            total_h += ts.height;
            self.desc_layout = Some(tl);
        }

        Size::new(w, total_h + self.padding.top + self.padding.bottom)
    }

    fn paint(&self, canvas: &mut Canvas, rect: Rect) {
        canvas.fill_rounded_rect(rect, self.corners, self.bg_color());
        canvas.stroke_rounded_rect(rect, self.corners, 1, self.border_color());

        let mut y = rect.y1 + self.padding.top;
        let x = rect.x1 + self.padding.left;

        if let Some(ref tl) = self.title_layout {
            canvas.draw_text(tl, x as i32, y as i32);
            let ts = tl.size();
            y += ts.height + 4.0;
        }
        if let Some(ref tl) = self.desc_layout {
            canvas.draw_text(tl, x as i32, y as i32);
        }
    }

    fn children(&self) -> &[Box<dyn Widget>] {
        &[]
    }

    fn event(&mut self, _event: &WidgetEvent, _rect: Rect) -> EventResponse {
        EventResponse::default()
    }
}
