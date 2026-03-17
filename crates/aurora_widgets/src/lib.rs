/// A colored rectangle container with optional child, corners, and padding.
pub mod box_widget;
/// The [`Widget`](widgets::Widget) trait and layout context types.
pub mod widgets;
/// Layout containers: Column, Row, Stack, Positioned.
pub mod layout;

/// Text display widget (requires the `text` feature).
#[cfg(feature = "text")]
pub mod text_widget;
/// Interactive widgets: TouchArea, Button.
pub mod interactables;
/// Stateful composite widget with rebuild-on-change semantics.
pub mod composite;

/// Raster image widget (requires the `image` feature).
#[cfg(feature = "image")]
pub mod image_widget;

/// SVG image widget (requires the `svg` feature).
#[cfg(feature = "svg")]
pub mod svg_widget;
