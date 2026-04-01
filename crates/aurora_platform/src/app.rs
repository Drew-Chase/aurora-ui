#![doc = include_str!("../../../.wiki/App.md")]

use crate::errors::app::AppError;
use aurora_core::color::Color;
use aurora_core::direction::TextDirection;
use aurora_core::geometry::point::Point;
use aurora_core::geometry::rect::Rect;
use aurora_core::geometry::size::Size;
use aurora_core::kmi::WidgetEvent;
use aurora_core::kmi::cursor_icon::CursorIcon;
use aurora_core::kmi::drag::{DragEvent, FileDropEvent};
use aurora_core::kmi::keyboard::{Key, KeyboardEvent, Modifiers};
use aurora_core::kmi::mouse::{MouseButton, MouseClickEvent, MouseEvent, MouseState};
use aurora_gpu::gpu_context::GpuContext;
use aurora_render::canvas::Canvas;
#[cfg(feature = "text")]
use aurora_text::text_layout::TextLayout;
use aurora_widgets::widgets::{EventResponse, LayoutCtx, Widget};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
#[cfg(feature = "menu")]
use std::sync::Mutex;
use winit::application::ApplicationHandler;
use winit::dpi;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoopProxy};
use winit::window::{WindowAttributes, WindowButtons, WindowId};

// ---------------------------------------------------------------------------
// Public enums
// ---------------------------------------------------------------------------

/// Which monitor to target for window placement.
///
/// Used with [`WindowPosition`] to control where a window spawns.
#[derive(Debug, Clone, Copy, Default)]
pub enum WindowMonitor {
    /// The primary display.
    ///
    /// Falls back to the first available monitor if no primary is reported.
    #[default]
    Primary,
    /// The monitor where the cursor is currently located.
    ///
    /// On platforms that do not support pre-window cursor queries this
    /// falls back to [`Primary`](Self::Primary).
    Active,
    /// A specific monitor by its zero-based index.
    ///
    /// Falls back to [`Primary`](Self::Primary) if the index is out of range.
    Index(usize),
}

/// Initial window placement on the target monitor.
///
/// Used with the [`App::position`] builder method.
#[derive(Debug, Clone, Copy)]
pub enum WindowPosition {
    /// Center the window on the target monitor.
    Center,
    /// Place the window at logical coordinates relative to the target
    /// monitor's top-left corner.
    At(Point),
}

/// Controls when the application exits in multi-window mode.
#[derive(Debug, Clone, Copy, Default)]
pub enum ExitBehavior {
    /// Exit when the last window is closed.
    #[default]
    LastWindowClosed,
    /// Exit when the primary (first) window is closed.
    PrimaryWindowClosed,
}

// ---------------------------------------------------------------------------
// WindowConfig — configuration for a single window
// ---------------------------------------------------------------------------

/// Configuration for creating a window.
///
/// Constructed via builder methods. For the primary window this is built
/// internally from [`App`]; for secondary windows the user creates one
/// directly with [`WindowConfig::new()`] and passes it to
/// [`AppWindow::open_window`].
#[derive(Clone)]
pub struct WindowConfig {
    pub title: String,
    pub size: Size,
    pub min_size: Option<Size>,
    pub resizable: bool,
    pub decorations: bool,
    pub custom_titlebar: bool,
    pub position: Option<WindowPosition>,
    pub monitor: WindowMonitor,
    pub background_color: Color,
    pub use_system_font: bool,
    pub icon: Option<IconData>,
    pub locale_tag: String,
    pub text_direction: TextDirection,
    #[cfg(feature = "text")]
    pub font_options: aurora_text::font_options::FontOptions,
    #[cfg(feature = "text")]
    pub fonts: Vec<&'static [u8]>,
    /// Native OS menu bar for this window (requires `menu` feature).
    #[cfg(feature = "menu")]
    pub menu: Option<crate::menu::NativeMenu>,
    /// If true, closing this window exits the application.
    pub primary: bool,
    /// Parent window — closing the parent closes children too.
    pub parent: Option<WindowId>,
    /// If true, this is a modal window that blocks input on its parent.
    pub modal: bool,
    /// Callback invoked when the window is closed (by any means: X button,
    /// `close_window()`, or parent closing). Use this to clean up shared
    /// state such as "is open" flags.
    pub on_close: Option<Arc<dyn Fn() + Send + Sync>>,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            title: "Aurora Window".to_string(),
            size: Size::new(800.0, 600.0),
            min_size: None,
            resizable: true,
            decorations: true,
            custom_titlebar: false,
            position: None,
            monitor: WindowMonitor::Primary,
            background_color: Color::WHITE,
            use_system_font: false,
            icon: None,
            locale_tag: String::new(),
            text_direction: TextDirection::Ltr,
            #[cfg(feature = "text")]
            font_options: aurora_text::font_options::FontOptions::default(),
            #[cfg(feature = "text")]
            fonts: vec![],
            #[cfg(feature = "menu")]
            menu: None,
            primary: false,
            parent: None,
            modal: false,
            on_close: None,
        }
    }
}

impl WindowConfig {
    /// Creates a new window configuration with defaults.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the window title.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    /// Sets the initial window size.
    pub fn size(mut self, size: impl Into<Size>) -> Self {
        self.size = size.into();
        self
    }

    /// Sets the minimum window size.
    pub fn min_size(mut self, min_size: impl Into<Size>) -> Self {
        self.min_size = Some(min_size.into());
        self
    }

    /// Enables a custom (chromeless) titlebar.
    pub fn custom_titlebar(mut self, custom_titlebar: bool) -> Self {
        self.custom_titlebar = custom_titlebar;
        self.decorations = false;
        self
    }

    /// Enables or disables OS window decorations.
    pub fn decorations(mut self, decorations: bool) -> Self {
        if self.custom_titlebar {
            self.decorations = false;
            return self;
        }
        self.decorations = decorations;
        self
    }

    /// Enables or disables window resizing.
    pub fn resizable(mut self, resizable: bool) -> Self {
        self.resizable = resizable;
        self
    }

    /// Sets the initial window position on the target monitor.
    pub fn position(mut self, position: WindowPosition) -> Self {
        self.position = Some(position);
        self
    }

    /// Selects which monitor the window spawns on.
    pub fn monitor(mut self, monitor: WindowMonitor) -> Self {
        self.monitor = monitor;
        self
    }

    /// Sets the background color.
    pub fn background_color(mut self, color: impl Into<Color>) -> Self {
        self.background_color = color.into();
        self
    }

    /// Enables system font discovery.
    pub fn use_system_fonts(mut self) -> Self {
        self.use_system_font = true;
        self
    }

    /// Sets the BCP 47 locale tag.
    pub fn locale(mut self, tag: impl Into<String>) -> Self {
        self.locale_tag = tag.into();
        self
    }

    /// Sets the text direction for layout.
    pub fn text_direction(mut self, direction: TextDirection) -> Self {
        self.text_direction = direction;
        self
    }

    /// Registers a font from a static byte slice.
    #[cfg(feature = "text")]
    pub fn font(mut self, bytes: &'static [u8]) -> Self {
        self.fonts.push(bytes);
        self
    }

    /// Sets the global font options.
    #[cfg(feature = "text")]
    pub fn font_options(mut self, opts: aurora_text::font_options::FontOptions) -> Self {
        self.font_options = opts;
        self
    }

    /// Sets a callback invoked when the window is closed by any means.
    pub fn on_close(mut self, f: impl Fn() + Send + Sync + 'static) -> Self {
        self.on_close = Some(Arc::new(f));
        self
    }

    /// Sets a native OS menu bar for this window.
    #[cfg(feature = "menu")]
    pub fn menu(mut self, menu: crate::menu::NativeMenu) -> Self {
        self.menu = Some(menu);
        self
    }
}

// ---------------------------------------------------------------------------
// App — builder for the primary window + application entry point
// ---------------------------------------------------------------------------

