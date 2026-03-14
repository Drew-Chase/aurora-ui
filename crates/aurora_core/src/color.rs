#![doc = include_str!("../../../.wiki/Color.md")]

use std::fmt::Display;

/// A trait for converting string representations of hex colors into [`Color`] values.
///
/// Implementors parse a hex string (without a leading `#`) and return the corresponding color.
pub trait IntoColor {
    /// Parses `self` as a hex color string and returns the corresponding [`Color`].
    ///
    /// Returns black (`#000000`) if the string is not a valid hex number.
    fn color(&self, has_alpha: bool) -> Color;
}

/// An RGBA color with 8-bit channels.
///
/// Colors can be constructed from hex values, RGB/RGBA components, or HSL/HSLA values.
/// The [`Display`] implementation formats the color as a CSS-style `#rrggbb` hex string.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    /// Red channel (0–255).
    pub red: u8,
    /// Green channel (0–255).
    pub green: u8,
    /// Blue channel (0–255).
    pub blue: u8,
    /// Alpha channel (0–255), where 255 is fully opaque.
    pub alpha: u8,
}

impl Color {
    pub const BLACK: Color = Color::new(0, 0, 0, 255);
    pub const WHITE: Color = Color::new(255, 255, 255, 255);
    pub const RED: Color = Color::new(255, 0, 0, 255);
    pub const GREEN: Color = Color::new(0, 255, 0, 255);
    pub const BLUE: Color = Color::new(0, 0, 255, 255);
    pub const TRANSPARENT: Color = Color::new(0, 0, 0, 0);
    
    /// Creates a new color from individual RGBA channel values.
    pub const fn new(red: u8, green: u8, blue: u8, alpha: u8) -> Self {
        Color {
            red,
            green,
            blue,
            alpha,
        }
    }

    /// Creates a fully opaque color from RGB channel values.
    ///
    /// Alpha is set to 255.
    pub fn from_rgb(red: u8, green: u8, blue: u8) -> Self {
        Color {
            red,
            green,
            blue,
            alpha: 255,
        }
    }

    /// Creates a color from RGBA channel values.
    pub fn from_rgba(red: u8, green: u8, blue: u8, alpha: u8) -> Self {
        Color {
            red,
            green,
            blue,
            alpha,
        }
    }

    /// Creates a fully opaque color from HSL (hue, saturation, lightness) values.
    ///
    /// This is a convenience wrapper around [`from_hsla`](Self::from_hsla) with alpha set to 1.0.
    ///
    /// - `hue` — degrees on the color wheel (0–360, wraps via modulo).
    /// - `saturation` — 0.0 (grey) to 1.0 (full color), clamped.
    /// - `lightness` — 0.0 (black) to 1.0 (white), clamped.
    pub fn from_hsl<T, F>(hue: T, saturation: F, lightness: F) -> Self
    where
        T: Into<i32>,
        F: Into<f32>,
    {
        Self::from_hsla(hue, saturation.into(), lightness.into(), 1f32)
    }

    /// Creates a color from HSLA (hue, saturation, lightness, alpha) values.
    ///
    /// - `hue` — degrees on the color wheel (0–360, wraps via modulo).
    /// - `saturation` — 0.0 (grey) to 1.0 (full color), clamped.
    /// - `lightness` — 0.0 (black) to 1.0 (white), clamped.
    /// - `alpha` — 0.0 (fully transparent) to 1.0 (fully opaque), clamped.
    pub fn from_hsla<T, F>(hue: T, saturation: F, lightness: F, alpha: F) -> Self
    where
        T: Into<i32>,
        F: Into<f32>,
    {
        let h = hue.into() as f32 % 360.0;
        let s = saturation.into().clamp(0.0, 1.0);
        let l = lightness.into().clamp(0.0, 1.0);
        let alpha = (alpha.into().clamp(0.0, 1.0) * 255.0) as u8;

        let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
        let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
        let m = l - c / 2.0;

        let (r1, g1, b1) = match h as u32 {
            0..60 => (c, x, 0.0),
            60..120 => (x, c, 0.0),
            120..180 => (0.0, c, x),
            180..240 => (0.0, x, c),
            240..300 => (x, 0.0, c),
            _ => (c, 0.0, x),
        };

        Self {
            red: ((r1 + m) * 255.0).round() as u8,
            green: ((g1 + m) * 255.0).round() as u8,
            blue: ((b1 + m) * 255.0).round() as u8,
            alpha,
        }
    }

