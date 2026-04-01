use crate::widgets::{EventResponse, EventStatus, LayoutCtx, Widget};
use aurora_core::color::Color;
use aurora_core::geometry::corners::Corners;
use aurora_core::geometry::rect::Rect;
use aurora_core::geometry::size::Size;
use aurora_core::kmi::WidgetEvent;
use aurora_core::kmi::cursor_icon::CursorIcon;
use aurora_core::kmi::mouse::{MouseEvent, MouseState};
use aurora_render::canvas::Canvas;

use super::colors;

/// Converts HSV (hue, saturation, value/brightness) to an RGBA [`Color`].
///
/// - `h` — hue in degrees (0.0..360.0)
/// - `s` — saturation (0.0..1.0)
/// - `v` — value/brightness (0.0..1.0)
fn hsv_to_rgb(h: f32, s: f32, v: f32) -> Color {
    let c = v * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = v - c;
    let (r, g, b) = match (h / 60.0) as u32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    Color::new(
        ((r + m) * 255.0) as u8,
        ((g + m) * 255.0) as u8,
        ((b + m) * 255.0) as u8,
        255,
    )
}

/// Converts an RGB [`Color`] to HSV components `(hue, saturation, value)`.
///
/// - Hue is in degrees (0.0..360.0).
/// - Saturation and value are in the range 0.0..1.0.
fn rgb_to_hsv(color: Color) -> (f32, f32, f32) {
    let r = color.red as f32 / 255.0;
    let g = color.green as f32 / 255.0;
    let b = color.blue as f32 / 255.0;
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;
    let h = if delta == 0.0 {
        0.0
    } else if max == r {
        60.0 * (((g - b) / delta) % 6.0)
    } else if max == g {
        60.0 * ((b - r) / delta + 2.0)
    } else {
        60.0 * ((r - g) / delta + 4.0)
    };
    let h = if h < 0.0 { h + 360.0 } else { h };
    let s = if max == 0.0 { 0.0 } else { delta / max };
    (h, s, max)
}

/// Formats a [`Color`] as a hex string `#RRGGBB`.
fn color_to_hex_string(color: Color) -> String {
    format!("#{:02X}{:02X}{:02X}", color.red, color.green, color.blue)
}

/// Formats a [`Color`] as an RGB display string `R: RR  G: GG  B: BB`.
fn color_to_rgb_string(color: Color) -> String {
    format!("R: {}  G: {}  B: {}", color.red, color.green, color.blue)
}

/// A color selection widget with a saturation/brightness area, hue slider,
/// preview swatch, hex/RGB display, and optional preset palette.
///
/// # Example
/// ```ignore
/// ColorPicker::new()
///     .hue(210.0)
///     .saturation(0.8)
///     .brightness(0.9)
///     .palette(vec![Color::RED, Color::GREEN, Color::BLUE])
///     .on_change(|color| println!("color: {color}"))
/// ```
pub struct ColorPicker {
    /// Hue angle in degrees (0.0..360.0).
    hue: f32,
    /// Saturation (0.0..1.0).
    saturation: f32,
    /// Brightness / value (0.0..1.0).
    brightness: f32,
    /// Whether to show the hex string.
    show_hex: bool,
    /// Whether to show the RGB values.
    show_rgb: bool,
    /// Optional preset color palette.
    palette: Vec<Color>,
    /// Explicit width, or fill available.
    width: Option<f32>,
    /// Callback fired when the selected color changes.
    on_change: Option<Box<dyn FnMut(Color)>>,
    /// Whether the user is dragging inside the SV area.
    sv_dragging: bool,
    /// Whether the user is dragging on the hue bar.
    hue_dragging: bool,
    /// Height of the saturation/brightness gradient area in pixels.
    sv_area_height: f32,
    /// Height of the hue slider bar in pixels.
    hue_bar_height: f32,
    /// Size of each palette swatch in pixels.
    swatch_size: f32,
    /// Cached text layout for the hex value.
    #[cfg(feature = "text")]
    hex_layout: Option<aurora_text::text_layout::TextLayout>,
    /// Cached text layout for the RGB values.
    #[cfg(feature = "text")]
    rgb_layout: Option<aurora_text::text_layout::TextLayout>,
    /// Background color of the widget.
    background: Color,
    /// Border color of the widget.
    border_color: Color,
    /// Corner radii for the outer border.
    corners: Corners,
}

