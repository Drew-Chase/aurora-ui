use aurora_ui::aurora_widgets::components::*;
use aurora_ui::prelude::*;

pub fn page_installation() -> impl Widget {
    col!()
        .spacing(24.0)
        .padding(Edges::new(0.0, 24.0, 0.0, 0.0))
        .child(crate::page_header(
            "Installation",
            "Add Aurora UI to your Rust project and configure feature flags.",
        ))
        .child(crate::example_section(
            "Add the dependency",
            "Add aurora_ui to your Cargo.toml with the features you need.",
        ))
        .child(
            code_block::CodeBlock::new()
                .language("toml")
                .code(
r#"[dependencies]
aurora_ui = { version = "0.1", features = ["opengl", "text"] }
aurora_theme = "0.1"
aurora_iconify = "0.1"  # optional, for icon support"#,
                )
                .font_size(13.0),
        )
        .child(crate::example_section(
            "Feature flags",
            "Aurora UI uses feature gates to keep binary size small. Only enable what you need.",
        ))
        .child(
            code_block::CodeBlock::new()
                .language("toml")
                .code(
r#"[dependencies.aurora_ui]
version = "0.1"
features = [
    "opengl",   # GPU-accelerated rendering via OpenGL
    "text",     # Text rendering and font shaping
    "animate",  # Animation system for transitions
    "svg",      # SVG rendering support
    "syntax",   # Syntax highlighting for code blocks
    "image",    # Raster image loading and display
]"#,
                )
                .font_size(13.0),
        )
        .child(crate::example_section(
            "Cargo profile",
            "For release builds, optimize for size with these recommended profile settings.",
        ))
        .child(
            code_block::CodeBlock::new()
                .language("toml")
                .code(
r#"[profile.release]
opt-level = "z"       # optimize for size
lto = "fat"           # link-time optimization
codegen-units = 1     # single codegen unit
panic = "abort"       # smaller panic handling
strip = "symbols"     # strip debug symbols"#,
                )
                .font_size(13.0),
        )
        .child(crate::example_section(
            "Verify it works",
            "Create a minimal main.rs to confirm everything compiles.",
        ))
        .child(
            code_block::CodeBlock::new()
                .language("rust")
                .code(
r#"use aurora_ui::prelude::*;

fn main() {
    App::new()
        .title("Hello Aurora")
        .size((800, 600))
        .run(|_window, _frame| {})
        .expect("Failed to run app");
}"#,
                )
                .font_size(13.0),
        )
}
