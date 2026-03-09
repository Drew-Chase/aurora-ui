use std::fmt::Display;

/// A trait for converting string representations of hex colors into [`Color`] values.
///
/// Implementors parse a hex string (without a leading `#`) and return the corresponding color.
pub trait StringColor {
    /// Parses `self` as a hex color string and returns the corresponding [`Color`].
    ///
    /// Returns black (`#000000`) if the string is not a valid hex number.
    fn color(&self) -> Color;
}

/// An RGBA color with 8-bit channels.
///
/// Colors can be constructed from hex values, RGB/RGBA components, or HSL/HSLA values.
/// The [`Display`] implementation formats the color as a CSS-style `#rrggbb` hex string.
#[derive(Debug, Clone)]
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
    /// Creates a new color from individual RGBA channel values.
    pub fn new(red: u8, green: u8, blue: u8, alpha: u8) -> Self {
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
    pub fn from_hex(hex: u64) -> Self {
        let has_alpha = hex > 0xffffff;
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
}

/// Formats the color as a CSS-style `#rrggbbaa` hex string
impl Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "#{:02x}{:02x}{:02x}{:02x}", self.red, self.green, self.blue, self.alpha)
    }
}

impl StringColor for String {
    fn color(&self) -> Color {
        let hex = u64::from_str_radix(self, 16).unwrap_or(0);
        Color::from_hex(hex)
    }
}

impl StringColor for &str {
    fn color(&self) -> Color {
        let hex = u64::from_str_radix(self, 16).unwrap_or(0);
        Color::from_hex(hex)
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

#[cfg(test)]
mod test {
    use crate::color::{Color, StringColor};

    #[test]
    fn test_from_hex() {
        let hex_color = 0xff00ff;
        let color = Color::from_hex(hex_color);
        assert_eq!(color.red, 255);
        assert_eq!(color.green, 0);
        assert_eq!(color.blue, 255);
        assert_eq!(color.alpha, 255);

        let hex_color = 0x00ff00;
        let color = Color::from_hex(hex_color);
        assert_eq!(color.red, 0);
        assert_eq!(color.green, 255);
        assert_eq!(color.blue, 0);
        assert_eq!(color.alpha, 255);

        let hex_color = 0xffff0080;
        let color = Color::from_hex(hex_color);

        assert_eq!(color.red, 255);
        assert_eq!(color.green, 255);
        assert_eq!(color.blue, 0);
        assert_eq!(color.alpha, 128); // 80 = 8 × 16 + 0 = 128

        let hex_color = 0x72203a;
        let color = Color::from_hex(hex_color);

        assert_eq!(color.red, 114);
        assert_eq!(color.green, 32);
        assert_eq!(color.blue, 58);
        assert_eq!(color.alpha, 255);

        let hex_color = 0x1ce783;
        let color = Color::from_hex(hex_color);

        assert_eq!(color.red, 28);
        assert_eq!(color.green, 231);
        assert_eq!(color.blue, 131);
        assert_eq!(color.alpha, 255);
    }

    #[test]
    fn test_to_hex() {
        let color = Color::from_hex(0xff00ffff);
        let hex_color = color.to_hex();
        assert_eq!(hex_color, 0xff00ffff);

        let color = Color::from_hex(0xff00ff);
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
        let color = Color::from_hex(0xff00ff);
        assert_eq!(format!("{}", color), "#ff00ffff");

        let color = Color::from_hex(0x000000);
        assert_eq!(format!("{}", color), "#000000ff");
    }

    #[test]
    fn test_string_color_trait() {
        let color = "ff00ff".to_string().color();
        assert_eq!(color.red, 255);
        assert_eq!(color.green, 0);
        assert_eq!(color.blue, 255);
        assert_eq!(color.alpha, 255);

        let color = "00ff00".color();
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
}