impl ColorPicker {
    /// Spacing between internal sections.
    const SPACING: f32 = 8.0;
    /// Size of the preview swatch.
    const SWATCH_PREVIEW_SIZE: f32 = 32.0;
    /// Height of the info row (preview + text).
    const INFO_ROW_HEIGHT: f32 = 34.0;

    pub fn new() -> Self {
        Self {
            hue: 0.0,
            saturation: 1.0,
            brightness: 1.0,
            show_hex: true,
            show_rgb: true,
            palette: Vec::new(),
            width: None,
            on_change: None,
            sv_dragging: false,
            hue_dragging: false,
            sv_area_height: 180.0,
            hue_bar_height: 20.0,
            swatch_size: 24.0,
            #[cfg(feature = "text")]
            hex_layout: None,
            #[cfg(feature = "text")]
            rgb_layout: None,
            background: colors::card(),
            border_color: colors::border(),
            corners: Corners::all(8.0),
        }
    }

    /// Sets the hue angle (0.0..360.0).
    pub fn hue(mut self, hue: f32) -> Self {
        self.hue = hue.clamp(0.0, 360.0);
        self
    }

    /// Sets the saturation (0.0..1.0).
    pub fn saturation(mut self, saturation: f32) -> Self {
        self.saturation = saturation.clamp(0.0, 1.0);
        self
    }

    /// Sets the brightness / value (0.0..1.0).
    pub fn brightness(mut self, brightness: f32) -> Self {
        self.brightness = brightness.clamp(0.0, 1.0);
        self
    }

    /// Sets the color from an existing [`Color`], converting RGB to HSV.
    pub fn color(mut self, color: Color) -> Self {
        let (h, s, v) = rgb_to_hsv(color);
        self.hue = h;
        self.saturation = s;
        self.brightness = v;
        self
    }

    /// Whether to show the hex string (default: true).
    pub fn show_hex(mut self, show: bool) -> Self {
        self.show_hex = show;
        self
    }

    /// Whether to show the RGB values (default: true).
    pub fn show_rgb(mut self, show: bool) -> Self {
        self.show_rgb = show;
        self
    }

    /// Sets a preset color palette displayed as clickable swatches.
    pub fn palette(mut self, colors: Vec<Color>) -> Self {
        self.palette = colors;
        self
    }

    /// Sets an explicit width. If not set, fills available space.
    pub fn width(mut self, width: impl Into<f32>) -> Self {
        self.width = Some(width.into());
        self
    }

    /// Callback invoked whenever the selected color changes.
    pub fn on_change(mut self, cb: impl FnMut(Color) + 'static) -> Self {
        self.on_change = Some(Box::new(cb));
        self
    }

    /// Sets the height of the saturation/brightness gradient area.
    pub fn sv_area_height(mut self, height: f32) -> Self {
        self.sv_area_height = height;
        self
    }

    /// Sets the height of the hue slider bar.
    pub fn hue_bar_height(mut self, height: f32) -> Self {
        self.hue_bar_height = height;
        self
    }

    /// Sets the size of each palette swatch.
    pub fn swatch_size(mut self, size: f32) -> Self {
        self.swatch_size = size;
        self
    }

    /// Sets the background color.
    pub fn background(mut self, color: Color) -> Self {
        self.background = color;
        self
    }

    /// Sets the border color.
    pub fn border_color(mut self, color: Color) -> Self {
        self.border_color = color;
        self
    }

    /// Sets the corner radii.
    pub fn corners(mut self, corners: Corners) -> Self {
        self.corners = corners;
        self
    }

    /// Returns the currently selected color as an RGB [`Color`].
    pub fn selected_color(&self) -> Color {
        hsv_to_rgb(self.hue, self.saturation, self.brightness)
    }

    // ---- internal layout helpers ----

    /// Returns the rect for the SV gradient area, given the outer content rect.
    fn sv_rect(&self, content: &Rect) -> Rect {
        Rect::new(
            content.x1,
            content.y1,
            content.x2,
            content.y1 + self.sv_area_height,
        )
    }

    /// Returns the rect for the hue slider bar.
    fn hue_rect(&self, content: &Rect) -> Rect {
        let y = content.y1 + self.sv_area_height + Self::SPACING;
        Rect::new(content.x1, y, content.x2, y + self.hue_bar_height)
    }

    /// Returns the rect for the info row (preview swatch + text).
    fn info_rect(&self, content: &Rect) -> Rect {
        let y = content.y1 + self.sv_area_height + Self::SPACING + self.hue_bar_height + Self::SPACING;
        Rect::new(content.x1, y, content.x2, y + Self::INFO_ROW_HEIGHT)
    }

