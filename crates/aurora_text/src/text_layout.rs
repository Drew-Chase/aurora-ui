use crate::font_manager::FontManager;
use crate::font_options::FontOptions;
use aurora_core::color::Color;
use aurora_core::geometry::size::Size;

/// A shaped and measured text block ready for rendering.
///
/// Wraps a `cosmic_text::Buffer` to perform font shaping and layout, then
/// provides a software rasteriser that alpha-blends glyphs into a raw pixel buffer.
#[derive(Clone)]
pub struct TextLayout {
    buffer: cosmic_text::Buffer,
    color: Color,
}

impl TextLayout {
    /// Creates a new text layout, shaping the given text immediately.
    ///
    /// Font size, line height, family, weight, style, and stretch are read
    /// from `font_options`. Pass an `align` value to control horizontal
    /// alignment (`Left`, `Center`, `Right`, etc.).
    pub fn new(
        font_manager: &mut FontManager,
        text: &str,
        font_options: &FontOptions,
        color: Color,
        align: Option<cosmic_text::Align>,
    ) -> Self {
        let size = font_options.effective_size();
        let line_height = font_options.effective_line_height();
        let metrics = cosmic_text::Metrics::new(size, line_height);
        let mut buffer = cosmic_text::Buffer::new(font_manager.font_system_mut(), metrics);
        let attrs = font_options.to_cosmic_attrs();

        buffer.set_text(
            font_manager.font_system_mut(),
            text,
            &attrs,
            cosmic_text::Shaping::Advanced,
            align,
        );

        buffer.shape_until_scroll(font_manager.font_system_mut(), false);

        Self { buffer, color }
    }
    /// Sets the maximum width for line wrapping and re-shapes the buffer.
    pub fn set_max_width(&mut self, font_manager: &mut FontManager, width: f32) {
        self.buffer
            .set_size(font_manager.font_system_mut(), Some(width), None);
        self.buffer
            .shape_until_scroll(font_manager.font_system_mut(), false);
    }

    /// Returns the bounding size of the laid-out text.
    pub fn size(&self) -> Size {
        let mut width: f32 = 0.0;
        let mut height: f32 = 0.0;

        for run in self.buffer.layout_runs() {
            width = width.max(run.line_w);
            height = height.max(run.line_top + run.line_height);
        }

        Size::new(width, height)
    }

    /// Rasterises every glyph into a raw `0x00RRGGBB` pixel buffer.
    ///
    /// Sub-pixel coverage is alpha-blended against the existing buffer contents.
    /// Out-of-bounds glyphs are clipped.
    pub fn render(
        &self,
        cache: &mut cosmic_text::SwashCache,
        font_manager: &mut FontManager,
        buffer: &mut [u32],
        width: u32,
        x_offset: i32,
        y_offset: i32,
    ) {
        let pixel = self.color.to_rgb_u32();

        for run in self.buffer.layout_runs() {
            for glyph in run.glyphs.iter() {
                let physical_glyph = glyph.physical((x_offset as f32, y_offset as f32), 1.0);

                if let Some(image) =
                    cache.get_image(font_manager.font_system_mut(), physical_glyph.cache_key)
                {
                    let glyph_w = image.placement.width as i32;
                    let glyph_h = image.placement.height as i32;
                    let glyph_left = physical_glyph.x + image.placement.left;
                    let glyph_top = physical_glyph.y - image.placement.top;

                    for gy in 0..glyph_h {
                        let py = glyph_top + gy;
                        if py < 0 || py >= buffer.len() as i32 / width as i32 {
                            continue;
                        }
                        for gx in 0..glyph_w {
                            let px = glyph_left + gx;
                            if px < 0 || px >= width as i32 {
                                continue;
                            }

                            let alpha = image.data[(gy * glyph_w + gx) as usize];
                            if alpha == 0 {
                                continue;
                            }

                            let idx = (py as u32 * width + px as u32) as usize;
                            if idx < buffer.len() {
                                if alpha == 255 {
                                    buffer[idx] = pixel;
                                } else {
                                    // Alpha blend with existing pixel
                                    let bg = buffer[idx];
                                    let bg_r = (bg >> 16) & 0xFF;
                                    let bg_g = (bg >> 8) & 0xFF;
                                    let bg_b = bg & 0xFF;
                                    let fg_r = (pixel >> 16) & 0xFF;
                                    let fg_g = (pixel >> 8) & 0xFF;
                                    let fg_b = pixel & 0xFF;
                                    let a = alpha as u32;
                                    let inv_a = 255 - a;
                                    let r = (fg_r * a + bg_r * inv_a) / 255;
                                    let g = (fg_g * a + bg_g * inv_a) / 255;
                                    let b = (fg_b * a + bg_b * inv_a) / 255;
                                    buffer[idx] = (r << 16) | (g << 8) | b;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
