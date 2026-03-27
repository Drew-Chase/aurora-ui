//! Runtime profile switching and default theme for Aurora UI.
//!
//! Provides:
//! - A thread-local profile index switched via [`set_profile`]
//! - A built-in default theme (light mode) that widgets use as baseline
//! - Re-exports of core types used by generated theme code
//!
//! The default theme can be overridden by `aurora_theming::config!()`.

pub use aurora_core;
pub use aurora_core::color::Color;
pub use aurora_core::geometry::corners::Corners;
pub use aurora_core::geometry::edges::Edges;

use std::cell::Cell;
use std::sync::OnceLock;

// ---------------------------------------------------------------------------
// Profile switching
// ---------------------------------------------------------------------------

thread_local! {
    static CURRENT_PROFILE: Cell<usize> = const { Cell::new(0) };
}

/// Sets the active profile by index.
pub fn set_profile(id: usize) {
    CURRENT_PROFILE.with(|c| c.set(id));
}

/// Returns the active profile index.
pub fn current_profile() -> usize {
    CURRENT_PROFILE.with(|c| c.get())
}

// ---------------------------------------------------------------------------
// Default theme (light mode baseline)
// ---------------------------------------------------------------------------

/// The built-in default colors (light theme, shadcn/ui zinc).
/// These are used when no user theme is loaded, or as fallback.
pub static DEFAULT_COLORS: [Color; 38] = [
    Color::new(255, 255, 255, 255),   //  0 BACKGROUND
    Color::new(9, 9, 11, 255),        //  1 FOREGROUND
    Color::new(244, 244, 245, 255),   //  2 MUTED
    Color::new(113, 113, 122, 255),   //  3 MUTED_FOREGROUND
    Color::new(228, 228, 231, 255),   //  4 BORDER
    Color::new(228, 228, 231, 255),   //  5 INPUT_BORDER
    Color::new(9, 9, 11, 255),        //  6 RING
    Color::new(9, 9, 11, 255),        //  7 PRIMARY
    Color::new(250, 250, 250, 255),   //  8 PRIMARY_FOREGROUND
    Color::new(244, 244, 245, 255),   //  9 SECONDARY
    Color::new(9, 9, 11, 255),        // 10 SECONDARY_FOREGROUND
    Color::new(244, 244, 245, 255),   // 11 ACCENT
    Color::new(9, 9, 11, 255),        // 12 ACCENT_FOREGROUND
    Color::new(239, 68, 68, 255),     // 13 DESTRUCTIVE
    Color::new(250, 250, 250, 255),   // 14 DESTRUCTIVE_FOREGROUND
    Color::new(34, 197, 94, 255),     // 15 SUCCESS
    Color::new(250, 250, 250, 255),   // 16 SUCCESS_FOREGROUND
    Color::new(234, 179, 8, 255),     // 17 WARNING
    Color::new(66, 32, 6, 255),       // 18 WARNING_FOREGROUND
    Color::new(59, 130, 246, 255),    // 19 INFO
    Color::new(250, 250, 250, 255),   // 20 INFO_FOREGROUND
    Color::new(255, 255, 255, 255),   // 21 CARD
    Color::new(9, 9, 11, 255),        // 22 CARD_FOREGROUND
    Color::new(255, 255, 255, 255),   // 23 POPOVER
    Color::new(9, 9, 11, 255),        // 24 POPOVER_FOREGROUND
    Color::new(0, 0, 0, 128),         // 25 OVERLAY
    // Syntax highlighting (One Dark defaults)
    Color::new(198, 120, 221, 255),   // 26 SYNTAX_KEYWORD     (#c678dd)
    Color::new(152, 195, 121, 255),   // 27 SYNTAX_STRING      (#98c379)
    Color::new(92, 99, 112, 255),     // 28 SYNTAX_COMMENT     (#5c6370)
    Color::new(209, 154, 102, 255),   // 29 SYNTAX_NUMBER      (#d19a66)
    Color::new(97, 175, 239, 255),    // 30 SYNTAX_FUNCTION    (#61afef)
    Color::new(229, 192, 123, 255),   // 31 SYNTAX_TYPE        (#e5c07b)
    Color::new(86, 182, 194, 255),    // 32 SYNTAX_OPERATOR    (#56b6c2)
    Color::new(171, 178, 191, 255),   // 33 SYNTAX_PUNCTUATION (#abb2bf)
    Color::new(209, 154, 102, 255),   // 34 SYNTAX_ATTRIBUTE   (#d19a66)
    Color::new(224, 108, 117, 255),   // 35 SYNTAX_TAG         (#e06c75)
    Color::new(209, 154, 102, 255),   // 36 SYNTAX_CONSTANT    (#d19a66)
    Color::new(171, 178, 191, 255),   // 37 SYNTAX_PLAIN       (#abb2bf)
];

