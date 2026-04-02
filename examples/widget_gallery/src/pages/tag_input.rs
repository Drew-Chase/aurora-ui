use aurora_ui::aurora_widgets::components::*;
use aurora_ui::prelude::*;

pub fn page_tag_input() -> impl Widget {
    col!()
        .spacing(24.0)
        .padding(Edges::new(0.0, 24.0, 0.0, 0.0))
        .child(crate::page_header(
            "Tag Input",
            "Multi-value input with removable tags.",
        ))
        .child(crate::example_section(
            "Default",
            "Type and press Enter to add tags. Click X to remove.",
        ))
        .child(crate::example_card(
            col!()
                .spacing(16.0)
                .child(
                    tag_input::TagInput::new()
                        .tags(vec!["Rust", "Aurora", "UI"])
                        .width(400.0),
                )
                .child(
                    tag_input::TagInput::new()
                        .placeholder("Add framework...")
                        .max_tags(5)
                        .width(400.0),
                ),
        ))
        .child(
            code_block::CodeBlock::new()
                .language("rust")
                .code(
                    r#"TagInput::new()
    .tags(vec!["Rust", "Aurora", "UI"])
    .placeholder("Add tag...")
    .max_tags(10)
    .width(400.0)
    .on_change(|tags| println!("{tags:?}"))"#,
                )
                .font_size(13.0),
        )
}
