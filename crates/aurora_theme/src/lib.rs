//! Runtime theme profile management for Aurora UI.
//!
//! This crate provides the runtime infrastructure for theming:
//! - A global profile registry initialized by the `aurora_theming::config!()` macro
//! - Zero-cost accessors for colors, edges, corners, and f32 values
//! - Profile switching via [`set_profile`]
//!
//! Widgets read from the active profile through the [`colors`] module
//! (well-known keys) or through index-based access for custom keys.

use aurora_core::color::Color;
use aurora_core::geometry::corners::Corners;
use aurora_core::geometry::edges::Edges;
use std::cell::Cell;
use std::sync::OnceLock;

/// Raw theme data registered by the `config!()` macro.
///
/// Each `Vec` holds one entry per profile. Each entry is an array of values —
/// index positions are assigned by the proc macro and match the generated
/// accessor functions.
pub struct ThemeData {
    pub profile_names: Vec<&'static str>,
    pub colors: Vec<Vec<Color>>,
    pub edges: Vec<Vec<Edges>>,
    pub corners: Vec<Vec<Corners>>,
    pub values: Vec<Vec<f32>>,
}

static THEME: OnceLock<ThemeData> = OnceLock::new();

thread_local! {
    static CURRENT_PROFILE: Cell<usize> = const { Cell::new(0) };
}

/// Registers theme data. Called once by generated `config!()` code.
pub fn init(data: ThemeData) {
    let _ = THEME.set(data);
}

/// Sets the active profile by index.
pub fn set_profile(id: usize) {
    CURRENT_PROFILE.with(|c| c.set(id));
}

/// Returns the active profile index.
pub fn current_profile() -> usize {
    CURRENT_PROFILE.with(|c| c.get())
}

/// Returns the name of the active profile.
pub fn current_profile_name() -> &'static str {
    let id = current_profile();
    THEME
        .get()
        .and_then(|t| t.profile_names.get(id).copied())
        .unwrap_or("default")
}

/// Returns the number of registered profiles.
pub fn profile_count() -> usize {
    THEME.get().map_or(0, |t| t.profile_names.len())
}

/// Returns the name of profile at `index`.
pub fn profile_name(index: usize) -> Option<&'static str> {
    THEME.get().and_then(|t| t.profile_names.get(index).copied())
}

/// Looks up a color by slot index in the active profile.
pub fn color(index: usize) -> Color {
    let profile = current_profile();
    THEME
        .get()
        .and_then(|t| t.colors.get(profile))
        .and_then(|c| c.get(index).copied())
        .unwrap_or(Color::BLACK)
}

/// Looks up edges by slot index in the active profile.
pub fn edges(index: usize) -> Edges {
    let profile = current_profile();
    THEME
        .get()
        .and_then(|t| t.edges.get(profile))
        .and_then(|e| e.get(index).copied())
        .unwrap_or(Edges::zero())
}

/// Looks up corners by slot index in the active profile.
pub fn corners(index: usize) -> Corners {
    let profile = current_profile();
    THEME
        .get()
        .and_then(|t| t.corners.get(profile))
        .and_then(|c| c.get(index).copied())
        .unwrap_or(Corners::zero())
}

/// Looks up an f32 value by slot index in the active profile.
pub fn value(index: usize) -> f32 {
    let profile = current_profile();
    THEME
        .get()
        .and_then(|t| t.values.get(profile))
        .and_then(|v| v.get(index).copied())
        .unwrap_or(0.0)
}

// ---------------------------------------------------------------------------
// Well-known color slots (indices match the default theme)
// ---------------------------------------------------------------------------

/// Well-known color slot indices used by Aurora widgets.
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
}

/// Convenience accessors for well-known theme colors.
///
/// These return the value from the active profile, falling back to the
/// hardcoded dark-theme default if no theme has been registered.
pub mod colors {
    use super::*;

