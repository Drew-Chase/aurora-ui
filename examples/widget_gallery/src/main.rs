#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use aurora_ui::prelude::*;
use aurora_ui::aurora_widgets::components::*;

// Sidebar background and active highlight colors
const SIDEBAR_BG: Color = Color::new(15, 15, 15, 255);
const SIDEBAR_ACTIVE_BG: Color = Color::new(39, 39, 42, 255);
const SIDEBAR_TEXT: Color = Color::new(161, 161, 170, 255);
const SIDEBAR_ACTIVE_TEXT: Color = Color::new(250, 250, 250, 255);
const CONTENT_BG: Color = Color::new(9, 9, 11, 255);

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
        .background_color(CONTENT_BG)
        .font_options(FontOptions::new().family("Segoe UI"))
        .run(move |window, _frame_info| {
            if !initialized {
                window.root(gallery());
                initialized = true;
            }
        })
        .expect("Failed to run app");
}

fn gallery() -> impl Widget {
    Composite::new(0usize, move |&page_index, set_state| {
        Box::new(
            row!()
                .align(Align::Stretch)
                // Sidebar
                .child(sidebar_widget(page_index, set_state.clone()))
                // Main content
                .child(content_area(page_index)),
        )
    })
}

// ---------------------------------------------------------------------------
// Sidebar
// ---------------------------------------------------------------------------

fn sidebar_widget(active: usize, setter: StateSetter<usize>) -> impl Widget {
    let mut sidebar = col!()
        .width(220)
        .padding(Edges::new(16.0, 12.0, 16.0, 12.0))
        .spacing(2.0);

    // Title
    sidebar = sidebar.child(
        Text::new("Components")
            .font_size(13.0)
            .font_weight(FontWeight::SemiBold)
            .color(Color::new(161, 161, 170, 255))
            .padding(Edges::new(8.0, 8.0, 16.0, 8.0)),
    );

    for (i, name) in PAGES.iter().enumerate() {
        let is_active = i == active;
        let set = setter.clone();
        sidebar = sidebar.child(sidebar_item(name, is_active, move || {
            let set = set.clone();
            set.set(move |state| *state = i);
        }));
    }

    BoxWidget::new()
        .width(220)
        .background_color(SIDEBAR_BG)
        .child(ScrollView::new().scrollbar_width(4.0).scrollbar_thumb_color(Color::WHITE).child(sidebar))
}

