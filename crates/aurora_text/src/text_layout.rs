use crate::font_manager::FontManager;
use crate::font_options::FontOptions;
use aurora_core::color::Color;
use aurora_core::geometry::rect::Rect;
use aurora_core::geometry::size::Size;
use std::ops::Range;

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

    /// Returns the right-edge x position of each glyph in layout order.
    ///
    /// Index `i` gives the x-coordinate of the right edge of glyph `i`,
    /// suitable for cursor positioning in single-line text inputs.
    pub fn char_x_positions(&self) -> Vec<f32> {
        let mut positions = Vec::new();
        for run in self.buffer.layout_runs() {
            for glyph in run.glyphs.iter() {
                positions.push(glyph.x + glyph.w);
            }
        }
        positions
    }

    /// Rasterises every glyph into a raw `0x00RRGGBB` pixel buffer.
    ///
    /// Sub-pixel coverage is alpha-blended against the existing buffer contents.
    /// Out-of-bounds glyphs are clipped. An optional `clip` rect restricts
    /// rendering to a sub-region of the buffer.
    #[allow(clippy::too_many_arguments)]
    pub fn render(
        &self,
        cache: &mut cosmic_text::SwashCache,
        font_manager: &mut FontManager,
        buffer: &mut [u32],
        width: u32,
        x_offset: i32,
        y_offset: i32,
        clip: Option<&Rect>,
    ) {
        let pixel = self.color.to_rgb_u32();
        let height = buffer.len() as i32 / width as i32;

        let (clip_x0, clip_y0, clip_x1, clip_y1) = match clip {
            Some(c) => (
                c.x1.max(0.0) as i32,
                c.y1.max(0.0) as i32,
                (c.x2 as i32).min(width as i32),
                (c.y2 as i32).min(height),
            ),
            None => (0, 0, width as i32, height),
        };

        for run in self.buffer.layout_runs() {
            for glyph in run.glyphs.iter() {
                let physical_glyph =
                    glyph.physical((x_offset as f32, y_offset as f32 + run.line_y), 1.0);

                if let Some(image) =
                    cache.get_image(font_manager.font_system_mut(), physical_glyph.cache_key)
                {
                    let glyph_w = image.placement.width as i32;
                    let glyph_h = image.placement.height as i32;
                    let glyph_left = physical_glyph.x.saturating_add(image.placement.left);
                    let glyph_top = physical_glyph.y.saturating_sub(image.placement.top);

                    // Skip glyphs entirely outside the clip region
                    if glyph_top.saturating_add(glyph_h) < clip_y0
                        || glyph_top >= clip_y1
                        || glyph_left.saturating_add(glyph_w) < clip_x0
                        || glyph_left >= clip_x1
                    {
                        continue;
                    }

                    for gy in 0..glyph_h {
                        let py = glyph_top.saturating_add(gy);
                        if py < clip_y0 || py >= clip_y1 {
                            continue;
                        }
                        for gx in 0..glyph_w {
                            let px = glyph_left.saturating_add(gx);
                            if px < clip_x0 || px >= clip_x1 {
                                continue;
                            }

                            let alpha = image.data[(gy * glyph_w + gx) as usize];
                            if alpha == 0 {
                                continue;
                            }

                            if py < 0 || px < 0 {
                                continue;
                            }
                            let idx = (py as u32 * width + px as u32) as usize;
                            if idx < buffer.len() {
                                if alpha == 255 {
                                    buffer[idx] = pixel;
                                } else {
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

    /// Rasterises glyphs with per-range colors into a raw `0x00RRGGBB` pixel buffer.
    ///
    /// For each glyph, the byte offset (`glyph.start`) is looked up in `ranges`.
    /// If a range matches, that range's color is used; otherwise `self.color` is used.
    #[allow(clippy::too_many_arguments)]
    pub fn render_rich(
        &self,
        cache: &mut cosmic_text::SwashCache,
        font_manager: &mut FontManager,
        buffer: &mut [u32],
        width: u32,
        x_offset: i32,
        y_offset: i32,
        clip: Option<&Rect>,
        ranges: &[(Range<usize>, Color)],
    ) {
        let default_pixel = self.color.to_rgb_u32();
        let height = buffer.len() as i32 / width as i32;

        let (clip_x0, clip_y0, clip_x1, clip_y1) = match clip {
            Some(c) => (
                c.x1.max(0.0) as i32,
                c.y1.max(0.0) as i32,
                (c.x2 as i32).min(width as i32),
                (c.y2 as i32).min(height),
            ),
            None => (0, 0, width as i32, height),
        };

        for run in self.buffer.layout_runs() {
            for glyph in run.glyphs.iter() {
                // Look up per-glyph color from ranges (binary search, assumes sorted by start)
                let byte_offset = glyph.start;
                let pixel = {
                    let idx = ranges.partition_point(|(r, _)| r.start <= byte_offset);
                    if idx > 0 && ranges[idx - 1].0.contains(&byte_offset) {
                        ranges[idx - 1].1.to_rgb_u32()
                    } else {
                        default_pixel
                    }
                };

                let physical_glyph =
                    glyph.physical((x_offset as f32, y_offset as f32 + run.line_y), 1.0);

                if let Some(image) =
                    cache.get_image(font_manager.font_system_mut(), physical_glyph.cache_key)
                {
                    let glyph_w = image.placement.width as i32;
                    let glyph_h = image.placement.height as i32;
                    let glyph_left = physical_glyph.x.saturating_add(image.placement.left);
                    let glyph_top = physical_glyph.y.saturating_sub(image.placement.top);

                    if glyph_top.saturating_add(glyph_h) < clip_y0
                        || glyph_top >= clip_y1
                        || glyph_left.saturating_add(glyph_w) < clip_x0
                        || glyph_left >= clip_x1
                    {
                        continue;
                    }

                    for gy in 0..glyph_h {
                        let py = glyph_top.saturating_add(gy);
                        if py < clip_y0 || py >= clip_y1 {
                            continue;
                        }
                        for gx in 0..glyph_w {
                            let px = glyph_left.saturating_add(gx);
                            if px < clip_x0 || px >= clip_x1 {
                                continue;
                            }

                            let alpha = image.data[(gy * glyph_w + gx) as usize];
                            if alpha == 0 {
                                continue;
                            }

                            if py < 0 || px < 0 {
                                continue;
                            }
                            let idx = (py as u32 * width + px as u32) as usize;
                            if idx < buffer.len() {
                                if alpha == 255 {
                                    buffer[idx] = pixel;
                                } else {
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