    // Hardcoded fallbacks (dark theme) when no config!() is used.
    const FALLBACKS: [Color; 26] = [
        Color::new(9, 9, 11, 255),       // BACKGROUND
        Color::new(250, 250, 250, 255),   // FOREGROUND
        Color::new(39, 39, 42, 255),      // MUTED
        Color::new(161, 161, 170, 255),   // MUTED_FOREGROUND
        Color::new(39, 39, 42, 255),      // BORDER
        Color::new(39, 39, 42, 255),      // INPUT_BORDER
        Color::new(212, 212, 216, 255),   // RING
        Color::new(250, 250, 250, 255),   // PRIMARY
        Color::new(9, 9, 11, 255),        // PRIMARY_FOREGROUND
        Color::new(39, 39, 42, 255),      // SECONDARY
        Color::new(250, 250, 250, 255),   // SECONDARY_FOREGROUND
        Color::new(39, 39, 42, 255),      // ACCENT
        Color::new(250, 250, 250, 255),   // ACCENT_FOREGROUND
        Color::new(127, 29, 29, 255),     // DESTRUCTIVE
        Color::new(250, 250, 250, 255),   // DESTRUCTIVE_FOREGROUND
        Color::new(34, 197, 94, 255),     // SUCCESS
        Color::new(250, 250, 250, 255),   // SUCCESS_FOREGROUND
        Color::new(234, 179, 8, 255),     // WARNING
        Color::new(9, 9, 11, 255),        // WARNING_FOREGROUND
        Color::new(59, 130, 246, 255),    // INFO
        Color::new(250, 250, 250, 255),   // INFO_FOREGROUND
        Color::new(9, 9, 11, 255),        // CARD
        Color::new(250, 250, 250, 255),   // CARD_FOREGROUND
        Color::new(9, 9, 11, 255),        // POPOVER
        Color::new(250, 250, 250, 255),   // POPOVER_FOREGROUND
        Color::new(0, 0, 0, 128),         // OVERLAY
    ];

    fn get(slot: usize) -> Color {
        let profile = current_profile();
        THEME
            .get()
            .and_then(|t| t.colors.get(profile))
            .and_then(|c| c.get(slot).copied())
            .unwrap_or(FALLBACKS[slot])
    }

    pub fn background() -> Color { get(slots::BACKGROUND) }
    pub fn foreground() -> Color { get(slots::FOREGROUND) }
    pub fn muted() -> Color { get(slots::MUTED) }
    pub fn muted_foreground() -> Color { get(slots::MUTED_FOREGROUND) }
    pub fn border() -> Color { get(slots::BORDER) }
    pub fn input_border() -> Color { get(slots::INPUT_BORDER) }
    pub fn ring() -> Color { get(slots::RING) }
    pub fn primary() -> Color { get(slots::PRIMARY) }
    pub fn primary_foreground() -> Color { get(slots::PRIMARY_FOREGROUND) }
    pub fn secondary() -> Color { get(slots::SECONDARY) }
    pub fn secondary_foreground() -> Color { get(slots::SECONDARY_FOREGROUND) }
    pub fn accent() -> Color { get(slots::ACCENT) }
    pub fn accent_foreground() -> Color { get(slots::ACCENT_FOREGROUND) }
    pub fn destructive() -> Color { get(slots::DESTRUCTIVE) }
    pub fn destructive_foreground() -> Color { get(slots::DESTRUCTIVE_FOREGROUND) }
    pub fn success() -> Color { get(slots::SUCCESS) }
    pub fn success_foreground() -> Color { get(slots::SUCCESS_FOREGROUND) }
    pub fn warning() -> Color { get(slots::WARNING) }
    pub fn warning_foreground() -> Color { get(slots::WARNING_FOREGROUND) }
    pub fn info() -> Color { get(slots::INFO) }
    pub fn info_foreground() -> Color { get(slots::INFO_FOREGROUND) }
    pub fn card() -> Color { get(slots::CARD) }
    pub fn card_foreground() -> Color { get(slots::CARD_FOREGROUND) }
    pub fn popover() -> Color { get(slots::POPOVER) }
    pub fn popover_foreground() -> Color { get(slots::POPOVER_FOREGROUND) }
    pub fn overlay() -> Color { get(slots::OVERLAY) }
}
