#![doc = include_str!("../../../.wiki/Canvas.md")]

use aurora_core::color::Color;
use aurora_core::geometry::corners::Corners;
use aurora_core::geometry::point::Point;
use aurora_core::geometry::rect::Rect;
use aurora_core::geometry::size::Size;
#[cfg(feature = "text")]
use aurora_text::cosmic_text::SwashCache;
#[cfg(feature = "text")]
use aurora_text::font_manager::FontManager;
#[cfg(feature = "text")]
use aurora_text::text_layout::TextLayout;

/// A 2D drawing surface backed by a raw pixel buffer.
///
/// Borrows a `&mut [u32]` buffer (typically from a [`GpuContext`](aurora_gpu::gpu_context::GpuContext))
/// and provides shape-drawing methods. All coordinates are in physical pixels.
/// The buffer is row-major with pixels in `0x00RRGGBB` format.
pub struct Canvas<'a> {
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) buffer: &'a mut [u32],
    #[cfg(feature = "text")]
    pub(crate) font_manager: &'a mut FontManager,
    #[cfg(feature = "text")]
    pub(crate) swash_cache: &'a mut SwashCache,
}

impl<'a> Canvas<'a> {

    #[cfg(not(feature = "text"))]
    /// Creates a canvas from a pixel buffer and its dimensions.
    ///
    /// The buffer is expected to be row-major with `width * height` entries.
    pub fn new(width: u32, height: u32, buffer: &'a mut [u32]) -> Self {
        Canvas {
            width,
            height,
            buffer,
        }
    }
    #[cfg(feature = "text")]
    /// Creates a canvas from a pixel buffer and its dimensions.
    ///
    /// The buffer is expected to be row-major with `width * height` entries.
    pub fn new(width: u32, height: u32, buffer: &'a mut [u32], font_manager: &'a mut FontManager, swash_cache: &'a mut SwashCache) -> Self {
        Canvas {
            width,
            height,
            buffer,
            font_manager,
            swash_cache,
        }
    }

    /// Draws a [`TextLayout`] at the given pixel offset.
    ///
    /// Delegates to [`TextLayout::render`], passing the canvas buffer,
    /// font manager, and glyph cache.
    #[cfg(feature = "text")]
    pub fn draw_text(&mut self, layout: &TextLayout, x: i32, y: i32) {
        layout.render(
            self.swash_cache,
            self.font_manager,
            self.buffer,
            self.width,
            x,
            y,
        );
    }

    /// Fills an axis-aligned rectangle with a solid color.
    ///
    /// Coordinates are clamped to the canvas bounds.
    pub fn fill_rect(&mut self, rect: impl Into<Rect>, color: impl Into<Color>) {
        let rect = rect.into();
        let pixel = color.into().to_rgb_u32();

        let x0 = (rect.x1.max(0.0) as u32).min(self.width);
        let y0 = (rect.y1.max(0.0) as u32).min(self.height);
        let x1 = (rect.x2.max(0.0) as u32).min(self.width);
        let y1 = (rect.y2.max(0.0) as u32).min(self.height);

        for y in y0..y1 {
            let row_start = (y * self.width + x0) as usize;
            let row_end = (y * self.width + x1) as usize;
            if row_end <= self.buffer.len() {
                self.buffer[row_start..row_end].fill(pixel);
            }
        }
    }

    /// Draws a filled circle.
    ///
    /// Convenience wrapper around [`fill_rounded_rect`](Self::fill_rounded_rect) that
    /// creates a square rect from `position` and `size`, then rounds all corners
    /// with `radius`.
    pub fn circle(
        &mut self,
        position: impl Into<Point>,
        size: impl Into<f32>,
        radius: impl Into<f32>,
        color: impl Into<Color>,
    ) {
        let size = size.into();
        let rect = Rect::from_origin_size(position.into(), Size::new(size, size));
        let radius = radius.into();
        let corner = Corners::all(radius);

        self.fill_rounded_rect(rect, corner, color)
    }