    /// Returns the Y offset where the palette starts.
    fn palette_y(&self, content: &Rect) -> f32 {
        content.y1
            + self.sv_area_height
            + Self::SPACING
            + self.hue_bar_height
            + Self::SPACING
            + Self::INFO_ROW_HEIGHT
            + Self::SPACING
    }

    /// Computes the content rect (inside padding) from the outer bounds.
    fn content_rect(&self, rect: &Rect) -> Rect {
        let pad = 8.0;
        Rect::new(
            rect.x1 + pad,
            rect.y1 + pad,
            rect.x2 - pad,
            rect.y2 - pad,
        )
    }

    /// Total height of the widget.
    fn total_height(&self) -> f32 {
        let pad = 8.0;
        let mut h = pad
            + self.sv_area_height
            + Self::SPACING
            + self.hue_bar_height
            + Self::SPACING
            + Self::INFO_ROW_HEIGHT
            + pad;
        if !self.palette.is_empty() {
            // One row of palette swatches + spacing
            h += Self::SPACING + self.swatch_size;
        }
        h
    }

    /// Updates SV from a mouse position within the SV area rect.
    fn update_sv_from_pos(&mut self, pos_x: f32, pos_y: f32, sv: &Rect) {
        let w = sv.width();
        let h = sv.height();
        if w > 0.0 && h > 0.0 {
            self.saturation = ((pos_x - sv.x1) / w).clamp(0.0, 1.0);
            self.brightness = 1.0 - ((pos_y - sv.y1) / h).clamp(0.0, 1.0);
        }
    }

    /// Updates hue from a mouse position within the hue bar rect.
    fn update_hue_from_pos(&mut self, pos_x: f32, hr: &Rect) {
        let w = hr.width();
        if w > 0.0 {
            self.hue = (((pos_x - hr.x1) / w).clamp(0.0, 1.0) * 360.0).min(359.99);
        }
    }

    /// Fires the on_change callback with the current color.
    fn fire_change(&mut self) {
        let color = self.selected_color();
        if let Some(ref mut cb) = self.on_change {
            cb(color);
        }
    }

    // ---- paint helpers ----

    /// Paints the saturation/brightness gradient area.
    fn paint_sv_area(&self, canvas: &mut Canvas, sv: Rect) {
        let cols = 24u32;
        let rows = 16u32;
        let cell_w = sv.width() / cols as f32;
        let cell_h = sv.height() / rows as f32;

        for row in 0..rows {
            let v = 1.0 - (row as f32 + 0.5) / rows as f32;
            for col in 0..cols {
                let s = (col as f32 + 0.5) / cols as f32;
                let color = hsv_to_rgb(self.hue, s, v);
                let x1 = sv.x1 + col as f32 * cell_w;
                let y1 = sv.y1 + row as f32 * cell_h;
                let x2 = if col == cols - 1 { sv.x2 } else { x1 + cell_w };
                let y2 = if row == rows - 1 { sv.y2 } else { y1 + cell_h };
                canvas.fill_rect(Rect::new(x1, y1, x2, y2), color);
            }
        }

        // Border around SV area
        canvas.stroke_rect(sv, 1u32, self.border_color);

        // Crosshair indicator
        let cx = sv.x1 + self.saturation * sv.width();
        let cy = sv.y1 + (1.0 - self.brightness) * sv.height();
        let arm = 6.0;
        let thick = 2.0;

        // Horizontal bar
        canvas.fill_rect(
            Rect::new(cx - arm, cy - thick / 2.0, cx + arm, cy + thick / 2.0),
            Color::WHITE,
        );
        // Vertical bar
        canvas.fill_rect(
            Rect::new(cx - thick / 2.0, cy - arm, cx + thick / 2.0, cy + arm),
            Color::WHITE,
        );
        // Dark outline for visibility against light backgrounds
        canvas.stroke_rect(
            Rect::new(
                cx - arm - 1.0,
                cy - arm - 1.0,
                cx + arm + 1.0,
                cy + arm + 1.0,
            ),
            1u32,
            Color::new(0, 0, 0, 120),
        );
    }

