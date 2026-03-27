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

/// Avatar size presets.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum AvatarSize {
    Small,
    #[default]
    Medium,
    Large,
}

/// A circular avatar displaying initials or a colored background.
///
/// # Example
/// ```ignore
/// Avatar::new()
///     .initials("DC")
///     .size(AvatarSize::Large)
///     .background_color(Color::new(59, 130, 246, 255))
/// ```
pub struct Avatar {
    initials: Option<String>,
    size_preset: AvatarSize,
    background_color: Color,
    foreground_color: Color,
    custom_size: Option<f32>,
    layout: Option<TextLayout>,
}

impl Avatar {
    pub fn new() -> Self {
        Self {
            initials: None,
            size_preset: AvatarSize::Medium,
            background_color: colors::muted(),
            foreground_color: colors::muted_foreground(),
            custom_size: None,
            layout: None,
        }
    }

    pub fn initials(mut self, initials: impl Into<String>) -> Self {
        self.initials = Some(initials.into());
        self
    }

    pub fn size(mut self, size: AvatarSize) -> Self {
        self.size_preset = size;
        self
    }

    pub fn custom_size(mut self, size: f32) -> Self {
        self.custom_size = Some(size);
        self
    }

    pub fn background_color(mut self, color: Color) -> Self {
        self.background_color = color;
        self
    }

    pub fn foreground_color(mut self, color: Color) -> Self {
        self.foreground_color = color;
        self
    }

    fn pixel_size(&self) -> f32 {
        self.custom_size.unwrap_or(match self.size_preset {
            AvatarSize::Small => 32.0,
            AvatarSize::Medium => 40.0,
            AvatarSize::Large => 56.0,
        })
    }
}

impl Default for Avatar {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for Avatar {
    fn layout(&mut self, _available: Size, ctx: &mut LayoutCtx) -> Size {
        let s = self.pixel_size();
        if let Some(ref initials) = self.initials {
            let font_size = s * 0.4;
            let mut opts = ctx.font_options.clone();
            opts.size = Some(font_size);
            opts.weight = Some(FontWeight::Medium);
            let mut tl = TextLayout::new(ctx.font_manager, initials, &opts, self.foreground_color, None);
            tl.set_max_width(ctx.font_manager, s);
            self.layout = Some(tl);
        }
        Size::new(s, s)
    }

    fn paint(&self, canvas: &mut Canvas, rect: Rect) {
        let corners = Corners::all(self.pixel_size() / 2.0);
        canvas.fill_rounded_rect(rect, corners, self.background_color);

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
        let mut info = aurora_a11y::NodeInfo::new(aurora_a11y::accesskit::Role::Image);
        if let Some(ref initials) = self.initials {
            info = info.with_label(initials.clone());
        }
        info
    }
}
