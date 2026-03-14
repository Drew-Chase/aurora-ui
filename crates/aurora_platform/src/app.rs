#![doc = include_str!("../../../.wiki/App.md")]

use crate::errors::app::AppError;
use aurora_core::color::Color;
use aurora_core::geometry::point::Point;
use aurora_core::geometry::rect::Rect;
use aurora_core::geometry::size::Size;
use aurora_core::kmi::cursor_icon::CursorIcon;
use aurora_core::kmi::mouse::{MouseButton, MouseClickEvent, MouseEvent, MouseState};
use aurora_gpu::gpu_context::GpuContext;
use aurora_render::canvas::Canvas;
#[cfg(feature = "text")]
use aurora_text::text_layout::TextLayout;
use aurora_widgets::widgets::{LayoutCtx, Widget};
use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::dpi;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{WindowAttributes, WindowButtons, WindowId};

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
    #[cfg(feature = "text")]
    pub font_options: aurora_text::font_options::FontOptions,
    #[cfg(feature = "text")]
    pub fonts: Vec<&'static [u8]>,
}

/// Handle to the underlying OS window.
///
/// Provided to the render callback on each frame. Use it to query window
/// properties or request redraws.
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
    pub(crate) cursor: winit::window::CursorIcon,
    pub(crate) last_mouse_position: Option<Point>,
}

struct AppHandler<F> {
    config: App,
    on_render: F,
    window: Option<AppWindow>,
    pub(crate) current_cursor_position: Option<Point>,
}

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
    pub fn use_system_fonts(mut self)->Self{
        self.use_system_font = true;
        self
    }

    /// Enables or disables system font discovery.
    ///
    /// Enabling system font discovery can cause a ~200ms delay on startup but allows for more font options.
    pub fn set_use_system_font(mut self, use_system_font: bool)->Self{
        self.use_system_font = use_system_font;
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

    /// Opens the window and enters the event loop.
    ///
    /// The `on_render` callback is invoked on every [`RedrawRequested`](WindowEvent::RedrawRequested)
    /// event with a reference to the [`AppWindow`] and the current [`FrameInfo`].
    ///
    /// Returns an [`AppError`] if the window or event loop could not be created.
    pub fn run<F>(self, on_render: F) -> Result<(), AppError>
    where
        F: FnMut(&mut AppWindow, FrameInfo) + 'static,
    {
        let mut app_handler = AppHandler {
            config: self,
            on_render,
            window: None,
            current_cursor_position: None,
        };
        let event_loop = winit::event_loop::EventLoop::new().map_err(AppError::from)?;
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
            #[cfg(feature = "text")]
            font_options: aurora_text::font_options::FontOptions::default(),
            #[cfg(feature = "text")]
            fonts: vec![],
        }
    }
}

