use crate::widgets::{LayoutCtx, Widget};
use aurora_core::geometry::rect::Rect;
use aurora_core::geometry::size::Size;
use aurora_render::canvas::Canvas;
use aurora_render::svg_data::SvgData;
use std::sync::Arc;

/// A widget that displays an SVG image, rasterized at layout size.
///
/// The SVG data is shared via `Arc` so cloning the widget is cheap.
/// Parse the SVG once with [`SvgData::from_bytes`] and wrap it in an `Arc`.
///
/// The SVG is re-rasterized whenever the layout size changes, producing
/// crisp output at any resolution.
///
/// # Example
///
/// ```no_run
/// use std::sync::Arc;
/// use aurora_render::svg_data::SvgData;
/// use aurora_widgets::svg_widget::Svg;
///
/// let data = Arc::new(SvgData::from_bytes(include_bytes!("icon.svg")).unwrap());
/// let widget = Svg::new(data).width(48.0).height(48.0);
/// ```
pub struct Svg {
    data: Arc<SvgData>,
    width: Option<f32>,
    height: Option<f32>,
    rasterized: Option<RasterizedSvg>,
}

struct RasterizedSvg {
    pixels: Vec<u8>,
    width: u32,
    height: u32,
}

impl Svg {
    /// Creates a new SVG widget from shared SVG data.
    pub fn new(data: Arc<SvgData>) -> Self {
        Self {
            data,
            width: None,
            height: None,
            rasterized: None,
        }
    }

    /// Sets a fixed width in pixels.
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    /// Sets a fixed height in pixels.
    pub fn height(mut self, height: f32) -> Self {
        self.height = Some(height);
        self
    }
}

impl Widget for Svg {
    fn layout(&mut self, available: Size, _ctx: &mut LayoutCtx) -> Size {
        let (svg_w, svg_h) = self.data.size();
        let w = self.width.unwrap_or(svg_w).min(available.width);
        let h = self.height.unwrap_or(svg_h).min(available.height);

        let rw = w as u32;
        let rh = h as u32;

        // Re-rasterize if size changed
        let needs_raster = match &self.rasterized {
            Some(r) => r.width != rw || r.height != rh,
            None => true,
        };
        if needs_raster && rw > 0 && rh > 0 {
            let pixels = self.data.render(rw, rh);
            self.rasterized = Some(RasterizedSvg {
                pixels,
                width: rw,
                height: rh,
            });
        }

        Size::new(w, h)
    }

    fn paint(&self, canvas: &mut Canvas, rect: Rect) {
        if let Some(r) = &self.rasterized {
            let dest = Rect::new(rect.x1, rect.y1, rect.x1 + r.width as f32, rect.y1 + r.height as f32);
            canvas.draw_image(&r.pixels, r.width, r.height, dest);
        }
    }

    fn children(&self) -> &[Box<dyn Widget>] {
        &[]
    }
}
