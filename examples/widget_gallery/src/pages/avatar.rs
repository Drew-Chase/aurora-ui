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
                .child(avatar::Avatar::new().initials("SM").size(avatar::AvatarSize::Small).background_color(colors::info()).foreground_color(colors::info_foreground()))
                .child(avatar::Avatar::new().initials("MD").background_color(colors::destructive()).foreground_color(colors::destructive_foreground()))
                .child(avatar::Avatar::new().initials("LG").size(avatar::AvatarSize::Large).background_color(colors::success()).foreground_color(colors::success_foreground()))
        ))
        .child(code_block::CodeBlock::new().language("rust").code(
r#"// Small avatar
Avatar::new()
    .initials("SM")
    .size(AvatarSize::Small)
    .background_color(colors::info())
    .foreground_color(colors::info_foreground())

// Medium avatar (default size)
Avatar::new()
    .initials("MD")
    .background_color(colors::destructive())
    .foreground_color(colors::destructive_foreground())

// Large avatar
Avatar::new()
    .initials("LG")
    .size(AvatarSize::Large)
    .background_color(colors::success())
    .foreground_color(colors::success_foreground())"#
        ).font_size(13.0))
}
