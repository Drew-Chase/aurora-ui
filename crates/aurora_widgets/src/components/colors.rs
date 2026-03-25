use aurora_core::color::Color;

// Default dark-theme constants used by widgets.
// When a user sets up aurora_theming::config!(), their generated
// theme::colors module overrides these at the call site.
pub const BACKGROUND: Color = Color::new(9, 9, 11, 255);
pub const FOREGROUND: Color = Color::new(250, 250, 250, 255);
pub const MUTED: Color = Color::new(39, 39, 42, 255);
pub const MUTED_FOREGROUND: Color = Color::new(161, 161, 170, 255);
pub const BORDER: Color = Color::new(39, 39, 42, 255);
pub const INPUT_BORDER: Color = Color::new(39, 39, 42, 255);
pub const RING: Color = Color::new(212, 212, 216, 255);
pub const PRIMARY: Color = Color::new(250, 250, 250, 255);
pub const PRIMARY_FOREGROUND: Color = Color::new(9, 9, 11, 255);
pub const SECONDARY: Color = Color::new(39, 39, 42, 255);
pub const SECONDARY_FOREGROUND: Color = Color::new(250, 250, 250, 255);
pub const ACCENT: Color = Color::new(39, 39, 42, 255);
pub const ACCENT_FOREGROUND: Color = Color::new(250, 250, 250, 255);
pub const DESTRUCTIVE: Color = Color::new(127, 29, 29, 255);
pub const DESTRUCTIVE_FOREGROUND: Color = Color::new(250, 250, 250, 255);
pub const SUCCESS: Color = Color::new(34, 197, 94, 255);
pub const SUCCESS_FOREGROUND: Color = Color::new(250, 250, 250, 255);
pub const WARNING: Color = Color::new(234, 179, 8, 255);
pub const WARNING_FOREGROUND: Color = Color::new(9, 9, 11, 255);
pub const INFO: Color = Color::new(59, 130, 246, 255);
pub const INFO_FOREGROUND: Color = Color::new(250, 250, 250, 255);
pub const CARD: Color = Color::new(9, 9, 11, 255);
pub const CARD_FOREGROUND: Color = Color::new(250, 250, 250, 255);
pub const POPOVER: Color = Color::new(9, 9, 11, 255);
pub const POPOVER_FOREGROUND: Color = Color::new(250, 250, 250, 255);
pub const OVERLAY: Color = Color::new(0, 0, 0, 128);