    /// Creates a color from a hex integer in either RGB (`0xRRGGBB`) or RGBA (`0xRRGGBBAA`) format.
    ///
    /// Values greater than `0xFFFFFF` are interpreted as RGBA; otherwise as RGB with alpha set to 255.
    pub fn from_hex(hex: u64, has_alpha: bool) -> Self {
        if has_alpha {
            let red = ((hex >> 24) & 0xff) as u8;
            let green = ((hex >> 16) & 0xff) as u8;
            let blue = ((hex >> 8) & 0xff) as u8;
            let alpha = (hex & 0xff) as u8;
            Self {
                red,
                green,
                blue,
                alpha,
            }
        } else {
            let red = ((hex >> 16) & 0xff) as u8;
            let green = ((hex >> 8) & 0xff) as u8;
            let blue = (hex & 0xff) as u8;
            let alpha = 0xff;
            Self {
                red,
                green,
                blue,
                alpha,
            }
        }
    }

    /// Converts this color to a hex integer in RGBA format (`0xRRGGBBAA`).
    pub fn to_hex(&self) -> u64 {
        let red = self.red as u64;
        let green = self.green as u64;
        let blue = self.blue as u64;
        let alpha = self.alpha as u64;
        (red << 24) | (green << 16) | (blue << 8) | alpha
    }

    /// Packs this color into a `u32` in `0x00RRGGBB` format.
    ///
    /// Alpha is discarded. This is the format expected by softbuffer.
    pub fn to_rgb_u32(&self) -> u32 {
        (self.red as u32) << 16 | (self.green as u32) << 8 | self.blue as u32
    }

    /// Packs this color into a `u32` in `0xAARRGGBB` format.
    ///
    /// Common in Direct2D, GDI+, and Windows-native APIs.
    pub fn to_argb_u32(&self) -> u32 {
        (self.alpha as u32) << 24
            | (self.red as u32) << 16
            | (self.green as u32) << 8
            | self.blue as u32
    }

    /// Packs this color into a `u32` in `0xAABBGGRR` format.
    ///
    /// Common in OpenGL, Vulkan, and other GPU APIs that expect little-endian RGBA byte order.
    pub fn to_abgr_u32(&self) -> u32 {
        (self.alpha as u32) << 24
            | (self.blue as u32) << 16
            | (self.green as u32) << 8
            | self.red as u32
    }

    /// Converts this color to an array of f32 values in RGBA format (`[r, g, b, a]`).
    pub fn to_array(&self) -> [f32; 4] {
        [
            self.red as f32 / 255.0,
            self.green as f32 / 255.0,
            self.blue as f32 / 255.0,
            self.alpha as f32 / 255.0,
        ]
    }

    /// Linearly interpolates between `self` and `other` by `time`.
    ///
    /// - `time` = 0.0 returns `self`.
    /// - `time` = 1.0 returns `other`.
    /// - Values outside 0.0–1.0 are clamped.
    pub fn lerp(&self, other: &Self, time: f32) -> Self {
        let t = time.clamp(0.0, 1.0);
        Self {
            red: (self.red as f32 + (other.red as f32 - self.red as f32) * t).round() as u8,
            green: (self.green as f32 + (other.green as f32 - self.green as f32) * t).round() as u8,
            blue: (self.blue as f32 + (other.blue as f32 - self.blue as f32) * t).round() as u8,
            alpha: (self.alpha as f32 + (other.alpha as f32 - self.alpha as f32) * t).round() as u8,
        }
    }