/// Builder for configuring and launching an application window.
///
/// Uses the builder pattern — each setter consumes and returns `self` so calls
/// can be chained. Call [`run`](Self::run) to open the window and enter the
/// event loop.
///
/// # Defaults
///
/// | Property | Default |
/// |---|---|
/// | `title` | `"Aurora App"` |
/// | `size` | 800 x 600 |
/// | `min_size` | `None` |
/// | `resizable` | `true` |
/// | `decorations` | `true` |
/// | `custom_titlebar` | `false` |
///
/// # Examples
///
/// ```no_run
/// use aurora_platform::app::App;
///
/// App::new()
///     .title("My App")
///     .size((1280, 720))
///     .min_size((400, 300))
///     .resizable(true)
///     .run(|window, frame| {
///         // render callback
///     })
///     .expect("Failed to run app");
/// ```
#[derive(Clone)]
pub struct App {
    pub title: String,
    pub size: Size,
    pub min_size: Option<Size>,
    pub resizable: bool,
    pub decorations: bool,
    pub custom_titlebar: bool,
    pub position: Option<WindowPosition>,
    pub monitor: WindowMonitor,
    pub background_color: Color,
    pub use_system_font: bool,
    pub icon: Option<IconData>,
    pub locale_tag: String,
    pub text_direction: TextDirection,
    #[cfg(feature = "text")]
    pub font_options: aurora_text::font_options::FontOptions,
    #[cfg(feature = "text")]
    pub fonts: Vec<&'static [u8]>,
    exit_on: ExitBehavior,
    #[cfg(feature = "menu")]
    pub(crate) menu: Option<crate::menu::NativeMenu>,
    #[cfg(feature = "tray")]
    pub(crate) tray: Option<crate::tray::TrayConfig>,
}

/// Decoded RGBA icon data for setting the window icon.
///
/// Create with [`IconData::from_rgba`] (always available) or
/// [`App::icon`] (requires the `image` feature for PNG/JPEG decoding).
#[derive(Clone)]
pub struct IconData {
    /// RGBA pixel data, 4 bytes per pixel.
    pub rgba: Vec<u8>,
    /// Icon width in pixels.
    pub width: u32,
    /// Icon height in pixels.
    pub height: u32,
}

impl App {
    /// Creates a new application builder with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the window title.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    /// Sets the initial window size.
    pub fn size(mut self, size: impl Into<Size>) -> Self {
        self.size = size.into();
        self
    }

    /// Sets the initial window width, leaving height unchanged.
    pub fn width(mut self, width: impl Into<f32>) -> Self {
        self.size.width = width.into();
        self
    }

    /// Sets the initial window height, leaving width unchanged.
    pub fn height(mut self, height: impl Into<f32>) -> Self {
        self.size.height = height.into();
        self
    }

    /// Sets the minimum window size.
    pub fn min_size(mut self, min_size: impl Into<Size>) -> Self {
        self.min_size = Some(min_size.into());
        self
    }

    /// Enables a custom (chromeless) titlebar.
    ///
    /// When enabled, OS decorations are automatically disabled.
    pub fn custom_titlebar(mut self, custom_titlebar: bool) -> Self {
        self.custom_titlebar = custom_titlebar;
        self.decorations = false;
        self
    }

    /// Enables or disables OS window decorations (title bar, borders).
    ///
    /// This is a no-op if [`custom_titlebar`](Self::custom_titlebar) is enabled.
    pub fn decorations(mut self, decorations: bool) -> Self {
        if self.custom_titlebar {
            self.decorations = false;
            return self;
        }
        self.decorations = decorations;
        self
    }

    /// Enables or disables window resizing.
    pub fn resizable(mut self, resizable: bool) -> Self {
        self.resizable = resizable;
        self
    }

    /// Sets the initial window position on the target monitor.
    ///
    /// When not set, the OS chooses the position.
    pub fn position(mut self, position: WindowPosition) -> Self {
        self.position = Some(position);
        self
    }

    /// Enables the use of system font discovery.
    ///
    /// This can cause a ~200ms delay on startup but allows for more font options.
    pub fn use_system_fonts(mut self) -> Self {
        self.use_system_font = true;
        self
    }

    /// Enables or disables system font discovery.
    ///
    /// Enabling system font discovery can cause a ~200ms delay on startup but allows for more font options.
    pub fn set_use_system_font(mut self, use_system_font: bool) -> Self {
        self.use_system_font = use_system_font;
        self
    }

    /// Sets the BCP 47 locale tag (e.g. `"en-US"`, `"ar-SA"`, `"he-IL"`).
    ///
    /// This is passed to the font system for locale-aware font fallback
    /// (e.g. CJK disambiguation). When the `i18n` feature is enabled,
    /// [`system_locale`](Self::system_locale) can auto-detect this.
    pub fn locale(mut self, tag: impl Into<String>) -> Self {
        self.locale_tag = tag.into();
        self
    }

    /// Sets the text direction for layout (LTR or RTL).
    ///
    /// Controls how [`Row`] children are ordered, how `Align::Start`/`End`
    /// resolve in [`Column`], and how text alignment maps to physical
    /// left/right. Defaults to [`TextDirection::Ltr`].
    pub fn text_direction(mut self, direction: TextDirection) -> Self {
        self.text_direction = direction;
        self
    }

    /// Detects the system locale and sets both [`locale`](Self::locale) and
    /// [`text_direction`](Self::text_direction) from it.
    ///
    /// Requires the `i18n` feature.
    #[cfg(feature = "i18n")]
    pub fn system_locale(mut self) -> Self {
        let locale = aurora_i18n::locale::Locale::system();
        self.locale_tag = locale.tag().to_string();
        self.text_direction = locale.direction();
        self
    }

    /// Selects which monitor the window spawns on.
    ///
    /// Defaults to [`WindowMonitor::Primary`]. Only takes effect when
    /// [`position`](Self::position) is also set.
    pub fn monitor(mut self, monitor: WindowMonitor) -> Self {
        self.monitor = monitor;
        self
    }

    /// Sets the background color used to clear the window before each frame.
    pub fn background_color(mut self, background_color: impl Into<Color>) -> Self {
        self.background_color = background_color.into();
        self
    }

    /// Sets the global font options applied to all text widgets by default.
    ///
    /// Individual widgets can override any field via their own builder methods.
    #[cfg(feature = "text")]
    pub fn font_options(mut self, font_options: aurora_text::font_options::FontOptions) -> Self {
        self.font_options = font_options;
        self
    }

    /// Registers a font from a static byte slice (e.g. `include_bytes!`).
    ///
    /// The font is loaded into the [`FontManager`](aurora_text::font_manager::FontManager)
    /// when the window is created.
    #[cfg(feature = "text")]
    pub fn font(mut self, bytes: &'static [u8]) -> Self {
        self.fonts.push(bytes);
        self
    }

    /// Sets the window icon from pre-decoded RGBA pixel data.
    ///
    /// The data must be `width * height * 4` bytes (one `[R, G, B, A]` per pixel).
    /// This sets the taskbar icon and the titlebar icon on platforms that support it.
    ///
    /// # Panics
    /// Panics if `rgba.len() != width * height * 4`, or if dimensions are zero or overflow.
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

