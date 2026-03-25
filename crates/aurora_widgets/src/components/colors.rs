use aurora_core::color::Color;

// All color accessors delegate to the aurora_theme runtime.
// When a theme has been registered via aurora_theming::config!(), values come
// from the active profile. Otherwise, the built-in dark-theme fallbacks are
// used automatically.

pub fn background() -> Color { aurora_theme::colors::background() }
pub fn foreground() -> Color { aurora_theme::colors::foreground() }
pub fn muted() -> Color { aurora_theme::colors::muted() }
pub fn muted_foreground() -> Color { aurora_theme::colors::muted_foreground() }
pub fn border() -> Color { aurora_theme::colors::border() }
pub fn input_border() -> Color { aurora_theme::colors::input_border() }
pub fn ring() -> Color { aurora_theme::colors::ring() }
pub fn primary() -> Color { aurora_theme::colors::primary() }
pub fn primary_foreground() -> Color { aurora_theme::colors::primary_foreground() }
pub fn secondary() -> Color { aurora_theme::colors::secondary() }
pub fn secondary_foreground() -> Color { aurora_theme::colors::secondary_foreground() }
pub fn accent() -> Color { aurora_theme::colors::accent() }
pub fn accent_foreground() -> Color { aurora_theme::colors::accent_foreground() }
pub fn destructive() -> Color { aurora_theme::colors::destructive() }
pub fn destructive_foreground() -> Color { aurora_theme::colors::destructive_foreground() }
pub fn success() -> Color { aurora_theme::colors::success() }
pub fn success_foreground() -> Color { aurora_theme::colors::success_foreground() }
pub fn warning() -> Color { aurora_theme::colors::warning() }
pub fn warning_foreground() -> Color { aurora_theme::colors::warning_foreground() }
pub fn info() -> Color { aurora_theme::colors::info() }
pub fn info_foreground() -> Color { aurora_theme::colors::info_foreground() }
pub fn card() -> Color { aurora_theme::colors::card() }
pub fn card_foreground() -> Color { aurora_theme::colors::card_foreground() }
pub fn popover() -> Color { aurora_theme::colors::popover() }
pub fn popover_foreground() -> Color { aurora_theme::colors::popover_foreground() }
pub fn overlay() -> Color { aurora_theme::colors::overlay() }

// Legacy constants — these call the theme functions so existing code using
// `colors::FOREGROUND` continues to compile. Prefer the function form for
// new code since the constant form cannot change at runtime.
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