    /// Fills a rounded rectangle with per-corner radii.
    ///
    /// Uses CSS-style radius clamping — if adjacent corner radii exceed the
    /// available edge length, all radii are scaled proportionally.
    /// Falls back to [`fill_rect`](Self::fill_rect) when all radii are zero.
    pub fn fill_rounded_rect(
        &mut self,
        rect: impl Into<Rect>,
        corners: impl Into<Corners>,
        color: impl Into<Color>,
    ) {
        let corners = corners.into();
        if corners.is_zero() {
            return self.fill_rect(rect, color);
        }
        let rect = rect.into();
        let pixel = color.into().to_rgb_u32();

        let w = rect.x2 - rect.x1;
        let h = rect.y2 - rect.y1;

        // CSS-style radius clamping: scale all radii proportionally so
        // adjacent corners never overlap. This means a 100x100 rect with
        // all corners set to 100 produces a circle (radii clamped to 50).
        let max_top = corners.top_left + corners.top_right;
        let max_bottom = corners.bottom_left + corners.bottom_right;
        let max_left = corners.top_left + corners.bottom_left;
        let max_right = corners.top_right + corners.bottom_right;
        let mut scale = 1.0_f32;
        if max_top > 0.0 {
            scale = scale.min(w / max_top);
        }
        if max_bottom > 0.0 {
            scale = scale.min(w / max_bottom);
        }
        if max_left > 0.0 {
            scale = scale.min(h / max_left);
        }
        if max_right > 0.0 {
            scale = scale.min(h / max_right);
        }
        let tl = corners.top_left * scale;
        let tr = corners.top_right * scale;
        let bl = corners.bottom_left * scale;
        let br = corners.bottom_right * scale;

        let x0 = (rect.x1.max(0.0) as u32).min(self.width);
        let y0 = (rect.y1.max(0.0) as u32).min(self.height);
        let x1 = (rect.x2.max(0.0) as u32).min(self.width);
        let y1 = (rect.y2.max(0.0) as u32).min(self.height);

        // Corner circle centers
        let tl_cx = rect.x1 + tl;
        let tl_cy = rect.y1 + tl;
        let tr_cx = rect.x2 - tr;
        let tr_cy = rect.y1 + tr;
        let bl_cx = rect.x1 + bl;
        let bl_cy = rect.y2 - bl;
        let br_cx = rect.x2 - br;
        let br_cy = rect.y2 - br;

        // Band boundaries (clamped to rect)
        let top_band_end = (tl_cy.max(tr_cy) as u32).min(y1);
        let bot_band_start = (bl_cy.min(br_cy).max(0.0) as u32).max(y0);

        for y in y0..y1 {
            let fy = y as f32 + 0.5;
            let mut row_x0 = x0;
            let mut row_x1 = x1;

            if y < top_band_end {
                // Top-left corner clips the left edge rightward
                if tl > 0.0 && fy < tl_cy {
                    let dy = fy - tl_cy;
                    let r2 = tl * tl;
                    let dx2 = r2 - dy * dy;
                    if dx2 > 0.0 {
                        let edge = (tl_cx - dx2.sqrt()) as u32;
                        row_x0 = row_x0.max(edge.max(x0));
                    } else {
                        row_x0 = row_x0.max((tl_cx as u32).min(x1));
                    }
                }
                // Top-right corner clips the right edge leftward
                if tr > 0.0 && fy < tr_cy {
                    let dy = fy - tr_cy;
                    let r2 = tr * tr;
                    let dx2 = r2 - dy * dy;
                    if dx2 > 0.0 {
                        let edge = (tr_cx + dx2.sqrt()).ceil() as u32;
                        row_x1 = row_x1.min(edge.max(x0));
                    } else {
                        row_x1 = row_x1.min((tr_cx as u32).max(x0));
                    }
                }
            }

            if y >= bot_band_start {
                // Bottom-left corner clips the left edge rightward
                if bl > 0.0 && fy > bl_cy {
                    let dy = fy - bl_cy;
                    let r2 = bl * bl;
                    let dx2 = r2 - dy * dy;
                    if dx2 > 0.0 {
                        let edge = (bl_cx - dx2.sqrt()) as u32;
                        row_x0 = row_x0.max(edge.max(x0));
                    } else {
                        row_x0 = row_x0.max((bl_cx as u32).min(x1));
                    }
                }
                // Bottom-right corner clips the right edge leftward
                if br > 0.0 && fy > br_cy {
                    let dy = fy - br_cy;
                    let r2 = br * br;
                    let dx2 = r2 - dy * dy;
                    if dx2 > 0.0 {
                        let edge = (br_cx + dx2.sqrt()).ceil() as u32;
                        row_x1 = row_x1.min(edge.max(x0));
                    } else {
                        row_x1 = row_x1.min((br_cx as u32).max(x0));
                    }
                }
            }

            if row_x0 < row_x1 {
                let row_start = (y * self.width + row_x0) as usize;
                let row_end = (y * self.width + row_x1) as usize;
                if row_end <= self.buffer.len() {
                    self.buffer[row_start..row_end].fill(pixel);
                }
            }
        }
    }

