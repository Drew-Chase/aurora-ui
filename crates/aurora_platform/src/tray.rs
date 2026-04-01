//! System tray icon support via [`tray_icon`].
//!
//! Enable the `tray` feature to use this module. Enabling `tray` automatically
//! enables the `menu` feature for tray context menus.
//!
//! # Example
//!
//! ```ignore
//! use aurora_platform::tray::*;
//! use aurora_platform::menu::*;
//!
//! let tray = TrayConfig::new()
//!     .tooltip("My App")
//!     .menu(
//!         NativeMenu::new()
//!             .item(MenuItemBuilder::new("show", "Show Window").build())
//!             .separator()
//!             .item(MenuItemBuilder::new("quit", "Quit").build()),
//!     )
//!     .on_click(|interaction| {
//!         println!("Tray: {interaction:?}");
//!     });
//! ```

use std::sync::Arc;

use crate::app::IconData;
use crate::menu::NativeMenu;

// ---------------------------------------------------------------------------
// TrayInteraction
// ---------------------------------------------------------------------------

/// Describes how the user interacted with the system tray icon.
#[derive(Debug, Clone)]
pub enum TrayInteraction {
    /// Single left-click.
    Click,
    /// Double left-click.
    DoubleClick,
    /// Right-click (usually opens context menu automatically).
    RightClick,
}

// ---------------------------------------------------------------------------
// TrayConfig
// ---------------------------------------------------------------------------

/// Configuration for a system tray icon.
///
/// Built via the builder pattern and attached to [`App`](crate::app::App).
#[derive(Clone)]
pub struct TrayConfig {
    pub(crate) icon: Option<IconData>,
    pub(crate) tooltip: Option<String>,
    pub(crate) menu: Option<NativeMenu>,
    pub(crate) on_click: Option<Arc<dyn Fn(TrayInteraction) + Send + Sync>>,
}

impl TrayConfig {
    pub fn new() -> Self {
        Self {
            icon: None,
            tooltip: None,
            menu: None,
            on_click: None,
        }
    }

    /// Sets the tray icon from pre-decoded RGBA pixel data.
    ///
    /// # Panics
    ///
    /// Panics if `rgba.len() != width * height * 4` or if dimensions are zero.
    pub fn icon_rgba(mut self, rgba: Vec<u8>, width: u32, height: u32) -> Self {
        assert!(
            width > 0 && height > 0,
            "icon_rgba: dimensions must be non-zero"
        );
        let expected = (width as usize)
            .checked_mul(height as usize)
            .and_then(|n| n.checked_mul(4))
            .expect("icon_rgba: dimensions overflow");
        assert_eq!(
            rgba.len(),
            expected,
            "icon_rgba: buffer length must be width * height * 4"
        );
        self.icon = Some(IconData {
            rgba,
            width,
            height,
        });
        self
    }

    /// Sets the tray icon by decoding an image (PNG, JPEG) from raw bytes.
    ///
    /// Typically used with `include_bytes!("icon.png")`.
    #[cfg(feature = "image")]
    pub fn icon(mut self, bytes: &[u8]) -> Self {
        match image::load_from_memory(bytes) {
            Ok(img) => {
                let rgba = img.to_rgba8();
                self.icon = Some(IconData {
                    width: rgba.width(),
                    height: rgba.height(),
                    rgba: rgba.into_raw(),
                });
            }
            Err(e) => {
                log::error!("Failed to decode tray icon image: {}", e);
            }
        }
        self
    }

    /// Sets the tooltip text shown when hovering the tray icon.
    pub fn tooltip(mut self, tooltip: impl Into<String>) -> Self {
        self.tooltip = Some(tooltip.into());
        self
    }

    /// Sets the context menu shown on right-click.
    pub fn menu(mut self, menu: NativeMenu) -> Self {
        self.menu = Some(menu);
        self
    }

    /// Sets the click handler for tray icon interactions.
    pub fn on_click(
        mut self,
        f: impl Fn(TrayInteraction) + Send + Sync + 'static,
    ) -> Self {
        self.on_click = Some(Arc::new(f));
        self
    }
}

impl Default for TrayConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for TrayConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TrayConfig")
            .field("icon", &self.icon.as_ref().map(|i| format!("{}x{}", i.width, i.height)))
            .field("tooltip", &self.tooltip)
            .field("menu", &self.menu)
            .field("on_click", &self.on_click.as_ref().map(|_| "..."))
            .finish()
    }
}

// ---------------------------------------------------------------------------
// Internal: build the tray-icon instance
// ---------------------------------------------------------------------------

/// Creates a [`tray_icon::TrayIcon`] from a [`TrayConfig`].
///
/// Must be called **after** the event loop has started (e.g. in `resumed()`).
/// Returns the built tray icon, or an error description on failure.
pub(crate) fn build_tray_icon(
    config: &TrayConfig,
    tray_menu: Option<Box<dyn muda::ContextMenu>>,
) -> Result<tray_icon::TrayIcon, String> {
    let mut builder = tray_icon::TrayIconBuilder::new();

    if let Some(ref icon_data) = config.icon {
        let icon = tray_icon::Icon::from_rgba(
            icon_data.rgba.clone(),
            icon_data.width,
            icon_data.height,
        )
        .map_err(|e| format!("Invalid tray icon data: {e}"))?;
        builder = builder.with_icon(icon);
    }

    if let Some(ref tooltip) = config.tooltip {
        builder = builder.with_tooltip(tooltip);
    }

    if let Some(menu) = tray_menu {
        builder = builder.with_menu(menu);
    }

    builder.build().map_err(|e| format!("Failed to build tray icon: {e}"))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tray_config_default_is_empty() {
        let config = TrayConfig::new();
        assert!(config.icon.is_none());
        assert!(config.tooltip.is_none());
        assert!(config.menu.is_none());
        assert!(config.on_click.is_none());
    }

    #[test]
    fn tray_config_builder_chain() {
        let config = TrayConfig::new()
            .tooltip("My App")
            .on_click(|_| {});

        assert_eq!(config.tooltip.as_deref(), Some("My App"));
        assert!(config.on_click.is_some());
    }

    #[test]
    fn tray_interaction_debug() {
        // Verify Debug is implemented and doesn't panic.
        let _ = format!("{:?}", TrayInteraction::Click);
        let _ = format!("{:?}", TrayInteraction::DoubleClick);
        let _ = format!("{:?}", TrayInteraction::RightClick);
    }

    #[test]
    #[should_panic(expected = "dimensions must be non-zero")]
    fn tray_icon_data_validation_zero_size() {
        let _ = TrayConfig::new().icon_rgba(vec![], 0, 0);
    }

    #[test]
    #[should_panic(expected = "buffer length must be width * height * 4")]
    fn tray_icon_data_validation_wrong_length() {
        let _ = TrayConfig::new().icon_rgba(vec![0; 10], 2, 2);
    }
}
