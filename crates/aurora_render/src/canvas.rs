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

/// Signed distance from point (fx, fy) to the boundary of a rounded rect.
/// Returns negative values inside, positive values outside.
#[inline(always)]
fn rounded_rect_sdf(
    fx: f32,
    fy: f32,
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
    tl: f32,
    tr: f32,
    bl: f32,
    br: f32,
    tl_cx: f32,
    tl_cy: f32,
    tr_cx: f32,
    tr_cy: f32,
    bl_cx: f32,
    bl_cy: f32,
    br_cx: f32,
    br_cy: f32,
) -> f32 {
    if fx < tl_cx && fy < tl_cy && tl > 0.0 {
        let dx = fx - tl_cx;
        let dy = fy - tl_cy;
        return (dx * dx + dy * dy).sqrt() - tl;
    }
    if fx > tr_cx && fy < tr_cy && tr > 0.0 {
        let dx = fx - tr_cx;
        let dy = fy - tr_cy;
        return (dx * dx + dy * dy).sqrt() - tr;
    }
    if fx < bl_cx && fy > bl_cy && bl > 0.0 {
        let dx = fx - bl_cx;
        let dy = fy - bl_cy;
        return (dx * dx + dy * dy).sqrt() - bl;
    }
    if fx > br_cx && fy > br_cy && br > 0.0 {
        let dx = fx - br_cx;
        let dy = fy - br_cy;
        return (dx * dx + dy * dy).sqrt() - br;
    }
    // Not in any corner: signed distance to nearest straight edge
    -((fx - x1).min(x2 - fx).min(fy - y1).min(y2 - fy))
}

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
    /// Blends a single foreground pixel into the buffer at `index` with the given alpha.
    ///
    /// Used for antialiased edge pixels where coverage produces a reduced effective alpha.
    #[inline]
    fn blend_pixel(buffer: &mut [u32], index: usize, pixel: u32, alpha: u8) {
        if alpha == 0 || index >= buffer.len() {
            return;
        }
        if alpha == 255 {
            buffer[index] = pixel;
            return;
        }
        let a = alpha as u32;
        let inv_a = 255 - a;
        let fg_r = (pixel >> 16) & 0xFF;
        let fg_g = (pixel >> 8) & 0xFF;
        let fg_b = pixel & 0xFF;
        let bg = buffer[index];
        let bg_r = (bg >> 16) & 0xFF;
        let bg_g = (bg >> 8) & 0xFF;
        let bg_b = bg & 0xFF;
        let r = (fg_r * a + bg_r * inv_a) / 255;
        let g = (fg_g * a + bg_g * inv_a) / 255;
        let b = (fg_b * a + bg_b * inv_a) / 255;
        buffer[index] = (r << 16) | (g << 8) | b;
    }

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

    /// Draws text with per-range coloring for syntax highlighting.
    #[cfg(feature = "text")]
    pub fn draw_rich_text(
        &mut self,
        layout: &TextLayout,
        x: i32,
        y: i32,
        ranges: &[(std::ops::Range<usize>, aurora_core::color::Color)],
    ) {
        let clip = self.clip_stack.last().copied();
        layout.render_rich(
            self.swash_cache,
            self.font_manager,
            self.buffer,
            self.width,
            x,
            y,
            clip.as_ref(),
            ranges,
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

        if x0 >= x1 || y0 >= y1 {
            return;
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
    /// Uses SDF-based anti-aliasing for smooth corners at all angles.
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
        // adjacent corners never overlap.
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
        let mut x1 = (rect.x2.ceil().max(0.0) as u32).min(self.width);
        let mut y1 = (rect.y2.ceil().max(0.0) as u32).min(self.height);

        if let Some(clip) = self.current_clip() {
            x0 = x0.max(clip.x1.max(0.0) as u32);
            y0 = y0.max(clip.y1.max(0.0) as u32);
            x1 = x1.min(clip.x2.max(0.0) as u32);
            y1 = y1.min(clip.y2.max(0.0) as u32);
        }

        if x0 >= x1 || y0 >= y1 {
            return;
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

        // Band boundaries for optimization
        let top_band = tl_cy.max(tr_cy);
        let bot_band = bl_cy.min(br_cy);

        for y in y0..y1 {
            let fy = y as f32 + 0.5;
            let row = (y * self.width) as usize;

            // Middle band: no corners, full-width solid span
            if fy >= top_band && fy <= bot_band {
                let end = row + x1 as usize;
                if end <= self.buffer.len() {
                    Self::blend_span(
                        &mut self.buffer[row + x0 as usize..end],
                        pixel,
                        alpha,
                    );
                }
                continue;
            }

            // Corner band: per-pixel SDF with solid-run accumulation
            let mut run_start: Option<u32> = None;

            for x in x0..x1 {
                let fx = x as f32 + 0.5;

                let dist = rounded_rect_sdf(
                    fx, fy, rect.x1, rect.y1, rect.x2, rect.y2, tl, tr, bl, br, tl_cx,
                    tl_cy, tr_cx, tr_cy, bl_cx, bl_cy, br_cx, br_cy,
                );
                let coverage = (0.5 - dist).clamp(0.0, 1.0);

                if coverage >= 1.0 {
                    if run_start.is_none() {
                        run_start = Some(x);
                    }
                } else {
                    if let Some(start) = run_start.take() {
                        let s = row + start as usize;
                        let e = row + x as usize;
                        if e <= self.buffer.len() {
                            Self::blend_span(&mut self.buffer[s..e], pixel, alpha);
                        }
                    }
                    if coverage > 0.0 {
                        let eff_alpha = (coverage * alpha as f32) as u8;
                        Self::blend_pixel(self.buffer, row + x as usize, pixel, eff_alpha);
                    }
                }
            }
            if let Some(start) = run_start {
                let s = row + start as usize;
                let e = row + x1 as usize;
                if e <= self.buffer.len() {
                    Self::blend_span(&mut self.buffer[s..e], pixel, alpha);
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
    /// Strokes a rounded rectangle outline with per-corner radii.
    ///
    /// Uses CSS-style radius clamping and SDF-based anti-aliasing for smooth
    /// edges on both the outer and inner boundaries of the stroke.
    /// Falls back to [`stroke_rect`](Self::stroke_rect) when all radii are zero.
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
        let mut x1 = (rect.x2.ceil().max(0.0) as u32).min(self.width);
        let mut y1 = (rect.y2.ceil().max(0.0) as u32).min(self.height);

        if let Some(clip) = self.current_clip() {
            x0 = x0.max(clip.x1.max(0.0) as u32);
            y0 = y0.max(clip.y1.max(0.0) as u32);
            x1 = x1.min(clip.x2.max(0.0) as u32);
            y1 = y1.min(clip.y2.max(0.0) as u32);
        }

        if x0 >= x1 || y0 >= y1 {
            return;
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
        let in_x1 = (rect.x1 + t).min(rect.x2);
        let in_y1 = (rect.y1 + t).min(rect.y2);
        let in_x2 = (rect.x2 - t).max(rect.x1);
        let in_y2 = (rect.y2 - t).max(rect.y1);

        // Inner corner centers
        let in_tl_cx = in_x1 + tl_in;
        let in_tl_cy = in_y1 + tl_in;
        let in_tr_cx = in_x2 - tr_in;
        let in_tr_cy = in_y1 + tr_in;
        let in_bl_cx = in_x1 + bl_in;
        let in_bl_cy = in_y2 - bl_in;
        let in_br_cx = in_x2 - br_in;
        let in_br_cy = in_y2 - br_in;

        // Band boundaries for optimization
        let top_band = tl_cy.max(tr_cy);
        let bot_band = bl_cy.min(br_cy);

        // Edge region width for middle-band optimization
        let left_end = ((in_x1 + 1.5).ceil().max(0.0) as u32).min(x1);
        let right_start = ((in_x2 - 1.5).floor().max(0.0) as u32).max(x0);

        for y in y0..y1 {
            let fy = y as f32 + 0.5;
            let row = (y * self.width) as usize;

            let in_corner = fy < top_band || fy > bot_band;

            // Build x-ranges to iterate: corner bands check full row,
            // middle band only checks left/right edge regions
            let ranges: [(u32, u32); 2] = if in_corner {
                [(x0, x1), (x1, x1)]
            } else {
                [(x0, left_end), (right_start, x1)]
            };

            for &(rx0, rx1) in &ranges {
                for x in rx0..rx1 {
                    let fx = x as f32 + 0.5;

                    // Quick reject: definitely inside interior (no stroke here)
                    if fx > in_x1 + 0.5
                        && fx < in_x2 - 0.5
                        && fy > in_y1 + 0.5
                        && fy < in_y2 - 0.5
                    {
                        continue;
                    }

                    // Outer coverage
                    let outer_dist = rounded_rect_sdf(
                        fx, fy, rect.x1, rect.y1, rect.x2, rect.y2, tl, tr, bl, br, tl_cx,
                        tl_cy, tr_cx, tr_cy, bl_cx, bl_cy, br_cx, br_cy,
                    );
                    let outer_cov = (0.5 - outer_dist).clamp(0.0, 1.0);
                    if outer_cov <= 0.0 {
                        continue;
                    }

                    // Inner coverage
                    let inner_dist = rounded_rect_sdf(
                        fx, fy, in_x1, in_y1, in_x2, in_y2, tl_in, tr_in, bl_in, br_in,
                        in_tl_cx, in_tl_cy, in_tr_cx, in_tr_cy, in_bl_cx, in_bl_cy, in_br_cx,
                        in_br_cy,
                    );
                    let inner_cov = (0.5 - inner_dist).clamp(0.0, 1.0);

                    let coverage = outer_cov - inner_cov;
                    if coverage > 0.0 {
                        let eff_alpha = (coverage * alpha as f32) as u8;
                        Self::blend_pixel(self.buffer, row + x as usize, pixel, eff_alpha);
                    }
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

    /// Draws an anti-aliased line between two points with the given thickness and color.
    ///
    /// Uses a signed distance field capsule approach: each pixel's distance to the
    /// line segment determines its coverage, producing smooth edges and rounded endpoints.
    /// Respects the active clip rect.
    pub fn draw_line(
        &mut self,
        from: impl Into<Point>,
        to: impl Into<Point>,
        thickness: f32,
        color: impl Into<Color>,
    ) {
        let from = from.into();
        let to = to.into();
        let color = color.into();
        if color.alpha == 0 || thickness <= 0.0 {
            return;
        }
        let pixel = color.to_rgb_u32();
        let base_alpha = color.alpha;
        let half_t = thickness * 0.5;

        let dx = to.x - from.x;
        let dy = to.y - from.y;
        let len_sq = dx * dx + dy * dy;

        // Pre-compute distance thresholds for early-out
        let outer = half_t + 0.5;
        let outer_sq = outer * outer;
        let inner = half_t - 0.5;
        let inner_sq = if inner > 0.0 { inner * inner } else { -1.0 };

        // Bounding box of the capsule, expanded by 1px for AA fringe
        let min_x = from.x.min(to.x) - half_t - 1.0;
        let min_y = from.y.min(to.y) - half_t - 1.0;
        let max_x = from.x.max(to.x) + half_t + 1.0;
        let max_y = from.y.max(to.y) + half_t + 1.0;

        // Clamp to canvas and clip rect
        let mut x0 = (min_x.floor().max(0.0) as u32).min(self.width);
        let mut y0 = (min_y.floor().max(0.0) as u32).min(self.height);
        let mut x1 = (max_x.ceil().max(0.0) as u32).min(self.width);
        let mut y1 = (max_y.ceil().max(0.0) as u32).min(self.height);

        if let Some(clip) = self.current_clip() {
            x0 = x0.max(clip.x1.max(0.0) as u32);
            y0 = y0.max(clip.y1.max(0.0) as u32);
            x1 = x1.min(clip.x2.max(0.0) as u32);
            y1 = y1.min(clip.y2.max(0.0) as u32);
        }

        // Reciprocal of len_sq for projection (avoid division in inner loop)
        let inv_len_sq = if len_sq > 0.0 { 1.0 / len_sq } else { 0.0 };

        for py in y0..y1 {
            let sy = py as f32 + 0.5;
            let vy = sy - from.y;
            let row_start = (py * self.width) as usize;

            for px in x0..x1 {
                let sx = px as f32 + 0.5;
                let vx = sx - from.x;

                // Project sample point onto line segment, clamped to [0, 1]
                let t = if len_sq > 0.0 {
                    ((vx * dx + vy * dy) * inv_len_sq).clamp(0.0, 1.0)
                } else {
                    0.0 // degenerate: point at `from`
                };

                // Nearest point on segment
                let nx = from.x + t * dx;
                let ny = from.y + t * dy;

                // Squared distance from sample to nearest point
                let dist_dx = sx - nx;
                let dist_dy = sy - ny;
                let dist_sq = dist_dx * dist_dx + dist_dy * dist_dy;

                // Early-out: fully outside the capsule
                if dist_sq >= outer_sq {
                    continue;
                }

                let idx = row_start + px as usize;

                // Fully inside the capsule (no sqrt needed)
                if dist_sq <= inner_sq {
                    Self::blend_pixel(self.buffer, idx, pixel, base_alpha);
                    continue;
                }

                // AA fringe: compute actual distance for coverage
                let dist = dist_sq.sqrt();
                let coverage = (half_t + 0.5 - dist).clamp(0.0, 1.0);
                let eff_alpha = (coverage * base_alpha as f32) as u8;
                if eff_alpha > 0 {
                    Self::blend_pixel(self.buffer, idx, pixel, eff_alpha);
                }
            }
        }
    }
}
