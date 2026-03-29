use aurora_ui::aurora_widgets::components::*;
use aurora_ui::prelude::*;

pub fn page_badge() -> impl Widget {
    col!()
        .spacing(24.0)
        .padding(Edges::new(0.0, 24.0, 0.0, 0.0))
        .child(crate::page_header(
            "Badge",
            "Displays a badge or a component that looks like a badge.",
        ))
        .child(crate::example_section(
            "Variants",
            "Badges come in several visual styles.",
        ))
        .child(crate::example_card(
            row!()
                .spacing(8.0)
                .align(Align::Center)
                .child(badge::Badge::new("Default"))
                .child(badge::Badge::new("Outline").variant(badge::BadgeVariant::Outline))
                .child(badge::Badge::new("Success").variant(badge::BadgeVariant::Success))
                .child(badge::Badge::new("Warning").variant(badge::BadgeVariant::Warning))
                .child(badge::Badge::new("Destructive").variant(badge::BadgeVariant::Destructive))
                .child(badge::Badge::new("Info").variant(badge::BadgeVariant::Info)),
        ))
        .child(
            code_block::CodeBlock::new()
                .language("rust")
                .code(
                    r#"Badge::new("Default")
Badge::new("Outline").variant(BadgeVariant::Outline)
Badge::new("Success").variant(BadgeVariant::Success)
Badge::new("Warning").variant(BadgeVariant::Warning)
Badge::new("Destructive").variant(BadgeVariant::Destructive)
Badge::new("Info").variant(BadgeVariant::Info)"#,
                )
                .font_size(13.0),
        )
}
