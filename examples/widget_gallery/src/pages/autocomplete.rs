use aurora_ui::aurora_widgets::components::*;
use aurora_ui::prelude::*;

pub fn page_autocomplete() -> impl Widget {
    col!()
        .spacing(24.0)
        .padding(Edges::new(0.0, 24.0, 0.0, 0.0))
        .child(crate::page_header("AutoComplete", "Text input with suggestion dropdown and fuzzy search."))
        .child(crate::example_section("Default", "Type to filter suggestions."))
        .child(crate::example_card(
            col!().spacing(16.0)
                .child(autocomplete::AutoComplete::new()
                    .placeholder("Search fruits...")
                    .suggestion("Apple").suggestion("Banana").suggestion("Cherry")
                    .suggestion("Date").suggestion("Elderberry").suggestion("Fig")
                    .suggestion("Grape").suggestion("Honeydew")
                    .width(300.0))
        ))
        .child(code_block::CodeBlock::new().language("rust").code(r#"AutoComplete::new()
    .placeholder("Search...")
    .suggestion("Apple")
    .suggestion("Banana")
    .suggestion("Cherry")
    .max_suggestions(8)
    .width(300.0)
    .on_select(|idx| println!("selected: {idx}"))"#).font_size(13.0))
}
