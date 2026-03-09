#[derive(Debug)]
pub enum AppError {
    WindowCreationFailed(String),
    EventLoopFailed(String),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::WindowCreationFailed(msg) => write!(f, "Window creation failed: {}", msg),
            AppError::EventLoopFailed(msg) => write!(f, "Event loop failed: {}", msg),
        }
    }
}

impl std::error::Error for AppError {}

impl From<winit::error::OsError> for AppError {
    fn from(err: winit::error::OsError) -> Self {
        AppError::WindowCreationFailed(err.to_string())
    }
}

impl From<winit::error::EventLoopError> for AppError {
    fn from(err: winit::error::EventLoopError) -> Self {
        AppError::EventLoopFailed(err.to_string())
    }
}
