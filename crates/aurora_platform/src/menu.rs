//! Native OS menu bars via [`muda`].
//!
//! This module provides a builder API for creating native menu bars that render
//! using the operating system's own menu chrome. On macOS the menu bar is
//! app-wide; on Windows and Linux it is attached per-window.
//!
//! Enable the `menu` feature to use this module.
//!
//! # Example
//!
//! ```ignore
//! use aurora_platform::menu::*;
//!
//! let menu = NativeMenu::new()
//!     .submenu(
//!         SubmenuBuilder::new("File")
//!             .item(MenuItemBuilder::new("new", "New")
//!                 .accelerator(MenuAccelerator::new(MenuModifiers::ctrl(), 'N'))
//!                 .build())
//!             .separator()
//!             .item(MenuItemBuilder::new("quit", "Quit").build())
//!             .build(),
//!     )
//!     .on_item_click(|id| {
//!         println!("clicked: {id}");
//!     });
//! ```

use std::collections::HashMap;
use std::sync::Arc;

// ---------------------------------------------------------------------------
// MenuModifiers
// ---------------------------------------------------------------------------

/// Modifier keys for a menu keyboard shortcut.
#[derive(Debug, Clone, Default)]
pub struct MenuModifiers {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    /// Command on macOS, Super/Win on Windows/Linux.
    pub meta: bool,
}

impl MenuModifiers {
    /// No modifier keys.
    pub fn none() -> Self {
        Self::default()
    }

    /// Ctrl only.
    pub fn ctrl() -> Self {
        Self {
            ctrl: true,
            ..Self::default()
        }
    }

    /// Ctrl + Shift.
    pub fn ctrl_shift() -> Self {
        Self {
            ctrl: true,
            shift: true,
            ..Self::default()
        }
    }

    /// Alt only.
    pub fn alt() -> Self {
        Self {
            alt: true,
            ..Self::default()
        }
    }

    /// Meta only (Cmd on macOS, Win on Windows).
    pub fn meta() -> Self {
        Self {
            meta: true,
            ..Self::default()
        }
    }

    /// Meta + Shift.
    pub fn meta_shift() -> Self {
        Self {
            meta: true,
            shift: true,
            ..Self::default()
        }
    }
}

// ---------------------------------------------------------------------------
// MenuAccelerator
// ---------------------------------------------------------------------------

/// A keyboard shortcut bound to a menu item.
#[derive(Debug, Clone)]
pub struct MenuAccelerator {
    pub modifiers: MenuModifiers,
    pub key: char,
}

impl MenuAccelerator {
    pub fn new(modifiers: MenuModifiers, key: char) -> Self {
        Self { modifiers, key }
    }
}

// ---------------------------------------------------------------------------
// PredefinedItem
// ---------------------------------------------------------------------------

/// Predefined OS menu items with platform-native text and behaviour.
#[derive(Debug, Clone)]
pub enum PredefinedItem {
    /// Horizontal separator line.
    Separator,
    /// "About <name>" item (opens the platform About dialog where supported).
    About(String),
    /// Quit / Exit item.
    Quit,
    /// Edit > Undo.
    Undo,
    /// Edit > Redo.
    Redo,
    /// Edit > Cut.
    Cut,
    /// Edit > Copy.
    Copy,
    /// Edit > Paste.
    Paste,
    /// Edit > Select All.
    SelectAll,
    /// Window > Minimize.
    Minimize,
    /// Window > Close Window.
    CloseWindow,
    /// Window > Fullscreen.
    Fullscreen,
}

// ---------------------------------------------------------------------------
// NativeMenuItem
// ---------------------------------------------------------------------------

/// A single entry in a native menu (item, check, separator, submenu, or
/// predefined OS item).
#[derive(Debug, Clone)]
pub enum NativeMenuItem {
    /// A regular clickable text item.
    Item {
        id: String,
        text: String,
        enabled: bool,
        accelerator: Option<MenuAccelerator>,
    },
    /// A toggleable check item.
    Check {
        id: String,
        text: String,
        enabled: bool,
        checked: bool,
        accelerator: Option<MenuAccelerator>,
    },
    /// A horizontal separator line.
    Separator,
    /// A nested submenu.
    Submenu {
        label: String,
        enabled: bool,
        items: Vec<NativeMenuItem>,
    },
    /// A predefined OS menu item.
    Predefined(PredefinedItem),
}

// ---------------------------------------------------------------------------
// MenuItemBuilder
// ---------------------------------------------------------------------------