fn sidebar_item(
    name: &str,
    active: bool,
    on_click: impl FnMut() + 'static,
) -> impl Widget {
    let bg = if active { SIDEBAR_ACTIVE_BG } else { Color::TRANSPARENT };
    let text_color = if active { SIDEBAR_ACTIVE_TEXT } else { SIDEBAR_TEXT };
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
    ScrollView::new()
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
// Helper builders
// ---------------------------------------------------------------------------

fn page_header(title: &str, description: &str) -> Column {
    col!()
        .spacing(4.0)
        .child(typography::Typography::h1(title))
        .child(typography::Typography::muted(description))
        .child(BoxWidget::new().height(24))
        .child(separator::Separator::new())
}

fn example_section(title: &str, description: &str) -> Column {
    col!()
        .spacing(4.0)
        .child(typography::Typography::h4(title))
        .child(typography::Typography::muted(description))
}

fn example_card(child: impl Widget + 'static) -> impl Widget {
    BoxWidget::new()
        .background_color(Color::new(24, 24, 27, 255))
        .corners(Corners::all(8.0))
        .padding(Edges::all(32.0))
        .child(
            col!()
                .align(Align::Center)
                .justify(Justify::Center)
                .child(child),
        )
}

// ---------------------------------------------------------------------------
// Component pages
// ---------------------------------------------------------------------------

fn page_accordion() -> impl Widget {
    col!()
        .spacing(24.0)
        .child(page_header(
            "Accordion",
            "A vertically stacked set of interactive headings that reveal content.",
        ))
        .child(example_section("Default", "Click a heading to expand its content."))
        .child(example_card(
            accordion::Accordion::new()
                .width(500.0)
                .section("Is it accessible?", typography::Typography::paragraph("Yes. It adheres to the WAI-ARIA design pattern."))
                .section("Is it styled?", typography::Typography::paragraph("Yes. It comes with default styles that match the other components."))
                .section("Is it animated?", typography::Typography::paragraph("Yes. It smoothly animates the content open and closed."))
                .expanded(0),
        ))
}

fn page_alert() -> impl Widget {
    col!()
        .spacing(24.0)
        .child(page_header(
            "Alert",
            "Displays a callout for important information.",
        ))
        .child(example_section("Variants", "Alerts come in several semantic variants."))
        .child(example_card(
            col!()
                .spacing(12.0)
                .child(alert::Alert::new().title("Heads up!").description("You can add components to your app using the CLI."))
                .child(alert::Alert::new().variant(alert::AlertVariant::Success).title("Success").description("Your changes have been saved."))
                .child(alert::Alert::new().variant(alert::AlertVariant::Warning).title("Warning").description("This action cannot be undone."))
                .child(alert::Alert::new().variant(alert::AlertVariant::Destructive).title("Error").description("Something went wrong. Please try again."))
        ))
}

fn page_avatar() -> impl Widget {
    col!()
        .spacing(24.0)
        .child(page_header(
            "Avatar",
            "An image element with a fallback for representing the user.",
        ))
        .child(example_section("Sizes", "Avatars come in small, medium, and large sizes."))
        .child(example_card(
            row!()
                .spacing(16.0)
                .align(Align::Center)
                .child(avatar::Avatar::new().initials("SM").size(avatar::AvatarSize::Small).background_color(Color::new(59, 130, 246, 255)).foreground_color(Color::WHITE))
                .child(avatar::Avatar::new().initials("MD").background_color(Color::new(234, 67, 53, 255)).foreground_color(Color::WHITE))
                .child(avatar::Avatar::new().initials("LG").size(avatar::AvatarSize::Large).background_color(Color::new(76, 175, 80, 255)).foreground_color(Color::WHITE))
        ))
}

fn page_badge() -> impl Widget {
    col!()
        .spacing(24.0)
        .child(page_header(
            "Badge",
            "Displays a badge or a component that looks like a badge.",
        ))
        .child(example_section("Variants", "Badges come in several visual styles."))
        .child(example_card(
            row!()
                .spacing(8.0)
                .align(Align::Center)
                .child(badge::Badge::new("Default"))
                .child(badge::Badge::new("Outline").variant(badge::BadgeVariant::Outline))
                .child(badge::Badge::new("Success").variant(badge::BadgeVariant::Success))
                .child(badge::Badge::new("Warning").variant(badge::BadgeVariant::Warning))
                .child(badge::Badge::new("Destructive").variant(badge::BadgeVariant::Destructive))
                .child(badge::Badge::new("Info").variant(badge::BadgeVariant::Info))
        ))
}

fn page_breadcrumb() -> impl Widget {
    col!()
        .spacing(24.0)
        .child(page_header(
            "Breadcrumb",
            "Displays a breadcrumb navigation trail.",
        ))
        .child(example_section("Default", "A simple breadcrumb trail."))
        .child(example_card(
            breadcrumb::Breadcrumb::new()
                .item("Home")
                .item("Components")
                .item("Breadcrumb")
        ))
}

fn page_button() -> impl Widget {
    col!()
        .spacing(24.0)
        .child(page_header(
            "Button",
            "Displays a button or a component that looks like a button.",
        ))
        .child(example_section("Default", "A standard button with hover animation."))
        .child(example_card(
            row!()
                .spacing(12.0)
                .align(Align::Center)
                .child(
                    button!("Primary")
                        .background_color(colors::PRIMARY)
                        .hover_background_color(Color::new(220, 220, 255, 255))
                        .border_radius(Corners::all(6.0))
                        .width(100)
                        .height(36)
                )
                .child(
                    button!("Secondary")
                        .background_color(colors::SECONDARY)
                        .hover_background_color(Color::new(50, 50, 54, 255))
                        .border_radius(Corners::all(6.0))
                        .width(110)
                        .height(36)
                )
                .child(
                    button!("Destructive")
                        .background_color(colors::DESTRUCTIVE)
                        .hover_background_color(Color::new(220, 50, 50, 255))
                        .border_radius(Corners::all(6.0))
                        .width(120)
                        .height(36)
                )
        ))
}

fn page_card() -> impl Widget {
    col!()
        .spacing(24.0)
        .child(page_header(
            "Card",
            "Displays a card with header, content, and footer.",
        ))
        .child(example_section("Default", "A card with title and description."))
        .child(example_card(
            card::Card::new()
                .width(400.0)
                .child(typography::Typography::h4("Card Title"))
                .child(typography::Typography::muted(
                    "This is a card component. Cards group related content and actions about a single subject.",
                ))
        ))
}

fn page_checkbox() -> impl Widget {
    col!()
        .spacing(24.0)
        .child(page_header(
            "Checkbox",
            "A control that allows the user to toggle between checked and not checked.",
        ))
        .child(example_section("Basic", "A simple checkbox with a label."))
        .child(example_card(
            col!()
                .spacing(12.0)
                .child(checkbox::Checkbox::new().checked(true).label("Accept terms and conditions"))
                .child(checkbox::Checkbox::new().label("Subscribe to newsletter"))
                .child(checkbox::Checkbox::new().disabled(true).label("Disabled option"))
        ))
}

fn page_collapsible() -> impl Widget {
    col!()
        .spacing(24.0)
        .child(page_header(
            "Collapsible",
            "An interactive component that expands and collapses content.",
        ))
        .child(example_section("Default", "Click the header to toggle content."))
        .child(example_card(
            collapsible::Collapsible::new("Click to expand")
                .width(400.0)
                .child(typography::Typography::paragraph(
                    "This content is revealed when the collapsible is expanded. It smoothly animates open and closed.",
                ))
        ))
}

fn page_dropdown_menu() -> impl Widget {
    col!()
        .spacing(24.0)
        .child(page_header(
            "Dropdown Menu",
            "A menu that appears on trigger click with a list of actions.",
        ))
        .child(example_section("Default", "Click the button to open the menu."))
        .child(example_card(
            dropdown_menu::DropdownMenu::new()
                .trigger(
                    button!("Open Menu")
                        .background_color(colors::SECONDARY)
                        .hover_background_color(Color::new(50, 50, 54, 255))
                        .border_radius(Corners::all(6.0))
                        .width(120)
                        .height(36)
                )
                .item("Edit")
                .item("Duplicate")
                .separator()
                .item("Archive")
                .item("Delete")
        ))
}

fn page_empty() -> impl Widget {
    col!()
        .spacing(24.0)
        .child(page_header(
            "Empty State",
            "Placeholder shown when there is no content to display.",
        ))
        .child(example_section("Default", "A centered empty state message."))
        .child(example_card(
            empty::Empty::new()
                .title("No results found")
                .description("Try adjusting your search or filter criteria.")
                .height(120.0)
        ))
}

fn page_input() -> impl Widget {
    col!()
        .spacing(24.0)
        .child(page_header(
            "Input",
            "A text input field for user data entry.",
        ))
        .child(example_section("Default", "A basic text input."))
        .child(example_card(
            col!()
                .spacing(12.0)
                .child(
                    field::Field::new("Email")
                        .width(350.0)
                        .input(TextInput::new().placeholder("Enter your email"))
                )
                .child(
                    field::Field::new("Password")
                        .width(350.0)
                        .input(TextInput::new().placeholder("Enter your password"))
                )
        ))
}

fn page_kbd() -> impl Widget {
    col!()
        .spacing(24.0)
        .child(page_header(
            "Keyboard Shortcut",
            "A visual indicator for keyboard shortcuts.",
        ))
        .child(example_section("Default", "Display keyboard shortcuts."))
        .child(example_card(
            row!()
                .spacing(8.0)
                .align(Align::Center)
                .child(kbd::Kbd::new("Ctrl"))
                .child(label::Label::new("+"))
                .child(kbd::Kbd::new("S"))
                .child(BoxWidget::new().width(24))
                .child(kbd::Kbd::new("Ctrl+Shift+P"))
        ))
}

fn page_label() -> impl Widget {
    col!()
        .spacing(24.0)
        .child(page_header(
            "Label",
            "Renders an accessible label associated with controls.",
        ))
        .child(example_section("Default", "Labels for form controls."))
        .child(example_card(
            col!()
                .spacing(8.0)
                .child(label::Label::new("Username"))
                .child(label::Label::new("Email address").font_size(12.0))
                .child(label::Label::new("Bold label").font_weight(FontWeight::Bold))
        ))
}

fn page_pagination() -> impl Widget {
    col!()
        .spacing(24.0)
        .child(page_header(
            "Pagination",
            "Navigation between pages of content.",
        ))
        .child(example_section("Default", "A basic pagination control."))
        .child(example_card(
            pagination::Pagination::new()
                .total_pages(10)
                .current_page(3)
        ))
}

fn page_progress() -> impl Widget {
    col!()
        .spacing(24.0)
        .child(page_header(
            "Progress",
            "Displays an indicator showing the completion progress of a task.",
        ))
        .child(example_section("Default", "Progress bars at different values."))
        .child(example_card(
            col!()
                .spacing(16.0)
                .child(progress::Progress::new().value(0.25).width(400.0))
                .child(progress::Progress::new().value(0.50).width(400.0).color(Color::new(59, 130, 246, 255)))
                .child(progress::Progress::new().value(0.75).width(400.0).color(Color::new(76, 175, 80, 255)))
                .child(progress::Progress::new().value(1.0).width(400.0).color(Color::new(234, 67, 53, 255)))
        ))
}

fn page_radio_group() -> impl Widget {
    col!()
        .spacing(24.0)
        .child(page_header(
            "Radio Group",
            "A set of checkable buttons where only one can be checked at a time.",
        ))
        .child(example_section("Default", "Select one option from the group."))
        .child(example_card(
            radio_group::RadioGroup::new()
                .item("Default")
                .item("Comfortable")
                .item("Compact")
                .selected(0)
        ))
}

fn page_select() -> impl Widget {
    col!()
        .spacing(24.0)
        .child(page_header(
            "Select",
            "Displays a list of options for the user to pick from.",
        ))
        .child(example_section("Default", "Click to open the dropdown."))
        .child(example_card(
            select::Select::new()
                .placeholder("Select a fruit...")
                .option("Apple")
                .option("Banana")
                .option("Cherry")
                .option("Date")
                .option("Elderberry")
                .width(250.0)
        ))
}

fn page_separator() -> impl Widget {
    col!()
        .spacing(24.0)
        .child(page_header(
            "Separator",
            "Visually separates content.",
        ))
        .child(example_section("Default", "A horizontal rule to divide sections."))
        .child(example_card(
            col!()
                .spacing(16.0)
                .child(typography::Typography::paragraph("Content above"))
                .child(separator::Separator::new())
                .child(typography::Typography::paragraph("Content below"))
        ))
}

fn page_skeleton() -> impl Widget {
    col!()
        .spacing(24.0)
        .child(page_header(
            "Skeleton",
            "Used to show a placeholder while content is loading.",
        ))
        .child(example_section("Default", "Skeleton shapes for loading states."))
        .child(example_card(
            row!()
                .spacing(12.0)
                .align(Align::Center)
                .child(skeleton::Skeleton::circle(40.0))
                .child(
                    col!()
                        .spacing(8.0)
                        .child(skeleton::Skeleton::new().width(200.0).height(16.0))
                        .child(skeleton::Skeleton::new().width(150.0).height(16.0))
                )
        ))
}

fn page_slider() -> impl Widget {
    col!()
        .spacing(24.0)
        .child(page_header(
            "Slider",
            "An input where the user selects a value from within a given range.",
        ))
        .child(example_section("Default", "Drag the thumb to set a value."))
        .child(example_card(
            col!()
                .spacing(16.0)
                .child(slider::Slider::new().value(0.3).width(400.0))
                .child(slider::Slider::new().value(0.7).width(400.0))
        ))
}

fn page_spinner() -> impl Widget {
    col!()
        .spacing(24.0)
        .child(page_header(
            "Spinner",
            "A loading indicator that rotates continuously.",
        ))
        .child(example_section("Sizes", "Spinners at different sizes."))
        .child(example_card(
            row!()
                .spacing(24.0)
                .align(Align::Center)
                .child(spinner::Spinner::new())
                .child(spinner::Spinner::new().size(32.0).color(Color::new(59, 130, 246, 255)))
                .child(spinner::Spinner::new().size(48.0).color(Color::new(76, 175, 80, 255)).thickness(4.0))
        ))
}

fn page_switch() -> impl Widget {
    col!()
        .spacing(24.0)
        .child(page_header(
            "Switch",
            "A control that allows the user to toggle between on and off.",
        ))
        .child(example_section("Default", "Toggle switches in different states."))
        .child(example_card(
            row!()
                .spacing(24.0)
                .align(Align::Center)
                .child(
                    col!()
                        .spacing(8.0)
                        .align(Align::Center)
                        .child(switch::Switch::new().checked(true))
                        .child(label::Label::new("On").font_size(12.0))
                )
                .child(
                    col!()
                        .spacing(8.0)
                        .align(Align::Center)
                        .child(switch::Switch::new())
                        .child(label::Label::new("Off").font_size(12.0))
                )
                .child(
                    col!()
                        .spacing(8.0)
                        .align(Align::Center)
                        .child(switch::Switch::new().disabled(true))
                        .child(label::Label::new("Disabled").font_size(12.0))
                )
        ))
}

fn page_table() -> impl Widget {
    col!()
        .spacing(24.0)
        .child(page_header(
            "Table",
            "A responsive table component for displaying tabular data.",
        ))
        .child(example_section("Default", "A basic data table."))
        .child(example_card(
            table::Table::new()
                .headers(vec!["Name", "Email", "Role"])
                .row(vec!["Alice Chen", "alice@example.com", "Admin"])
                .row(vec!["Bob Smith", "bob@example.com", "User"])
                .row(vec!["Carol White", "carol@example.com", "Editor"])
                .row(vec!["Dave Jones", "dave@example.com", "User"])
                .width(500.0)
        ))
}

fn page_tabs() -> impl Widget {
    col!()
        .spacing(24.0)
        .child(page_header(
            "Tabs",
            "A set of layered sections of content shown one at a time.",
        ))
        .child(example_section("Default", "Click a tab to switch content."))
        .child(example_card(
            tabs::Tabs::new()
                .width(500.0)
                .tab("Account", typography::Typography::paragraph("Make changes to your account here."))
                .tab("Password", typography::Typography::paragraph("Change your password here."))
                .tab("Settings", typography::Typography::paragraph("Manage your settings and preferences."))
        ))
}

fn page_toggle() -> impl Widget {
    col!()
        .spacing(24.0)
        .child(page_header(
            "Toggle",
            "A two-state button that can be on or off.",
        ))
        .child(example_section("Variants", "Default and outline toggle styles."))
        .child(example_card(
            row!()
                .spacing(12.0)
                .align(Align::Center)
                .child(toggle::Toggle::new("Bold").pressed(true))
                .child(toggle::Toggle::new("Italic"))
                .child(toggle::Toggle::new("Outline").variant(toggle::ToggleVariant::Outline))
        ))
}

fn page_toggle_group() -> impl Widget {
    col!()
        .spacing(24.0)
        .child(page_header(
            "Toggle Group",
            "A set of two-state buttons that can be toggled on or off.",
        ))
        .child(example_section("Default", "Select one option from the group."))
        .child(example_card(
            toggle_group::ToggleGroup::new()
                .item("Left")
                .item("Center")
                .item("Right")
                .selected(1)
        ))
}

fn page_typography() -> impl Widget {
    col!()
        .spacing(24.0)
        .child(page_header(
            "Typography",
            "Styles for headings, paragraphs, and other text elements.",
        ))
        .child(example_section("Headings", "Heading levels from h1 to h4."))
        .child(example_card(
            col!()
                .spacing(12.0)
                .child(typography::Typography::h1("Heading 1"))
                .child(typography::Typography::h2("Heading 2"))
                .child(typography::Typography::h3("Heading 3"))
                .child(typography::Typography::h4("Heading 4"))
        ))
        .child(example_section("Body text", "Paragraph, lead, small, and muted text."))
        .child(example_card(
            col!()
                .spacing(12.0)
                .child(typography::Typography::paragraph("This is a paragraph of body text. It uses the default font size and color."))
                .child(typography::Typography::lead("This is lead text — slightly larger and lighter."))
                .child(typography::Typography::small("This is small text."))
                .child(typography::Typography::muted("This is muted text."))
        ))
}
