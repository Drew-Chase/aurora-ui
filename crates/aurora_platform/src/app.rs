use crate::errors::app::AppError;
use aurora_core::geometry::size::Size;
use std::process::ExitCode;
use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::dpi;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{WindowAttributes, WindowId};

#[derive(Debug, Clone)]
pub struct App {
    pub title: String,
    pub size: Size,
    pub min_size: Option<Size>,
    pub resizable: bool,
    pub decorations: bool,
    pub custom_titlebar: bool,
}

pub struct AppWindow {
    window_handle: Arc<winit::window::Window>,
}

struct AppHandler<F> {
    config: App,
    on_render: F,
    window: Option<AppWindow>,
}

#[derive(Debug)]
pub struct FrameInfo {
    pub width: u32,
    pub height: u32,
    pub scale_factor: f32,
}

impl App {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }
    pub fn size(mut self, size: impl Into<Size>) -> Self {
        self.size = size.into();
        self
    }
    pub fn width(mut self, width: impl Into<f32>) -> Self {
        self.size.width = width.into();
        self
    }
    pub fn height(mut self, height: impl Into<f32>) -> Self {
        self.size.height = height.into();
        self
    }
    pub fn min_size(mut self, min_size: impl Into<Size>) -> Self {
        self.min_size = Some(min_size.into());
        self
    }
    pub fn custom_titlebar(mut self, custom_titlebar: bool) -> Self {
        self.custom_titlebar = custom_titlebar;
        self.decorations = false;
        self
    }
    pub fn decorations(mut self, decorations: bool) -> Self {
        if self.custom_titlebar {
            self.decorations = false;
            return self;
        }
        self.decorations = decorations;
        self
    }
    pub fn resizable(mut self, resizable: bool) -> Self {
        self.resizable = resizable;
        self
    }

    pub fn run<F>(self, on_render: F) -> Result<ExitCode, AppError>
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
        Ok(ExitCode::SUCCESS)
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
    pub fn inner_size(&self) -> Size {
        let inner_size = self.window_handle.inner_size();
        Size::new(inner_size.width as f32, inner_size.height as f32)
    }
    pub fn physical_size(&self) -> Size {
        let inner_size = self.window_handle.outer_size();
        Size::new(inner_size.width as f32, inner_size.height as f32)
    }
    pub fn scale_factor(&self) -> f32 {
        self.window_handle.scale_factor() as f32
    }
    pub fn request_redraw(&self) {
        self.window_handle.request_redraw();
    }
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

                let frame_info = FrameInfo {
                    width: window.inner_size().width as u32,
                    height: window.inner_size().height as u32,
                    scale_factor: window.scale_factor(),
                };

                (self.on_render)(window, frame_info);
            }
            _ => {}
        }
    }
}
