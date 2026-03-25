#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use aurora_ui::prelude::*;
use aurora_ui::aurora_widgets::components::*;

fn main() {
    App::new()
        .title("Aurora Widget Gallery")
        .size((900, 700))
        .position(WindowPosition::Center)
        .use_system_fonts()
        .font_options(FontOptions::new().family("Segoe UI"))
        .run(|window, _frame_info| {
            window.root(gallery());
        })
        .expect("Failed to run app");
}

fn gallery() -> impl Widget {
    ScrollView::new()
        .child(
            col!()
                .spacing(24.0)
                .padding(Edges::new(32.0, 32.0, 32.0, 32.0))
                // Title
                .child(typography::Typography::h1("Aurora Widget Gallery"))
                .child(typography::Typography::muted(
                    "A comprehensive showcase of all Aurora UI components.",
                ))
                .child(separator::Separator::new())

                // Typography section
                .child(section_title("Typography"))
                .child(typography::Typography::h2("Heading 2"))
                .child(typography::Typography::h3("Heading 3"))
                .child(typography::Typography::h4("Heading 4"))
                .child(typography::Typography::paragraph(
                    "This is a paragraph with regular body text. Aurora UI provides a complete set of typography components for consistent text styling across your application.",
                ))
                .child(typography::Typography::lead("This is lead text — slightly larger and muted."))
                .child(typography::Typography::small("This is small text."))
                .child(separator::Separator::new())

                // Labels & Badges
                .child(section_title("Labels & Badges"))
                .child(
                    row!()
                        .spacing(8.0)
                        .align(Align::Center)
                        .child(label::Label::new("Username"))
                        .child(badge::Badge::new("New"))
                        .child(badge::Badge::new("Outline").variant(badge::BadgeVariant::Outline))
                        .child(badge::Badge::new("Success").variant(badge::BadgeVariant::Success))
                        .child(badge::Badge::new("Warning").variant(badge::BadgeVariant::Warning))
                        .child(badge::Badge::new("Error").variant(badge::BadgeVariant::Destructive))
                        .child(badge::Badge::new("Info").variant(badge::BadgeVariant::Info))
                )
                .child(separator::Separator::new())

                // Keyboard Shortcuts
                .child(section_title("Keyboard Shortcuts"))
                .child(
                    row!()
                        .spacing(8.0)
                        .align(Align::Center)
                        .child(kbd::Kbd::new("Ctrl"))
                        .child(label::Label::new("+"))
                        .child(kbd::Kbd::new("S"))
                        .child(label::Label::new("  "))
                        .child(kbd::Kbd::new("Ctrl+Shift+P"))
                )
                .child(separator::Separator::new())

                // Avatars
                .child(section_title("Avatars"))
                .child(
                    row!()
                        .spacing(12.0)
                        .align(Align::Center)
                        .child(avatar::Avatar::new().initials("DC").size(avatar::AvatarSize::Small).background_color(Color::new(59, 130, 246, 255)).foreground_color(Color::WHITE))
                        .child(avatar::Avatar::new().initials("AB").background_color(Color::new(234, 67, 53, 255)).foreground_color(Color::WHITE))
                        .child(avatar::Avatar::new().initials("XY").size(avatar::AvatarSize::Large).background_color(Color::new(76, 175, 80, 255)).foreground_color(Color::WHITE))
                )
                .child(separator::Separator::new())

                // Card
                .child(section_title("Card"))
                .child(
                    card::Card::new()
                        .width(400.0)
                        .child(typography::Typography::h4("Card Title"))
                        .child(typography::Typography::muted("This is a card component with a title and description. Cards are used to group related content."))
                )
                .child(separator::Separator::new())

                // Alert
                .child(section_title("Alerts"))
                .child(alert::Alert::new().title("Heads up!").description("This is a default alert."))
                .child(alert::Alert::new().variant(alert::AlertVariant::Success).title("Success").description("Your changes have been saved."))
                .child(alert::Alert::new().variant(alert::AlertVariant::Warning).title("Warning").description("This action cannot be undone."))
                .child(alert::Alert::new().variant(alert::AlertVariant::Destructive).title("Error").description("Something went wrong."))
                .child(separator::Separator::new())

                // Progress
                .child(section_title("Progress"))
                .child(progress::Progress::new().value(0.33).width(400.0))
                .child(progress::Progress::new().value(0.66).width(400.0).color(Color::new(76, 175, 80, 255)))
                .child(progress::Progress::new().value(1.0).width(400.0).color(Color::new(59, 130, 246, 255)))
                .child(separator::Separator::new())

                // Skeleton
                .child(section_title("Skeleton"))
                .child(
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
                )
                .child(separator::Separator::new())

                // Checkbox, Switch, Toggle
                .child(section_title("Checkbox, Switch, Toggle"))
                .child(
                    row!()
                        .spacing(24.0)
                        .align(Align::Center)
                        .child(checkbox::Checkbox::new().checked(true).label("Accept terms"))
                        .child(checkbox::Checkbox::new().label("Subscribe"))
                        .child(switch::Switch::new().checked(true))
                        .child(switch::Switch::new())
                )
                .child(separator::Separator::new())

                // Radio Group
                .child(section_title("Radio Group"))
                .child(
                    radio_group::RadioGroup::new()
                        .item("Option A")
                        .item("Option B")
                        .item("Option C")
                        .selected(0)
                )
                .child(separator::Separator::new())

                // Toggle Group
                .child(section_title("Toggle Group"))
                .child(
                    toggle_group::ToggleGroup::new()
                        .item("Left")
                        .item("Center")
                        .item("Right")
                        .selected(1)
                )
                .child(separator::Separator::new())

                // Slider
                .child(section_title("Slider"))
                .child(slider::Slider::new().value(0.4).width(400.0))
                .child(separator::Separator::new())

                // Spinner
                .child(section_title("Spinner"))
                .child(
                    row!()
                        .spacing(16.0)
                        .align(Align::Center)
                        .child(spinner::Spinner::new())
                        .child(spinner::Spinner::new().size(32.0).color(Color::new(59, 130, 246, 255)))
                )
                .child(separator::Separator::new())

                // Empty State
                .child(section_title("Empty State"))
                .child(
                    empty::Empty::new()
                        .title("No results found")
                        .description("Try adjusting your search or filter criteria.")
                        .height(120.0)
                ),
        )
}

fn section_title(text: &str) -> impl Widget {
    typography::Typography::h3(text)
}
