pub mod colors;

// Code rendering
#[cfg(feature = "syntax")]
pub mod code_block;

// Display widgets
pub mod alert;
pub mod aspect_ratio;
pub mod avatar;
pub mod badge;
pub mod card;
pub mod empty;
pub mod kbd;
pub mod label;
pub mod progress;
pub mod separator;
pub mod skeleton;
pub mod spinner;
pub mod typography;

// Interactive widgets
pub mod accordion;
pub mod button_group;
pub mod checkbox;
pub mod collapsible;
pub mod field;
pub mod input_group;
pub mod radio_group;
pub mod slider;
pub mod switch;
pub mod tabs;
pub mod toggle;
pub mod toggle_group;

// Data/container widgets
pub mod breadcrumb;
pub mod item;
pub mod pagination;
pub mod resizable;
pub mod scroll_area;
pub mod sidebar;
pub mod table;
pub mod textarea;

// Overlay widgets
pub mod dialog;
pub mod dropdown_menu;
pub mod hover_card;
pub mod popover;
pub mod select;
pub mod toast;
pub mod tooltip;

// Complex widgets
pub mod calendar;
pub mod carousel;
pub mod combobox;
pub mod command;
pub mod data_table;
pub mod date_picker;
pub mod input_otp;
pub mod menubar;
pub mod navigation_menu;