    /// Sets the window icon by decoding an image (PNG, JPEG) from raw bytes.
    ///
    /// Typically used with `include_bytes!("icon.png")`.
    /// This sets the taskbar icon and the titlebar icon on platforms that support it.
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
                log::error!("Failed to decode icon image: {}", e);
            }
        }
        self
    }

    /// Sets the exit behavior for multi-window applications.
    ///
    /// Defaults to [`ExitBehavior::LastWindowClosed`].
    pub fn exit_on(mut self, behavior: ExitBehavior) -> Self {
        self.exit_on = behavior;
        self
    }

    /// Sets the native OS menu bar for the primary window.
    ///
    /// On macOS this becomes the app-wide menu bar. On Windows and Linux it
    /// is attached to the primary window.
    #[cfg(feature = "menu")]
    pub fn menu(mut self, menu: crate::menu::NativeMenu) -> Self {
        self.menu = Some(menu);
        self
    }

    /// Configures a system tray icon for the application.
    ///
    /// The tray icon is created when the event loop starts. Requires the
    /// `tray` feature.
    #[cfg(feature = "tray")]
    pub fn tray(mut self, config: crate::tray::TrayConfig) -> Self {
        self.tray = Some(config);
        self
    }

    /// Opens the window and enters the event loop.
    ///
    /// The `on_render` callback is invoked on every [`RedrawRequested`](WindowEvent::RedrawRequested)
    /// event with a reference to the [`AppWindow`] and the current [`FrameInfo`].
    ///
    /// Returns an [`AppError`] if the window or event loop could not be created.
    #[allow(unused_mut)]
    pub fn run<F>(mut self, on_render: F) -> Result<(), AppError>
    where
        F: FnMut(&mut AppWindow, FrameInfo) + 'static,
    {
        let benchmark = std::env::var("AURORA_BENCHMARK").is_ok();
        let start_time = std::time::Instant::now();
        let exit_on = self.exit_on;

        // Extract menu/tray configs before converting to WindowConfig
        // (which consumes `self`).
        #[cfg(feature = "menu")]
        let menu_config = self.menu.take();
        #[cfg(feature = "tray")]
        let tray_config = self.tray.take();

        let config = WindowConfig::from(self);

        let event_loop = winit::event_loop::EventLoop::<AppEvent>::with_user_event()
            .build()
            .map_err(AppError::from)?;
        let proxy = event_loop.create_proxy();

        // Set up the global muda MenuEvent handler that forwards menu item
        // clicks into our event loop.
        #[cfg(feature = "menu")]
        let menu_id_map: Arc<Mutex<HashMap<muda::MenuId, String>>> =
            Arc::new(Mutex::new(HashMap::new()));
        #[cfg(feature = "menu")]
        {
            let proxy_clone = proxy.clone();
            let map = menu_id_map.clone();
            muda::MenuEvent::set_event_handler(Some(move |event: muda::MenuEvent| {
                if let Ok(guard) = map.lock()
                    && let Some(user_id) = guard.get(&event.id)
                {
                    let _ = proxy_clone.send_event(AppEvent::MenuEvent(user_id.clone()));
                }
            }));
        }

        // Set up the global TrayIconEvent handler.
        #[cfg(feature = "tray")]
        {
            let proxy_clone = proxy.clone();
            tray_icon::TrayIconEvent::set_event_handler(Some(move |event| {
                let interaction = match event {
                    tray_icon::TrayIconEvent::Click {
                        button: tray_icon::MouseButton::Left,
                        button_state: tray_icon::MouseButtonState::Up,
                        ..
                    } => Some(crate::tray::TrayInteraction::Click),
                    tray_icon::TrayIconEvent::DoubleClick {
                        button: tray_icon::MouseButton::Left,
                        ..
                    } => Some(crate::tray::TrayInteraction::DoubleClick),
                    tray_icon::TrayIconEvent::Click {
                        button: tray_icon::MouseButton::Right,
                        button_state: tray_icon::MouseButtonState::Up,
                        ..
                    } => Some(crate::tray::TrayInteraction::RightClick),
                    _ => None,
                };
                if let Some(interaction) = interaction {
                    let _ = proxy_clone.send_event(AppEvent::TrayEvent(interaction));
                }
            }));
        }

        let mut app_handler = AppHandler {
            windows: HashMap::new(),
            pending_windows: vec![(config, Box::new(on_render))],
            current_modifiers: winit::keyboard::ModifiersState::default(),
            proxy,
            primary_window: None,
            benchmark,
            start_time,
            exit_on,
            #[cfg(feature = "menu")]
            menu_config,
            #[cfg(feature = "menu")]
            menu_id_map,
            #[cfg(feature = "menu")]
            menu_callbacks: Vec::new(),
            #[cfg(feature = "menu")]
            muda_menus: Vec::new(),
            #[cfg(feature = "tray")]
            tray_config,
            #[cfg(feature = "tray")]
            tray_icon: None,
            #[cfg(feature = "tray")]
            tray_click_callback: None,
        };

        event_loop
            .run_app(&mut app_handler)
            .map_err(AppError::from)?;
        Ok(())
    }
}

impl Default for App {
    fn default() -> Self {
        Self {
            title: "Aurora App".to_string(),
            size: Size::new(800.0, 600.0),
            min_size: None,
            resizable: true,
            decorations: true,
            custom_titlebar: false,
            position: None,
            monitor: WindowMonitor::Primary,
            background_color: Color::WHITE,
            use_system_font: false,
            icon: None,
            locale_tag: String::new(),
            text_direction: TextDirection::Ltr,
            #[cfg(feature = "text")]
            font_options: aurora_text::font_options::FontOptions::default(),
            #[cfg(feature = "text")]
            fonts: vec![],
            exit_on: ExitBehavior::LastWindowClosed,
            #[cfg(feature = "menu")]
            menu: None,
            #[cfg(feature = "tray")]
            tray: None,
        }
    }
}

impl From<App> for WindowConfig {
    fn from(app: App) -> Self {
        Self {
            title: app.title,
            size: app.size,
            min_size: app.min_size,
            resizable: app.resizable,
            decorations: app.decorations,
            custom_titlebar: app.custom_titlebar,
            position: app.position,
            monitor: app.monitor,
            background_color: app.background_color,
            use_system_font: app.use_system_font,
            icon: app.icon,
            locale_tag: app.locale_tag,
            text_direction: app.text_direction,
            #[cfg(feature = "text")]
            font_options: app.font_options,
            #[cfg(feature = "text")]
            fonts: app.fonts,
            #[cfg(feature = "menu")]
            menu: app.menu,
            primary: true,
            parent: None,
            modal: false,
            on_close: None,
        }
    }
}

// ---------------------------------------------------------------------------
// FrameInfo
// ---------------------------------------------------------------------------

/// Per-frame information passed to the render callback.
///
/// Contains the current drawable dimensions and the display scale factor.
#[derive(Debug)]
pub struct FrameInfo {
    /// Drawable width in physical pixels.
    pub width: u32,
    /// Drawable height in physical pixels.
    pub height: u32,
    /// Display scale factor (e.g. `2.0` on Retina/HiDPI displays).
    pub scale_factor: f64,
}

// ---------------------------------------------------------------------------
// AppContext — handle for multi-window operations
// ---------------------------------------------------------------------------

/// Handle for managing the application from within render callbacks.
///
/// Cloneable and `Send`, allowing cross-window and cross-thread communication.
/// Stored on every [`AppWindow`] — access via [`AppWindow::app_context`].
#[derive(Clone)]
pub struct AppContext {
    proxy: EventLoopProxy<AppEvent>,
}

impl AppContext {
    /// Opens a new window with the given configuration and render callback.
    ///
    /// The window is created asynchronously on the next event loop tick.
    pub fn open_window<F>(&self, config: WindowConfig, on_render: F)
    where
        F: FnMut(&mut AppWindow, FrameInfo) + Send + 'static,
    {
        let _ = self
            .proxy
            .send_event(AppEvent::OpenWindow(Box::new(OpenWindowRequest {
                config,
                on_render: Box::new(on_render),
            })));
    }

    /// Closes a specific window by ID.
    pub fn close_window(&self, id: WindowId) {
        let _ = self.proxy.send_event(AppEvent::CloseWindow(id));
    }

    /// Sends a typed message to a specific window (or broadcasts to all if `None`).
    pub fn send_message<T: std::any::Any + Send>(&self, target: Option<WindowId>, message: T) {
        let _ = self.proxy.send_event(AppEvent::UserMessage {
            target,
            payload: Box::new(message),
        });
    }

    /// Requests a redraw for a specific window.
    pub fn request_redraw(&self, id: WindowId) {
        let _ = self.proxy.send_event(AppEvent::RequestRedraw(id));
    }
}

// ---------------------------------------------------------------------------
// Type aliases
// ---------------------------------------------------------------------------

/// Render callback type for a window.
type RenderCallback = dyn FnMut(&mut AppWindow, FrameInfo);
/// Render callback that can be sent across threads (for `AppEvent`).
type SendRenderCallback = dyn FnMut(&mut AppWindow, FrameInfo) + Send;

// ---------------------------------------------------------------------------
// AppEvent — user events for the event loop
// ---------------------------------------------------------------------------

pub(crate) enum AppEvent {
    OpenWindow(Box<OpenWindowRequest>),
    CloseWindow(WindowId),
    UserMessage {
        target: Option<WindowId>,
        payload: Box<dyn std::any::Any + Send>,
    },
    RequestRedraw(WindowId),
    #[cfg(feature = "menu")]
    MenuEvent(String),
    #[cfg(feature = "tray")]
    TrayEvent(crate::tray::TrayInteraction),
}

/// Boxed to avoid large enum variant size difference in `AppEvent`.
pub(crate) struct OpenWindowRequest {
    config: WindowConfig,
    on_render: Box<SendRenderCallback>,
}

// ---------------------------------------------------------------------------
// WindowState — per-window event-loop state
// ---------------------------------------------------------------------------

struct WindowState {
    app_window: AppWindow,
    on_render: Box<RenderCallback>,
    // Per-window event tracking (previously global in AppHandler):
    current_cursor_position: Option<Point>,
    resized_this_cycle: bool,
    drag_press_position: Option<Point>,
    drag_active: bool,
    os_drag_paths: Vec<PathBuf>,
    // Multi-window relationships:
    parent: Option<WindowId>,
    modal_child: Option<WindowId>,
    on_close: Option<Arc<dyn Fn() + Send + Sync>>,
}

// ---------------------------------------------------------------------------
// AppHandler — the event loop handler
// ---------------------------------------------------------------------------

