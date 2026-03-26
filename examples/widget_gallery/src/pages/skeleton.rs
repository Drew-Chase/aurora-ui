use aurora_ui::aurora_widgets::components::*;
use aurora_ui::prelude::*;

pub fn page_skeleton() -> impl Widget {
    col!()
        .spacing(24.0)
        .padding(Edges::new(0.0, 24.0, 0.0, 0.0))
        .child(crate::page_header("Skeleton", "Used to show a placeholder while content is loading."))
        .child(crate::example_section("Default", "Skeleton shapes for loading states."))
        .child(crate::example_card(
            row!()
                .spacing(12.0)
                .align(Align::Center)
                .child(skeleton::Skeleton::circle(40.0))
                .child(col!().spacing(8.0).child(skeleton::Skeleton::new().width(200.0).height(16.0)).child(skeleton::Skeleton::new().width(150.0).height(16.0)))
        ))
        .child(code_block::CodeBlock::new().language("rust").code(
r#"// Circle avatar skeleton
Skeleton::circle(40.0)

// Text line skeletons
Skeleton::new().width(200.0).height(16.0)
Skeleton::new().width(150.0).height(16.0)"#
        ).font_size(13.0))
}
