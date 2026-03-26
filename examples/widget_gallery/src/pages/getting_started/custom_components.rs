use aurora_ui::aurora_widgets::components::*;
use aurora_ui::prelude::*;

pub fn page_custom_components() -> impl Widget {
    col!()
        .spacing(24.0)
        .padding(Edges::new(0.0, 24.0, 0.0, 0.0))
        .child(crate::page_header(
            "Custom Components",
            "Build reusable components by implementing the Widget trait or composing existing widgets.",
        ))
        .child(crate::example_section(
            "Composition approach",
            "The simplest way to create a component is to compose existing widgets in a function.",
        ))
        .child(
            code_block::CodeBlock::new()
                .language("rust")
                .code(
r#"fn stat_card(label: &str, value: &str) -> impl Widget {
    Card::new()
        .padding(Edges::all(16.0))
        .child(
            col!()
                .spacing(4.0)
                .child(
                    Text::new(label)
                        .font_size(12.0)
                        .color(colors::muted_foreground())
                )
                .child(
                    Text::new(value)
                        .font_size(24.0)
                        .font_weight(FontWeight::Bold)
                )
        )
}

// Usage
row!()
    .spacing(12.0)
    .child(stat_card("Users", "1,234"))
    .child(stat_card("Revenue", "$5,678"))
    .child(stat_card("Growth", "+12%"))"#,
                )
                .font_size(13.0),
        )
        .child(crate::example_section(
            "The Widget trait",
            "For full control, implement the Widget trait directly. This gives you access to layout, painting, and event handling.",
        ))
        .child(
            code_block::CodeBlock::new()
                .language("rust")
                .code(
r#"use aurora_ui::prelude::*;

struct ColorSwatch {
    color: Color,
    size: f32,
}

impl ColorSwatch {
    fn new(color: Color) -> Self {
        Self { color, size: 40.0 }
    }

    fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }
}

impl Widget for ColorSwatch {
    fn layout(&mut self, _available: Size, _ctx: &mut LayoutCtx) -> Size {
        Size::new(self.size, self.size)
    }

    fn paint(&self, canvas: &mut Canvas, rect: Rect) {
        canvas.fill_rounded_rect(rect, Corners::all(4.0), self.color);
    }

    fn event(
        &mut self,
        _event: &WidgetEvent,
        _rect: Rect,
    ) -> EventResponse {
        EventResponse::default()
    }

    fn children(&self) -> &[Box<dyn Widget>] {
        &[]
    }
}"#,
                )
                .font_size(13.0),
        )
        .child(crate::example_section(
            "Stateful components with Composite",
            "Use Composite to create components with internal state that rebuild when the state changes.",
        ))
        .child(
            code_block::CodeBlock::new()
                .language("rust")
                .code(
r#"fn toggle_card(title: &str, description: &str) -> impl Widget {
    let title = title.to_string();
    let description = description.to_string();

    Composite::new(false, move |&enabled, setter| {
        let set = setter.clone();
        Box::new(
            Card::new()
                .padding(Edges::all(16.0))
                .child(
                    row!()
                        .justify(Justify::SpaceBetween)
                        .align(Align::Center)
                        .child(
                            col!()
                                .spacing(4.0)
                                .child(
                                    Text::new(&title)
                                        .font_weight(FontWeight::SemiBold)
                                )
                                .child(
                                    Text::new(&description)
                                        .font_size(12.0)
                                        .color(colors::muted_foreground())
                                )
                        )
                        .child(
                            Switch::new()
                                .checked(enabled)
                                .on_toggle(move |val| {
                                    set.set(move |s| *s = val);
                                })
                        )
                )
        )
    })
}"#,
                )
                .font_size(13.0),
        )
        .child(crate::example_section(
            "Builder pattern convention",
            "Follow Aurora's owned-self builder pattern: methods consume self and return Self for chaining.",
        ))
        .child(
            code_block::CodeBlock::new()
                .language("rust")
                .code(
r#"struct InfoBanner {
    title: String,
    subtitle: Option<String>,
    background: Color,
}

impl InfoBanner {
    fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            subtitle: None,
            background: colors::primary(),
        }
    }

    // Owned-self builder — consumes self, returns Self
    fn subtitle(mut self, text: impl Into<String>) -> Self {
        self.subtitle = Some(text.into());
        self
    }

    fn background(mut self, color: Color) -> Self {
        self.background = color;
        self
    }
}

// Usage chains naturally:
// InfoBanner::new("Welcome")
//     .subtitle("Getting started guide")
//     .background(colors::accent())"#,
                )
                .font_size(13.0),
        )
}
