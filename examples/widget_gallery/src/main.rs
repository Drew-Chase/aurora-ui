#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub mod pages;

use aurora_ui::aurora_widgets::components::*;
use aurora_ui::prelude::*;

// Load theme profiles from JSON at compile time
aurora_ui::aurora_theming::config!("themes.json");

/// Names shown in the sidebar, in display order.
const PAGES: &[&str] = &[
    "Accordion",
    "Alert",
    "Avatar",
    "Badge",
    "Breadcrumb",
    "Button",
    "Card",
    "Checkbox",
    "Collapsible",
    "Dropdown Menu",
    "Empty State",
    "Input",
    "Kbd",
    "Label",
    "Pagination",
    "Progress",
    "Radio Group",
    "Select",
    "Separator",
    "Skeleton",
    "Slider",
    "Spinner",
    "Switch",
    "Table",
    "Tabs",
    "Toggle",
    "Toggle Group",
    "Typography",
];

fn main() {
    let mut initialized = false;

    App::new()
        .title("Aurora Widget Gallery")
        .size((1200, 800))
        .min_size((900, 600))
        .position(WindowPosition::Center)
        .use_system_fonts()
        .font_options(FontOptions::new().family("Segoe UI"))
        .run(move |window, _frame_info| {
            // Update background color each frame so theme changes take effect
            window.set_background_color(theme::colors::background());
            if !initialized {
                window.root(gallery());
                initialized = true;
            }
        })
        .expect("Failed to run app");
}

/// App state: selected page + selected theme profile.
#[derive(Clone, Copy)]
struct GalleryState {
    page: usize,
    profile: usize,
}

fn gallery() -> impl Widget {
    Composite::new(
        GalleryState { page: 0, profile: 0 },
        move |state, set_state| {
            // Apply the selected theme profile
            theme::set(unsafe {
                std::mem::transmute::<usize, theme::ProfileId>(
                    state.profile.min(theme::profile_count() - 1),
                )
            });

            Box::new(
                row!()
                    .align(Align::Stretch)
                    .child(sidebar_widget(state.page, state.profile, set_state.clone()))
                    .child(content_area(state.page)),
            )
        },
    )
}

// ---------------------------------------------------------------------------
// Sidebar
// ---------------------------------------------------------------------------

fn sidebar_widget(
    active_page: usize,
    active_profile: usize,
    setter: StateSetter<GalleryState>,
) -> impl Widget {
    // Theme profile dropdown
    let profile_setter = setter.clone();
    let theme_select = select::Select::new()
        .placeholder("Theme")
        .option("Dark")
        .option("Candy")
        .option("Light")
        .selected(active_profile)
        .width(196.0)
        .height(32.0)
        .on_change(move |idx| {
            profile_setter.set(move |s| s.profile = idx);
        });

    // Scrollable component list
    let mut items = col!()
        .width(220)
        .padding(Edges::new(0.0, 12.0, 16.0, 12.0))
        .spacing(2.0);

    for (i, name) in PAGES.iter().enumerate() {
        let is_active = i == active_page;
        let set = setter.clone();
        items = items.child(sidebar_item(name, is_active, move || {
            let set = set.clone();
            set.set(move |s| s.page = i);
        }));
    }

    // Layout: fixed header (dropdown + title) above scrollable list
    BoxWidget::new()
        .width(220)
        .background_color(theme::colors::card())
        .child(
            col!()
                .width(220)
                .spacing(0.0)
                // Fixed header: dropdown + "Components" label
                .child(
                    col!()
                        .width(220)
                        .padding(Edges::new(12.0, 12.0, 0.0, 12.0))
                        .spacing(12.0)
                        .child(theme_select)
                        .child(
                            Text::new("Components")
                                .font_size(13.0)
                                .font_weight(FontWeight::SemiBold)
                                .color(Color::new(161, 161, 170, 255))
                                .padding(Edges::new(0.0, 8.0, 4.0, 8.0)),
                        ),
                )
                // Scrollable item list
                .child(
                    ScrollView::new()
                        .scrollbar_width(4.0)
                        .scrollbar_thumb_color(Color::WHITE.opacity(0.5))
                        .child(items),
                ),
        )
}

fn sidebar_item(name: &str, active: bool, on_click: impl FnMut() + 'static) -> impl Widget {
    let bg = if active {
        theme::colors::accent()
    } else {
        Color::TRANSPARENT
    };
    let text_color = if active {
        theme::colors::foreground()
    } else {
        theme::colors::muted_foreground()
    };
    let mut on_click = on_click;

    TouchArea::new()
        .hover_cursor(CursorIcon::Pointer)
        .on_click(move |_| {
            on_click();
        })
        .child(
            BoxWidget::new()
                .background_color(bg)
                .corners(Corners::all(6.0))
                .height(32)
                .padding(Edges::new(0.0, 8.0, 0.0, 8.0))
                .child(
                    Text::new(name)
                        .font_size(13.0)
                        .color(text_color)
                        .height(32.0)
                        .justify(Justify::Center),
                ),
        )
}

// ---------------------------------------------------------------------------
// Main content area
// ---------------------------------------------------------------------------

fn content_area(page_index: usize) -> impl Widget {
    use pages::*;
    ScrollView::new()
        .scrollbar_thumb_color(Color::WHITE.opacity(0.5))
        .padding(Edges::new(48.0, 48.0, 48.0, 48.0))
        .child(
            ContentSwitch::new()
                .selected(page_index)
                .item(page_accordion())
                .item(page_alert())
                .item(page_avatar())
                .item(page_badge())
                .item(page_breadcrumb())
                .item(page_button())
                .item(page_card())
                .item(page_checkbox())
                .item(page_collapsible())
                .item(page_dropdown_menu())
                .item(page_empty())
                .item(page_input())
                .item(page_kbd())
                .item(page_label())
                .item(page_pagination())
                .item(page_progress())
                .item(page_radio_group())
                .item(page_select())
                .item(page_separator())
                .item(page_skeleton())
                .item(page_slider())
                .item(page_spinner())
                .item(page_switch())
                .item(page_table())
                .item(page_tabs())
                .item(page_toggle())
                .item(page_toggle_group())
                .item(page_typography()),
        )
}

// ---------------------------------------------------------------------------
// Helper builders (pub so page modules can use them via crate::)
// ---------------------------------------------------------------------------

pub fn page_header(title: &str, description: &str) -> Column {
    col!()
        .spacing(4.0)
        .child(
            Text::new(title)
                .font_size(32.0)
                .font_weight(FontWeight::Bold)
                .color(theme::colors::foreground()),
        )
        .child(Text::new(description).font_size(14.0).color(theme::colors::muted_foreground()))
        .child(BoxWidget::new().height(24))
        .child(separator::Separator::new())
}

pub fn example_section(title: &str, description: &str) -> Column {
    col!()
        .spacing(4.0)
        .child(
            Text::new(title)
                .font_size(16.0)
                .font_weight(FontWeight::SemiBold)
                .color(theme::colors::foreground()),
        )
        .child(Text::new(description).font_size(14.0).color(theme::colors::muted_foreground()))
}

pub fn example_card(child: impl Widget + 'static) -> impl Widget {
    BoxWidget::new()
        .background_color(Color::new(24, 24, 27, 255))
        .corners(Corners::all(8.0))
        .padding(Edges::all(32.0))
        .child(child)
}
