use aurora_ui::aurora_widgets::components::*;
use aurora_ui::prelude::*;
use crate::theme;

pub fn page_form_builder() -> impl Widget {
    col!()
        .spacing(24.0)
        .padding(Edges::new(0.0, 24.0, 0.0, 0.0))
        .child(crate::page_header(
            "Form Builder",
            "Create structured forms with inputs, selects, checkboxes, and validation patterns.",
        ))
        .child(crate::example_section(
            "Live demo",
            "A registration form with multiple field types.",
        ))
        .child(crate::example_card(form_demo()))
        .child(crate::example_section(
            "Step 1: Define form state",
            "Create a struct that holds the value of every field in the form.",
        ))
        .child(
            code_block::CodeBlock::new()
                .language("rust")
                .code(
r#"#[derive(Clone, Default)]
struct FormData {
    name: String,
    email: String,
    role: usize,       // index into role options
    newsletter: bool,
    agree_terms: bool,
}"#,
                )
                .font_size(13.0),
        )
        .child(crate::example_section(
            "Step 2: Build labeled fields",
            "Use Label + Input pairs in a Column for a clean form layout.",
        ))
        .child(
            code_block::CodeBlock::new()
                .language("rust")
                .code(
r#"fn form_field(
    label_text: &str,
    value: &str,
    placeholder: &str,
    setter: StateSetter<FormData>,
    update: impl Fn(&mut FormData, String) + 'static,
) -> impl Widget {
    col!()
        .spacing(4.0)
        .child(
            label::Label::new(label_text)
                .font_size(14.0)
                .font_weight(FontWeight::Medium)
        )
        .child(
            TextInput::new()
                .value(value)
                .placeholder(placeholder)
                .height(36.0)
                .on_change(move |text| {
                    let text = text.to_string();
                    setter.set(move |s| update(s, text.clone()));
                })
        )
}

// Usage:
form_field(
    "Full Name", &state.name, "John Doe",
    setter.clone(),
    |s, v| s.name = v,
)"#,
                )
                .font_size(13.0),
        )
        .child(crate::example_section(
            "Step 3: Add a Select dropdown",
            "Use Select for enumerated choices like roles or categories.",
        ))
        .child(
            code_block::CodeBlock::new()
                .language("rust")
                .code(
r#"let role_setter = setter.clone();

col!()
    .spacing(4.0)
    .child(label::Label::new("Role").font_size(14.0))
    .child(
        select::Select::new()
            .placeholder("Select a role")
            .option("Developer")
            .option("Designer")
            .option("Manager")
            .option("Other")
            .selected(state.role)
            .height(36.0)
            .on_change(move |idx| {
                role_setter.set(move |s| s.role = idx);
            })
    )"#,
                )
                .font_size(13.0),
        )
        .child(crate::example_section(
            "Step 4: Checkboxes and submit",
            "Add checkboxes for boolean options and a submit button that reads the full state.",
        ))
        .child(
            code_block::CodeBlock::new()
                .language("rust")
                .code(
r#"let terms_setter = setter.clone();
let news_setter = setter.clone();

col!()
    .spacing(8.0)
    .child(
        checkbox::Checkbox::new()
            .checked(state.newsletter)
            .label("Subscribe to newsletter")
            .on_change(move |val| {
                news_setter.set(move |s| s.newsletter = val);
            })
    )
    .child(
        checkbox::Checkbox::new()
            .checked(state.agree_terms)
            .label("I agree to the terms")
            .on_change(move |val| {
                terms_setter.set(move |s| s.agree_terms = val);
            })
    )
    .child(BoxWidget::new().height(8))
    .child(
        button!("Submit")
            .background_color(colors::primary())
            .foreground_color(colors::primary_foreground())
            .width(120).height(36)
    )"#,
                )
                .font_size(13.0),
        )
        .child(crate::example_section(
            "Key concepts",
            "What you learned in this tutorial.",
        ))
        .child(crate::example_card(
            col!()
                .spacing(8.0)
                .child(
                    Text::new("• Group form state into a single struct for Composite")
                        .font_size(14.0)
                        .color(theme::colors::foreground()),
                )
                .child(
                    Text::new("• Use helper functions to reduce repetition across fields")
                        .font_size(14.0)
                        .color(theme::colors::foreground()),
                )
                .child(
                    Text::new("• Select maps choice index to state — convert to enum as needed")
                        .font_size(14.0)
                        .color(theme::colors::foreground()),
                )
                .child(
                    Text::new("• Each field's on_change clones the setter and updates one field")
                        .font_size(14.0)
                        .color(theme::colors::foreground()),
                ),
        ))
}

