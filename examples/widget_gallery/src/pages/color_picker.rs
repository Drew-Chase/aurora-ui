use aurora_ui::aurora_widgets::components::*;
use aurora_ui::prelude::*;

pub fn page_color_picker() -> impl Widget {
    col!()
        .spacing(24.0)
        .padding(Edges::new(0.0, 24.0, 0.0, 0.0))
        .child(crate::page_header(
            "Color Picker",
            "A color selection widget with saturation/brightness area, hue slider, and hex/RGB display.",
        ))
        .child(crate::example_section(
            "Default",
            "Click and drag the gradient area to set saturation and brightness. Use the hue bar below to change the hue.",
        ))
        .child(crate::example_card(
            color_picker::ColorPicker::new()
                .hue(210.0)
                .saturation(0.8)
                .brightness(0.9)
                .width(320.0),
        ))
        .child(
            code_block::CodeBlock::new()
                .language("rust")
                .code(
                    r#"ColorPicker::new()
    .hue(210.0)
    .saturation(0.8)
    .brightness(0.9)
    .width(320.0)
    .on_change(|color| {
        println!("Selected: {color}");
    })"#,
                )
                .font_size(13.0),
        )
        .child(crate::example_section(
            "With Palette",
            "A preset palette of colors that can be clicked to select.",
        ))
        .child(crate::example_card(
            color_picker::ColorPicker::new()
                .hue(0.0)
                .saturation(1.0)
                .brightness(1.0)
                .width(320.0)
                .palette(vec![
                    Color::from_hex(0xef4444, false),
                    Color::from_hex(0xf97316, false),
                    Color::from_hex(0xeab308, false),
                    Color::from_hex(0x22c55e, false),
                    Color::from_hex(0x3b82f6, false),
                    Color::from_hex(0x8b5cf6, false),
                    Color::from_hex(0xec4899, false),
                    Color::from_hex(0xffffff, false),
                    Color::from_hex(0x000000, false),
                ]),
        ))
        .child(
            code_block::CodeBlock::new()
                .language("rust")
                .code(
                    r#"ColorPicker::new()
    .hue(0.0)
    .saturation(1.0)
    .brightness(1.0)
    .width(320.0)
    .palette(vec![
        Color::from_hex(0xef4444, false), // red
        Color::from_hex(0xf97316, false), // orange
        Color::from_hex(0xeab308, false), // yellow
        Color::from_hex(0x22c55e, false), // green
        Color::from_hex(0x3b82f6, false), // blue
        Color::from_hex(0x8b5cf6, false), // violet
        Color::from_hex(0xec4899, false), // pink
        Color::from_hex(0xffffff, false), // white
        Color::from_hex(0x000000, false), // black
    ])"#,
                )
                .font_size(13.0),
        )
        .child(crate::example_section(
            "From Existing Color",
            "Initialize from an RGB color value.",
        ))
        .child(crate::example_card(
            color_picker::ColorPicker::new()
                .color(Color::from_hex(0x22c55e, false))
                .width(320.0),
        ))
        .child(
            code_block::CodeBlock::new()
                .language("rust")
                .code(
                    r#"ColorPicker::new()
    .color(Color::from_hex(0x22c55e, false))
    .width(320.0)"#,
                )
                .font_size(13.0),
        )
}
