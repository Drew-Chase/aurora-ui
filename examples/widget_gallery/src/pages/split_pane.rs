use aurora_ui::aurora_widgets::components::*;
use aurora_ui::prelude::*;

pub fn page_split_pane() -> impl Widget {
    col!()
        .spacing(24.0)
        .padding(Edges::new(0.0, 24.0, 0.0, 0.0))
        .child(crate::page_header(
            "Split Pane",
            "A resizable split view with two children and a draggable divider.",
        ))
        // Horizontal split
        .child(crate::example_section(
            "Horizontal",
            "Left and right panels separated by a vertical drag handle.",
        ))
        .child(crate::example_card(
            BoxWidget::new().height(300).child(
                split_pane::SplitPane::new()
                    .direction(split_pane::SplitDirection::Horizontal)
                    .ratio(0.5)
                    .min_first(80.0)
                    .min_second(80.0)
                    .first(
                        BoxWidget::new()
                            .background_color(crate::theme::colors::muted())
                            .corners(Corners::new(8.0, 0.0, 0.0, 8.0))
                            .padding(Edges::all(16.0))
                            .child(Text::new("Left panel")),
                    )
                    .second(
                        BoxWidget::new()
                            .background_color(crate::theme::colors::muted())
                            .corners(Corners::new(0.0, 8.0, 8.0, 0.0))
                            .padding(Edges::all(16.0))
                            .child(Text::new("Right panel")),
                    ),
            ),
        ))
        .child(
            code_block::CodeBlock::new()
                .language("rust")
                .code(
                    r#"SplitPane::new()
    .direction(SplitDirection::Horizontal)
    .ratio(0.5)
    .min_first(80.0)
    .min_second(80.0)
    .first(left_panel)
    .second(right_panel)"#,
                )
                .font_size(13.0),
        )
        // Vertical split
        .child(crate::example_section(
            "Vertical",
            "Top and bottom panels separated by a horizontal drag handle.",
        ))
        .child(crate::example_card(
            BoxWidget::new().height(300).child(
                split_pane::SplitPane::new()
                    .direction(split_pane::SplitDirection::Vertical)
                    .ratio(0.4)
                    .min_first(60.0)
                    .min_second(60.0)
                    .first(
                        BoxWidget::new()
                            .background_color(crate::theme::colors::muted())
                            .corners(Corners::new(8.0, 8.0, 0.0, 0.0))
                            .padding(Edges::all(16.0))
                            .child(Text::new("Top panel")),
                    )
                    .second(
                        BoxWidget::new()
                            .background_color(crate::theme::colors::muted())
                            .corners(Corners::new(0.0, 0.0, 8.0, 8.0))
                            .padding(Edges::all(16.0))
                            .child(Text::new("Bottom panel")),
                    ),
            ),
        ))
        .child(
            code_block::CodeBlock::new()
                .language("rust")
                .code(
                    r#"SplitPane::new()
    .direction(SplitDirection::Vertical)
    .ratio(0.4)
    .min_first(60.0)
    .min_second(60.0)
    .first(top_panel)
    .second(bottom_panel)"#,
                )
                .font_size(13.0),
        )
}
