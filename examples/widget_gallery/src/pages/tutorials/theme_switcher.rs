use crate::theme;
use aurora_ui::aurora_widgets::components::*;
use aurora_ui::prelude::*;

pub fn page_theme_switcher() -> impl Widget {
    col!()
        .spacing(24.0)
        .padding(Edges::new(0.0, 24.0, 0.0, 0.0))
        .child(crate::page_header(
            "Theme Switcher",
            "Learn how Aurora's theming system works and build a runtime theme selector.",
        ))
        .child(crate::example_section(
            "How theming works",
            "Aurora themes are defined in a JSON file and loaded at compile time with the config! macro.",
        ))
        .child(
            code_block::CodeBlock::new()
                .language("rust")
                .code(
r#"// Load theme profiles from a JSON file at compile time
aurora_ui::aurora_theming::config!("themes.json");"#,
                )
                .font_size(13.0),
        )
        .child(crate::example_section(
            "Theme JSON structure",
            "Each profile defines colors for every semantic role — background, foreground, primary, etc.",
        ))
        .child(
            code_block::CodeBlock::new()
                .language("json")
                .code(
r##"{
    "profiles": {
        "dark": {
            "background": "#09090b",
            "foreground": "#fafafa",
            "primary": "#3b82f6",
            "primary_foreground": "#ffffff",
            "secondary": "#27272a",
            "secondary_foreground": "#fafafa",
            "accent": "#27272a",
            "muted": "#27272a",
            "muted_foreground": "#a1a1aa",
            "border": "#27272a",
            "card": "#0a0a0c",
            "destructive": "#dc2626"
        },
        "light": {
            "background": "#ffffff",
            "foreground": "#09090b",
            "primary": "#2563eb",
            "primary_foreground": "#ffffff"
        }
    }
}"##,
                )
                .font_size(13.0),
        )
        .child(crate::example_section(
            "Switching themes at runtime",
            "Use theme::set() to apply a profile. Colors update on the next frame.",
        ))
        .child(
            code_block::CodeBlock::new()
                .language("rust")
                .code(
r#"use aurora_ui::prelude::*;

// Initialize the theming system once at startup
theme::init();

// In your run loop or event handler, switch profiles:
theme::set(theme::ProfileId::Default);  // dark theme
theme::set(theme::ProfileId::Light);    // light theme
theme::set(theme::ProfileId::Candy);    // custom theme

// All color functions automatically return the active profile's colors:
let bg = theme::colors::background();      // adapts to current theme
let fg = theme::colors::foreground();      // adapts to current theme
let primary = theme::colors::primary();    // adapts to current theme"#,
                )
                .font_size(13.0),
        )
        .child(crate::example_section(
            "Building a theme selector",
            "Combine a Select dropdown with Composite state to let users pick a theme.",
        ))
        .child(
            code_block::CodeBlock::new()
                .language("rust")
                .code(
r#"fn theme_selector() -> impl Widget {
    Composite::new(0usize, |&selected, setter| {
        let profiles = [
            theme::ProfileId::Default,
            theme::ProfileId::Candy,
            theme::ProfileId::Light,
        ];
        theme::set(profiles[selected.min(profiles.len() - 1)]);

        let set = setter.clone();
        Box::new(
            select::Select::new()
                .placeholder("Theme")
                .option("Dark")
                .option("Candy")
                .option("Light")
                .selected(selected)
                .on_change(move |idx| {
                    set.set(move |s| *s = idx);
                })
        )
    })
}"#,
                )
                .font_size(13.0),
        )
        .child(crate::example_section(
            "Using semantic colors",
            "Always use theme::colors::* functions instead of hardcoded colors for theme support.",
        ))
        .child(
            code_block::CodeBlock::new()
                .language("rust")
                .code(
r##"// ✓ Correct — adapts to theme changes
Text::new("Hello")
    .color(theme::colors::foreground())
BoxWidget::new()
    .background_color(theme::colors::card())
button!("Submit")
    .background_color(colors::primary())
    .foreground_color(colors::primary_foreground())

// ✗ Avoid — hardcoded colors ignore the active theme
Text::new("Hello")
    .color(Color::WHITE)
BoxWidget::new()
    .background_color(Color::from_hex("#1a1a2e"))"##,
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
                    Text::new("• Themes are compiled from JSON via config! macro")
                        .font_size(14.0)
                        .color(theme::colors::foreground()),
                )
                .child(
                    Text::new("• theme::set() switches profiles — colors update next frame")
                        .font_size(14.0)
                        .color(theme::colors::foreground()),
                )
                .child(
                    Text::new("• Always use colors::* functions for theme-aware styling")
                        .font_size(14.0)
                        .color(theme::colors::foreground()),
                )
                .child(
                    Text::new("• Combine Select + Composite for a user-facing theme picker")
                        .font_size(14.0)
                        .color(theme::colors::foreground()),
                ),
        ))
}
