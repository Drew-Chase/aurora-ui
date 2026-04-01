// Core types
pub use crate::aurora_core::{hex, hexa, hsl, hsla, rgb, rgba};
pub use aurora_core::color::{Color, IntoColor};
pub use aurora_core::direction::TextDirection;
pub use aurora_core::geometry::corners::Corners;
pub use aurora_core::geometry::edges::Edges;
pub use aurora_core::geometry::point::Point;
pub use aurora_core::geometry::rect::Rect;
pub use aurora_core::geometry::size::Size;
pub use aurora_core::kmi::WidgetEvent;
pub use aurora_core::kmi::cursor_icon::CursorIcon;
pub use aurora_core::kmi::drag::{DragEvent, FileDropEvent};
pub use aurora_core::kmi::keyboard::{Key, KeyboardEvent, Modifiers};
pub use aurora_core::kmi::mouse::{MouseButton, MouseClickEvent, MouseEvent, MouseState};

// Platform
pub use aurora_platform::app::{
    App, AppContext, AppWindow, ExitBehavior, FrameInfo, IconData, WindowConfig, WindowMonitor,
    WindowPosition,
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
pub use aurora_widgets::interactables::draggable::Draggable;
pub use aurora_widgets::interactables::drop_zone::DropZone;
pub use aurora_widgets::interactables::touch_area::TouchArea;
pub use aurora_widgets::layout::column::Column;
pub use aurora_widgets::layout::content_switch::ContentSwitch;
pub use aurora_widgets::layout::position::{Position, Positioned};
pub use aurora_widgets::layout::row::Row;
pub use aurora_widgets::layout::scrollview::{ScrollState, ScrollView};
pub use aurora_widgets::layout::stack::Stack;
pub use aurora_widgets::layout::virtual_list::VirtualList;
pub use aurora_widgets::layout::{Align, Justify};
#[cfg(feature = "regex")]
pub use aurora_widgets::validation::Pattern;
pub use aurora_widgets::validation::{
    Custom, Email, FormValidator, MaxLength, MinLength, NumericRange, Required, ValidationResult,
    Validator, ValidatorChain,
};
pub use aurora_widgets::widgets::{EventResponse, EventStatus, LayoutCtx, Widget};
pub use aurora_widgets::{button, col, row};

// Text (feature-gated)
#[cfg(feature = "text")]
pub use aurora_text::font_options::{FontOptions, FontStretch, FontStyle, FontWeight};
#[cfg(feature = "text")]
pub use aurora_text::{font_manager::FontManager, text_layout::TextLayout};
#[cfg(feature = "text")]
pub use aurora_widgets::components::combobox::Combobox;
#[cfg(feature = "text")]
pub use aurora_widgets::components::field::Field;
#[cfg(feature = "text")]
pub use aurora_widgets::components::input_group::InputGroup;
#[cfg(feature = "text")]
pub use aurora_widgets::components::label::Label;
#[cfg(feature = "text")]
pub use aurora_widgets::components::select::Select;
#[cfg(feature = "text")]
pub use aurora_widgets::components::textarea::TextArea;
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
pub use aurora_platform::dialogs::{FileDialog, FileFilter, PendingDialog};

// Native Menu (feature-gated)
#[cfg(feature = "menu")]
pub use aurora_platform::menu::{
    CheckMenuItemBuilder, MenuAccelerator, MenuItemBuilder, MenuModifiers, NativeMenu,
    NativeMenuItem, PredefinedItem, SubmenuBuilder,
};

// System Tray (feature-gated)
#[cfg(feature = "tray")]
pub use aurora_platform::tray::{TrayConfig, TrayInteraction};

// Animation (feature-gated)
#[cfg(feature = "animate")]
pub use aurora_animate::{
    Animatable, Easing, Keyframe, KeyframeAnimation, LoopMode, Preset, Timeline, Tween,
};