    /// Paints the hue slider bar.
    fn paint_hue_bar(&self, canvas: &mut Canvas, hr: Rect) {
        // Draw hue spectrum in segments
        let segments = 36u32;
        let seg_w = hr.width() / segments as f32;

        for i in 0..segments {
            let h = (i as f32 + 0.5) / segments as f32 * 360.0;
            let color = hsv_to_rgb(h, 1.0, 1.0);
            let x1 = hr.x1 + i as f32 * seg_w;
            let x2 = if i == segments - 1 {
                hr.x2
            } else {
                x1 + seg_w
            };
            canvas.fill_rect(Rect::new(x1, hr.y1, x2, hr.y2), color);
        }

        // Border
        canvas.stroke_rect(hr, 1u32, self.border_color);

        // Position indicator — small vertical bar
        let ix = hr.x1 + (self.hue / 360.0) * hr.width();
        let indicator_w = 4.0;
        let indicator = Rect::new(
            ix - indicator_w / 2.0,
            hr.y1 - 2.0,
            ix + indicator_w / 2.0,
            hr.y2 + 2.0,
        );
        canvas.fill_rect(indicator, Color::WHITE);
        canvas.stroke_rect(indicator, 1u32, Color::new(0, 0, 0, 180));
    }

    /// Paints the info row: preview swatch + hex + RGB text.
    #[allow(unused_variables)]
    fn paint_info_row(&self, canvas: &mut Canvas, info: Rect) {
        let color = self.selected_color();

        // Preview swatch
        let swatch = Rect::new(
            info.x1,
            info.y1,
            info.x1 + Self::SWATCH_PREVIEW_SIZE,
            info.y1 + Self::SWATCH_PREVIEW_SIZE,
        );
        let swatch_corners = Corners::all(4.0);
        canvas.fill_rounded_rect(swatch, swatch_corners, color);
        canvas.stroke_rounded_rect(swatch, swatch_corners, 1, self.border_color);

        // Text labels (hex and RGB)
        #[cfg(feature = "text")]
        {
            let text_x = (swatch.x2 + 10.0) as i32;
            if self.show_hex
                && let Some(ref layout) = self.hex_layout
            {
                canvas.draw_text(layout, text_x, info.y1 as i32);
            }
            if self.show_rgb
                && let Some(ref layout) = self.rgb_layout
            {
                let y_offset = if self.show_hex { 16.0 } else { 0.0 };
                canvas.draw_text(layout, text_x, (info.y1 + y_offset) as i32);
            }
        }
    }

    /// Paints the preset palette swatches.
    fn paint_palette(&self, canvas: &mut Canvas, content: &Rect) {
        if self.palette.is_empty() {
            return;
        }
        let y = self.palette_y(content);
        let spacing = 4.0;
        let swatch_corners = Corners::all(4.0);
        let current = self.selected_color();

        let mut x = content.x1;
        for &color in &self.palette {
            if x + self.swatch_size > content.x2 {
                break; // Don't overflow
            }
            let swatch = Rect::new(x, y, x + self.swatch_size, y + self.swatch_size);
            canvas.fill_rounded_rect(swatch, swatch_corners, color);

            // Highlight if this is the currently selected color
            let border = if color == current {
                colors::ring()
            } else {
                self.border_color
            };
            canvas.stroke_rounded_rect(swatch, swatch_corners, 1, border);

            x += self.swatch_size + spacing;
        }
    }

    /// Finds which palette swatch was clicked, if any.
    fn palette_hit(&self, pos_x: f32, pos_y: f32, content: &Rect) -> Option<usize> {
        if self.palette.is_empty() {
            return None;
        }
        let y = self.palette_y(content);
        let spacing = 4.0;

        if pos_y < y || pos_y > y + self.swatch_size {
            return None;
        }

        let mut x = content.x1;
        for (i, _color) in self.palette.iter().enumerate() {
            if x + self.swatch_size > content.x2 {
                break;
            }
            if pos_x >= x && pos_x <= x + self.swatch_size {
                return Some(i);
            }
            x += self.swatch_size + spacing;
        }
        None
    }
}

impl Default for ColorPicker {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for ColorPicker {
    fn layout(&mut self, available: Size, _ctx: &mut LayoutCtx) -> Size {
        let w = self.width.unwrap_or(available.width).min(available.width);

        // Build text layouts for hex and RGB display
        #[cfg(feature = "text")]
        {
            let color = self.selected_color();
            let mut opts = _ctx.font_options.clone();
            opts.size = Some(13.0);
            opts.weight = Some(aurora_text::font_options::FontWeight::Normal);

            if self.show_hex {
                let hex = color_to_hex_string(color);
                let mut tl = aurora_text::text_layout::TextLayout::new(
                    _ctx.font_manager,
                    &hex,
                    &opts,
                    colors::foreground(),
                    None,
                );
                tl.set_max_width(_ctx.font_manager, w);
                self.hex_layout = Some(tl);
            }

            if self.show_rgb {
                let rgb = color_to_rgb_string(color);
                let mut tl = aurora_text::text_layout::TextLayout::new(
                    _ctx.font_manager,
                    &rgb,
                    &opts,
                    colors::muted_foreground(),
                    None,
                );
                tl.set_max_width(_ctx.font_manager, w);
                self.rgb_layout = Some(tl);
            }
        }

        Size::new(w, self.total_height())
    }