/// Builder for a regular clickable menu item.
pub struct MenuItemBuilder {
    id: String,
    text: String,
    enabled: bool,
    accelerator: Option<MenuAccelerator>,
}

impl MenuItemBuilder {
    pub fn new(id: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            text: text.into(),
            enabled: true,
            accelerator: None,
        }
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn accelerator(mut self, accel: MenuAccelerator) -> Self {
        self.accelerator = Some(accel);
        self
    }

    pub fn build(self) -> NativeMenuItem {
        NativeMenuItem::Item {
            id: self.id,
            text: self.text,
            enabled: self.enabled,
            accelerator: self.accelerator,
        }
    }
}

// ---------------------------------------------------------------------------
// CheckMenuItemBuilder
// ---------------------------------------------------------------------------

/// Builder for a toggleable check menu item.
pub struct CheckMenuItemBuilder {
    id: String,
    text: String,
    enabled: bool,
    checked: bool,
    accelerator: Option<MenuAccelerator>,
}

impl CheckMenuItemBuilder {
    pub fn new(id: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            text: text.into(),
            enabled: true,
            checked: false,
            accelerator: None,
        }
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = checked;
        self
    }

    pub fn accelerator(mut self, accel: MenuAccelerator) -> Self {
        self.accelerator = Some(accel);
        self
    }

    pub fn build(self) -> NativeMenuItem {
        NativeMenuItem::Check {
            id: self.id,
            text: self.text,
            enabled: self.enabled,
            checked: self.checked,
            accelerator: self.accelerator,
        }
    }
}

// ---------------------------------------------------------------------------
// SubmenuBuilder
// ---------------------------------------------------------------------------

/// Builder for a nested submenu.
pub struct SubmenuBuilder {
    label: String,
    enabled: bool,
    items: Vec<NativeMenuItem>,
}

impl SubmenuBuilder {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            enabled: true,
            items: Vec::new(),
        }
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn item(mut self, item: NativeMenuItem) -> Self {
        self.items.push(item);
        self
    }

    pub fn separator(mut self) -> Self {
        self.items.push(NativeMenuItem::Separator);
        self
    }

    pub fn build(self) -> NativeMenuItem {
        NativeMenuItem::Submenu {
            label: self.label,
            enabled: self.enabled,
            items: self.items,
        }
    }
}

// ---------------------------------------------------------------------------
// NativeMenu
// ---------------------------------------------------------------------------

/// Configuration for a native OS menu bar.
///
/// Built via the builder pattern and attached to [`App`](crate::app::App) or
/// [`WindowConfig`](crate::app::WindowConfig).
/// Callback type for menu item click events.
pub type MenuClickHandler = Arc<dyn Fn(&str) + Send + Sync>;

#[derive(Clone)]
pub struct NativeMenu {
    pub(crate) items: Vec<NativeMenuItem>,
    pub(crate) on_item_click: Option<MenuClickHandler>,
}

impl NativeMenu {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            on_item_click: None,
        }
    }

    /// Appends a top-level submenu. This is the typical way to add "File",
    /// "Edit", etc. to the menu bar.
    pub fn submenu(mut self, item: NativeMenuItem) -> Self {
        self.items.push(item);
        self
    }

    /// Appends any [`NativeMenuItem`] to the top level.
    pub fn item(mut self, item: NativeMenuItem) -> Self {
        self.items.push(item);
        self
    }

    /// Appends a separator to the top level.
    pub fn separator(mut self) -> Self {
        self.items.push(NativeMenuItem::Separator);
        self
    }

    /// Sets the callback invoked when any item in this menu is clicked.
    ///
    /// The callback receives the string ID of the clicked item.
    pub fn on_item_click(mut self, f: impl Fn(&str) + Send + Sync + 'static) -> Self {
        self.on_item_click = Some(Arc::new(f));
        self
    }
}

impl Default for NativeMenu {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for NativeMenu {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NativeMenu")
            .field("items", &self.items)
            .field(
                "on_item_click",
                &self.on_item_click.as_ref().map(|_| "..."),
            )
            .finish()
    }
}

// ---------------------------------------------------------------------------
// muda conversion helpers (crate-internal)
// ---------------------------------------------------------------------------