#[derive(Clone, Default)]
struct FormData {
    name: String,
    email: String,
    role: usize,
    newsletter: bool,
    agree_terms: bool,
}

fn form_demo() -> impl Widget {
    Composite::new(FormData::default(), |state, setter| {
        let name_setter = setter.clone();
        let email_setter = setter.clone();
        let role_setter = setter.clone();
        let news_setter = setter.clone();
        let terms_setter = setter.clone();

        Box::new(
            col!()
                .spacing(16.0)
                .child(
                    col!()
                        .spacing(4.0)
                        .child(
                            label::Label::new("Full Name")
                                .font_size(14.0)
                                .font_weight(FontWeight::Medium)
                                .color(theme::colors::foreground()),
                        )
                        .child(
                            TextInput::new()
                                .text(&state.name)
                                .placeholder("John Doe")
                                .width(300.0)
                                .height(36.0)
                                .on_change(move |text| {
                                    let text = text.to_string();
                                    name_setter.set(move |s| s.name = text.clone());
                                }),
                        ),
                )
                .child(
                    col!()
                        .spacing(4.0)
                        .child(
                            label::Label::new("Email")
                                .font_size(14.0)
                                .font_weight(FontWeight::Medium)
                                .color(theme::colors::foreground()),
                        )
                        .child(
                            TextInput::new()
                                .text(&state.email)
                                .placeholder("john@example.com")
                                .width(300.0)
                                .height(36.0)
                                .on_change(move |text| {
                                    let text = text.to_string();
                                    email_setter.set(move |s| s.email = text.clone());
                                }),
                        ),
                )
                .child(
                    col!()
                        .spacing(4.0)
                        .child(
                            label::Label::new("Role")
                                .font_size(14.0)
                                .font_weight(FontWeight::Medium)
                                .color(theme::colors::foreground()),
                        )
                        .child(
                            select::Select::new()
                                .placeholder("Select a role")
                                .option("Developer")
                                .option("Designer")
                                .option("Manager")
                                .option("Other")
                                .selected(state.role)
                                .width(300.0)
                                .height(36.0)
                                .background(theme::colors::sidebar_accent())
                                .border_color(theme::colors::border())
                                .on_change(move |idx| {
                                    role_setter.set(move |s| s.role = idx);
                                }),
                        ),
                )
                .child(
                    checkbox::Checkbox::new()
                        .checked(state.newsletter)
                        .label("Subscribe to newsletter")
                        .on_change(move |val| {
                            news_setter.set(move |s| s.newsletter = val);
                        }),
                )
                .child(
                    checkbox::Checkbox::new()
                        .checked(state.agree_terms)
                        .label("I agree to the terms and conditions")
                        .on_change(move |val| {
                            terms_setter.set(move |s| s.agree_terms = val);
                        }),
                )
                .child(BoxWidget::new().height(4))
                .child(
                    button!("Submit")
                        .background_color(colors::primary())
                        .hover_background_color(colors::primary().opacity(0.8))
                        .foreground_color(colors::primary_foreground())
                        .border_radius(theme::corners::button_radius())
                        .width(120)
                        .height(36),
                ),
        )
    })
}