    fn paint(&self, canvas: &mut Canvas, rect: Rect) {
        // Outer background + border
        canvas.fill_rounded_rect(rect, self.corners, self.background);
        canvas.stroke_rounded_rect(rect, self.corners, 1, self.border_color);

        let content = self.content_rect(&rect);

        // SV area
        self.paint_sv_area(canvas, self.sv_rect(&content));

        // Hue bar
        self.paint_hue_bar(canvas, self.hue_rect(&content));

        // Info row
        self.paint_info_row(canvas, self.info_rect(&content));

        // Palette
        self.paint_palette(canvas, &content);
    }

    fn children(&self) -> &[Box<dyn Widget>] {
        &[]
    }

    fn event(&mut self, event: &WidgetEvent, rect: Rect) -> EventResponse {
        let content = self.content_rect(&rect);
        let sv = self.sv_rect(&content);
        let hr = self.hue_rect(&content);

        match event {
            WidgetEvent::Mouse(MouseEvent::MouseClickEvent(e)) => {
                if e.state == MouseState::Pressed {
                    // Check SV area
                    if sv.contains(&e.position) {
                        self.sv_dragging = true;
                        self.update_sv_from_pos(e.position.x, e.position.y, &sv);
                        self.fire_change();
                        return EventResponse {
                            status: EventStatus::Consumed,
                            cursor: Some(CursorIcon::Pointer),
                            ..Default::default()
                        };
                    }

                    // Check hue bar
                    if hr.contains(&e.position) {
                        self.hue_dragging = true;
                        self.update_hue_from_pos(e.position.x, &hr);
                        self.fire_change();
                        return EventResponse {
                            status: EventStatus::Consumed,
                            cursor: Some(CursorIcon::Pointer),
                            ..Default::default()
                        };
                    }

                    // Check palette swatches
                    if let Some(idx) = self.palette_hit(e.position.x, e.position.y, &content) {
                        let color = self.palette[idx];
                        let (h, s, v) = rgb_to_hsv(color);
                        self.hue = h;
                        self.saturation = s;
                        self.brightness = v;
                        self.fire_change();
                        return EventResponse {
                            status: EventStatus::Consumed,
                            cursor: Some(CursorIcon::Pointer),
                            ..Default::default()
                        };
                    }
                } else {
                    // Released
                    if self.sv_dragging || self.hue_dragging {
                        self.sv_dragging = false;
                        self.hue_dragging = false;
                        return EventResponse {
                            status: EventStatus::Consumed,
                            ..Default::default()
                        };
                    }
                }
                EventResponse::default()
            }
            WidgetEvent::Mouse(MouseEvent::MouseMoveEvent(pos)) => {
                if self.sv_dragging {
                    self.update_sv_from_pos(pos.x, pos.y, &sv);
                    self.fire_change();
                    return EventResponse {
                        status: EventStatus::Consumed,
                        cursor: Some(CursorIcon::Pointer),
                        ..Default::default()
                    };
                }
                if self.hue_dragging {
                    self.update_hue_from_pos(pos.x, &hr);
                    self.fire_change();
                    return EventResponse {
                        status: EventStatus::Consumed,
                        cursor: Some(CursorIcon::Pointer),
                        ..Default::default()
                    };
                }

                // Hover cursors
                if sv.contains(pos) {
                    return EventResponse {
                        cursor: Some(CursorIcon::Pointer),
                        ..Default::default()
                    };
                }
                if hr.contains(pos) || self.palette_hit(pos.x, pos.y, &content).is_some() {
                    return EventResponse {
                        cursor: Some(CursorIcon::Pointer),
                        ..Default::default()
                    };
                }

                EventResponse::default()
            }
            _ => EventResponse::default(),
        }
    }

