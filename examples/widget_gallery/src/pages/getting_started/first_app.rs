use aurora_ui::aurora_widgets::components::*;
use aurora_ui::prelude::*;

pub fn page_first_app() -> impl Widget {
    col!()
        .spacing(24.0)
        .padding(Edges::new(0.0, 24.0, 0.0, 0.0))
        .child(crate::page_header(
            "Your First App",
            "Create a window, set up the event loop, and render your first widget.",
        ))
        .child(crate::example_section(
            "The App builder",
            "Aurora uses an owned-self builder pattern. Chain configuration methods and finish with run().",
        ))
        .child(
            code_block::CodeBlock::new()
                .language("rust")
                .code(
r#"use aurora_ui::prelude::*;

fn main() {
    App::new()
        .title("My App")
        .size((800, 600))
        .min_size((400, 300))
        .position(WindowPosition::Center)
        .use_system_fonts()
        .font_options(FontOptions::new().family("Segoe UI"))
        .run(|window, frame_info| {
            // This closure runs every frame
        })
        .expect("Failed to run app");
}"#,
                )
                .font_size(13.0),
        )
        .child(crate::example_section(
            "Setting a background color",
            "Use window.set_background_color() inside the run loop to set the window background.",
        ))
        .child(
            code_block::CodeBlock::new()
                .language("rust")
                .code(
r##"App::new()
    .title("Colored Background")
    .size((800, 600))
    .run(|window, _frame| {
        window.set_background_color(Color::from_hex("#1a1a2e"));
    })
    .expect("Failed to run app");"##,
                )
                .font_size(13.0),
        )
        .child(crate::example_section(
            "Adding a widget",
            "Use window.root() to set the root widget. Initialize it once using a boolean flag.",
        ))
        .child(
            code_block::CodeBlock::new()
                .language("rust")
                .code(
r#"fn main() {
    let mut initialized = false;

    App::new()
        .title("Hello Widgets")
        .size((800, 600))
        .run(move |window, _frame| {
            if !initialized {
                window.root(
                    col!()
                        .spacing(12.0)
                        .child(
                            Text::new("Hello, Aurora!")
                                .font_size(24.0)
                                .font_weight(FontWeight::Bold)
                        )
                        .child(
                            Text::new("Welcome to your first app.")
                                .font_size(14.0)
                        )
                );
                initialized = true;
            }
        })
        .expect("Failed to run app");
}"#,
                )
                .font_size(13.0),
        )
        .child(crate::example_section(
            "Using FrameInfo",
            "The frame_info parameter provides the window dimensions and scale factor each frame.",
        ))
        .child(
            code_block::CodeBlock::new()
                .language("rust")
                .code(
r#"App::new()
    .title("Frame Info")
    .size((800, 600))
    .run(|window, frame_info| {
        let width = frame_info.width;    // physical pixels (u32)
        let height = frame_info.height;  // physical pixels (u32)
        let scale = frame_info.scale_factor; // DPI scale (f64)
    })
    .expect("Failed to run app");"#,
                )
                .font_size(13.0),
        )
}
