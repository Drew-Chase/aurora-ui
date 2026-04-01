/// Derive macro for generating builder-pattern setters (without Widget impl).
pub use aurora_macros::CompositeWidget;
/// Attribute macro that turns a config struct into a full composite widget.
pub use aurora_macros::composite_widget;

/// A colored rectangle container with optional child, corners, and padding.
pub mod box_widget;
/// Layout containers: Column, Row, Stack, Positioned.
pub mod layout;
/// The [`Widget`](widgets::Widget) trait and layout context types.
pub mod widgets;

/// Stateful composite widget with rebuild-on-change semantics.
pub mod composite;
/// Interactive widgets: TouchArea, Button.
pub mod interactables;
/// Single-line text input widget (requires the `text` feature).
#[cfg(feature = "text")]
pub mod text_input;
/// Text display widget (requires the `text` feature).
#[cfg(feature = "text")]
pub mod text_widget;

/// Raster image widget (requires the `image` feature).
#[cfg(feature = "image")]
pub mod image_widget;

/// SVG image widget (requires the `svg` feature).
#[cfg(feature = "svg")]
pub mod svg_widget;

/// Form validation: [`Validator`](validation::Validator) trait, built-in validators, and [`FormValidator`](validation::FormValidator).
pub mod validation;

/// Component library — display, interactive, data, and overlay widgets.
#[cfg(feature = "text")]
pub mod components;
