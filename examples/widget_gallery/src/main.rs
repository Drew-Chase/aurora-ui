#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub mod pages;

use aurora_ui::aurora_widgets::components::*;
use aurora_ui::prelude::*;

// Load theme profiles from JSON at compile time
aurora_ui::aurora_theming::config!("themes.json");

/// A sidebar section with a title and its page names.
struct Section {
    title: &'static str,
    pages: &'static [&'static str],
}

/// Sidebar sections and their pages, in display order.
const SECTIONS: &[Section] = &[
    Section {
        title: "Getting Started",
        pages: &["Installation", "Your First App", "Custom Components"],
    },
    Section {
        title: "Components",
        pages: &[
            "Accordion",
            "Alert",
            "Avatar",
            "Badge",
            "Breadcrumb",
            "Button",
            "Card",
            "Code Block",
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
        ],
    },
    Section {
        title: "Tutorials",
        pages: &["Counter", "Todo App", "Theme Switcher", "Form Builder", "Navigation"],
    },
];

fn main() {
    theme::init();
    let mut initialized = false;

    App::new()
        .title("Aurora Widget Gallery")
        .icon(include_bytes!("../../../logo.png"))
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
#[derive(Clone)]
struct GalleryState {
    page: usize,
    profile: usize,
    sidebar_scroll: ScrollState,
    content_scroll: ScrollState,
}

fn gallery() -> impl Widget {
    Composite::new(
        GalleryState {
            page: 0,
            profile: 0,
            sidebar_scroll: ScrollState::new(),
            content_scroll: ScrollState::new(),
        },
        move |state, set_state| {
            // Apply the selected theme profile
            let profiles = [theme::ProfileId::Default, theme::ProfileId::Candy, theme::ProfileId::Light];
            theme::set(profiles[state.profile.min(profiles.len() - 1)]);

            Box::new(
                row!()
                    .align(Align::Stretch)
                    .child(sidebar_widget(
                        state.page,
                        state.profile,
                        state.sidebar_scroll.clone(),
                        set_state.clone(),
                    ))
                    .child(content_area(state.page, state.content_scroll.clone())),
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
    scroll: ScrollState,
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
        .background(theme::colors::sidebar_accent())
        .border_color(theme::colors::border())
        .on_change(move |idx| {
            profile_setter.set(move |s| s.profile = idx);
        });

    // Scrollable section list
    let mut items = col!()
        .padding(Edges::new(0.0, 12.0, 16.0, 12.0))
        .spacing(2.0);

    let mut global_index = 0usize;
    for (section_idx, section) in SECTIONS.iter().enumerate() {
        // Section header (non-clickable)
        let top_pad = if section_idx == 0 { 0.0 } else { 12.0 };
        items = items.child(
            Text::new(&section.title.to_uppercase())
                .font_size(11.0)
                .font_weight(FontWeight::SemiBold)
                .color(theme::colors::sidebar_foreground().opacity(0.7))
                .padding(Edges::new(top_pad, 8.0, 4.0, 8.0)),
        );

        // Page items within this section
        for name in section.pages {
            let i = global_index;
            let is_active = i == active_page;
            let set = setter.clone();
            items = items.child(sidebar_item(name, is_active, move || {
                let set = set.clone();
                set.set(move |s| {
                    s.page = i;
                    s.content_scroll.set(0.0);
                });
            }));
            global_index += 1;
        }
    }

    // Layout: fixed header (theme dropdown) above scrollable list
    BoxWidget::new()
        .width(220)
        .background_color(theme::colors::sidebar())
        .child(
            col!()
                .width(220)
                .spacing(0.0)
                .child(
                    col!()
                        .width(220)
                        .padding(Edges::new(12.0, 12.0, 8.0, 12.0))
                        .child(theme_select),
                )
                // Scrollable item list
                .child(
                    ScrollView::new()
                        .scrollbar_width(4.0)
                        .scrollbar_thumb_color(theme::colors::foreground().opacity(0.5))
                        .state(scroll)
                        .child(items),
                ),
        )
}

fn sidebar_item(name: &str, active: bool, on_click: impl FnMut() + 'static) -> impl Widget {
    use std::cell::RefCell;
    use std::rc::Rc;

    let name = name.to_string();
    let on_click = Rc::new(RefCell::new(on_click));

    Composite::new(false, move |&hovered, set_hover| {
        let bg = if active {
            theme::colors::sidebar_accent()
        } else if hovered {
            theme::colors::sidebar_accent().opacity(0.5)
        } else {
            Color::TRANSPARENT
        };
        let text_color = if active {
            theme::colors::sidebar_accent_foreground()
        } else {
            theme::colors::sidebar_foreground()
        };

        let prev = hovered;
        let set_hover = set_hover.clone();
        let on_click = on_click.clone();

        Box::new(
            TouchArea::new()
                .hover_cursor(CursorIcon::Pointer)
                .on_hover(move |_rect, is_hovered| {
                    if is_hovered != prev {
                        set_hover.set(move |h| *h = is_hovered);
                    }
                })
                .on_click(move |_| {
                    (on_click.borrow_mut())();
                })
                .child(
                    BoxWidget::new()
                        .background_color(bg)
                        .corners(Corners::all(6.0))
                        .height(32)
                        .padding(Edges::new(0.0, 8.0, 0.0, 8.0))
                        .child(
                            Text::new(&name)
                                .font_size(13.0)
                                .color(text_color)
                                .height(32.0)
                                .justify(Justify::Center),
                        ),
                ),
        )
    })
}

// ---------------------------------------------------------------------------
// Main content area
// ---------------------------------------------------------------------------

fn content_area(page_index: usize, scroll: ScrollState) -> impl Widget {
    use pages::*;
    ScrollView::new()
        .scrollbar_thumb_color(theme::colors::foreground().opacity(0.5))
        .padding(Edges::new(10.0, 10.0, 10.0, 20.0))
        .state(scroll)
        .child(
            ContentSwitch::new()
                .selected(page_index)
                // Getting Started (indices 0-2)
                .item(page_installation())
                .item(page_first_app())
                .item(page_custom_components())
                // Components (indices 3-31)
                .item(page_accordion())
                .item(page_alert())
                .item(page_avatar())
                .item(page_badge())
                .item(page_breadcrumb())
                .item(page_button())
                .item(page_card())
                .item(page_code_block())
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
                .item(page_typography())
                // Tutorials (indices 32-36)
                .item(page_counter())
                .item(page_todo_app())
                .item(page_theme_switcher())
                .item(page_form_builder())
                .item(page_navigation()),
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
        .background_color(theme::colors::card())
        .corners(Corners::all(8.0))
        .padding(Edges::all(32.0))
        .child(child)
}
