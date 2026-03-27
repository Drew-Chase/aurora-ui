use aurora_ui::aurora_widgets::components::*;
use aurora_ui::prelude::*;

pub fn page_combobox() -> impl Widget {
    col!()
        .spacing(24.0)
        .padding(Edges::new(0.0, 24.0, 0.0, 0.0))
        .child(crate::page_header("Combobox", "A searchable dropdown that combines text input with a select list."))
        .child(crate::example_section("Default", "Type to filter options, then select from the list."))
        .child(crate::example_card(
            combobox::Combobox::new()
                .placeholder("Search frameworks...")
                .option("Aurora")
                .option("React")
                .option("Vue")
                .option("Angular")
                .option("Svelte")
        ))
        .child(code_block::CodeBlock::new().language("rust").code(
r#"Combobox::new()
    .placeholder("Search frameworks...")
    .option("Aurora")
    .option("React")
    .option("Vue")
    .option("Angular")
    .option("Svelte")"#
        ).font_size(13.0))
}