struct AppHandler {
    windows: HashMap<WindowId, WindowState>,
    pending_windows: Vec<(WindowConfig, Box<RenderCallback>)>,
    current_modifiers: winit::keyboard::ModifiersState,
    proxy: EventLoopProxy<AppEvent>,
    primary_window: Option<WindowId>,
    benchmark: bool,
    start_time: std::time::Instant,
    exit_on: ExitBehavior,
    // Native menu support
    #[cfg(feature = "menu")]
    menu_config: Option<crate::menu::NativeMenu>,
    #[cfg(feature = "menu")]
    menu_id_map: Arc<Mutex<HashMap<muda::MenuId, String>>>,
    #[cfg(feature = "menu")]
    menu_callbacks: Vec<crate::menu::MenuClickHandler>,
    /// Keep `muda::Menu` objects alive for the lifetime of the app.
    #[cfg(feature = "menu")]
    muda_menus: Vec<muda::Menu>,
    // System tray support
    #[cfg(feature = "tray")]
    tray_config: Option<crate::tray::TrayConfig>,
    #[cfg(feature = "tray")]
    tray_icon: Option<tray_icon::TrayIcon>,
    #[cfg(feature = "tray")]
    tray_click_callback: Option<Arc<dyn Fn(crate::tray::TrayInteraction) + Send + Sync>>,
}

/// Minimum cursor distance in pixels to trigger a drag gesture.
const DRAG_THRESHOLD: f32 = 4.0;

impl AppHandler {
    fn create_window(
        &mut self,
        event_loop: &ActiveEventLoop,
        config: WindowConfig,
        on_render: Box<RenderCallback>,
    ) {
        let is_primary = config.primary;
        let parent = config.parent;
        let modal = config.modal;
        let on_close = config.on_close.clone();

        let attributes = build_window_attributes(&config, event_loop);

        match event_loop.create_window(attributes) {
            Ok(window) => {
                let handle = Arc::new(window);
                #[cfg(target_os = "windows")]
                if config.custom_titlebar {
                    crate::windows_titlebar::apply(&handle);
                }

                match AppWindow::new(handle.clone(), &config, event_loop) {
                    Ok(mut app_window) => {
                        let window_id = handle.id();

                        // Set multi-window fields
                        app_window.window_id = Some(window_id);
                        app_window.app_context = Some(AppContext {
                            proxy: self.proxy.clone(),
                        });

                        // Initialize accessibility adapter
                        #[cfg(feature = "a11y")]
                        {
                            let a11y_state = crate::a11y::A11yState::new(
                                event_loop,
                                &app_window.window_handle,
                                &config.title,
                            );
                            app_window.a11y = Some(a11y_state);
                        }

                        handle.set_visible(true);

                        // Attach per-window native menu bar (if configured).
                        #[cfg(feature = "menu")]
                        if let Some(ref menu_cfg) = config.menu {
                            let (muda_menu, id_map) = crate::menu::build_muda_menu(menu_cfg);

                            #[cfg(target_os = "macos")]
                            crate::menu::attach_menu_to_app(&muda_menu);

                            #[cfg(target_os = "windows")]
                            crate::menu::attach_menu_to_window(&muda_menu, &handle);

                            if let Ok(mut guard) = self.menu_id_map.lock() {
                                guard.extend(id_map);
                            }
                            if let Some(ref cb) = menu_cfg.on_item_click {
                                self.menu_callbacks.push(cb.clone());
                            }
                            self.muda_menus.push(muda_menu);
                        }

                        let state = WindowState {
                            app_window,
                            on_render,
                            current_cursor_position: None,
                            resized_this_cycle: false,
                            drag_press_position: None,
                            drag_active: false,
                            os_drag_paths: Vec::new(),
                            parent,
                            modal_child: None,
                            on_close,
                        };

                        self.windows.insert(window_id, state);

                        if is_primary || self.primary_window.is_none() {
                            self.primary_window = Some(window_id);
                        }

                        // Mark parent as having a modal child
                        if modal
                            && let Some(pid) = parent
                            && let Some(parent_state) = self.windows.get_mut(&pid)
                        {
                            parent_state.modal_child = Some(window_id);
                        }
                    }
                    Err(err) => {
                        log::error!("Failed to create AppWindow: {}", err);
                    }
                }
            }
            Err(err) => {
                log::error!("Failed to create OS window: {}", err);
            }
        }
    }

    fn close_window(&mut self, id: WindowId, event_loop: &ActiveEventLoop) {
        // Collect and close children first
        let child_ids: Vec<WindowId> = self
            .windows
            .iter()
            .filter(|(_, s)| s.parent == Some(id))
            .map(|(wid, _)| *wid)
            .collect();
        for child_id in child_ids {
            self.close_window(child_id, event_loop);
        }

        // Clear parent's modal_child
        if let Some(state) = self.windows.get(&id)
            && let Some(pid) = state.parent
            && let Some(parent_state) = self.windows.get_mut(&pid)
            && parent_state.modal_child == Some(id)
        {
            parent_state.modal_child = None;
        }

        // Fire the on_close callback before dropping the window state
        if let Some(state) = self.windows.get(&id)
            && let Some(ref cb) = state.on_close
        {
            cb();
        }

        self.windows.remove(&id);

        let should_exit = match self.exit_on {
            ExitBehavior::LastWindowClosed => self.windows.is_empty(),
            ExitBehavior::PrimaryWindowClosed => {
                self.primary_window == Some(id) || self.windows.is_empty()
            }
        };

        if should_exit {
            event_loop.exit();
        }
    }
}

// ---------------------------------------------------------------------------
// ApplicationHandler impl
// ---------------------------------------------------------------------------

impl ApplicationHandler<AppEvent> for AppHandler {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.windows.is_empty() {
            let pending = std::mem::take(&mut self.pending_windows);
            for (config, on_render) in pending {
                self.create_window(event_loop, config, on_render);
            }

            // --- Initialize native menu bar for the primary window ---
            #[cfg(feature = "menu")]
            if let Some(menu_config) = self.menu_config.take() {
                let (muda_menu, id_map) = crate::menu::build_muda_menu(&menu_config);

                // On macOS the menu bar is app-wide.
                #[cfg(target_os = "macos")]
                crate::menu::attach_menu_to_app(&muda_menu);

                // On Windows/Linux attach to the primary window.
                #[cfg(target_os = "windows")]
                if let Some(primary_id) = self.primary_window
                    && let Some(state) = self.windows.get(&primary_id)
                {
                    crate::menu::attach_menu_to_window(&muda_menu, &state.app_window.window_handle);
                }

                // Merge ID map into the shared map.
                if let Ok(mut guard) = self.menu_id_map.lock() {
                    guard.extend(id_map);
                }

                // Store the callback.
                if let Some(cb) = menu_config.on_item_click {
                    self.menu_callbacks.push(cb);
                }

                // Keep the muda menu alive.
                self.muda_menus.push(muda_menu);
            }