    /// Linearly interpolates across a sequence of colors by `time`.
    ///
    /// `time` is clamped to 0.0–1.0 and mapped evenly across the color stops.
    /// For example, with 4 colors, `time` 0.0–0.33 blends between the first and second,
    /// 0.33–0.66 between the second and third, and 0.66–1.0 between the third and fourth.
    ///
    /// Panics if `colors` is empty.
    pub fn lerp_many(colors: &[Color], time: f32) -> Self {
        assert!(!colors.is_empty(), "lerp_many requires at least one color");
        if colors.len() == 1 {
            return colors[0];
        }
        let t = time.clamp(0.0, 1.0);
        let segments = (colors.len() - 1) as f32;
        let scaled = t * segments;
        let index = (scaled as usize).min(colors.len() - 2);
        let local_t = scaled - index as f32;
        colors[index].lerp(&colors[index + 1], local_t)
    }

    pub fn opacity(mut self, opacity: f32) -> Self {
        let opacity = opacity.clamp(0.0, 1.0);
        self.alpha = (255f32 * opacity) as u8;
        self
    }

    pub fn alpha(mut self, alpha: u8) -> Self {
        self.alpha = alpha;
        self
    }
    pub fn red(mut self, red: u8) -> Self {
        self.red = red;
        self
    }
    pub fn green(mut self, green: u8) -> Self {
        self.green = green;
        self
    }
    pub fn blue(mut self, blue: u8) -> Self {
        self.blue = blue;
        self
    }
}

/// Formats the color as a CSS-style `#rrggbbaa` hex string
impl Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "#{:02x}{:02x}{:02x}{:02x}",
            self.red, self.green, self.blue, self.alpha
        )
    }
}

impl IntoColor for String {
    fn color(&self, has_alpha: bool) -> Color {
        let hex = u64::from_str_radix(self, 16).unwrap_or(0);
        Color::from_hex(hex, has_alpha)
    }
}

impl IntoColor for &str {
    fn color(&self, has_alpha: bool) -> Color {
        let hex = u64::from_str_radix(self, 16).unwrap_or(0);
        Color::from_hex(hex, has_alpha)
    }
}

/// Creates a [`Color`] from RGBA channel values.
///
/// Shorthand for [`Color::from_rgba`].
#[macro_export]
macro_rules! rgba {
    ($r:expr,$g:expr,$b:expr,$a:expr) => {
        Color::from_rgba($r, $g, $b, $a)
    };
}

/// Creates a fully opaque [`Color`] from RGB channel values.
///
/// Shorthand for [`Color::from_rgb`].
#[macro_export]
macro_rules! rgb {
    ($r:expr,$g:expr,$b:expr) => {
        Color::from_rgb($r, $g, $b)
    };
}

#[macro_export]
macro_rules! hex {
    ($hex:expr) => {
        Color::from_hex($hex, false)
    };
}
#[macro_export]
macro_rules! hexa {
    ($hex:expr) => {
        Color::from_hex($hex, true)
    };
}


/// Creates a [`Color`] from HSLA values.
///
/// Shorthand for [`Color::from_hsla`].
#[macro_export]
macro_rules! hsla {
    ($h:expr,$s:expr,$l:expr,$a:expr) => {
        Color::from_hsla($h, $s, $l, $a)
    };
}

/// Creates a fully opaque [`Color`] from HSL values.
///
/// Shorthand for [`Color::from_hsl`].
#[macro_export]
macro_rules! hsl {
    ($h:expr,$s:expr,$l:expr) => {
        Color::from_hsl($h, $s, $l)
    };
}

/// Interpolates across multiple colors by a `time` value (0.0–1.0).
///
/// The time is distributed evenly across adjacent color pairs.
///
/// Shorthand for [`Color::lerp_many`].
#[macro_export]
macro_rules! lerp {
    ($time:expr, $($color:expr),+ $(,)?) => {
        Color::lerp_many(&[$($color),+], $time)
    };
}

#[cfg(test)]
mod test {
    use crate::color::{Color, IntoColor};