    /// Draws a rectangular outline with the given pixel thickness.
    ///
    /// Only the border pixels are filled — the interior is untouched.
    pub fn stroke_rect(&mut self, bounds: impl Into<Rect>, thickness: impl Into<u32>, color: impl Into<Color>) {
        let rect = bounds.into();
        let thickness = thickness.into();
        let pixel = color.into().to_rgb_u32();

        let x0 = (rect.x1.max(0.0) as u32).min(self.width);
        let y0 = (rect.y1.max(0.0) as u32).min(self.height);
        let x1 = (rect.x2.max(0.0) as u32).min(self.width);
        let y1 = (rect.y2.max(0.0) as u32).min(self.height);

        let inner_x0 = (x0 + thickness).min(x1);
        let inner_y0 = (y0 + thickness).min(y1);
        let inner_x1 = x1.saturating_sub(thickness).max(x0);
        let inner_y1 = y1.saturating_sub(thickness).max(y0);

        for y in y0..y1 {
            if y < inner_y0 || y >= inner_y1 {
                // Top or bottom edge — fill the entire row
                let row_start = (y * self.width + x0) as usize;
                let row_end = (y * self.width + x1) as usize;
                if row_end <= self.buffer.len() {
                    self.buffer[row_start..row_end].fill(pixel);
                }
            } else {
                // Middle rows — fill only the left and right edges
                if inner_x0 > x0 {
                    let start = (y * self.width + x0) as usize;
                    let end = (y * self.width + inner_x0) as usize;
                    if end <= self.buffer.len() {
                        self.buffer[start..end].fill(pixel);
                    }
                }
                if inner_x1 < x1 {
                    let start = (y * self.width + inner_x1) as usize;
                    let end = (y * self.width + x1) as usize;
                    if end <= self.buffer.len() {
                        self.buffer[start..end].fill(pixel);
                    }
                }
            }
        }
    }
    /// Draws a rounded rectangular outline with per-corner radii.
    ///
    /// The inner edge uses radii shrunk by the stroke thickness. Uses CSS-style
    /// radius clamping. Falls back to [`stroke_rect`](Self::stroke_rect) when
    /// all radii are zero.
    pub fn stroke_rounded_rect(
        &mut self,
        bounds: impl Into<Rect>,
        corners: impl Into<Corners>,
        thickness:u32,
        color: impl Into<Color>,
    ) {
        let corners = corners.into();
        if corners.is_zero() {
            return self.stroke_rect(bounds, thickness, color);
        }
        let rect = bounds.into();
        let t = thickness as f32;
        let pixel = color.into().to_rgb_u32();

        let w = rect.x2 - rect.x1;
        let h = rect.y2 - rect.y1;

        // CSS-style radius clamping
        let max_top = corners.top_left + corners.top_right;
        let max_bottom = corners.bottom_left + corners.bottom_right;
        let max_left = corners.top_left + corners.bottom_left;
        let max_right = corners.top_right + corners.bottom_right;
        let mut scale = 1.0_f32;
        if max_top > 0.0 { scale = scale.min(w / max_top); }
        if max_bottom > 0.0 { scale = scale.min(w / max_bottom); }
        if max_left > 0.0 { scale = scale.min(h / max_left); }
        if max_right > 0.0 { scale = scale.min(h / max_right); }
        let tl = corners.top_left * scale;
        let tr = corners.top_right * scale;
        let bl = corners.bottom_left * scale;
        let br = corners.bottom_right * scale;

        // Inner radii: same center, radius shrunk by thickness
        let tl_in = (tl - t).max(0.0);
        let tr_in = (tr - t).max(0.0);
        let bl_in = (bl - t).max(0.0);
        let br_in = (br - t).max(0.0);

        let x0 = (rect.x1.max(0.0) as u32).min(self.width);
        let y0 = (rect.y1.max(0.0) as u32).min(self.height);
        let x1 = (rect.x2.max(0.0) as u32).min(self.width);
        let y1 = (rect.y2.max(0.0) as u32).min(self.height);

        // Outer corner circle centers
        let tl_cx = rect.x1 + tl;
        let tl_cy = rect.y1 + tl;
        let tr_cx = rect.x2 - tr;
        let tr_cy = rect.y1 + tr;
        let bl_cx = rect.x1 + bl;
        let bl_cy = rect.y2 - bl;
        let br_cx = rect.x2 - br;
        let br_cy = rect.y2 - br;

        // Inner rect edges (clamped so they don't invert)
        let in_left = (rect.x1 + t).min(rect.x2);
        let in_right = (rect.x2 - t).max(rect.x1);
        let in_top = (rect.y1 + t).min(rect.y2);
        let in_bottom = (rect.y2 - t).max(rect.y1);

        let top_band_end = (tl_cy.max(tr_cy) as u32).min(y1);
        let bot_band_start = (bl_cy.min(br_cy).max(0.0) as u32).max(y0);

        for y in y0..y1 {
            let fy = y as f32 + 0.5;

            // Outer edges (clipped by corner arcs)
            let mut out_left = x0;
            let mut out_right = x1;

            if y < top_band_end {
                if tl > 0.0 && fy < tl_cy {
                    let dy = fy - tl_cy;
                    let dx2 = tl * tl - dy * dy;
                    if dx2 > 0.0 {
                        out_left = out_left.max((tl_cx - dx2.sqrt()) as u32).max(x0);
                    } else {
                        out_left = out_left.max((tl_cx as u32).min(x1));
                    }
                }
                if tr > 0.0 && fy < tr_cy {
                    let dy = fy - tr_cy;
                    let dx2 = tr * tr - dy * dy;
                    if dx2 > 0.0 {
                        out_right = out_right.min((tr_cx + dx2.sqrt()).ceil() as u32).max(x0);
                    } else {
                        out_right = out_right.min((tr_cx as u32).max(x0));
                    }
                }
            }
            if y >= bot_band_start {
                if bl > 0.0 && fy > bl_cy {
                    let dy = fy - bl_cy;
                    let dx2 = bl * bl - dy * dy;
                    if dx2 > 0.0 {
                        out_left = out_left.max((bl_cx - dx2.sqrt()) as u32).max(x0);
                    } else {
                        out_left = out_left.max((bl_cx as u32).min(x1));
                    }
                }
                if br > 0.0 && fy > br_cy {
                    let dy = fy - br_cy;
                    let dx2 = br * br - dy * dy;
                    if dx2 > 0.0 {
                        out_right = out_right.min((br_cx + dx2.sqrt()).ceil() as u32).max(x0);
                    } else {
                        out_right = out_right.min((br_cx as u32).max(x0));
                    }
                }
            }

            if out_left >= out_right {
                continue;
            }

            // If this row is in the top/bottom thickness band, fill entire outer span
            if fy < in_top || fy >= in_bottom {
                let start = (y * self.width + out_left) as usize;
                let end = (y * self.width + out_right) as usize;
                if end <= self.buffer.len() {
                    self.buffer[start..end].fill(pixel);
                }
                continue;
            }

            // Inner edges (clipped by inner corner arcs)
            let mut inn_left = (in_left.max(0.0) as u32).min(self.width);
            let mut inn_right = (in_right.max(0.0) as u32).min(self.width);

            // Inner corners use the same centers but smaller radii
            if y < top_band_end {
                if tl_in > 0.0 && fy < tl_cy {
                    let dy = fy - tl_cy;
                    let dx2 = tl_in * tl_in - dy * dy;
                    if dx2 > 0.0 {
                        inn_left = inn_left.max((tl_cx - dx2.sqrt()) as u32).max(x0);
                    } else {
                        inn_left = inn_left.max((tl_cx as u32).min(x1));
                    }
                }
                if tr_in > 0.0 && fy < tr_cy {
                    let dy = fy - tr_cy;
                    let dx2 = tr_in * tr_in - dy * dy;
                    if dx2 > 0.0 {
                        inn_right = inn_right.min((tr_cx + dx2.sqrt()).ceil() as u32).max(x0);
                    } else {
                        inn_right = inn_right.min((tr_cx as u32).max(x0));
                    }
                }
            }
            if y >= bot_band_start {
                if bl_in > 0.0 && fy > bl_cy {
                    let dy = fy - bl_cy;
                    let dx2 = bl_in * bl_in - dy * dy;
                    if dx2 > 0.0 {
                        inn_left = inn_left.max((bl_cx - dx2.sqrt()) as u32).max(x0);
                    } else {
                        inn_left = inn_left.max((bl_cx as u32).min(x1));
                    }
                }
                if br_in > 0.0 && fy > br_cy {
                    let dy = fy - br_cy;
                    let dx2 = br_in * br_in - dy * dy;
                    if dx2 > 0.0 {
                        inn_right = inn_right.min((br_cx + dx2.sqrt()).ceil() as u32).max(x0);
                    } else {
                        inn_right = inn_right.min((br_cx as u32).max(x0));
                    }
                }
            }

            // Left border strip
            if out_left < inn_left {
                let start = (y * self.width + out_left) as usize;
                let end = (y * self.width + inn_left.min(out_right)) as usize;
                if end <= self.buffer.len() {
                    self.buffer[start..end].fill(pixel);
                }
            }
            // Right border strip
            if inn_right < out_right {
                let start = (y * self.width + inn_right.max(out_left)) as usize;
                let end = (y * self.width + out_right) as usize;
                if end <= self.buffer.len() {
                    self.buffer[start..end].fill(pixel);
                }
            }
        }
    }
}