            // --- Initialize system tray icon ---
            #[cfg(feature = "tray")]
            if let Some(tray_config) = self.tray_config.take() {
                self.tray_click_callback = tray_config.on_click.clone();

                // Build the tray context menu (if any).
                let tray_menu = tray_config.menu.as_ref().map(|m| {
                    let (menu, id_map) = crate::menu::build_muda_menu(m);
                    // Merge IDs into the shared map so the global handler
                    // can resolve tray-menu clicks too.
                    if let Ok(mut guard) = self.menu_id_map.lock() {
                        guard.extend(id_map);
                    }
                    // Store the tray menu's callback.
                    if let Some(ref cb) = m.on_item_click {
                        self.menu_callbacks.push(cb.clone());
                    }
                    menu
                });

                let tray_menu_box: Option<Box<dyn muda::ContextMenu>> =
                    tray_menu.map(|m| Box::new(m) as Box<dyn muda::ContextMenu>);

                match crate::tray::build_tray_icon(&tray_config, tray_menu_box) {
                    Ok(icon) => {
                        self.tray_icon = Some(icon);
                    }
                    Err(e) => {
                        log::error!("Failed to create tray icon: {e}");
                    }
                }
            }
        } else {
            for state in self.windows.values() {
                state.app_window.window_handle.focus_window();
            }
        }
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: AppEvent) {
        match event {
            AppEvent::OpenWindow(req) => {
                self.create_window(event_loop, req.config, req.on_render);
            }
            AppEvent::CloseWindow(id) => {
                self.close_window(id, event_loop);
            }
            AppEvent::UserMessage { target, payload } => {
                if let Some(id) = target
                    && let Some(state) = self.windows.get_mut(&id)
                {
                    state.app_window.pending_messages.push(payload);
                    state.app_window.window_handle.request_redraw();
                }
            }
            AppEvent::RequestRedraw(id) => {
                if let Some(state) = self.windows.get(&id) {
                    state.app_window.window_handle.request_redraw();
                }
            }
            #[cfg(feature = "menu")]
            AppEvent::MenuEvent(id) => {
                for cb in &self.menu_callbacks {
                    cb(&id);
                }
                // Request redraw on all windows so the UI can react.
                for state in self.windows.values() {
                    state.app_window.window_handle.request_redraw();
                }
            }
            #[cfg(feature = "tray")]
            AppEvent::TrayEvent(interaction) => {
                if let Some(ref cb) = self.tray_click_callback {
                    cb(interaction);
                }
            }
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        // Drain pending window creation requests
        let pending = std::mem::take(&mut self.pending_windows);
        for (config, on_render) in pending {
            self.create_window(event_loop, config, on_render);
        }

        let mut any_animating = false;

        // Collect window IDs to iterate without borrowing self mutably twice
        let window_ids: Vec<WindowId> = self.windows.keys().copied().collect();

        for wid in window_ids {
            let Some(state) = self.windows.get_mut(&wid) else {
                continue;
            };

            let resized = std::mem::take(&mut state.resized_this_cycle);
            let window = &mut state.app_window;

            // Poll cursor position during OS file drag
            if !state.os_drag_paths.is_empty()
                && let Some(pos) = query_cursor_in_window(&window.window_handle)
            {
                state.current_cursor_position = Some(pos);
                for path in state.os_drag_paths.clone() {
                    window.dispatch_event(&WidgetEvent::FileDrop(FileDropEvent::FileHovered {
                        path,
                        position: pos,
                    }));
                }
                window.request_next_frame();
            }

            if window.next_frame_requested && !resized {
                window.next_frame_requested = false;

                let physical = window.window_handle.inner_size();
                let frame_info = FrameInfo {
                    width: physical.width,
                    height: physical.height,
                    scale_factor: window.window_handle.scale_factor(),
                };
                window.gpu.resize(frame_info.width, frame_info.height);
                window.clear(window.background_color);
                (state.on_render)(window, frame_info);
                window.layout_and_paint();
                window.present();
            }

            if window.next_frame_requested || !state.os_drag_paths.is_empty() {
                any_animating = true;
            }
        }

        if any_animating {
            event_loop.set_control_flow(winit::event_loop::ControlFlow::WaitUntil(
                std::time::Instant::now() + std::time::Duration::from_millis(16),
            ));
        } else {
            event_loop.set_control_flow(winit::event_loop::ControlFlow::Wait);
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(state) = self.windows.get_mut(&window_id) else {
            return;
        };

        // Block input events if this window has a modal child open
        if state.modal_child.is_some() {
            match &event {
                WindowEvent::CloseRequested => {} // allow close
                WindowEvent::KeyboardInput { .. }
                | WindowEvent::MouseInput { .. }
                | WindowEvent::CursorMoved { .. }
                | WindowEvent::MouseWheel { .. } => return, // block input
                _ => {}                           // allow resize, scale change, etc.
            }
        }

        let _window = &mut state.app_window;

        // Forward event to AccessKit adapter
        #[cfg(feature = "a11y")]
        if let Some(ref mut a11y) = window.a11y {
            a11y.process_event(&window.window_handle, &event);
        }

        // Process accessibility action requests
        #[cfg(feature = "a11y")]
        if let Some(ref a11y) = window.a11y {
            for request in a11y.drain_actions() {
                if let Some(widget_event) = crate::a11y::action_to_widget_event(&request) {
                    window.dispatch_event(&widget_event);
                }
            }
        }

        match event {
            WindowEvent::CloseRequested => {
                log::trace!("Window close requested: {:?}", window_id);
                self.close_window(window_id, event_loop);
            }
            WindowEvent::RedrawRequested => {
                let state = self.windows.get_mut(&window_id);
                let Some(state) = state else { return };
                let window = &mut state.app_window;

                if self.benchmark {
                    let elapsed = self.start_time.elapsed();
                    println!("STARTUP_MS={:.2}", elapsed.as_secs_f64() * 1000.0);
                    println!("MEMORY_KB={}", current_memory_kb());
                    event_loop.exit();
                    return;
                }
                log::trace!("Window redraw requested");
                window.window_handle.pre_present_notify();
                let physical = window.window_handle.inner_size();
                let frame_info = FrameInfo {
                    width: physical.width,
                    height: physical.height,
                    scale_factor: window.window_handle.scale_factor(),
                };
                window.gpu.resize(frame_info.width, frame_info.height);
                window.clear(window.background_color);
                window.next_frame_requested = false;
                (state.on_render)(window, frame_info);
                window.layout_and_paint();
                window.present();
            }
            WindowEvent::Resized(physical_size) => {
                let state = self.windows.get_mut(&window_id);
                let Some(state) = state else { return };
                let window = &mut state.app_window;

                let frame_info = FrameInfo {
                    width: physical_size.width,
                    height: physical_size.height,
                    scale_factor: window.window_handle.scale_factor(),
                };
                window.gpu.resize(frame_info.width, frame_info.height);
                window.clear(window.background_color);
                (state.on_render)(window, frame_info);
                window.layout_and_paint();
                window.present();
                state.resized_this_cycle = true;
            }
            WindowEvent::ScaleFactorChanged { .. } => {
                let state = self.windows.get_mut(&window_id);
                let Some(state) = state else { return };
                state.app_window.request_next_frame();
            }
            WindowEvent::CursorMoved { position, .. } => {
                let state = self.windows.get_mut(&window_id);
                let Some(state) = state else { return };
                let window = &mut state.app_window;

                let pos = Point::new(position.x as f32, position.y as f32);
                state.current_cursor_position = Some(pos);

                if let Some(press_pos) = state.drag_press_position {
                    if !state.drag_active {
                        let dx = pos.x - press_pos.x;
                        let dy = pos.y - press_pos.y;
                        if (dx * dx + dy * dy).sqrt() >= DRAG_THRESHOLD {
                            state.drag_active = true;
                            window.dispatch_event(&WidgetEvent::Drag(DragEvent::DragStart {
                                origin: press_pos,
                            }));
                        }
                    }
                    if state.drag_active {
                        window.dispatch_event(&WidgetEvent::Drag(DragEvent::DragMove {
                            origin: press_pos,
                            current: pos,
                        }));
                        window.request_next_frame();
                        return;
                    }
                }

                if !state.os_drag_paths.is_empty() {
                    for path in state.os_drag_paths.clone() {
                        window.dispatch_event(&WidgetEvent::FileDrop(FileDropEvent::FileHovered {
                            path,
                            position: pos,
                        }));
                    }
                    window.request_next_frame();
                    return;
                }

                window.dispatch_event(&WidgetEvent::Mouse(MouseEvent::MouseMoveEvent(pos)));
                window.request_next_frame();
            }
            WindowEvent::MouseInput {
                state: btn_state,
                button,
                ..
            } => {
                let state = self.windows.get_mut(&window_id);
                let Some(state) = state else { return };
                let window = &mut state.app_window;

                let position = state.current_cursor_position.unwrap_or_default();

                if button == winit::event::MouseButton::Left
                    && btn_state == winit::event::ElementState::Released
                    && state.drag_active
                    && let Some(origin) = state.drag_press_position.take()
                {
                    state.drag_active = false;
                    window.dispatch_event(&WidgetEvent::Drag(DragEvent::DragEnd {
                        origin,
                        current: position,
                    }));
                    window.request_next_frame();
                    return;
                }

                if button == winit::event::MouseButton::Left
                    && btn_state == winit::event::ElementState::Released
                {
                    state.drag_press_position = None;
                    state.drag_active = false;
                }

                let response = window.dispatch_event(&WidgetEvent::Mouse(
                    MouseEvent::MouseClickEvent(MouseClickEvent {
                        button: translate_mouse_button(button),
                        state: translate_mouse_state(btn_state),
                        position,
                    }),
                ));

                if button == winit::event::MouseButton::Left
                    && btn_state == winit::event::ElementState::Pressed
                    && !response.status.is_handled()
                {
                    state.drag_press_position = Some(position);
                    state.drag_active = false;
                }

                window.request_next_frame();
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let state = self.windows.get_mut(&window_id);
                let Some(state) = state else { return };
                let window = &mut state.app_window;

                let scroll_delta = match delta {
                    winit::event::MouseScrollDelta::LineDelta(_, y) => y,
                    winit::event::MouseScrollDelta::PixelDelta(pos) => {
                        const PIXELS_PER_SCROLL_LINE: f32 = 40.0;
                        pos.y as f32 / PIXELS_PER_SCROLL_LINE
                    }
                };
                window.dispatch_event(&WidgetEvent::Mouse(MouseEvent::MouseScrollEvent(
                    scroll_delta,
                )));
                window.request_next_frame();
            }
            WindowEvent::KeyboardInput { event, .. } => {
                let state = self.windows.get_mut(&window_id);
                let Some(state) = state else { return };
                let window = &mut state.app_window;

                let modifiers = Modifiers {
                    shift: self.current_modifiers.shift_key(),
                    ctrl: self.current_modifiers.control_key(),
                    alt: self.current_modifiers.alt_key(),
                };

                let key = translate_key(&event.logical_key);

                match event.state {
                    winit::event::ElementState::Pressed => {
                        window.dispatch_event(&WidgetEvent::Keyboard(KeyboardEvent::KeyPressed {
                            key,
                            modifiers,
                        }));

                        if !modifiers.ctrl && !modifiers.alt {
                            match &event.logical_key {
                                winit::keyboard::Key::Character(c) => {
                                    for ch in c.chars() {
                                        window.dispatch_event(&WidgetEvent::Keyboard(
                                            KeyboardEvent::CharTyped(ch),
                                        ));
                                    }
                                }
                                winit::keyboard::Key::Named(winit::keyboard::NamedKey::Space) => {
                                    window.dispatch_event(&WidgetEvent::Keyboard(
                                        KeyboardEvent::CharTyped(' '),
                                    ));
                                }
                                _ => {}
                            }
                        }
                    }
                    winit::event::ElementState::Released => {
                        window.dispatch_event(&WidgetEvent::Keyboard(KeyboardEvent::KeyReleased {
                            key,
                            modifiers,
                        }));
                    }
                }
                window.request_next_frame();
            }
            WindowEvent::ModifiersChanged(mods) => {
                self.current_modifiers = mods.state();
            }
            WindowEvent::HoveredFile(path) => {
                let state = self.windows.get_mut(&window_id);
                let Some(state) = state else { return };
                let window = &mut state.app_window;

                let position = query_cursor_in_window(&window.window_handle)
                    .or(state.current_cursor_position)
                    .unwrap_or_default();
                state.current_cursor_position = Some(position);
                state.os_drag_paths.push(path.clone());
                window.dispatch_event(&WidgetEvent::FileDrop(FileDropEvent::FileHovered {
                    path,
                    position,
                }));
                window.request_next_frame();
            }
            WindowEvent::DroppedFile(path) => {
                let state = self.windows.get_mut(&window_id);
                let Some(state) = state else { return };
                let window = &mut state.app_window;

                let position = query_cursor_in_window(&window.window_handle)
                    .or(state.current_cursor_position)
                    .unwrap_or_default();
                state.current_cursor_position = Some(position);
                state.os_drag_paths.retain(|p| p != &path);
                window.dispatch_event(&WidgetEvent::FileDrop(FileDropEvent::FileDropped {
                    path,
                    position,
                }));
                window.request_next_frame();
            }
            WindowEvent::HoveredFileCancelled => {
                let state = self.windows.get_mut(&window_id);
                let Some(state) = state else { return };
                let window = &mut state.app_window;

                state.os_drag_paths.clear();
                window.dispatch_event(&WidgetEvent::FileDrop(FileDropEvent::FileCancelled));
                window.request_next_frame();
            }
            _ => {}
        }
    }
}

// ---------------------------------------------------------------------------
// AppWindow
// ---------------------------------------------------------------------------

/// Handle to the underlying OS window.
///
/// Provided to the render callback on each frame. Use it to query window
/// properties, set the root widget, or open additional windows.
pub struct AppWindow {
    window_handle: Arc<winit::window::Window>,
    gpu: Box<dyn GpuContext>,
    root_widget: Option<Box<dyn Widget>>,
    #[cfg(feature = "text")]
    font_manager: aurora_text::font_manager::FontManager,
    #[cfg(feature = "text")]
    font_options: aurora_text::font_options::FontOptions,
    #[cfg(feature = "text")]
    pub swash_cache: aurora_text::cosmic_text::SwashCache,
    text_direction: TextDirection,
    pub(crate) _cursor: winit::window::CursorIcon,
    pub(crate) last_mouse_position: Option<Point>,
    pub(crate) focused_widget_id: Option<u64>,
    next_frame_requested: bool,
    pub(crate) background_color: Color,
    #[cfg(feature = "a11y")]
    a11y: Option<crate::a11y::A11yState>,
    // Multi-window fields
    app_context: Option<AppContext>,
    window_id: Option<WindowId>,
    pending_messages: Vec<Box<dyn std::any::Any + Send>>,
}

impl AppWindow {
    pub(crate) fn new(
        window_handle: Arc<winit::window::Window>,
        config: &WindowConfig,
        event_loop: &ActiveEventLoop,
    ) -> Result<Self, AppError> {
        let gpu: Box<dyn GpuContext> = {
            #[cfg(feature = "wgpu_backend")]
            {
                let _ = event_loop;
                let backend = aurora_gpu::backend::wgpu::WgpuBackend::new(window_handle.clone())
                    .map_err(AppError::GpuInitializationError)?;
                Box::new(backend)
            }
            #[cfg(all(feature = "opengl", not(feature = "wgpu_backend")))]
            {
                let backend =
                    aurora_gpu::backend::glow::GlowBackend::new(&window_handle, event_loop)
                        .map_err(AppError::GpuInitializationError)?;
                Box::new(backend)
            }
            #[cfg(all(
                feature = "software",
                not(feature = "opengl"),
                not(feature = "wgpu_backend")
            ))]
            {
                let _ = event_loop;
                let backend =
                    aurora_gpu::backend::softbuffer::SoftbufferBackend::new(window_handle.clone())
                        .map_err(|err| AppError::GpuInitializationError(err.to_string()))?;
                Box::new(backend)
            }
        };
        #[cfg(feature = "text")]
        {
            let font_manager = {
                let mut fm = if config.use_system_font {
                    aurora_text::font_manager::FontManager::new_with_locale_and_system_db(
                        &config.locale_tag,
                    )
                } else {
                    aurora_text::font_manager::FontManager::new_with_locale(&config.locale_tag)
                };
                for bytes in &config.fonts {
                    fm.load_from_bytes(bytes);
                }
                fm
            };
            let swash_cache = aurora_text::cosmic_text::SwashCache::new();
            Ok(Self {
                window_handle,
                gpu,
                font_manager,
                font_options: config.font_options.clone(),
                swash_cache,
                text_direction: config.text_direction,
                root_widget: None,
                _cursor: winit::window::CursorIcon::Default,
                last_mouse_position: None,
                focused_widget_id: None,
                next_frame_requested: false,
                background_color: config.background_color,
                #[cfg(feature = "a11y")]
                a11y: None,
                app_context: None,
                window_id: None,
                pending_messages: Vec::new(),
            })
        }
        #[cfg(not(feature = "text"))]
        Ok(Self {
            window_handle,
            gpu,
            text_direction: config.text_direction,
            root_widget: None,
            _cursor: winit::window::CursorIcon::Default,
            last_mouse_position: None,
            focused_widget_id: None,
            next_frame_requested: false,
            background_color: config.background_color,
            #[cfg(feature = "a11y")]
            a11y: None,
            app_context: None,
            window_id: None,
            pending_messages: Vec::new(),
        })
    }

    /// Sets the root widget tree for this window.
    pub fn root(&mut self, widget: impl Widget + 'static) {
        self.root_widget = Some(Box::new(widget));
    }

    /// Returns this window's unique identifier.
    pub fn id(&self) -> WindowId {
        self.window_id.expect("WindowId not set")
    }

    /// Returns the multi-window context.
    pub fn app_context(&self) -> &AppContext {
        self.app_context
            .as_ref()
            .expect("AppContext not initialized")
    }

    /// Opens a new window with the given configuration and render callback.
    pub fn open_window<F>(&self, config: WindowConfig, on_render: F)
    where
        F: FnMut(&mut AppWindow, FrameInfo) + Send + 'static,
    {
        self.app_context().open_window(config, on_render);
    }

    /// Opens a modal child window that blocks input on this (parent) window.
    pub fn open_modal<F>(&self, mut config: WindowConfig, on_render: F)
    where
        F: FnMut(&mut AppWindow, FrameInfo) + Send + 'static,
    {
        config.parent = self.window_id;
        config.modal = true;
        self.app_context().open_window(config, on_render);
    }

    /// Drains all pending messages from other windows.
    pub fn drain_messages(&mut self) -> Vec<Box<dyn std::any::Any + Send>> {
        std::mem::take(&mut self.pending_messages)
    }

    pub(crate) fn layout_and_paint(&mut self) {
        let (width, height) = self.gpu.size();
        let available = Size::new(width as f32, height as f32);

        if let Some(ref mut widget) = self.root_widget {
            // Layout phase
            {
                #[cfg(feature = "text")]
                let mut ctx = LayoutCtx {
                    font_manager: &mut self.font_manager,
                    font_options: &self.font_options,
                    direction: self.text_direction,
                };
                #[cfg(not(feature = "text"))]
                let mut ctx = LayoutCtx {
                    direction: self.text_direction,
                };

                widget.layout(available, &mut ctx);
            }

            // Restore focus
            if let Some(focus_id) = self.focused_widget_id {
                let rect = Rect::from_size(available);
                widget.event(&WidgetEvent::Focus(focus_id, false), rect);
            }

            // Restore hover state
            if let Some(pos) = self.last_mouse_position {
                let rect = Rect::from_size(available);
                let response =
                    widget.event(&WidgetEvent::Mouse(MouseEvent::MouseMoveEvent(pos)), rect);
                if let Some(cursor) = response.cursor {
                    let winit_cursor = match cursor {
                        CursorIcon::Default => winit::window::CursorIcon::Default,
                        CursorIcon::Pointer => winit::window::CursorIcon::Pointer,
                        CursorIcon::Text => winit::window::CursorIcon::Text,
                        CursorIcon::Grab => winit::window::CursorIcon::Grab,
                        CursorIcon::Grabbing => winit::window::CursorIcon::Grabbing,
                        CursorIcon::NotAllowed => winit::window::CursorIcon::NotAllowed,
                    };
                    self.window_handle.set_cursor(winit_cursor);
                }

                // Re-layout if hover dirtied composites
                {
                    #[cfg(feature = "text")]
                    let mut ctx = LayoutCtx {
                        font_manager: &mut self.font_manager,
                        font_options: &self.font_options,
                        direction: self.text_direction,
                    };
                    #[cfg(not(feature = "text"))]
                    let mut ctx = LayoutCtx {
                        direction: self.text_direction,
                    };

                    widget.layout(available, &mut ctx);
                }
            }

            // Paint phase
            let buffer = self.gpu.buffer_mut();
            #[cfg(feature = "text")]
            let mut canvas = Canvas::new(
                width,
                height,
                buffer,
                &mut self.font_manager,
                &mut self.swash_cache,
            );
            #[cfg(not(feature = "text"))]
            let mut canvas = Canvas::new(width, height, buffer);

            let rect = Rect::from_size(available);
            widget.paint(&mut canvas, rect);
            widget.paint_overlay(&mut canvas, rect);

            if widget.needs_animation() {
                self.next_frame_requested = true;
            }

            #[cfg(feature = "a11y")]
            if let Some(ref mut a11y) = self.a11y {
                let rect = Rect::from_size(available);
                a11y.update_tree(widget.as_ref(), self.focused_widget_id, rect);
            }
        }
    }

    pub(crate) fn dispatch_event(&mut self, event: &WidgetEvent) -> EventResponse {
        if let WidgetEvent::Mouse(MouseEvent::MouseMoveEvent(pos)) = event {
            self.last_mouse_position = Some(*pos);
        }
        let (width, height) = self.gpu.size();
        let rect = Rect::from_size((width as f32, height as f32).into());
        let Some(ref mut widget) = self.root_widget else {
            return EventResponse::default();
        };
        let overlay_response = widget.event_overlay(event, rect);

        let response = if overlay_response.status.stops_propagation() {
            overlay_response
        } else {
            let normal_response = widget.event(event, rect);
            EventResponse {
                status: if normal_response.status.stops_propagation() {
                    normal_response.status
                } else {
                    overlay_response.status
                },
                cursor: overlay_response.cursor.or(normal_response.cursor),
                request_focus: overlay_response
                    .request_focus
                    .or(normal_response.request_focus),
                focus_next: overlay_response.focus_next || normal_response.focus_next,
                focus_prev: overlay_response.focus_prev || normal_response.focus_prev,
            }
        };

        let is_mouse_move = matches!(event, WidgetEvent::Mouse(MouseEvent::MouseMoveEvent(_)));
        let cursor = response.cursor.or(if is_mouse_move {
            Some(CursorIcon::Default)
        } else {
            None
        });
        if let Some(cursor) = cursor {
            let winit_cursor = match cursor {
                CursorIcon::Default => winit::window::CursorIcon::Default,
                CursorIcon::Pointer => winit::window::CursorIcon::Pointer,
                CursorIcon::Text => winit::window::CursorIcon::Text,
                CursorIcon::Grab => winit::window::CursorIcon::Grab,
                CursorIcon::Grabbing => winit::window::CursorIcon::Grabbing,
                CursorIcon::NotAllowed => winit::window::CursorIcon::NotAllowed,
            };
            self.window_handle.set_cursor(winit_cursor);
        }

        if let Some(new_id) = response.request_focus {
            if let Some(old_id) = self.focused_widget_id
                && old_id != new_id
            {
                widget.event(&WidgetEvent::Blur(old_id), rect);
            }
            self.focused_widget_id = Some(new_id);
        } else if matches!(event, WidgetEvent::Mouse(MouseEvent::MouseClickEvent(_)))
            && response.status.is_handled()
        {
            if let Some(old_id) = self.focused_widget_id {
                widget.event(&WidgetEvent::Blur(old_id), rect);
            }
            self.focused_widget_id = None;
        }

        if response.focus_next || response.focus_prev {
            let mut tab_widgets: Vec<(u32, u64)> = Vec::new();
            collect_tab_widgets(widget.as_ref(), &mut tab_widgets);
            tab_widgets.sort_by_key(|(idx, _)| *idx);

            if !tab_widgets.is_empty() {
                let current_idx = response
                    .request_focus
                    .and_then(|id| tab_widgets.iter().position(|(_, wid)| *wid == id));

                let next = if response.focus_next {
                    match current_idx {
                        Some(i) => (i + 1) % tab_widgets.len(),
                        None => 0,
                    }
                } else {
                    match current_idx {
                        Some(i) if i > 0 => i - 1,
                        _ => tab_widgets.len() - 1,
                    }
                };

                let (_, target_id) = tab_widgets[next];
                if let Some(old_id) = self.focused_widget_id {
                    widget.event(&WidgetEvent::Blur(old_id), rect);
                }
                widget.event(&WidgetEvent::Focus(target_id, true), rect);
                self.focused_widget_id = Some(target_id);
            }
        }
        response
    }

    /// Returns a mutable reference to the window's [`FontManager`](aurora_text::font_manager::FontManager).
    #[cfg(feature = "text")]
    pub fn font_manager(&mut self) -> &mut aurora_text::font_manager::FontManager {
        &mut self.font_manager
    }

    /// Renders a [`TextLayout`] directly into the GPU buffer at the given pixel offset.
    #[cfg(feature = "text")]
    pub fn render_text(&mut self, layout: &TextLayout, x: i32, y: i32) {
        let (width, _) = self.gpu.size();
        let buffer = self.gpu.buffer_mut();
        layout.render(
            &mut self.swash_cache,
            &mut self.font_manager,
            buffer,
            width,
            x,
            y,
            None,
        );
    }

    /// Returns the inner (client-area) size in physical pixels.
    pub fn inner_size(&self) -> Size {
        let inner_size = self.window_handle.inner_size();
        Size::new(inner_size.width as f32, inner_size.height as f32)
    }

    /// Returns the outer (including decorations) size in physical pixels.
    pub fn outer_size(&self) -> Size {
        let outer_size = self.window_handle.outer_size();
        Size::new(outer_size.width as f32, outer_size.height as f32)
    }

    /// Returns the display scale factor (e.g. `2.0` on Retina/HiDPI).
    pub fn scale_factor(&self) -> f64 {
        self.window_handle.scale_factor()
    }

    /// Requests a redraw for the next frame.
    pub fn request_redraw(&self) {
        self.window_handle.request_redraw();
    }

    /// Requests that the render callback be invoked again on the next frame.
    pub fn request_next_frame(&mut self) {
        self.next_frame_requested = true;
    }

    /// Returns a shared reference to the underlying [`winit::window::Window`].
    pub fn window_handle(&self) -> Arc<winit::window::Window> {
        self.window_handle.clone()
    }

    /// Sets the background color used to clear the window before each frame.
    pub fn set_background_color(&mut self, color: Color) {
        self.background_color = color;
    }

    /// Clears the GPU buffer to the given color.
    pub fn clear(&mut self, color: Color) {
        self.gpu.clear(color);
    }

    /// Returns a mutable reference to the raw pixel buffer.
    pub fn buffer_mut(&mut self) -> &mut [u32] {
        self.gpu.buffer_mut()
    }

    /// Copies the buffer to the display surface.
    pub fn present(&mut self) {
        self.gpu.present();
    }

    /// Creates a [`Canvas`] from the GPU buffer and passes it to the closure.
    pub fn draw<F>(&mut self, f: F)
    where
        F: FnOnce(&mut Canvas),
    {
        let (width, height) = self.gpu.size();
        let buffer = self.gpu.buffer_mut();
        let mut canvas = {
            #[cfg(not(feature = "text"))]
            {
                Canvas::new(width, height, buffer)
            }
            #[cfg(feature = "text")]
            {
                Canvas::new(
                    width,
                    height,
                    buffer,
                    &mut self.font_manager,
                    &mut self.swash_cache,
                )
            }
        };
        f(&mut canvas);
    }
}