    #[test]
    fn test_from_hex() {
        let hex_color = 0xff00ff;
        let color = Color::from_hex(hex_color, false);
        assert_eq!(color.red, 255);
        assert_eq!(color.green, 0);
        assert_eq!(color.blue, 255);
        assert_eq!(color.alpha, 255);

        let hex_color = 0x00000080;
        let color = Color::from_hex(hex_color, true);
        assert_eq!(color.red, 0);
        assert_eq!(color.green, 0);
        assert_eq!(color.blue, 0);
        assert_eq!(color.alpha, 128);

        let hex_color = 0x00ff00;
        let color = Color::from_hex(hex_color, false);
        assert_eq!(color.red, 0);
        assert_eq!(color.green, 255);
        assert_eq!(color.blue, 0);
        assert_eq!(color.alpha, 255);

        let hex_color = 0xffff0080;
        let color = Color::from_hex(hex_color, true);

        assert_eq!(color.red, 255);
        assert_eq!(color.green, 255);
        assert_eq!(color.blue, 0);
        assert_eq!(color.alpha, 128); // 80 = 8 × 16 + 0 = 128

        let hex_color = 0x72203a;
        let color = Color::from_hex(hex_color, false);

        assert_eq!(color.red, 114);
        assert_eq!(color.green, 32);
        assert_eq!(color.blue, 58);
        assert_eq!(color.alpha, 255);

        let hex_color = 0x1ce783;
        let color = Color::from_hex(hex_color, false);

        assert_eq!(color.red, 28);
        assert_eq!(color.green, 231);
        assert_eq!(color.blue, 131);
        assert_eq!(color.alpha, 255);
    }

    #[test]
    fn test_to_hex() {
        let color = Color::from_hex(0xff00ffff, true);
        let hex_color = color.to_hex();
        assert_eq!(hex_color, 0xff00ffff);

        let color = Color::from_hex(0xff00ff, false);
        let hex_color = color.to_hex();
        assert_eq!(hex_color, 0xff00ffff);
    }

    #[test]
    fn test_from_hsla() {
        // Pure red: hsl(0, 1.0, 0.5)
        let color = Color::from_hsla(0, 1.0f32, 0.5f32, 1.0f32);
        assert_eq!(color.red, 255);
        assert_eq!(color.green, 0);
        assert_eq!(color.blue, 0);
        assert_eq!(color.alpha, 255);

        // Pure green: hsl(120, 1.0, 0.5)
        let color = Color::from_hsla(120, 1.0f32, 0.5f32, 1.0f32);
        assert_eq!(color.red, 0);
        assert_eq!(color.green, 255);
        assert_eq!(color.blue, 0);
        assert_eq!(color.alpha, 255);

        // Pure blue: hsl(240, 1.0, 0.5)
        let color = Color::from_hsla(240, 1.0f32, 0.5f32, 1.0f32);
        assert_eq!(color.red, 0);
        assert_eq!(color.green, 0);
        assert_eq!(color.blue, 255);
        assert_eq!(color.alpha, 255);

        // White: hsl(0, 0.0, 1.0)
        let color = Color::from_hsla(0, 0.0f32, 1.0f32, 1.0f32);
        assert_eq!(color.red, 255);
        assert_eq!(color.green, 255);
        assert_eq!(color.blue, 255);

        // Black: hsl(0, 0.0, 0.0)
        let color = Color::from_hsla(0, 0.0f32, 0.0f32, 1.0f32);
        assert_eq!(color.red, 0);
        assert_eq!(color.green, 0);
        assert_eq!(color.blue, 0);

        // Half alpha
        let color = Color::from_hsla(0, 1.0f32, 0.5f32, 0.5f32);
        assert_eq!(color.red, 255);
        assert_eq!(color.alpha, 127);
    }

    #[test]
    fn test_from_hsl() {
        let color = Color::from_hsl(0, 1.0f32, 0.5f32);
        assert_eq!(color.red, 255);
        assert_eq!(color.green, 0);
        assert_eq!(color.blue, 0);
        assert_eq!(color.alpha, 255);
    }

    #[test]
    fn test_display() {
        let color = Color::from_hex(0xff00ff, false);
        assert_eq!(format!("{}", color), "#ff00ffff");

        let color = Color::from_hex(0x000000, false);
        assert_eq!(format!("{}", color), "#000000ff");
    }