impl AppWindow {
    pub(crate) fn new(
        window_handle: Arc<winit::window::Window>,
        config: &App,
    ) -> Result<Self, AppError> {
        let gpu: Box<dyn GpuContext> = {
            #[cfg(feature = "software")]
            {
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
                    aurora_text::font_manager::FontManager::new_with_system_db()
                } else {
                    aurora_text::font_manager::FontManager::new()
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
                root_widget: None,
                cursor: winit::window::CursorIcon::Default,
                last_mouse_position: None,
            })
        }
        #[cfg(not(feature = "text"))]
        Ok(Self {
            window_handle,
            gpu,
            root_widget: None,
            cursor: winit::window::CursorIcon::Default,
            last_mouse_position: None,
        })
    }
    /// Lays out and paints a root widget tree into the window.
    ///
    /// Runs the layout phase (computing sizes) then the paint phase (drawing
    /// into the canvas) for the given widget and all its children.
    pub fn root(&mut self, widget: impl Widget + 'static) {
        if self.root_widget.is_none() {
            self.root_widget = Some(Box::new(widget));
        }
    }

    pub(crate) fn layout_and_paint(&mut self) {
        let (width, height) = self.gpu.size();
        let available = Size::new(width as f32, height as f32);

        if let Some(ref mut widget) = self.root_widget {
            // Layout phase — may rebuild dirty composites
            {
                #[cfg(feature = "text")]
                let mut ctx = LayoutCtx {
                    font_manager: &mut self.font_manager,
                    font_options: &self.font_options,
                };
                #[cfg(not(feature = "text"))]
                let mut ctx = LayoutCtx;

                widget.layout(available, &mut ctx);
            }

            // Restore hover state on rebuilt widgets so visual hover and cursor
            // survive composite rebuilds triggered by clicks or state changes.
            if let Some(pos) = self.last_mouse_position {
                let rect = Rect::from_size(available);
                let response = widget.event(&MouseEvent::MouseMoveEvent(pos), rect);
                let cursor = response.cursor.unwrap_or(CursorIcon::Default);
                let winit_cursor = match cursor {
                    CursorIcon::Default => winit::window::CursorIcon::Default,
                    CursorIcon::Pointer => winit::window::CursorIcon::Pointer,
                    CursorIcon::Text => winit::window::CursorIcon::Text,
                    CursorIcon::Grab => winit::window::CursorIcon::Grab,
                    CursorIcon::Grabbing => winit::window::CursorIcon::Grabbing,
                    CursorIcon::NotAllowed => winit::window::CursorIcon::NotAllowed,
                };
                self.window_handle.set_cursor(winit_cursor);

                // Re-layout if hover dirtied any composites
                {
                    #[cfg(feature = "text")]
                    let mut ctx = LayoutCtx {
                        font_manager: &mut self.font_manager,
                        font_options: &self.font_options,
                    };
                    #[cfg(not(feature = "text"))]
                    let mut ctx = LayoutCtx;

                    widget.layout(available, &mut ctx);
                }
            }

            // Paint phase — safe to borrow font_manager again
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
        }
    }

    pub(crate) fn dispatch_event(&mut self, event: &MouseEvent) {
        if let MouseEvent::MouseMoveEvent(pos) = event {
            self.last_mouse_position = Some(*pos);
        }
        let (width, height) = self.gpu.size();
        let rect = Rect::from_size((width as f32, height as f32).into());
        if let Some(ref mut widget) = self.root_widget {
            let response = widget.event(event, rect);
            let cursor = response.cursor.unwrap_or(CursorIcon::Default);
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
    }

    /// Returns a mutable reference to the window's [`FontManager`](aurora_text::font_manager::FontManager).
    #[cfg(feature = "text")]
    pub fn font_manager(&mut self) -> &mut aurora_text::font_manager::FontManager {
        &mut self.font_manager
    }

    /// Renders a [`TextLayout`] directly into the GPU buffer at the given pixel offset.
    ///
    /// For most use cases, prefer [`draw`](Self::draw) with [`Canvas::draw_text`](aurora_render::canvas::Canvas)
    /// or the [`Text`](aurora_widgets::text_widget::Text) widget instead.
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
        );
    }

    /// Returns the inner (client-area) size in logical pixels.
    pub fn inner_size(&self) -> Size {
        let inner_size = self.window_handle.inner_size();
        Size::new(inner_size.width as f32, inner_size.height as f32)
    }

    /// Returns the outer (including decorations) size in physical pixels.
    pub fn outer_size(&self) -> Size {
        let inner_size = self.window_handle.outer_size();
        Size::new(inner_size.width as f32, inner_size.height as f32)
    }

    /// Returns the display scale factor (e.g. `2.0` on Retina/HiDPI).
    pub fn scale_factor(&self) -> f64 {
        self.window_handle.scale_factor()
    }

    /// Requests a redraw for the next frame.
    pub fn request_redraw(&self) {
        self.window_handle.request_redraw();
    }

    /// Returns a shared reference to the underlying [`winit::window::Window`].
    pub fn window_handle(&self) -> Arc<winit::window::Window> {
        self.window_handle.clone()
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
    ///
    /// This is the primary way to draw content onto the window.
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

impl<F> ApplicationHandler for AppHandler<F>
where
    F: FnMut(&mut AppWindow, FrameInfo) + 'static,
{
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if let Some(window) = &self.window {
            window.window_handle.focus_window();
            return;
        }
        let config = &self.config;
        let title = config.title.clone();
        let size = config.size;
        let min_size = config.min_size;
        let resizable = config.resizable;
        let decorations = config.decorations;

        let mut attributes = WindowAttributes::default()
            .with_title(title)
            .with_decorations(decorations)
            .with_resizable(resizable)
            .with_inner_size(dpi::LogicalSize::new(size.width, size.height));
        if !resizable {
            attributes = attributes.with_enabled_buttons(
                WindowButtons::CLOSE | WindowButtons::MINIMIZE,
            );
        }
        if let Some(position) = config.position {
            if let Some(monitor) = resolve_monitor(event_loop, config.monitor) {
                let monitor_pos = monitor.position();
                let monitor_size = monitor.size();
                let scale = monitor.scale_factor();

                let pos = match position {
                    WindowPosition::Center => {
                        let win_w = (size.width as f64 * scale) as i32;
                        let win_h = (size.height as f64 * scale) as i32;
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
        }
        if let Some(min_size) = min_size {
            attributes = attributes
                .with_min_inner_size(dpi::LogicalSize::new(min_size.width, min_size.height));
        }
        self.window = match event_loop.create_window(attributes) {
            Ok(window) => {
                let handle = Arc::new(window);
                match AppWindow::new(handle, config) {
                    Ok(app_window) => Some(app_window),
                    Err(err) => {
                        log::error!("Failed to create window: {}", err);
                        event_loop.exit();
                        return;
                    }
                }
            }
            Err(err) => {
                log::error!("Failed to create window: {}", err);
                event_loop.exit();
                return;
            }
        };
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        let window = self
            .window
            .as_mut()
            .expect("Window redraw request without a valid window");
        let background_color = self.config.background_color;
        match event {
            WindowEvent::CloseRequested => {
                log::trace!("Window close requested");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                log::trace!("Window redraw requested");
                window.window_handle.pre_present_notify();
                let physical = window.window_handle.inner_size();
                let frame_info = FrameInfo {
                    width: physical.width,
                    height: physical.height,
                    scale_factor: window.window_handle.scale_factor(),
                };
                window.gpu.resize(frame_info.width, frame_info.height);
                window.clear(background_color);
                (self.on_render)(window, frame_info);
                window.layout_and_paint();
                window.present();
            }
            WindowEvent::Resized(physical_size) => {
                let frame_info = FrameInfo {
                    width: physical_size.width,
                    height: physical_size.height,
                    scale_factor: window.window_handle.scale_factor(),
                };
                window.gpu.resize(frame_info.width, frame_info.height);
                window.clear(background_color);
                (self.on_render)(window, frame_info);
                window.layout_and_paint();
                window.present();
                window.request_redraw();
            }
            WindowEvent::ScaleFactorChanged { .. } => {
                window.request_redraw();
            }
            WindowEvent::CursorMoved { position, .. } => {
                let pos = Point::new(position.x as f32, position.y as f32);
                self.current_cursor_position = Some(pos);
                let event = MouseEvent::MouseMoveEvent(pos);
                window.dispatch_event(&event);
                window.request_redraw();
            }
            WindowEvent::MouseInput { state, button, .. } => {
                if let Some(current_cursor_position) = self.current_cursor_position {
                    let event = MouseEvent::MouseClickEvent(MouseClickEvent {
                        button: translate_mouse_button(button),
                        state: translate_mouse_state(state),
                        position: current_cursor_position,
                    });
                    window.dispatch_event(&event);
                    window.request_redraw();
                }
            }
            _ => {}
        }
    }
}

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
        WindowMonitor::Index(index) => {
            event_loop.available_monitors().nth(index).or_else(primary)
        }
    }
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