/// Converts a [`MenuAccelerator`] to a [`muda::accelerator::Accelerator`].
pub(crate) fn to_muda_accelerator(accel: &MenuAccelerator) -> muda::accelerator::Accelerator {
    use muda::accelerator::{Accelerator, Modifiers};

    let mut mods = Modifiers::empty();
    if accel.modifiers.ctrl {
        mods |= Modifiers::CONTROL;
    }
    if accel.modifiers.alt {
        mods |= Modifiers::ALT;
    }
    if accel.modifiers.shift {
        mods |= Modifiers::SHIFT;
    }
    if accel.modifiers.meta {
        mods |= Modifiers::SUPER;
    }

    let code = char_to_code(accel.key);
    Accelerator::new(Some(mods), code)
}

/// Maps a character to a `muda::accelerator::Code`.
pub(crate) fn char_to_code(c: char) -> muda::accelerator::Code {
    use muda::accelerator::Code;
    match c.to_ascii_uppercase() {
        'A' => Code::KeyA,
        'B' => Code::KeyB,
        'C' => Code::KeyC,
        'D' => Code::KeyD,
        'E' => Code::KeyE,
        'F' => Code::KeyF,
        'G' => Code::KeyG,
        'H' => Code::KeyH,
        'I' => Code::KeyI,
        'J' => Code::KeyJ,
        'K' => Code::KeyK,
        'L' => Code::KeyL,
        'M' => Code::KeyM,
        'N' => Code::KeyN,
        'O' => Code::KeyO,
        'P' => Code::KeyP,
        'Q' => Code::KeyQ,
        'R' => Code::KeyR,
        'S' => Code::KeyS,
        'T' => Code::KeyT,
        'U' => Code::KeyU,
        'V' => Code::KeyV,
        'W' => Code::KeyW,
        'X' => Code::KeyX,
        'Y' => Code::KeyY,
        'Z' => Code::KeyZ,
        '0' => Code::Digit0,
        '1' => Code::Digit1,
        '2' => Code::Digit2,
        '3' => Code::Digit3,
        '4' => Code::Digit4,
        '5' => Code::Digit5,
        '6' => Code::Digit6,
        '7' => Code::Digit7,
        '8' => Code::Digit8,
        '9' => Code::Digit9,
        _ => Code::Unidentified,
    }
}

/// Converts a [`PredefinedItem`] to a `muda::PredefinedMenuItem`.
pub(crate) fn to_muda_predefined(item: &PredefinedItem) -> Box<dyn muda::IsMenuItem> {
    use muda::PredefinedMenuItem;
    match item {
        PredefinedItem::Separator => Box::new(PredefinedMenuItem::separator()),
        PredefinedItem::About(name) => {
            let meta = Some(muda::AboutMetadata {
                name: Some(name.clone()),
                ..Default::default()
            });
            Box::new(PredefinedMenuItem::about(None, meta))
        }
        PredefinedItem::Quit => Box::new(PredefinedMenuItem::quit(None)),
        PredefinedItem::Undo => Box::new(PredefinedMenuItem::undo(None)),
        PredefinedItem::Redo => Box::new(PredefinedMenuItem::redo(None)),
        PredefinedItem::Cut => Box::new(PredefinedMenuItem::cut(None)),
        PredefinedItem::Copy => Box::new(PredefinedMenuItem::copy(None)),
        PredefinedItem::Paste => Box::new(PredefinedMenuItem::paste(None)),
        PredefinedItem::SelectAll => Box::new(PredefinedMenuItem::select_all(None)),
        PredefinedItem::Minimize => Box::new(PredefinedMenuItem::minimize(None)),
        PredefinedItem::CloseWindow => Box::new(PredefinedMenuItem::close_window(None)),
        PredefinedItem::Fullscreen => Box::new(PredefinedMenuItem::fullscreen(None)),
    }
}