    #[test]
    fn test_string_color_trait() {
        let color = "ff00ff".to_string().color(false);
        assert_eq!(color.red, 255);
        assert_eq!(color.green, 0);
        assert_eq!(color.blue, 255);
        assert_eq!(color.alpha, 255);

        let color = "00ff00".color(false);
        assert_eq!(color.red, 0);
        assert_eq!(color.green, 255);
        assert_eq!(color.blue, 0);
        assert_eq!(color.alpha, 255);
    }

    #[test]
    fn test_rgb_macro() {
        let color = rgb!(255, 0, 255);
        assert_eq!(color.red, 255);
        assert_eq!(color.green, 0);
        assert_eq!(color.blue, 255);
        assert_eq!(color.alpha, 255);
    }

    #[test]
    fn test_rgba_macro() {
        let color = rgba!(255, 0, 255, 128);
        assert_eq!(color.red, 255);
        assert_eq!(color.green, 0);
        assert_eq!(color.blue, 255);
        assert_eq!(color.alpha, 128);
    }

    #[test]
    fn test_hsl_macro() {
        let color = hsl!(0, 1.0f32, 0.5f32);
        assert_eq!(color.red, 255);
        assert_eq!(color.green, 0);
        assert_eq!(color.blue, 0);
        assert_eq!(color.alpha, 255);
    }

    #[test]
    fn test_hsla_macro() {
        let color = hsla!(120, 1.0f32, 0.5f32, 0.5f32);
        assert_eq!(color.red, 0);
        assert_eq!(color.green, 255);
        assert_eq!(color.blue, 0);
        assert_eq!(color.alpha, 127);
    }

    #[test]
    fn test_to_rgb_u32() {
        let color = Color::from_rgba(255, 128, 0, 200);
        assert_eq!(color.to_rgb_u32(), 0x00FF8000);

        let color = Color::from_rgb(0, 0, 0);
        assert_eq!(color.to_rgb_u32(), 0x00000000);

        let color = Color::from_rgb(255, 255, 255);
        assert_eq!(color.to_rgb_u32(), 0x00FFFFFF);
    }

    #[test]
    fn test_to_argb_u32() {
        let color = Color::from_rgba(255, 128, 0, 200);
        assert_eq!(color.to_argb_u32(), 0xC8FF8000);

        let color = Color::from_rgb(255, 0, 255);
        assert_eq!(color.to_argb_u32(), 0xFFFF00FF);
    }

    #[test]
    fn test_to_abgr_u32() {
        let color = Color::from_rgba(255, 128, 0, 200);
        assert_eq!(color.to_abgr_u32(), 0xC80080FF);

        let color = Color::from_rgb(255, 0, 255);
        assert_eq!(color.to_abgr_u32(), 0xFFFF00FF);

        // Verify channel order: R and B are swapped compared to ARGB
        let color = Color::from_rgb(255, 0, 0); // pure red
        assert_eq!(color.to_argb_u32(), 0xFFFF0000);
        assert_eq!(color.to_abgr_u32(), 0xFF0000FF);
    }

    #[test]
    fn test_lerp_endpoints() {
        let black = Color::from_rgb(0, 0, 0);
        let white = Color::from_rgb(255, 255, 255);

        // t=0.0 returns self
        assert_eq!(black.lerp(&white, 0.0), black);
        // t=1.0 returns other
        assert_eq!(black.lerp(&white, 1.0), white);
    }

    #[test]
    fn test_lerp_midpoint() {
        let black = Color::from_rgb(0, 0, 0);
        let white = Color::from_rgb(255, 255, 255);

        let mid = black.lerp(&white, 0.5);
        assert_eq!(mid.red, 128);
        assert_eq!(mid.green, 128);
        assert_eq!(mid.blue, 128);
        assert_eq!(mid.alpha, 255);
    }

    #[test]
    fn test_lerp_quarter() {
        let red = Color::from_rgba(255, 0, 0, 255);
        let blue = Color::from_rgba(0, 0, 255, 255);

        let quarter = red.lerp(&blue, 0.25);
        assert_eq!(quarter.red, 191);
        assert_eq!(quarter.green, 0);
        assert_eq!(quarter.blue, 64);
    }

