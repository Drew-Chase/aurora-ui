use aurora_ui::aurora_widgets::components::*;
use aurora_ui::prelude::*;
use crate::theme;

pub fn page_navigation() -> impl Widget {
    col!()
        .spacing(24.0)
        .padding(Edges::new(0.0, 24.0, 0.0, 0.0))
        .child(crate::page_header(
            "Navigation",
            "Build multi-page apps with sidebar navigation and ContentSwitch routing.",
        ))
        .child(crate::example_section(
            "The ContentSwitch pattern",
            "ContentSwitch renders one child at a time based on a selected index — Aurora's equivalent of a router.",
        ))
        .child(
            code_block::CodeBlock::new()
                .language("rust")
                .code(
r#"ContentSwitch::new()
    .selected(current_page)  // usize index
    .item(page_home())       // index 0
    .item(page_about())      // index 1
    .item(page_settings())   // index 2"#,
                )
                .font_size(13.0),
        )
        .child(crate::example_section(
            "Building a sidebar",
            "Pair a fixed-width sidebar with a flexible content area using Row + BoxWidget.",
        ))
        .child(
            code_block::CodeBlock::new()
                .language("rust")
                .code(
r#"#[derive(Clone)]
struct AppState {
    page: usize,
    sidebar_scroll: ScrollState,
    content_scroll: ScrollState,
}

fn app_shell() -> impl Widget {
    Composite::new(
        AppState {
            page: 0,
            sidebar_scroll: ScrollState::new(),
            content_scroll: ScrollState::new(),
        },
        |state, setter| {
            Box::new(
                row!()
                    .align(Align::Stretch)
                    .child(sidebar(state.page, setter.clone()))
                    .child(content(state.page, state.content_scroll.clone()))
            )
        },
    )
}"#,
                )
                .font_size(13.0),
        )
        .child(crate::example_section(
            "Clickable sidebar items",
            "Each item updates the page index in state. Use TouchArea for hover and click handling.",
        ))
        .child(
            code_block::CodeBlock::new()
                .language("rust")
                .code(
r#"const PAGES: &[&str] = &["Home", "About", "Settings"];

fn sidebar(active: usize, setter: StateSetter<AppState>) -> impl Widget {
    let mut items = col!().spacing(2.0).padding(Edges::all(8.0));

    for (i, name) in PAGES.iter().enumerate() {
        let set = setter.clone();
        let is_active = i == active;

        let bg = if is_active {
            colors::accent()
        } else {
            Color::TRANSPARENT
        };

        items = items.child(
            TouchArea::new()
                .on_click(move |_| {
                    let set = set.clone();
                    set.set(move |s| {
                        s.page = i;
                        s.content_scroll.set(0.0); // scroll to top
                    });
                })
                .child(
                    BoxWidget::new()
                        .background_color(bg)
                        .corners(Corners::all(6.0))
                        .height(32)
                        .padding(Edges::new(0.0, 8.0, 0.0, 8.0))
                        .child(
                            Text::new(name)
                                .font_size(13.0)
                                .height(32.0)
                                .justify(Justify::Center)
                        )
                )
        );
    }

    BoxWidget::new()
        .width(200)
        .background_color(colors::sidebar())
        .child(items)
}"#,
                )
                .font_size(13.0),
        )
        .child(crate::example_section(
            "Adding sections to the sidebar",
            "Group navigation items under section headers for organized navigation.",
        ))
        .child(
            code_block::CodeBlock::new()
                .language("rust")
                .code(
r#"struct Section {
    title: &'static str,
    pages: &'static [&'static str],
}

const SECTIONS: &[Section] = &[
    Section { title: "General", pages: &["Home", "About"] },
    Section { title: "Account", pages: &["Profile", "Settings"] },
];

fn sidebar_with_sections(active: usize, setter: StateSetter<AppState>) -> impl Widget {
    let mut items = col!().spacing(2.0);
    let mut global_index = 0;

    for (i, section) in SECTIONS.iter().enumerate() {
        // Section header
        let top_pad = if i == 0 { 0.0 } else { 12.0 };
        items = items.child(
            Text::new(&section.title.to_uppercase())
                .font_size(11.0)
                .font_weight(FontWeight::SemiBold)
                .color(colors::muted_foreground())
                .padding(Edges::new(top_pad, 8.0, 4.0, 8.0))
        );

        // Section items
        for name in section.pages {
            let idx = global_index;
            let set = setter.clone();
            items = items.child(nav_item(name, idx == active, move || {
                let set = set.clone();
                set.set(move |s| s.page = idx);
            }));
            global_index += 1;
        }
    }

    BoxWidget::new()
        .width(200)
        .background_color(colors::sidebar())
        .child(items)
}"#,
                )
                .font_size(13.0),
        )
        .child(crate::example_section(
            "Scrollable content area",
            "Wrap the ContentSwitch in a ScrollView with persistent state for scroll position.",
        ))
        .child(
            code_block::CodeBlock::new()
                .language("rust")
                .code(
r#"fn content(page: usize, scroll: ScrollState) -> impl Widget {
    ScrollView::new()
        .state(scroll)
        .child(
            ContentSwitch::new()
                .selected(page)
                .item(page_home())
                .item(page_about())
                .item(page_settings())
        )
}"#,
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
                    Text::new("• ContentSwitch is a flat index-based page router")
                        .font_size(14.0)
                        .color(theme::colors::foreground()),
                )
                .child(
                    Text::new("• Store the active page index in Composite state")
                        .font_size(14.0)
                        .color(theme::colors::foreground()),
                )
                .child(
                    Text::new("• Use ScrollState.set(0.0) to scroll to top on page change")
                        .font_size(14.0)
                        .color(theme::colors::foreground()),
                )
                .child(
                    Text::new("• Section headers are non-clickable Text widgets above item groups")
                        .font_size(14.0)
                        .color(theme::colors::foreground()),
                ),
        ))
}
