#![doc = include_str!("../../README.md")]
#![doc = ""]
#![doc = "# Feature Reference"]
#![doc = ""]
#![cfg_attr(feature = "document-features", doc = document_features::document_features!())]
#![doc = ""]
#![doc = include_str!("../features.md")]

pub mod prelude;

#[cfg(not(any(feature = "software", feature = "opengl", feature = "wgpu_backend")))]
compile_error!(
    "At least one GPU backend feature must be enabled (e.g. 'software', 'opengl', 'wgpu_backend')"
);
#[cfg(feature = "animate")]
pub use aurora_animate;
pub use aurora_core;
pub use aurora_gpu;
#[cfg(feature = "i18n")]
pub use aurora_i18n;
#[cfg(feature = "i18n")]
pub use aurora_lang;
pub use aurora_platform;
pub use aurora_render;
#[cfg(feature = "text")]
pub use aurora_text;
pub use aurora_theme;
pub use aurora_theming;
pub use aurora_widgets;