// ---------------------------------------------------------------------------
// Helper: build WindowAttributes from WindowConfig
// ---------------------------------------------------------------------------

fn build_window_attributes(
    config: &WindowConfig,
    event_loop: &ActiveEventLoop,
) -> WindowAttributes {
    let mut attributes = WindowAttributes::default()
        .with_title(config.title.clone())
        .with_decorations(config.decorations)
        .with_resizable(config.resizable)
        .with_inner_size(dpi::LogicalSize::new(config.size.width, config.size.height))
        .with_visible(false);

    if !config.resizable {
        attributes =
            attributes.with_enabled_buttons(WindowButtons::CLOSE | WindowButtons::MINIMIZE);
    }

    if let Some(position) = config.position
        && let Some(monitor) = resolve_monitor(event_loop, config.monitor)
    {
        let monitor_pos = monitor.position();
        let monitor_size = monitor.size();
        let scale = monitor.scale_factor();

        let pos = match position {
            WindowPosition::Center => {
                let win_w = (config.size.width as f64 * scale) as i32;
                let win_h = (config.size.height as f64 * scale) as i32;
                dpi::PhysicalPosition::new(
                    monitor_pos.x + (monitor_size.width as i32 - win_w) / 2,
                    monitor_pos.y + (monitor_size.height as i32 - win_h) / 2,
                )
            }
            WindowPosition::At(point) => dpi::PhysicalPosition::new(
                monitor_pos.x + (point.x as f64 * scale) as i32,
                monitor_pos.y + (point.y as f64 * scale) as i32,
            ),
        };
        attributes = attributes.with_position(pos);
    }

    if let Some(min_size) = config.min_size {
        attributes =
            attributes.with_min_inner_size(dpi::LogicalSize::new(min_size.width, min_size.height));
    }

    if let Some(icon_data) = &config.icon {
        match winit::window::Icon::from_rgba(
            icon_data.rgba.clone(),
            icon_data.width,
            icon_data.height,
        ) {
            Ok(icon) => {
                attributes = attributes.with_window_icon(Some(icon));
            }
            Err(e) => {
                log::error!("Failed to create window icon: {}", e);
            }
        }
    }

    attributes
}

