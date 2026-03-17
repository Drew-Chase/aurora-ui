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
    pub(crate) clip_stack: Vec<Rect>,
    #[cfg(feature = "text")]
    pub(crate) font_manager: &'a mut FontManager,
    #[cfg(feature = "text")]
    pub(crate) swash_cache: &'a mut SwashCache,
}

impl<'a> Canvas<'a> {
    /// Blends a foreground color into a buffer span, handling both opaque and transparent cases.
    fn blend_span(buffer: &mut [u32], pixel: u32, alpha: u8) {
        if alpha == 255 {
            buffer.fill(pixel);
        } else {
            let a = alpha as u32;
            let inv_a = 255 - a;
            let fg_r = (pixel >> 16) & 0xFF;
            let fg_g = (pixel >> 8) & 0xFF;
            let fg_b = pixel & 0xFF;
            for px in buffer.iter_mut() {
                let bg = *px;
                let bg_r = (bg >> 16) & 0xFF;
                let bg_g = (bg >> 8) & 0xFF;
                let bg_b = bg & 0xFF;
                let r = (fg_r * a + bg_r * inv_a) / 255;
                let g = (fg_g * a + bg_g * inv_a) / 255;
                let b = (fg_b * a + bg_b * inv_a) / 255;
                *px = (r << 16) | (g << 8) | b;
            }
        }
    }

    #[cfg(not(feature = "text"))]
    /// Creates a canvas from a pixel buffer and its dimensions.
    ///
    /// The buffer is expected to be row-major with `width * height` entries.
    pub fn new(width: u32, height: u32, buffer: &'a mut [u32]) -> Self {
        Canvas {
            width,
            height,
            buffer,
            clip_stack: Vec::new(),
        }
    }
    /// Creates a canvas from a pixel buffer and its dimensions.
    ///
    /// The buffer is expected to be row-major with `width * height` entries.
    #[cfg(feature = "text")]
    pub fn new(
        width: u32,
        height: u32,
        buffer: &'a mut [u32],
        font_manager: &'a mut FontManager,
        swash_cache: &'a mut SwashCache,
    ) -> Self {
        Canvas {
            width,
            height,
            buffer,
            clip_stack: Vec::new(),
            font_manager,
            swash_cache,
        }
    }

    /// Pushes a clipping rectangle onto the clip stack.
    ///
    /// The effective clip is the intersection of the new rect with the
    /// current top of the stack, so nested clips always tighten the region.
    /// Call [`pop_clip`](Self::pop_clip) to restore the previous clip.
    pub fn push_clip(&mut self, rect: Rect) {
        let clipped = match self.clip_stack.last() {
            Some(existing) => existing.intersection(&rect).unwrap_or(rect),
            None => rect,
        };
        self.clip_stack.push(clipped);
    }

    /// Pops the most recent clipping rectangle, restoring the previous one.
    ///
    /// Every `push_clip` should be paired with a `pop_clip`.
    pub fn pop_clip(&mut self) {
        self.clip_stack.pop();
    }

    /// Returns the current effective clip rect, if any.
    pub fn current_clip(&self) -> Option<&Rect> {
        self.clip_stack.last()
    }

    /// Draws a [`TextLayout`] at the given pixel offset.
    ///
    /// Delegates to [`TextLayout::render`], passing the canvas buffer,
    /// font manager, glyph cache, and the active clip rect.
    #[cfg(feature = "text")]
    pub fn draw_text(&mut self, layout: &TextLayout, x: i32, y: i32) {
        let clip = self.clip_stack.last().copied();
        layout.render(
            self.swash_cache,
            self.font_manager,
            self.buffer,
            self.width,
            x,
            y,
            clip.as_ref(),
        );
    }

