use aurora_ui::aurora_widgets::components::*;
use aurora_ui::prelude::*;

pub fn page_aspect_ratio() -> impl Widget {
    col!()
        .spacing(24.0)
        .padding(Edges::new(0.0, 24.0, 0.0, 0.0))
        .child(crate::page_header("Aspect Ratio", "A container that maintains a fixed width-to-height ratio."))
        .child(crate::example_section("Presets", "Common aspect ratios have convenience constructors."))
        .child(crate::example_card(
            row!()
                .spacing(16.0)
                .align(Align::Center)
                .child(
                    col!().spacing(4.0)
                        .child(Text::new("16:9").font_size(12.0).color(crate::theme::colors::muted_foreground()))
                        .child(aspect_ratio::AspectRatio::widescreen().width(200.0).child(
                            BoxWidget::new().background_color(crate::theme::colors::muted()).corners(Corners::all(8.0))
                        ))
                )
                .child(
                    col!().spacing(4.0)
                        .child(Text::new("4:3").font_size(12.0).color(crate::theme::colors::muted_foreground()))
                        .child(aspect_ratio::AspectRatio::standard().width(200.0).child(
                            BoxWidget::new().background_color(crate::theme::colors::muted()).corners(Corners::all(8.0))
                        ))
                )
                .child(
                    col!().spacing(4.0)
                        .child(Text::new("1:1").font_size(12.0).color(crate::theme::colors::muted_foreground()))
                        .child(aspect_ratio::AspectRatio::square().width(150.0).child(
                            BoxWidget::new().background_color(crate::theme::colors::muted()).corners(Corners::all(8.0))
                        ))
                )
        ))
        .child(code_block::CodeBlock::new().language("rust").code(
r#"AspectRatio::widescreen().width(200.0)  // 16:9
AspectRatio::standard().width(200.0)    // 4:3
AspectRatio::square().width(150.0)      // 1:1
AspectRatio::new(21.0 / 9.0).width(200.0)  // custom"#
        ).font_size(13.0))
}
