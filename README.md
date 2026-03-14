<p align="center">
  <h1 align="center">AuroraUI</h1>
  <p align="center">
    A GPU-accelerated, feature-gated UI framework for building fast, lightweight desktop applications in Rust.
  </p>
</p>

<p align="center">
  <a href="#features"><strong>Features</strong></a> ·
  <a href="#quick-start"><strong>Quick Start</strong></a> ·
  <a href="#feature-flags"><strong>Feature Flags</strong></a> ·
  <a href="#architecture"><strong>Architecture</strong></a>
</p>

---
> [!WARNING]
> This framework is still in development and is not ready for production use. Use at your own risk.

AuroraUI is a cross-platform desktop UI framework that treats performance as a
first-class constraint — not an afterthought. Every subsystem (text rendering,
widgets, animation, accessibility) is behind a feature gate. You pay only for
what you use, in both binary size and startup time.

```rust
use aurora_ui::prelude::*;

fn main() {
    App::new()
        .title("Hello, Aurora")
        .size((400, 300))
        .font(include_bytes!("Roboto-Regular.ttf"))
        .run(|window, _frame| {
            window.root(
                col!()
                    .spacing(10.0)
                    .align(Align::Center)
                    .justify(Justify::Center)
                    .child(Text::new("Hello, world!").font_size(24.0))
                    .child(button(ButtonOptions {
                        text_options: Text::new("Click me")
                            .align(Align::Center)
                            .justify(Justify::Center),
                        on_click: Box::new(|_| println!("clicked")),
                        ..Default::default()
                    })),
            );
        })
        .expect("Failed to run app");
}
```

## Why AuroraUI?

| Problem                                   | Aurora's Answer                                                                                                               |
|-------------------------------------------|-------------------------------------------------------------------------------------------------------------------------------|
| Electron/Tauri apps are slow to start     | Feature-gated architecture keeps startup fast — load only what you need                                                       |
| Qt/GTK bundles are large                  | Minimal binary with aggressive feature gating — don't need text? Don't ship a font shaper                                    |
| No framework does custom titlebars right  | **Native Win11 rounded corners, drop shadows, and snap layouts.** macOS traffic lights with `fullSizeContentView`. Linux CSD. |
| Frameworks force you to bundle everything | **Feature-gated architecture.** Every subsystem is opt-in.                                                                    |

## Features

- **Software rendering** — CPU rendering via softbuffer (only backend currently implemented; GPU backends planned)
- **Custom window titlebars** — Windows 11 DWM integration and macOS transparent titlebar (planned/in-progress)
- **Feature-gated text rendering** — Font loading, shaping, and text widgets behind `text` feature flag
- **Layout system** — `col!` and `row!` macros with flex alignment, spacing, and justification
- **Composite stateful widgets** — `Composite<S>` for widgets that rebuild on state change
- **Incremental rendering** — Dirty-flag system skips unchanged subtrees
- **Statically linked binaries** — Single executable, no runtime dependencies

## Quick Start

AuroraUI is not yet published to crates.io. To use it, clone the repository and
reference it as a path dependency:

```toml
# Cargo.toml
[dependencies]
aurora_ui = { path = "path/to/aurora-ui/aurora" }
```

The `default` feature set includes the software rendering backend. To also
enable text rendering and widgets that depend on it:

```toml
[dependencies]
aurora_ui = { path = "path/to/aurora-ui/aurora", features = ["software", "text"] }
```

## Feature Flags

| Feature    | What it adds                                            | Default |
|------------|---------------------------------------------------------|---------|
| `software` | CPU software rendering via softbuffer                   | Yes     |
| `text`     | Font loading, shaping, text widgets, buttons            | No      |

## Architecture

AuroraUI is structured as a Cargo workspace of focused, single-responsibility
crates. This gives fast incremental compilation (~1-3s for widget changes) and
clean dependency boundaries.

```
aurora/
├── aurora_core       # Zero-dep types: color, geometry, events, IDs
├── aurora_platform   # Windowing, event loop, native handles, custom titlebar
├── aurora_gpu        # GPU abstraction with pluggable backends
├── aurora_render     # 2D drawing: rounded rects, paths, images
├── aurora_text       # Text shaping, layout, font management (optional)
├── aurora_layout     # Layout engine (planned)
├── aurora_widgets    # Widget library: layout, composites, interactables
├── aurora_theme      # Theming system (planned)
├── aurora_animate    # Animation system (planned)
├── aurora_a11y       # Accessibility via AccessKit (planned)
└── aurora (facade)   # Public API — re-exports everything
```

## Custom Titlebar

> [!NOTE]
> Custom titlebar support is planned/in-progress and not yet fully implemented.

Aurora aims to provide first-class support for custom window chrome that respects
platform conventions:

**Windows 11** — DWM frame extension with `DWMWA_WINDOW_CORNER_PREFERENCE` for
native rounded corners, `WM_NCHITTEST` for snap layout support on the maximize
button, and automatic drop shadow via `DwmExtendFrameIntoClientArea`.

**macOS** — `titlebarAppearsTransparent` with `fullSizeContentView` style mask.
Traffic light buttons are automatically positioned within your custom titlebar
content.

**Linux** — Client-side decorations on Wayland (deferred).

## Platform Support

| Platform        | Backend             | Status    |
|-----------------|---------------------|-----------|
| Windows 10/11   | Software (softbuffer) | Primary   |
| macOS 11+       | Software (softbuffer) | Primary   |
| Linux (X11)     | Software (softbuffer) | Supported |
| Linux (Wayland) | Software (softbuffer) | Supported |

GPU backends (OpenGL via glow, Vulkan/Metal/DX12 via wgpu) are planned but not yet implemented.

## Building from Source

```bash
git clone https://github.com/yourusername/aurora-ui.git
cd aurora-ui

# Run the minimal example (blank window)
cargo run -p minimal

# Run the counter example (requires text feature)
cargo run -p counter_example --features text

# Build a release binary and check size
cargo build --release -p minimal
```

### Development Setup

For fast iteration, install a faster linker:

```bash
# Linux — install mold
sudo apt install mold

# macOS — lld comes with Xcode command line tools

# Windows — lld comes with Visual Studio
```

The workspace's `.cargo/config.toml` is preconfigured to use the fast linker.
Incremental dev builds of widget code should take 1-3 seconds.

## Contributing

AuroraUI is currently a solo project in active development. Issues and
discussions are welcome. If you're interested in contributing, the most
impactful areas are:

- **Platform testing** — especially Linux compositors, multi-monitor HiDPI setups, and Windows accessibility
- **Widget implementations** — new widgets following the existing patterns
- **Examples and documentation** — real-world usage examples