    /// Fills an axis-aligned rectangle with a solid color.
    ///
    /// Coordinates are clamped to the canvas bounds and the active clip rect.
    pub fn fill_rect(&mut self, rect: impl Into<Rect>, color: impl Into<Color>) {
        let color = color.into();
        if color.alpha == 0 {
            return;
        }
        let rect = rect.into();
        let pixel = color.to_rgb_u32();

        let mut x0 = (rect.x1.max(0.0) as u32).min(self.width);
        let mut y0 = (rect.y1.max(0.0) as u32).min(self.height);
        let mut x1 = (rect.x2.max(0.0) as u32).min(self.width);
        let mut y1 = (rect.y2.max(0.0) as u32).min(self.height);

        if let Some(clip) = self.current_clip() {
            x0 = x0.max(clip.x1.max(0.0) as u32);
            y0 = y0.max(clip.y1.max(0.0) as u32);
            x1 = x1.min(clip.x2.max(0.0) as u32);
            y1 = y1.min(clip.y2.max(0.0) as u32);
        }

        for y in y0..y1 {
            let row_start = (y * self.width + x0) as usize;
            let row_end = (y * self.width + x1) as usize;
            Self::blend_span(&mut self.buffer[row_start..row_end], pixel, color.alpha);
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
        let color = color.into();
        if color.alpha == 0 {
            return;
        }
        let pixel = color.to_rgb_u32();
        let alpha = color.alpha;

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

        let mut x0 = (rect.x1.max(0.0) as u32).min(self.width);
        let mut y0 = (rect.y1.max(0.0) as u32).min(self.height);
        let mut x1 = (rect.x2.max(0.0) as u32).min(self.width);
        let mut y1 = (rect.y2.max(0.0) as u32).min(self.height);

        if let Some(clip) = self.current_clip() {
            x0 = x0.max(clip.x1.max(0.0) as u32);
            y0 = y0.max(clip.y1.max(0.0) as u32);
            x1 = x1.min(clip.x2.max(0.0) as u32);
            y1 = y1.min(clip.y2.max(0.0) as u32);
        }

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
                    Self::blend_span(&mut self.buffer[row_start..row_end], pixel, alpha);
                }
            }
        }
    }

    /// Draws a rectangular outline with the given pixel thickness.
    ///
    /// Only the border pixels are filled — the interior is untouched.
    pub fn stroke_rect(
        &mut self,
        bounds: impl Into<Rect>,
        thickness: impl Into<u32>,
        color: impl Into<Color>,
    ) {
        let rect = bounds.into();
        let thickness = thickness.into();
        let color = color.into();
        if color.alpha == 0 {
            return;
        }
        let pixel = color.to_rgb_u32();
        let alpha = color.alpha;

        let mut x0 = (rect.x1.max(0.0) as u32).min(self.width);
        let mut y0 = (rect.y1.max(0.0) as u32).min(self.height);
        let mut x1 = (rect.x2.max(0.0) as u32).min(self.width);
        let mut y1 = (rect.y2.max(0.0) as u32).min(self.height);

        if let Some(clip) = self.current_clip() {
            x0 = x0.max(clip.x1.max(0.0) as u32);
            y0 = y0.max(clip.y1.max(0.0) as u32);
            x1 = x1.min(clip.x2.max(0.0) as u32);
            y1 = y1.min(clip.y2.max(0.0) as u32);
        }

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
                    Self::blend_span(&mut self.buffer[row_start..row_end], pixel, alpha);
                }
            } else {
                // Middle rows — fill only the left and right edges
                if inner_x0 > x0 {
                    let start = (y * self.width + x0) as usize;
                    let end = (y * self.width + inner_x0) as usize;
                    if end <= self.buffer.len() {
                        Self::blend_span(&mut self.buffer[start..end], pixel, alpha);
                    }
                }
                if inner_x1 < x1 {
                    let start = (y * self.width + inner_x1) as usize;
                    let end = (y * self.width + x1) as usize;
                    if end <= self.buffer.len() {
                        Self::blend_span(&mut self.buffer[start..end], pixel, alpha);
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
        thickness: u32,
        color: impl Into<Color>,
    ) {
        let corners = corners.into();
        if corners.is_zero() {
            return self.stroke_rect(bounds, thickness, color);
        }
        let rect = bounds.into();
        let t = thickness as f32;
        let color = color.into();
        if color.alpha == 0 {
            return;
        }
        let pixel = color.to_rgb_u32();
        let alpha = color.alpha;

        let w = rect.x2 - rect.x1;
        let h = rect.y2 - rect.y1;

        // CSS-style radius clamping
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

        // Inner radii: same center, radius shrunk by thickness
        let tl_in = (tl - t).max(0.0);
        let tr_in = (tr - t).max(0.0);
        let bl_in = (bl - t).max(0.0);
        let br_in = (br - t).max(0.0);

        let mut x0 = (rect.x1.max(0.0) as u32).min(self.width);
        let mut y0 = (rect.y1.max(0.0) as u32).min(self.height);
        let mut x1 = (rect.x2.max(0.0) as u32).min(self.width);
        let mut y1 = (rect.y2.max(0.0) as u32).min(self.height);

        if let Some(clip) = self.current_clip() {
            x0 = x0.max(clip.x1.max(0.0) as u32);
            y0 = y0.max(clip.y1.max(0.0) as u32);
            x1 = x1.min(clip.x2.max(0.0) as u32);
            y1 = y1.min(clip.y2.max(0.0) as u32);
        }

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
                    Self::blend_span(&mut self.buffer[start..end], pixel, alpha);
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
                    Self::blend_span(&mut self.buffer[start..end], pixel, alpha);
                }
            }
            // Right border strip
            if inn_right < out_right {
                let start = (y * self.width + inn_right.max(out_left)) as usize;
                let end = (y * self.width + out_right) as usize;
                if end <= self.buffer.len() {
                    Self::blend_span(&mut self.buffer[start..end], pixel, alpha);
                }
            }
        }
    }

    /// Draws RGBA image data into the canvas at the destination rectangle.
    ///
    /// Scales the source image to fit `dest` using nearest-neighbor sampling.
    /// Supports alpha blending and respects the active clip rect.
    ///
    /// `pixels` must be RGBA with 4 bytes per pixel, `img_width * img_height * 4` total.
    pub fn draw_image(
        &mut self,
        pixels: &[u8],
        img_width: u32,
        img_height: u32,
        dest: impl Into<Rect>,
    ) {
        let dest = dest.into();
        let mut dx0 = (dest.x1.max(0.0) as u32).min(self.width);
        let mut dy0 = (dest.y1.max(0.0) as u32).min(self.height);
        let mut dx1 = (dest.x2.max(0.0) as u32).min(self.width);
        let mut dy1 = (dest.y2.max(0.0) as u32).min(self.height);

        if let Some(clip) = self.current_clip() {
            dx0 = dx0.max(clip.x1.max(0.0) as u32);
            dy0 = dy0.max(clip.y1.max(0.0) as u32);
            dx1 = dx1.min(clip.x2.max(0.0) as u32);
            dy1 = dy1.min(clip.y2.max(0.0) as u32);
        }

        let dest_w = (dest.x2 - dest.x1).max(1.0);
        let dest_h = (dest.y2 - dest.y1).max(1.0);

        for y in dy0..dy1 {
            for x in dx0..dx1 {
                let sx = ((x as f32 - dest.x1) / dest_w * img_width as f32) as u32;
                let sy = ((y as f32 - dest.y1) / dest_h * img_height as f32) as u32;

                if sx >= img_width || sy >= img_height {
                    continue;
                }

                let src_idx = ((sy * img_width + sx) * 4) as usize;
                if src_idx + 3 >= pixels.len() {
                    continue;
                }

                let r = pixels[src_idx] as u32;
                let g = pixels[src_idx + 1] as u32;
                let b = pixels[src_idx + 2] as u32;
                let a = pixels[src_idx + 3];

                if a == 0 {
                    continue;
                }

                let idx = (y * self.width + x) as usize;
                if idx >= self.buffer.len() {
                    continue;
                }

                if a == 255 {
                    self.buffer[idx] = (r << 16) | (g << 8) | b;
                } else {
                    let bg = self.buffer[idx];
                    let bg_r = (bg >> 16) & 0xFF;
                    let bg_g = (bg >> 8) & 0xFF;
                    let bg_b = bg & 0xFF;
                    let alpha = a as u32;
                    let inv = 255 - alpha;
                    let out_r = (r * alpha + bg_r * inv) / 255;
                    let out_g = (g * alpha + bg_g * inv) / 255;
                    let out_b = (b * alpha + bg_b * inv) / 255;
                    self.buffer[idx] = (out_r << 16) | (out_g << 8) | out_b;
                }
            }
        }
    }
}
