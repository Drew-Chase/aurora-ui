use aurora_core::color::Color;
use aurora_theme::slots;

/// All color accessors read from the active theme profile at runtime.
/// When a user theme is registered via aurora_theming::config!(), values
/// come from the active profile. Otherwise the built-in light-mode
/// defaults are used.

pub fn background() -> Color { aurora_theme::color(slots::BACKGROUND) }
pub fn foreground() -> Color { aurora_theme::color(slots::FOREGROUND) }
pub fn muted() -> Color { aurora_theme::color(slots::MUTED) }
pub fn muted_foreground() -> Color { aurora_theme::color(slots::MUTED_FOREGROUND) }
pub fn border() -> Color { aurora_theme::color(slots::BORDER) }
pub fn input_border() -> Color { aurora_theme::color(slots::INPUT_BORDER) }
pub fn ring() -> Color { aurora_theme::color(slots::RING) }
pub fn primary() -> Color { aurora_theme::color(slots::PRIMARY) }
pub fn primary_foreground() -> Color { aurora_theme::color(slots::PRIMARY_FOREGROUND) }
pub fn secondary() -> Color { aurora_theme::color(slots::SECONDARY) }
pub fn secondary_foreground() -> Color { aurora_theme::color(slots::SECONDARY_FOREGROUND) }
pub fn accent() -> Color { aurora_theme::color(slots::ACCENT) }
pub fn accent_foreground() -> Color { aurora_theme::color(slots::ACCENT_FOREGROUND) }
pub fn destructive() -> Color { aurora_theme::color(slots::DESTRUCTIVE) }
pub fn destructive_foreground() -> Color { aurora_theme::color(slots::DESTRUCTIVE_FOREGROUND) }
pub fn success() -> Color { aurora_theme::color(slots::SUCCESS) }
pub fn success_foreground() -> Color { aurora_theme::color(slots::SUCCESS_FOREGROUND) }
pub fn warning() -> Color { aurora_theme::color(slots::WARNING) }
pub fn warning_foreground() -> Color { aurora_theme::color(slots::WARNING_FOREGROUND) }
pub fn info() -> Color { aurora_theme::color(slots::INFO) }
pub fn info_foreground() -> Color { aurora_theme::color(slots::INFO_FOREGROUND) }
pub fn card() -> Color { aurora_theme::color(slots::CARD) }
pub fn card_foreground() -> Color { aurora_theme::color(slots::CARD_FOREGROUND) }
pub fn popover() -> Color { aurora_theme::color(slots::POPOVER) }
pub fn popover_foreground() -> Color { aurora_theme::color(slots::POPOVER_FOREGROUND) }
pub fn overlay() -> Color { aurora_theme::color(slots::OVERLAY) }
