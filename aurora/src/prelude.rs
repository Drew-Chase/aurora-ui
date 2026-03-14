// Core types
pub use aurora_core::color::{Color, IntoColor};
pub use aurora_core::geometry::corners::Corners;
pub use aurora_core::geometry::edges::Edges;
pub use aurora_core::geometry::point::Point;
pub use aurora_core::geometry::rect::Rect;
pub use aurora_core::geometry::size::Size;
pub use aurora_core::kmi::cursor_icon::CursorIcon;
pub use aurora_core::kmi::mouse::{MouseButton, MouseClickEvent, MouseEvent, MouseState};
pub use crate::aurora_core::{hex, hexa, rgb, rgba, hsl, hsla};

// Platform
pub use aurora_platform::app::{App, AppWindow, FrameInfo, WindowMonitor, WindowPosition};

// Rendering
pub use aurora_render::canvas::Canvas;

// Widgets
pub use aurora_widgets::box_widget::BoxWidget;
pub use aurora_widgets::composite::{Composite, StateSetter};
pub use aurora_widgets::interactables::touch_area::TouchArea;
pub use aurora_widgets::layout::column::Column;
pub use aurora_widgets::layout::row::Row;
pub use aurora_widgets::layout::{Align, Justify};
pub use aurora_widgets::widgets::{EventResponse, LayoutCtx, Widget};
pub use aurora_widgets::{col, row};

// Text (feature-gated)
#[cfg(feature = "text")]
pub use aurora_text::{font_manager::FontManager, text_layout::TextLayout};
#[cfg(feature = "text")]
pub use aurora_widgets::interactables::button::{button, ButtonOptions};
#[cfg(feature = "text")]
pub use aurora_widgets::text_widget::Text;
