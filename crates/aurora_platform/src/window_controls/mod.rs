use aurora_core::geometry::rect::Rect;
use aurora_core::geometry::size::Size;
use aurora_core::kmi::WidgetEvent;
use aurora_core::kmi::cursor_icon::CursorIcon;
use aurora_core::kmi::mouse::{MouseEvent, MouseState};
use aurora_render::canvas::Canvas;
use aurora_widgets::widgets::{EventResponse, EventStatus, LayoutCtx, Widget};
use std::sync::Arc;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;

/// Which button in the window controls.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ControlButton {
    Minimize,
    Maximize,
    Close,
}

/// Platform-native window control buttons (minimize, maximize, close).
///
/// Renders with the native style for the current platform:
/// - **Windows**: rectangular buttons on the right (Win11 style)
/// - **macOS**: stoplight circles on the left
/// - **Linux**: GTK-style circular buttons on the right
///
/// By default, each button performs its standard action. Use the
/// `on_minimize`, `on_maximize`, and `on_close` builder methods to
/// override with custom behavior (e.g. minimize-to-tray).
///
/// # Examples
///
/// ```ignore
/// // Default behavior:
/// WindowControls::new(window.window_handle())
///
/// // Custom close (minimize to tray):
/// WindowControls::new(window.window_handle())
///     .on_close(|| { /* minimize to tray */ })
///
/// // Hide minimize, disable maximize:
/// WindowControls::new(window.window_handle())
///     .show_minimize(false)
///     .maximize_enabled(false)
/// ```
pub struct WindowControls {
    window: Arc<winit::window::Window>,

    // Appearance
    dark: bool,

    // Visibility
    show_minimize: bool,
    show_maximize: bool,
    show_close: bool,

    // Enabled state
    minimize_enabled: bool,
    maximize_enabled: bool,
    close_enabled: bool,

    // Optional callback overrides
    on_minimize: Option<Box<dyn FnMut()>>,
    on_maximize: Option<Box<dyn FnMut()>>,
    on_close: Option<Box<dyn FnMut()>>,

    // Internal state
    hovered_button: Option<ControlButton>,
    pressed_button: Option<ControlButton>,
    button_rects: Vec<(ControlButton, Rect)>,
    last_size: Size,
}

impl WindowControls {
    /// Creates window controls with default behavior for the given window.
    pub fn new(window: Arc<winit::window::Window>) -> Self {
        Self {
            window,
            dark: false,
            show_minimize: true,
            show_maximize: true,
            show_close: true,
            minimize_enabled: true,
            maximize_enabled: true,
            close_enabled: true,
            on_minimize: None,
            on_maximize: None,
            on_close: None,
            hovered_button: None,
            pressed_button: None,
            button_rects: Vec::new(),
            last_size: Size::zero(),
        }
    }

    /// Sets whether the minimize button is visible.
    pub fn show_minimize(mut self, show: bool) -> Self {
        self.show_minimize = show;
        self
    }

    /// Sets whether the maximize button is visible.
    pub fn show_maximize(mut self, show: bool) -> Self {
        self.show_maximize = show;
        self
    }

    /// Sets whether the close button is visible.
    pub fn show_close(mut self, show: bool) -> Self {
        self.show_close = show;
        self
    }

    /// Hides the minimize button.
    pub fn hide_minimize(self) -> Self {
        self.show_minimize(false)
    }

    /// Hides the maximize button.
    pub fn hide_maximize(self) -> Self {
        self.show_maximize(false)
    }

    /// Hides the close button.
    pub fn hide_close(self) -> Self {
        self.show_close(false)
    }

    /// Sets whether the minimize button is clickable.
    pub fn minimize_enabled(mut self, enabled: bool) -> Self {
        self.minimize_enabled = enabled;
        self
    }

    /// Sets whether the maximize button is clickable.
    pub fn maximize_enabled(mut self, enabled: bool) -> Self {
        self.maximize_enabled = enabled;
        self
    }

    /// Sets whether the close button is clickable.
    pub fn close_enabled(mut self, enabled: bool) -> Self {
        self.close_enabled = enabled;
        self
    }

    /// Disables the minimize button (still visible but grayed out).
    pub fn disable_minimize(self) -> Self {
        self.minimize_enabled(false)
    }

    /// Disables the maximize button (still visible but grayed out).
    pub fn disable_maximize(self) -> Self {
        self.maximize_enabled(false)
    }

    /// Disables the close button (still visible but grayed out).
    pub fn disable_close(self) -> Self {
        self.close_enabled(false)
    }

    /// Overrides the minimize button action.
    pub fn on_minimize(mut self, f: impl FnMut() + 'static) -> Self {
        self.on_minimize = Some(Box::new(f));
        self
    }

    /// Overrides the maximize button action.
    pub fn on_maximize(mut self, f: impl FnMut() + 'static) -> Self {
        self.on_maximize = Some(Box::new(f));
        self
    }

