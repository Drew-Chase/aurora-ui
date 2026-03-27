use aurora_ui::aurora_widgets::components::*;
use aurora_ui::prelude::*;

pub fn page_command() -> impl Widget {
    col!()
        .spacing(24.0)
        .padding(Edges::new(0.0, 24.0, 0.0, 0.0))
        .child(crate::page_header("Command", "A command palette for keyboard-driven navigation."))
        .child(crate::example_section("Default", "A searchable command list, typically opened with Ctrl+K."))
        .child(crate::example_card(
            command::Command::new()
                .open(true)
                .command("New File", || {})
                .command("Open File", || {})
                .command("Save", || {})
                .command("Settings", || {})
        ))
        .child(code_block::CodeBlock::new().language("rust").code(
r#"Command::new()
    .open(true)
    .command("New File", || create_file())
    .command("Open File", || open_file())
    .command("Save", || save_file())
    .command("Settings", || open_settings())"#
        ).font_size(13.0))
}
