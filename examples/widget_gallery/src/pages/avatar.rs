use aurora_ui::aurora_widgets::components::*;
use aurora_ui::prelude::*;

pub fn page_avatar() -> impl Widget {
    col!()
        .spacing(24.0)
        .child(crate::page_header("Avatar", "An image element with a fallback for representing the user."))
        .child(crate::example_section("Sizes", "Avatars come in small, medium, and large sizes."))
        .child(crate::example_card(
            row!()
                .spacing(16.0)
                .align(Align::Center)
                .child(avatar::Avatar::new().initials("SM").size(avatar::AvatarSize::Small).background_color(Color::new(59, 130, 246, 255)).foreground_color(Color::WHITE))
                .child(avatar::Avatar::new().initials("MD").background_color(Color::new(234, 67, 53, 255)).foreground_color(Color::WHITE))
                .child(avatar::Avatar::new().initials("LG").size(avatar::AvatarSize::Large).background_color(Color::new(76, 175, 80, 255)).foreground_color(Color::WHITE))
        ))
}
