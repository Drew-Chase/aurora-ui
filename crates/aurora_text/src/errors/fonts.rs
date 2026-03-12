use std::fmt::Display;
use std::io;

pub enum FontError {
	FailedToLoadFont(io::Error)
}


impl Display for FontError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			FontError::FailedToLoadFont(err) => write!(f, "Failed to load font: {}", err),
		}
	}
}

impl From<io::Error> for FontError {
    fn from(err: io::Error) -> Self {
        FontError::FailedToLoadFont(err)
    }
}