    #[test]
    fn test_lerp_alpha() {
        let opaque = Color::from_rgba(255, 0, 0, 255);
        let transparent = Color::from_rgba(255, 0, 0, 0);

        let half = opaque.lerp(&transparent, 0.5);
        assert_eq!(half.red, 255);
        assert_eq!(half.alpha, 128);
    }

    #[test]
    fn test_lerp_clamps() {
        let a = Color::from_rgb(100, 100, 100);
        let b = Color::from_rgb(200, 200, 200);

        // Values outside 0.0–1.0 are clamped
        assert_eq!(a.lerp(&b, -1.0), a);
        assert_eq!(a.lerp(&b, 2.0), b);
    }

    #[test]
    fn test_to_array() {
        let color = Color::from_rgba(255, 128, 0, 255);
        let arr = color.to_array();
        assert!((arr[0] - 1.0).abs() < f32::EPSILON);
        assert!((arr[1] - 128.0 / 255.0).abs() < f32::EPSILON);
        assert!(arr[2].abs() < f32::EPSILON);
        assert!((arr[3] - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_lerp_many_endpoints() {
        let red = Color::from_rgb(255, 0, 0);
        let green = Color::from_rgb(0, 255, 0);
        let blue = Color::from_rgb(0, 0, 255);

        assert_eq!(Color::lerp_many(&[red, green, blue], 0.0), red);
        assert_eq!(Color::lerp_many(&[red, green, blue], 1.0), blue);
    }

    #[test]
    fn test_lerp_many_midpoints() {
        let red = Color::from_rgb(255, 0, 0);
        let green = Color::from_rgb(0, 255, 0);
        let blue = Color::from_rgb(0, 0, 255);

        // t=0.5 is exactly at the second color
        assert_eq!(Color::lerp_many(&[red, green, blue], 0.5), green);

        // t=0.25 is midpoint between red and green
        let mid = Color::lerp_many(&[red, green, blue], 0.25);
        assert_eq!(mid.red, 128);
        assert_eq!(mid.green, 128);
        assert_eq!(mid.blue, 0);

        // t=0.75 is midpoint between green and blue
        let mid = Color::lerp_many(&[red, green, blue], 0.75);
        assert_eq!(mid.red, 0);
        assert_eq!(mid.green, 128);
        assert_eq!(mid.blue, 128);
    }

    #[test]
    fn test_lerp_many_single_color() {
        let red = Color::from_rgb(255, 0, 0);
        assert_eq!(Color::lerp_many(&[red], 0.5), red);
    }

    #[test]
    fn test_lerp_macro() {
        let red = Color::from_rgb(255, 0, 0);
        let green = Color::from_rgb(0, 255, 0);
        let blue = Color::from_rgb(0, 0, 255);

        assert_eq!(lerp!(0.0, red, green, blue), red);
        assert_eq!(lerp!(0.5, red, green, blue), green);
        assert_eq!(lerp!(1.0, red, green, blue), blue);
    }

    #[test]
    fn test_lerp_macro_four_colors() {
        let c1 = Color::from_rgb(255, 0, 0);
        let c2 = Color::from_rgb(0, 255, 0);
        let c3 = Color::from_rgb(0, 0, 255);
        let c4 = Color::from_rgb(255, 255, 255);

        assert_eq!(lerp!(0.0, c1, c2, c3, c4), c1);
        assert_eq!(lerp!(1.0, c1, c2, c3, c4), c4);

        // t=1/3 lands exactly on c2
        let at_c2 = lerp!(1.0 / 3.0, c1, c2, c3, c4);
        assert_eq!(at_c2.red, 0);
        assert_eq!(at_c2.green, 255);
        assert_eq!(at_c2.blue, 0);

        // t=2/3 lands exactly on c3
        let at_c3 = lerp!(2.0 / 3.0, c1, c2, c3, c4);
        assert_eq!(at_c3.red, 0);
        assert_eq!(at_c3.green, 0);
        assert_eq!(at_c3.blue, 255);
    }
}