// ---------------------------------------------------------------------------
// Free functions
// ---------------------------------------------------------------------------

fn translate_mouse_button(button: winit::event::MouseButton) -> MouseButton {
    match button {
        winit::event::MouseButton::Left => MouseButton::Left,
        winit::event::MouseButton::Right => MouseButton::Right,
        winit::event::MouseButton::Middle => MouseButton::Middle,
        winit::event::MouseButton::Back => MouseButton::Back,
        winit::event::MouseButton::Forward => MouseButton::Forward,
        winit::event::MouseButton::Other(_) => MouseButton::Left,
    }
}

fn translate_mouse_state(state: winit::event::ElementState) -> MouseState {
    match state {
        winit::event::ElementState::Pressed => MouseState::Pressed,
        winit::event::ElementState::Released => MouseState::Released,
    }
}

/// Recursively collects `(tab_index, widget_id)` pairs from the widget tree.
fn collect_tab_widgets(widget: &dyn aurora_widgets::widgets::Widget, out: &mut Vec<(u32, u64)>) {
    if let (Some(tab), Some(id)) = (widget.tab_index(), widget.widget_id()) {
        out.push((tab, id));
    }
    for child in widget.children() {
        collect_tab_widgets(child.as_ref(), out);
    }
}

fn translate_key(key: &winit::keyboard::Key) -> Key {
    match key {
        winit::keyboard::Key::Named(named) => match named {
            winit::keyboard::NamedKey::Backspace => Key::Backspace,
            winit::keyboard::NamedKey::Delete => Key::Delete,
            winit::keyboard::NamedKey::ArrowLeft => Key::Left,
            winit::keyboard::NamedKey::ArrowRight => Key::Right,
            winit::keyboard::NamedKey::ArrowUp => Key::Up,
            winit::keyboard::NamedKey::ArrowDown => Key::Down,
            winit::keyboard::NamedKey::Home => Key::Home,
            winit::keyboard::NamedKey::End => Key::End,
            winit::keyboard::NamedKey::Enter => Key::Enter,
            winit::keyboard::NamedKey::Tab => Key::Tab,
            winit::keyboard::NamedKey::Escape => Key::Escape,
            winit::keyboard::NamedKey::Space => Key::Character(' '),
            _ => Key::Other,
        },
        winit::keyboard::Key::Character(c) => {
            let mut chars = c.chars();
            match (chars.next(), chars.next()) {
                (Some(ch), None) => Key::Character(ch),
                _ => Key::Other,
            }
        }
        _ => Key::Other,
    }
}

