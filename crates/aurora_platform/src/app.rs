use crate::errors::app::AppError;
use aurora_core::geometry::size::Size;
use std::process::ExitCode;
use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::dpi;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{WindowAttributes, WindowId};

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
#[derive(Debug, Clone)]
pub struct App {
    pub title: String,
    pub size: Size,
    pub min_size: Option<Size>,
    pub resizable: bool,
    pub decorations: bool,
    pub custom_titlebar: bool,
}

/// Handle to the underlying OS window.
///
/// Provided to the render callback on each frame. Use it to query window
/// properties or request redraws.
pub struct AppWindow {
    window_handle: Arc<winit::window::Window>,
}

struct AppHandler<F> {
    config: App,
    on_render: F,
    window: Option<AppWindow>,
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
    pub scale_factor: f32,
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

    /// Opens the window and enters the event loop.
    ///
    /// The `on_render` callback is invoked on every [`RedrawRequested`](WindowEvent::RedrawRequested)
    /// event with a reference to the [`AppWindow`] and the current [`FrameInfo`].
    ///
    /// Returns an [`AppError`] if the window or event loop could not be created.
    pub fn run<F>(self, on_render: F) -> Result<(), AppError>
    where
        F: FnMut(&AppWindow, FrameInfo) + 'static,
    {
        let mut app_handler = AppHandler {
            config: self,
            on_render,
            window: None,
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
        }
    }
}

impl AppWindow {
    pub(crate) fn new(window_handle: Arc<winit::window::Window>) -> Self {
        Self { window_handle }
    }

    /// Returns the inner (client-area) size in logical pixels.
    pub fn inner_size(&self) -> Size {
        let inner_size = self.window_handle.inner_size();
        Size::new(inner_size.width as f32, inner_size.height as f32)
    }

    /// Returns the outer (including decorations) size in logical pixels.
    pub fn physical_size(&self) -> Size {
        let inner_size = self.window_handle.outer_size();
        Size::new(inner_size.width as f32, inner_size.height as f32)
    }

    /// Returns the display scale factor (e.g. `2.0` on Retina/HiDPI).
    pub fn scale_factor(&self) -> f32 {
        self.window_handle.scale_factor() as f32
    }

    /// Requests a redraw for the next frame.
    pub fn request_redraw(&self) {
        self.window_handle.request_redraw();
    }

    /// Returns a shared reference to the underlying [`winit::window::Window`].
    pub fn window_handle(&self) -> Arc<winit::window::Window> {
        self.window_handle.clone()
    }
}

impl<F> ApplicationHandler for AppHandler<F>
where
    F: FnMut(&AppWindow, FrameInfo) + 'static,
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
        if let Some(min_size) = min_size {
            attributes = attributes
                .with_min_inner_size(dpi::LogicalSize::new(min_size.width, min_size.height));
        }
        self.window = match event_loop.create_window(attributes) {
            Ok(window) => Some(AppWindow::new(Arc::new(window))),
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
            .as_ref()
            .expect("Window redraw request without a valid window");
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
                    scale_factor: window.window_handle.scale_factor() as f32,
                };

                (self.on_render)(window, frame_info);
            }
            WindowEvent::Resized(physical_size) => {
                let frame_info = FrameInfo {
                    width: physical_size.width,
                    height: physical_size.height,
                    scale_factor: window.window_handle.scale_factor() as f32,
                };
                (self.on_render)(window, frame_info);
            }
            WindowEvent::ScaleFactorChanged { .. } => {
                window.request_redraw();
            }
            _ => {}
        }
    }
}
