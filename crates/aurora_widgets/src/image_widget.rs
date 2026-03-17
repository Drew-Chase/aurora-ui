use crate::widgets::{LayoutCtx, Widget};
use aurora_core::geometry::rect::Rect;
use aurora_core::geometry::size::Size;
use aurora_render::canvas::Canvas;
use aurora_render::image_data::ImageData;
use std::sync::Arc;

/// How an image scales to fit its layout bounds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ImageFit {
    /// Scale uniformly to fill the bounds, cropping if the aspect ratio differs.
    #[default]
    Cover,
    /// Scale uniformly to fit entirely within the bounds, letterboxing if needed.
    Contain,
    /// Stretch to exactly fill the bounds, ignoring aspect ratio.
    Fill,
    /// Display at the image's native pixel size, ignoring the bounds.
    None,
}

/// A widget that displays a raster image.
///
/// The image data is shared via `Arc` so cloning the widget is cheap.
/// Decode the image once with [`ImageData::from_bytes`] and wrap it in an `Arc`.
///
/// # Example
///
/// ```no_run
/// use std::sync::Arc;
/// use aurora_render::image_data::ImageData;
/// use aurora_widgets::image_widget::Image;
///
/// let data = Arc::new(ImageData::from_bytes(include_bytes!("logo.png")).unwrap());
/// let widget = Image::new(data).width(200.0).height(100.0);
/// ```
pub struct Image {
    data: Arc<ImageData>,
    width: Option<f32>,
    height: Option<f32>,
    fit: ImageFit,
}

impl Image {
    /// Creates a new image widget from shared image data.
    pub fn new(data: Arc<ImageData>) -> Self {
        Self {
            data,
            width: None,
            height: None,
            fit: ImageFit::default(),
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

    /// Sets how the image scales to fit its bounds.
    pub fn fit(mut self, fit: ImageFit) -> Self {
        self.fit = fit;
        self
    }
}

impl Widget for Image {
    fn layout(&mut self, available: Size, _ctx: &mut LayoutCtx) -> Size {
        let w = self.width.unwrap_or(available.width).min(available.width);
        let h = self.height.unwrap_or(available.height).min(available.height);
        Size::new(w, h)
    }

    fn paint(&self, canvas: &mut Canvas, rect: Rect) {
        if self.data.width == 0 || self.data.height == 0 {
            return;
        }

        let img_w = self.data.width as f32;
        let img_h = self.data.height as f32;
        let box_w = rect.width();
        let box_h = rect.height();

        let dest = match self.fit {
            ImageFit::Fill => rect,
            ImageFit::None => {
                Rect::new(rect.x1, rect.y1, rect.x1 + img_w, rect.y1 + img_h)
            }
            ImageFit::Contain => {
                let scale = (box_w / img_w).min(box_h / img_h);
                let w = img_w * scale;
                let h = img_h * scale;
                let x = rect.x1 + (box_w - w) / 2.0;
                let y = rect.y1 + (box_h - h) / 2.0;
                Rect::new(x, y, x + w, y + h)
            }
            ImageFit::Cover => {
                let scale = (box_w / img_w).max(box_h / img_h);
                let w = img_w * scale;
                let h = img_h * scale;
                let x = rect.x1 + (box_w - w) / 2.0;
                let y = rect.y1 + (box_h - h) / 2.0;
                Rect::new(x, y, x + w, y + h)
            }
        };

        canvas.draw_image(&self.data.pixels, self.data.width, self.data.height, dest);
    }

    fn children(&self) -> &[Box<dyn Widget>] {
        &[]
    }
}