fn resolve_monitor(
    event_loop: &ActiveEventLoop,
    monitor: WindowMonitor,
) -> Option<winit::monitor::MonitorHandle> {
    let primary = || {
        event_loop
            .primary_monitor()
            .or_else(|| event_loop.available_monitors().next())
    };

    match monitor {
        WindowMonitor::Primary => primary(),
        WindowMonitor::Active => {
            #[cfg(target_os = "windows")]
            if let Some(m) = cursor_monitor(event_loop) {
                return Some(m);
            }
            primary()
        }
        WindowMonitor::Index(index) => event_loop.available_monitors().nth(index).or_else(primary),
    }
}

/// Returns the current process working set size in KB.
#[cfg(target_os = "windows")]
fn current_memory_kb() -> u64 {
    use windows::Win32::System::ProcessStatus::{GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS};
    use windows::Win32::System::Threading::GetCurrentProcess;

    unsafe {
        let mut pmc = PROCESS_MEMORY_COUNTERS::default();
        let _ = GetProcessMemoryInfo(
            GetCurrentProcess(),
            &mut pmc,
            std::mem::size_of::<PROCESS_MEMORY_COUNTERS>() as u32,
        );
        pmc.WorkingSetSize as u64 / 1024
    }
}

/// Returns the current process RSS in KB.
#[cfg(target_os = "linux")]
fn current_memory_kb() -> u64 {
    std::fs::read_to_string("/proc/self/status")
        .ok()
        .and_then(|s| {
            s.lines()
                .find(|l| l.starts_with("VmRSS:"))
                .and_then(|l| l.split_whitespace().nth(1))
                .and_then(|v| v.parse().ok())
        })
        .unwrap_or(0)
}

/// Returns the current process memory in KB (stub for unsupported platforms).
#[cfg(not(any(target_os = "windows", target_os = "linux")))]
fn current_memory_kb() -> u64 {
    0
}

/// Queries the actual cursor position relative to the window's client area.
#[cfg(target_os = "windows")]
fn query_cursor_in_window(window: &winit::window::Window) -> Option<Point> {
    use windows::Win32::Foundation::POINT;
    use windows::Win32::UI::WindowsAndMessaging::GetCursorPos;

    let mut screen_pt = POINT::default();
    unsafe { GetCursorPos(&mut screen_pt) }.ok()?;
    let win_pos = window.inner_position().ok()?;
    Some(Point::new(
        (screen_pt.x - win_pos.x) as f32,
        (screen_pt.y - win_pos.y) as f32,
    ))
}

#[cfg(not(target_os = "windows"))]
fn query_cursor_in_window(_window: &winit::window::Window) -> Option<Point> {
    None
}

#[cfg(target_os = "windows")]
fn cursor_monitor(event_loop: &ActiveEventLoop) -> Option<winit::monitor::MonitorHandle> {
    use windows::Win32::Foundation::POINT;
    use windows::Win32::UI::WindowsAndMessaging::GetCursorPos;

    let mut point = POINT::default();
    unsafe { GetCursorPos(&mut point) }.ok()?;

    let cx = point.x;
    let cy = point.y;

    for monitor in event_loop.available_monitors() {
        let pos = monitor.position();
        let size = monitor.size();
        if cx >= pos.x
            && cx < pos.x + size.width as i32
            && cy >= pos.y
            && cy < pos.y + size.height as i32
        {
            return Some(monitor);
        }
    }
    None
}