/// Recursively adds [`NativeMenuItem`]s to a `muda::Submenu`, populating
/// `id_map` for each item that has a user-assigned ID.
pub(crate) fn add_items_to_submenu(
    submenu: &muda::Submenu,
    items: &[NativeMenuItem],
    id_map: &mut HashMap<muda::MenuId, String>,
) {
    for item in items {
        match item {
            NativeMenuItem::Item {
                id,
                text,
                enabled,
                accelerator,
            } => {
                let accel = accelerator.as_ref().map(to_muda_accelerator);
                let mi = muda::MenuItem::new(text, *enabled, accel);
                id_map.insert(mi.id().clone(), id.clone());
                let _ = submenu.append(&mi);
            }
            NativeMenuItem::Check {
                id,
                text,
                enabled,
                checked,
                accelerator,
            } => {
                let accel = accelerator.as_ref().map(to_muda_accelerator);
                let ci = muda::CheckMenuItem::new(text, *enabled, *checked, accel);
                id_map.insert(ci.id().clone(), id.clone());
                let _ = submenu.append(&ci);
            }
            NativeMenuItem::Separator => {
                let _ = submenu.append(&muda::PredefinedMenuItem::separator());
            }
            NativeMenuItem::Submenu {
                label,
                enabled,
                items: children,
            } => {
                let sub = muda::Submenu::new(label, *enabled);
                add_items_to_submenu(&sub, children, id_map);
                let _ = submenu.append(&sub);
            }
            NativeMenuItem::Predefined(p) => {
                let mi = to_muda_predefined(p);
                let _ = submenu.append(mi.as_ref());
            }
        }
    }
}

/// Builds a [`muda::Menu`] from a [`NativeMenu`] configuration.
///
/// Returns the menu and a mapping from `muda::MenuId` to the user-assigned
/// string IDs. Only `Item` and `Check` entries produce ID mappings.
pub(crate) fn build_muda_menu(config: &NativeMenu) -> (muda::Menu, HashMap<muda::MenuId, String>) {
    let menu = muda::Menu::new();
    let mut id_map = HashMap::new();

    for item in &config.items {
        match item {
            NativeMenuItem::Submenu {
                label,
                enabled,
                items,
            } => {
                let sub = muda::Submenu::new(label, *enabled);
                add_items_to_submenu(&sub, items, &mut id_map);
                let _ = menu.append(&sub);
            }
            NativeMenuItem::Item {
                id,
                text,
                enabled,
                accelerator,
            } => {
                let accel = accelerator.as_ref().map(to_muda_accelerator);
                let mi = muda::MenuItem::new(text, *enabled, accel);
                id_map.insert(mi.id().clone(), id.clone());
                let _ = menu.append(&mi);
            }
            NativeMenuItem::Check {
                id,
                text,
                enabled,
                checked,
                accelerator,
            } => {
                let accel = accelerator.as_ref().map(to_muda_accelerator);
                let ci = muda::CheckMenuItem::new(text, *enabled, *checked, accel);
                id_map.insert(ci.id().clone(), id.clone());
                let _ = menu.append(&ci);
            }
            NativeMenuItem::Separator => {
                let _ = menu.append(&muda::PredefinedMenuItem::separator());
            }
            NativeMenuItem::Predefined(p) => {
                let mi = to_muda_predefined(p);
                let _ = menu.append(mi.as_ref());
            }
        }
    }

    (menu, id_map)
}

// ---------------------------------------------------------------------------
// Platform-specific menu attachment
// ---------------------------------------------------------------------------

/// Attaches a `muda::Menu` to a window as its native menu bar.
#[cfg(target_os = "windows")]
pub(crate) fn attach_menu_to_window(menu: &muda::Menu, window: &winit::window::Window) {
    use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};
    if let Ok(handle) = window.window_handle()
        && let RawWindowHandle::Win32(h) = handle.as_ref()
    {
        let hwnd = h.hwnd.get();
        unsafe {
            let _ = menu.init_for_hwnd(hwnd as _);
        }
    }
}

/// Attaches a `muda::Menu` as the app-wide menu bar on macOS.
#[cfg(target_os = "macos")]
pub(crate) fn attach_menu_to_app(menu: &muda::Menu) {
    menu.init_for_nsapp();
}

/// Attaches a `muda::Menu` to a GTK window on Linux.
#[cfg(target_os = "linux")]
pub(crate) fn attach_menu_to_window(
    menu: &muda::Menu,
    window: &winit::window::Window,
    vbox: &gtk::Box,
) {
    use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};
    if let Ok(handle) = window.window_handle() {
        // On Linux muda requires a GTK window and container. This is a
        // best-effort integration; if the handle is not a GTK surface the
        // menu will silently not appear.
        unsafe {
            let _ = menu.init_for_gtk_window(
                gtk::prelude::WidgetExt::window(vbox).as_ref().unwrap(),
                Some(vbox),
            );
        }
    }
}