    /// Overrides the close button action.
    pub fn on_close(mut self, f: impl FnMut() + 'static) -> Self {
        self.on_close = Some(Box::new(f));
        self
    }

    /// Enables dark mode (white icons for dark titlebars).
    ///
    /// When `true`, icons render as white instead of black and hover
    /// backgrounds use semi-transparent white overlays. Use this when
    /// placing controls on a dark titlebar background.
    pub fn dark(mut self, dark: bool) -> Self {
        self.dark = dark;
        self
    }

    /// Returns whether the button is enabled.
    fn is_enabled(&self, button: ControlButton) -> bool {
        match button {
            ControlButton::Minimize => self.minimize_enabled,
            ControlButton::Maximize => self.maximize_enabled,
            ControlButton::Close => self.close_enabled,
        }
    }

    /// Executes the action for the given button.
    fn execute_action(&mut self, button: ControlButton) {
        if !self.is_enabled(button) {
            return;
        }
        match button {
            ControlButton::Minimize => {
                if let Some(ref mut f) = self.on_minimize {
                    f();
                } else {
                    self.window.set_minimized(true);
                }
            }
            ControlButton::Maximize => {
                if let Some(ref mut f) = self.on_maximize {
                    f();
                } else {
                    let is_maximized = self.window.is_maximized();
                    self.window.set_maximized(!is_maximized);
                }
            }
            ControlButton::Close => {
                if let Some(ref mut f) = self.on_close {
                    f();
                } else {
                    close_window(&self.window);
                }
            }
        }
    }

    /// Returns which visible buttons to render, in platform order.
    fn visible_buttons(&self) -> Vec<ControlButton> {
        let mut buttons = Vec::with_capacity(3);
        // macOS order: close, minimize, maximize (left to right)
        #[cfg(target_os = "macos")]
        {
            if self.show_close {
                buttons.push(ControlButton::Close);
            }
            if self.show_minimize {
                buttons.push(ControlButton::Minimize);
            }
            if self.show_maximize {
                buttons.push(ControlButton::Maximize);
            }
        }
        // Windows/Linux order: minimize, maximize, close (left to right)
        #[cfg(not(target_os = "macos"))]
        {
            if self.show_minimize {
                buttons.push(ControlButton::Minimize);
            }
            if self.show_maximize {
                buttons.push(ControlButton::Maximize);
            }
            if self.show_close {
                buttons.push(ControlButton::Close);
            }
        }
        buttons
    }
}

/// Platform-specific window close.
#[cfg(target_os = "windows")]
fn close_window(window: &winit::window::Window) {
    use ::windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
    use ::windows::Win32::UI::WindowsAndMessaging::{PostMessageW, WM_CLOSE};
    use std::ffi::c_void;
    use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};

    if let Ok(handle) = window.window_handle()
        && let RawWindowHandle::Win32(h) = handle.as_raw()
    {
        let hwnd = HWND(h.hwnd.get() as *mut c_void);
        unsafe {
            let _ = PostMessageW(Some(hwnd), WM_CLOSE, WPARAM(0), LPARAM(0));
        }
    }
}

#[cfg(target_os = "macos")]
fn close_window(window: &winit::window::Window) {
    use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};

    if let Ok(handle) = window.window_handle()
        && let RawWindowHandle::AppKit(h) = handle.as_raw()
    {
        // SAFETY: h.ns_view is a valid, non-null pointer to an NSView
        // provided by winit's HasWindowHandle implementation. The pointer
        // remains valid for the duration of this scope.
        unsafe {
            let ns_view: &objc2_app_kit::NSView =
                &*(h.ns_view.as_ptr() as *const objc2_app_kit::NSView);
            if let Some(ns_window) = ns_view.window() {
                ns_window.performClose(None);
            }
        }
    }
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
fn close_window(_window: &winit::window::Window) {
    std::process::exit(0);
}

// ─── Platform-specific rendering constants ───

#[cfg(target_os = "windows")]
const BUTTON_WIDTH: f32 = 46.0;
#[cfg(target_os = "windows")]
const BUTTON_HEIGHT: f32 = 32.0;

#[cfg(target_os = "macos")]
const BUTTON_DIAMETER: f32 = 12.0;
#[cfg(target_os = "macos")]
const BUTTON_SPACING: f32 = 8.0;
#[cfg(target_os = "macos")]
const STOPLIGHT_PADDING: f32 = 8.0;

#[cfg(target_os = "linux")]
const BUTTON_DIAMETER: f32 = 20.0;
#[cfg(target_os = "linux")]
const BUTTON_SPACING: f32 = 6.0;
#[cfg(target_os = "linux")]
const BUTTON_PADDING: f32 = 6.0;

// ─── Widget implementation ───

