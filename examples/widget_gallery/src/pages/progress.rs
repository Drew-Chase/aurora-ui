use aurora_ui::aurora_widgets::components::*;
use aurora_ui::prelude::*;

pub fn page_progress() -> impl Widget {
    col!()
        .spacing(24.0)
        .padding(Edges::new(0.0, 24.0, 0.0, 0.0))
        .child(crate::page_header(
            "Progress",
            "Displays an indicator showing the completion progress of a task.",
        ))
        .child(crate::example_section(
            "Default",
            "Progress bars at different values.",
        ))
        .child(crate::example_card(
            col!()
                .spacing(16.0)
                .child(progress::Progress::new().value(0.25).width(400.0))
                .child(
                    progress::Progress::new()
                        .value(0.50)
                        .width(400.0)
                        .color(colors::info()),
                )
                .child(
                    progress::Progress::new()
                        .value(0.75)
                        .width(400.0)
                        .color(colors::success()),
                )
                .child(
                    progress::Progress::new()
                        .value(1.0)
                        .width(400.0)
                        .color(colors::destructive()),
                ),
        ))
        .child(
            code_block::CodeBlock::new()
                .language("rust")
                .code(
                    r#"Progress::new()
    .value(0.50)
    .width(400.0)
    .color(colors::info())

Progress::new()
    .value(0.75)
    .width(400.0)
    .color(colors::success())"#,
                )
                .font_size(13.0),
        )
}