// Fallback for platforms without native menu bar support.
#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
pub(crate) fn attach_menu_to_window(_menu: &muda::Menu, _window: &winit::window::Window) {
    log::warn!("Native menu bars are not supported on this platform");
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn menu_item_builder_sets_fields() {
        let item = MenuItemBuilder::new("open", "Open File")
            .enabled(false)
            .accelerator(MenuAccelerator::new(MenuModifiers::ctrl(), 'O'))
            .build();

        match item {
            NativeMenuItem::Item {
                id,
                text,
                enabled,
                accelerator,
            } => {
                assert_eq!(id, "open");
                assert_eq!(text, "Open File");
                assert!(!enabled);
                let accel = accelerator.unwrap();
                assert!(accel.modifiers.ctrl);
                assert!(!accel.modifiers.alt);
                assert_eq!(accel.key, 'O');
            }
            _ => panic!("expected NativeMenuItem::Item"),
        }
    }

    #[test]
    fn check_menu_item_builder_defaults() {
        let item = CheckMenuItemBuilder::new("dark", "Dark Mode").build();

        match item {
            NativeMenuItem::Check {
                enabled, checked, ..
            } => {
                assert!(enabled, "should default to enabled");
                assert!(!checked, "should default to unchecked");
            }
            _ => panic!("expected NativeMenuItem::Check"),
        }
    }

    #[test]
    fn submenu_builder_nesting() {
        let sub = SubmenuBuilder::new("File")
            .item(MenuItemBuilder::new("new", "New").build())
            .separator()
            .item(
                SubmenuBuilder::new("Recent")
                    .item(MenuItemBuilder::new("r1", "file1.txt").build())
                    .build(),
            )
            .build();

        match sub {
            NativeMenuItem::Submenu { label, items, .. } => {
                assert_eq!(label, "File");
                assert_eq!(items.len(), 3);
                assert!(matches!(items[0], NativeMenuItem::Item { .. }));
                assert!(matches!(items[1], NativeMenuItem::Separator));
                assert!(matches!(items[2], NativeMenuItem::Submenu { .. }));
            }
            _ => panic!("expected NativeMenuItem::Submenu"),
        }
    }

    #[test]
    fn accelerator_creation() {
        let accel = MenuAccelerator::new(MenuModifiers::ctrl_shift(), 'S');
        assert!(accel.modifiers.ctrl);
        assert!(accel.modifiers.shift);
        assert!(!accel.modifiers.alt);
        assert!(!accel.modifiers.meta);
        assert_eq!(accel.key, 'S');
    }

    #[test]
    fn default_native_menu_is_empty() {
        let menu = NativeMenu::new();
        assert!(menu.items.is_empty());
        assert!(menu.on_item_click.is_none());
    }

    #[test]
    fn native_menu_builder_chain() {
        let menu = NativeMenu::new()
            .item(MenuItemBuilder::new("a", "A").build())
            .separator()
            .submenu(SubmenuBuilder::new("Sub").build());

        assert_eq!(menu.items.len(), 3);
        assert!(matches!(menu.items[0], NativeMenuItem::Item { .. }));
        assert!(matches!(menu.items[1], NativeMenuItem::Separator));
        assert!(matches!(menu.items[2], NativeMenuItem::Submenu { .. }));
    }

    #[test]
    fn build_muda_menu_maps_all_ids() {
        let menu = NativeMenu::new()
            .submenu(
                SubmenuBuilder::new("File")
                    .item(MenuItemBuilder::new("new", "New").build())
                    .item(MenuItemBuilder::new("open", "Open").build())
                    .separator()
                    .item(CheckMenuItemBuilder::new("wrap", "Word Wrap").build())
                    .build(),
            )
            .item(MenuItemBuilder::new("top", "Top Level").build());

        let (_muda_menu, id_map) = build_muda_menu(&menu);
        let user_ids: Vec<&str> = id_map.values().map(|s| s.as_str()).collect();
        assert!(user_ids.contains(&"new"));
        assert!(user_ids.contains(&"open"));
        assert!(user_ids.contains(&"wrap"));
        assert!(user_ids.contains(&"top"));
        assert_eq!(id_map.len(), 4);
    }

    #[test]
    fn predefined_items_produce_no_id_mapping() {
        let menu = NativeMenu::new().item(NativeMenuItem::Predefined(PredefinedItem::Quit));
        let (_muda_menu, id_map) = build_muda_menu(&menu);
        assert!(id_map.is_empty());
    }

    #[test]
    fn separator_produces_no_id_mapping() {
        let menu = NativeMenu::new().separator();
        let (_muda_menu, id_map) = build_muda_menu(&menu);
        assert!(id_map.is_empty());
    }
}
