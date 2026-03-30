#[cfg(feature = "a11y")]
pub(crate) mod a11y;
/// Cross-platform windowing, native window handles, and event loop management.
///
/// This crate provides the platform abstraction layer for AuroraUI. It wraps
/// [`winit`] to offer a builder-style API for creating and running application
/// windows.
///
/// # Quick start
///
/// ```no_run
/// use aurora_platform::app::App;
///
/// App::new()
///     .title("Hello, Aurora")
///     .size((800, 600))
///     .run(|window, frame| {
///         // render each frame here
///     })
///     .expect("Failed to start application");
/// ```
///
/// # Modules
///
/// - [`app`] — Application builder, window handle, and frame info.
/// - [`errors`] — Error types for window creation and event loop failures.
pub mod app;
#[cfg(feature = "dialogs")]
pub mod dialogs;
pub mod errors;
pub mod window_controls;
#[cfg(target_os = "windows")]
pub mod windows_titlebar;

pub use winit;
