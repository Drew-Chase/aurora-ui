use crate::widgets::{EventResponse, LayoutCtx, Widget};
use aurora_core::geometry::rect::Rect;
use aurora_core::geometry::size::Size;
use aurora_core::kmi::WidgetEvent;
use aurora_render::canvas::Canvas;
use aurora_text::font_options::FontWeight;
use aurora_text::text_layout::TextLayout;

use super::colors;

/// An empty state placeholder with optional title and description.
///
/// # Example
/// ```ignore
/// Empty::new()
///     .title("No results found")
///     .description("Try adjusting your search or filters.")
/// ```
pub struct Empty {
    title: Option<String>,
    description: Option<String>,
    width: Option<f32>,
    height: Option<f32>,
    title_layout: Option<TextLayout>,
    desc_layout: Option<TextLayout>,
}

impl Empty {
    pub fn new() -> Self {
        Self {
            title: None,
            description: None,
            width: None,
            height: Some(200.0),
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

    pub fn width(mut self, width: impl Into<f32>) -> Self {
        self.width = Some(width.into());
        self
    }

    pub fn height(mut self, height: impl Into<f32>) -> Self {
        self.height = Some(height.into());
        self
    }
}

impl Default for Empty {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for Empty {
    fn layout(&mut self, available: Size, ctx: &mut LayoutCtx) -> Size {
        let w = self.width.unwrap_or(available.width);

        if let Some(ref title) = self.title {
            let mut opts = ctx.font_options.clone();
            opts.size = Some(16.0);
            opts.weight = Some(FontWeight::Medium);
            let mut tl = TextLayout::new(ctx.font_manager, title, &opts, colors::foreground(), None);
            tl.set_max_width(ctx.font_manager, w);
            self.title_layout = Some(tl);
        }
        if let Some(ref desc) = self.description {
            let mut opts = ctx.font_options.clone();
            opts.size = Some(14.0);
            opts.weight = Some(FontWeight::Normal);
            let mut tl = TextLayout::new(ctx.font_manager, desc, &opts, colors::muted_foreground(), None);
            tl.set_max_width(ctx.font_manager, w);
            self.desc_layout = Some(tl);
        }

        Size::new(w, self.height.unwrap_or(200.0))
    }

    fn paint(&self, canvas: &mut Canvas, rect: Rect) {
        let cx = rect.x1 + rect.width() / 2.0;
        let mut y = rect.y1 + rect.height() / 2.0 - 20.0;

        if let Some(ref tl) = self.title_layout {
            let ts = tl.size();
            canvas.draw_text(tl, (cx - ts.width / 2.0) as i32, y as i32);
            y += ts.height + 8.0;
        }
        if let Some(ref tl) = self.desc_layout {
            let ts = tl.size();
            canvas.draw_text(tl, (cx - ts.width / 2.0) as i32, y as i32);
        }
    }

    fn children(&self) -> &[Box<dyn Widget>] {
        &[]
    }

    fn event(&mut self, _event: &WidgetEvent, _rect: Rect) -> EventResponse {
        EventResponse::default()
    }
}