    #[cfg(feature = "a11y")]
    fn access_info(&self) -> aurora_a11y::NodeInfo {
        aurora_a11y::NodeInfo::new(aurora_a11y::accesskit::Role::Group)
            .with_label("Color picker".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hsv_rgb_roundtrip() {
        // Test several hue values through a round-trip
        for h in [0.0, 30.0, 60.0, 120.0, 180.0, 240.0, 300.0] {
            let original_s = 0.8;
            let original_v = 0.9;
            let color = hsv_to_rgb(h, original_s, original_v);
            let (rh, rs, rv) = rgb_to_hsv(color);
            // Allow small rounding tolerance due to u8 quantization
            assert!(
                (rh - h).abs() < 2.0,
                "Hue mismatch: expected {h}, got {rh}"
            );
            assert!(
                (rs - original_s).abs() < 0.02,
                "Saturation mismatch at hue {h}: expected {original_s}, got {rs}"
            );
            assert!(
                (rv - original_v).abs() < 0.02,
                "Brightness mismatch at hue {h}: expected {original_v}, got {rv}"
            );
        }
    }

    #[test]
    fn hsv_pure_red() {
        let color = hsv_to_rgb(0.0, 1.0, 1.0);
        assert_eq!(color.red, 255);
        assert_eq!(color.green, 0);
        assert_eq!(color.blue, 0);
        assert_eq!(color.alpha, 255);
    }

    #[test]
    fn hsv_white() {
        let color = hsv_to_rgb(0.0, 0.0, 1.0);
        assert_eq!(color.red, 255);
        assert_eq!(color.green, 255);
        assert_eq!(color.blue, 255);
    }

    #[test]
    fn hsv_black() {
        let color = hsv_to_rgb(0.0, 0.0, 0.0);
        assert_eq!(color.red, 0);
        assert_eq!(color.green, 0);
        assert_eq!(color.blue, 0);
    }

    #[test]
    fn hex_format() {
        let color = Color::new(255, 128, 0, 255);
        let hex = color_to_hex_string(color);
        assert_eq!(hex, "#FF8000");

        let black = Color::new(0, 0, 0, 255);
        assert_eq!(color_to_hex_string(black), "#000000");

        let white = Color::new(255, 255, 255, 255);
        assert_eq!(color_to_hex_string(white), "#FFFFFF");
    }

    #[test]
    fn default_values() {
        let picker = ColorPicker::new();
        assert_eq!(picker.hue, 0.0);
        assert_eq!(picker.saturation, 1.0);
        assert_eq!(picker.brightness, 1.0);
        assert!(picker.show_hex);
        assert!(picker.show_rgb);
        assert!(picker.palette.is_empty());
        assert!(picker.width.is_none());
        assert!(!picker.sv_dragging);
        assert!(!picker.hue_dragging);
        assert_eq!(picker.sv_area_height, 180.0);
        assert_eq!(picker.hue_bar_height, 20.0);
        assert_eq!(picker.swatch_size, 24.0);
    }

    #[test]
    fn builder_methods() {
        let picker = ColorPicker::new()
            .hue(210.0)
            .saturation(0.5)
            .brightness(0.8)
            .show_hex(false)
            .show_rgb(false)
            .sv_area_height(200.0)
            .hue_bar_height(24.0)
            .swatch_size(32.0)
            .palette(vec![Color::RED, Color::GREEN]);

        assert_eq!(picker.hue, 210.0);
        assert_eq!(picker.saturation, 0.5);
        assert_eq!(picker.brightness, 0.8);
        assert!(!picker.show_hex);
        assert!(!picker.show_rgb);
        assert_eq!(picker.sv_area_height, 200.0);
        assert_eq!(picker.hue_bar_height, 24.0);
        assert_eq!(picker.swatch_size, 32.0);
        assert_eq!(picker.palette.len(), 2);
    }

    #[test]
    fn color_setter_converts_rgb_to_hsv() {
        let picker = ColorPicker::new().color(Color::RED);
        assert_eq!(picker.hue, 0.0);
        assert!((picker.saturation - 1.0).abs() < 0.01);
        assert!((picker.brightness - 1.0).abs() < 0.01);
    }

    #[test]
    fn selected_color_matches_hsv() {
        let picker = ColorPicker::new().hue(0.0).saturation(1.0).brightness(1.0);
        let color = picker.selected_color();
        assert_eq!(color.red, 255);
        assert_eq!(color.green, 0);
        assert_eq!(color.blue, 0);
    }

    #[test]
    fn rgb_format() {
        let color = Color::new(255, 128, 0, 255);
        let rgb = color_to_rgb_string(color);
        assert_eq!(rgb, "R: 255  G: 128  B: 0");
    }
}