/// Slot indices for the well-known colors.
pub mod slots {
    pub const BACKGROUND: usize = 0;
    pub const FOREGROUND: usize = 1;
    pub const MUTED: usize = 2;
    pub const MUTED_FOREGROUND: usize = 3;
    pub const BORDER: usize = 4;
    pub const INPUT_BORDER: usize = 5;
    pub const RING: usize = 6;
    pub const PRIMARY: usize = 7;
    pub const PRIMARY_FOREGROUND: usize = 8;
    pub const SECONDARY: usize = 9;
    pub const SECONDARY_FOREGROUND: usize = 10;
    pub const ACCENT: usize = 11;
    pub const ACCENT_FOREGROUND: usize = 12;
    pub const DESTRUCTIVE: usize = 13;
    pub const DESTRUCTIVE_FOREGROUND: usize = 14;
    pub const SUCCESS: usize = 15;
    pub const SUCCESS_FOREGROUND: usize = 16;
    pub const WARNING: usize = 17;
    pub const WARNING_FOREGROUND: usize = 18;
    pub const INFO: usize = 19;
    pub const INFO_FOREGROUND: usize = 20;
    pub const CARD: usize = 21;
    pub const CARD_FOREGROUND: usize = 22;
    pub const POPOVER: usize = 23;
    pub const POPOVER_FOREGROUND: usize = 24;
    pub const OVERLAY: usize = 25;
    // Syntax highlighting
    pub const SYNTAX_KEYWORD: usize = 26;
    pub const SYNTAX_STRING: usize = 27;
    pub const SYNTAX_COMMENT: usize = 28;
    pub const SYNTAX_NUMBER: usize = 29;
    pub const SYNTAX_FUNCTION: usize = 30;
    pub const SYNTAX_TYPE: usize = 31;
    pub const SYNTAX_OPERATOR: usize = 32;
    pub const SYNTAX_PUNCTUATION: usize = 33;
    pub const SYNTAX_ATTRIBUTE: usize = 34;
    pub const SYNTAX_TAG: usize = 35;
    pub const SYNTAX_CONSTANT: usize = 36;
    pub const SYNTAX_PLAIN: usize = 37;
}

// ---------------------------------------------------------------------------
// User theme override registry
// ---------------------------------------------------------------------------

/// User-provided theme color arrays (one per profile), set by config!().
struct ThemeOverride {
    colors: Vec<Vec<Color>>,
}

static USER_THEME: OnceLock<ThemeOverride> = OnceLock::new();

/// Register user theme color overrides. Called by generated config!() code.
pub fn register_colors(profiles: Vec<Vec<Color>>) {
    if USER_THEME.set(ThemeOverride { colors: profiles }).is_err() {
        log::warn!("Theme colors already registered; ignoring duplicate registration");
    }
}

/// Look up a color by slot, using user theme if available, else default.
pub fn color(slot: usize) -> Color {
    let profile = current_profile();
    USER_THEME
        .get()
        .and_then(|t| t.colors.get(profile))
        .and_then(|c| c.get(slot).copied())
        .unwrap_or_else(|| {
            DEFAULT_COLORS.get(slot).copied().unwrap_or_else(|| {
                log::warn!("Theme color slot {slot} out of bounds, using black");
                Color::BLACK
            })
        })
}
