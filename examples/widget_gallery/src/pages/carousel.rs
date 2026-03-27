use aurora_ui::aurora_widgets::components::*;
use aurora_ui::prelude::*;

pub fn page_carousel() -> impl Widget {
    col!()
        .spacing(24.0)
        .padding(Edges::new(0.0, 24.0, 0.0, 0.0))
        .child(crate::page_header("Carousel", "A content carousel with previous/next navigation."))
        .child(crate::example_section("Default", "Navigate between slides using the arrow controls."))
        .child(crate::example_card(
            carousel::Carousel::new()
                .slide(BoxWidget::new().height(200).background_color(Color::from_hex(0x3b82f6, false)).corners(Corners::all(8.0)).child(Text::new("Slide 1").color(Color::WHITE).font_size(24.0)))
                .slide(BoxWidget::new().height(200).background_color(Color::from_hex(0x22c55e, false)).corners(Corners::all(8.0)).child(Text::new("Slide 2").color(Color::WHITE).font_size(24.0)))
                .slide(BoxWidget::new().height(200).background_color(Color::from_hex(0xeab308, false)).corners(Corners::all(8.0)).child(Text::new("Slide 3").color(Color::WHITE).font_size(24.0)))
        ))
        .child(code_block::CodeBlock::new().language("rust").code(
r#"Carousel::new()
    .slide(slide_1_widget)
    .slide(slide_2_widget)
    .slide(slide_3_widget)"#
        ).font_size(13.0))
}
