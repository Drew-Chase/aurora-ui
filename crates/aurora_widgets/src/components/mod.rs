pub mod colors;

// Code rendering
#[cfg(feature = "syntax")]
pub mod code_block;

// Display widgets
pub mod label;
pub mod badge;
pub mod separator;
pub mod card;
pub mod alert;
pub mod avatar;
pub mod progress;
pub mod skeleton;
pub mod spinner;
pub mod kbd;
pub mod typography;
pub mod empty;
pub mod aspect_ratio;

// Interactive widgets
pub mod checkbox;
pub mod switch;
pub mod toggle;
pub mod toggle_group;
pub mod radio_group;
pub mod slider;
pub mod accordion;
pub mod collapsible;
pub mod tabs;
pub mod button_group;
pub mod input_group;
pub mod field;

// Data/container widgets
pub mod table;
pub mod breadcrumb;
pub mod pagination;
pub mod resizable;
pub mod sidebar;
pub mod textarea;
pub mod scroll_area;
pub mod item;

// Overlay widgets
pub mod tooltip;
pub mod popover;
pub mod hover_card;
pub mod dialog;
pub mod dropdown_menu;
pub mod select;
pub mod toast;

// Complex widgets
pub mod combobox;
pub mod command;
pub mod menubar;
pub mod navigation_menu;
pub mod calendar;
pub mod date_picker;
pub mod carousel;
pub mod data_table;
pub mod input_otp;
