use aurora_ui::aurora_widgets::components::*;
use aurora_ui::prelude::*;

pub fn page_resizable() -> impl Widget {
    col!()
        .spacing(24.0)
        .padding(Edges::new(0.0, 24.0, 0.0, 0.0))
        .child(crate::page_header("Resizable", "A container with a draggable resize handle."))
        .child(crate::example_section("Default", "Drag the handle to resize the panel."))
        .child(crate::example_card(
            resizable::Resizable::new()
                .child(BoxWidget::new().background_color(crate::theme::colors::muted()).corners(Corners::all(8.0)).child(Text::new("Drag to resize")))
                .initial_size(300.0)
                .min_size(100.0)
                .max_size(600.0)
        ))
        .child(code_block::CodeBlock::new().language("rust").code(
r#"Resizable::new()
    .child(content)
    .initial_size(300.0)
    .min_size(100.0)
    .max_size(600.0)"#
        ).font_size(13.0))
}