impl Widget for WindowControls {
    fn layout(&mut self, available: Size, _ctx: &mut LayoutCtx) -> Size {
        let buttons = self.visible_buttons();
        self.button_rects.clear();

        #[cfg(target_os = "windows")]
        {
            let total_width = buttons.len() as f32 * BUTTON_WIDTH;
            let height = BUTTON_HEIGHT.min(available.height);
            let start_x = available.width - total_width;
            for (i, button) in buttons.iter().enumerate() {
                let x = start_x + i as f32 * BUTTON_WIDTH;
                self.button_rects
                    .push((*button, Rect::new(x, 0.0, x + BUTTON_WIDTH, height)));
            }
            self.last_size = Size::new(total_width, height);
        }

        #[cfg(target_os = "macos")]
        {
            let total_width = STOPLIGHT_PADDING * 2.0
                + buttons.len() as f32 * BUTTON_DIAMETER
                + (buttons.len().saturating_sub(1)) as f32 * BUTTON_SPACING;
            let height = available.height;
            let center_y = height / 2.0;
            let mut x = STOPLIGHT_PADDING;
            for button in &buttons {
                let rect = Rect::new(
                    x,
                    center_y - BUTTON_DIAMETER / 2.0,
                    x + BUTTON_DIAMETER,
                    center_y + BUTTON_DIAMETER / 2.0,
                );
                self.button_rects.push((*button, rect));
                x += BUTTON_DIAMETER + BUTTON_SPACING;
            }
            self.last_size = Size::new(total_width, height);
        }

        #[cfg(target_os = "linux")]
        {
            let total_width = BUTTON_PADDING * 2.0
                + buttons.len() as f32 * BUTTON_DIAMETER
                + (buttons.len().saturating_sub(1)) as f32 * BUTTON_SPACING;
            let height = available.height;
            let center_y = height / 2.0;
            let start_x = available.width - total_width + BUTTON_PADDING;
            for (i, button) in buttons.iter().enumerate() {
                let x = start_x + i as f32 * (BUTTON_DIAMETER + BUTTON_SPACING);
                let rect = Rect::new(
                    x,
                    center_y - BUTTON_DIAMETER / 2.0,
                    x + BUTTON_DIAMETER,
                    center_y + BUTTON_DIAMETER / 2.0,
                );
                self.button_rects.push((*button, rect));
            }
            self.last_size = Size::new(total_width, height);
        }

        self.last_size
    }

    fn paint(&self, canvas: &mut Canvas, rect: Rect) {
        #[cfg(any(target_os = "windows", target_os = "linux"))]
        let is_maximized = self.window.is_maximized();

        for (button, button_rect) in &self.button_rects {
            let translated = button_rect.translate(&rect.origin());
            let is_hovered = self.hovered_button == Some(*button);
            let is_pressed = self.pressed_button == Some(*button);
            let is_enabled = self.is_enabled(*button);

            #[cfg(target_os = "windows")]
            windows::paint_button(
                canvas,
                translated,
                *button,
                is_hovered,
                is_pressed,
                is_enabled,
                is_maximized,
                self.dark,
            );

            #[cfg(target_os = "macos")]
            macos::paint_button(
                canvas, translated, *button, is_hovered, is_pressed, is_enabled,
            );

            #[cfg(target_os = "linux")]
            linux::paint_button(
                canvas,
                translated,
                *button,
                is_hovered,
                is_pressed,
                is_enabled,
                is_maximized,
                self.dark,
            );
        }
    }

    fn children(&self) -> &[Box<dyn Widget>] {
        &[]
    }

    fn event(&mut self, event: &WidgetEvent, rect: Rect) -> EventResponse {
        let mouse = match event {
            WidgetEvent::Mouse(e) => e,
            _ => return EventResponse::default(),
        };

        match mouse {
            MouseEvent::MouseMoveEvent(pos) => {
                let mut hit = None;
                for (button, button_rect) in &self.button_rects {
                    let translated = button_rect.translate(&rect.origin());
                    if translated.contains(pos) && self.is_enabled(*button) {
                        hit = Some(*button);
                        break;
                    }
                }
                self.hovered_button = hit;
                EventResponse {
                    status: if hit.is_some() {
                        EventStatus::Consumed
                    } else {
                        EventStatus::Ignored
                    },
                    cursor: if hit.is_some() {
                        Some(CursorIcon::Pointer)
                    } else {
                        Some(CursorIcon::Default)
                    },
                    ..Default::default()
                }
            }
            MouseEvent::MouseClickEvent(click) => {
                for (button, button_rect) in &self.button_rects {
                    let translated = button_rect.translate(&rect.origin());
                    if translated.contains(&click.position) && self.is_enabled(*button) {
                        if click.state == MouseState::Pressed {
                            self.pressed_button = Some(*button);
                            return EventResponse {
                                status: EventStatus::Consumed,
                                ..Default::default()
                            };
                        } else if click.state == MouseState::Released
                            && self.pressed_button == Some(*button)
                        {
                            self.pressed_button = None;
                            self.execute_action(*button);
                            return EventResponse {
                                status: EventStatus::Consumed,
                                ..Default::default()
                            };
                        }
                    }
                }
                if click.state == MouseState::Released {
                    self.pressed_button = None;
                }
                EventResponse::default()
            }
            _ => EventResponse::default(),
        }
    }
}
