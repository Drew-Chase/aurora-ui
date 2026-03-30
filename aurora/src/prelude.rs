// Core types
pub use crate::aurora_core::{hex, hexa, hsl, hsla, rgb, rgba};
pub use aurora_core::color::{Color, IntoColor};
pub use aurora_core::geometry::corners::Corners;
pub use aurora_core::geometry::edges::Edges;
pub use aurora_core::geometry::point::Point;
pub use aurora_core::geometry::rect::Rect;
pub use aurora_core::geometry::size::Size;
pub use aurora_core::kmi::WidgetEvent;
pub use aurora_core::kmi::cursor_icon::CursorIcon;
pub use aurora_core::kmi::keyboard::{Key, KeyboardEvent, Modifiers};
pub use aurora_core::kmi::mouse::{MouseButton, MouseClickEvent, MouseEvent, MouseState};

// Platform
pub use aurora_platform::app::{
    App, AppWindow, FrameInfo, IconData, WindowMonitor, WindowPosition,
};
pub use aurora_platform::errors::app::AppError;
pub use aurora_platform::window_controls::WindowControls;

// Rendering
pub use aurora_render::canvas::Canvas;

// Widgets
pub use aurora_macros::{CompositeWidget, composite_widget};
pub use aurora_widgets::box_widget::BoxWidget;
pub use aurora_widgets::composite::{Composite, CompositeBuilder, CompositeWrapper, StateSetter};
pub use aurora_widgets::interactables::button::Button;
pub use aurora_widgets::interactables::touch_area::TouchArea;
pub use aurora_widgets::layout::column::Column;
pub use aurora_widgets::layout::content_switch::ContentSwitch;
pub use aurora_widgets::layout::position::{Position, Positioned};
pub use aurora_widgets::layout::row::Row;
pub use aurora_widgets::layout::scrollview::{ScrollState, ScrollView};
pub use aurora_widgets::layout::stack::Stack;
pub use aurora_widgets::layout::{Align, Justify};
pub use aurora_widgets::widgets::{EventResponse, LayoutCtx, Widget};
pub use aurora_widgets::{button, col, row};

// Text (feature-gated)
#[cfg(feature = "text")]
pub use aurora_text::font_options::{FontOptions, FontStretch, FontStyle, FontWeight};
#[cfg(feature = "text")]
pub use aurora_text::{font_manager::FontManager, text_layout::TextLayout};
#[cfg(feature = "text")]
pub use aurora_widgets::text_input::TextInput;
#[cfg(feature = "text")]
pub use aurora_widgets::text_widget::Text;

// Image (feature-gated)
#[cfg(feature = "image")]
pub use aurora_render::image_data::ImageData;
#[cfg(feature = "image")]
pub use aurora_widgets::image_widget::{Image, ImageFit};

// SVG (feature-gated)
#[cfg(feature = "svg")]
pub use aurora_render::svg_data::SvgData;
#[cfg(feature = "svg")]
pub use aurora_widgets::svg_widget::Svg;

// Dialogs (feature-gated)
#[cfg(feature = "dialogs")]
pub use aurora_platform::dialogs::{FileDialog, FileFilter};

// Animation (feature-gated)
#[cfg(feature = "animate")]
pub use aurora_animate::{
    Animatable, Easing, Keyframe, KeyframeAnimation, LoopMode, Preset, Timeline, Tween,
};
