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

```rust,ignore
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
					.child(
						button!("Click me")
							.on_click(|_| println!("clicked")),
					),
			);
		})
		.expect("Failed to run app");
}
```

## Why AuroraUI?

| Problem                                   | Aurora's Answer                                                                                                               |
|-------------------------------------------|-------------------------------------------------------------------------------------------------------------------------------|
| Electron/Tauri apps are slow to start     | Feature-gated architecture keeps startup fast — load only what you need                                                       |
| Qt/GTK bundles are large                  | Minimal binary with aggressive feature gating — don't need text? Don't ship a font shaper                                     |
| No framework does custom titlebars right  | **Native Win11 rounded corners, drop shadows, and snap layouts.** macOS traffic lights with `fullSizeContentView`. Linux CSD. |
| Frameworks force you to bundle everything | **Feature-gated architecture.** Every subsystem is opt-in.                                                                    |

## Features

- **Software rendering** — CPU rendering via softbuffer (only backend currently implemented; GPU backends planned)
- **Custom window titlebars** — Windows 11 DWM rounded corners, drop shadow, and edge-resize via `WM_NCHITTEST` subclassing
- **Feature-gated text rendering** — Font loading, shaping, and text widgets behind `text` feature flag
- **Layout system** — `col!` and `row!` macros with flex alignment, `Stack`, `Positioned`, and `ScrollView`
- **Image and SVG support** — PNG/JPEG decoding, SVG rasterization with gradient support, feature-gated
- **Composite widget macro** — `#[composite_widget]` generates builder setters, `new()`, and `impl Widget` from a config struct
- **Iconify integration** — `aurora_iconify` fetches icons from [iconify.design](https://iconify.design) at compile time with type-safe access
- **Incremental rendering** — Dirty-flag system skips unchanged subtrees
- **Built-in benchmarking** — `just benchmark` reports binary size, startup time, and memory for every example
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

| Feature    | What it adds                                 | Default |
|------------|----------------------------------------------|---------|
| `software` | CPU software rendering via softbuffer        | Yes     |
| `text`     | Font loading, shaping, text widgets, buttons | No      |
| `image`    | PNG/JPEG decoding and Image widget           | No      |
| `svg`      | SVG parsing, rasterization, and Svg widget   | No      |

## Performance

Release-mode benchmarks on Windows 11 (`just benchmark`):

| Example                   | Binary (KB) | Startup (ms) | Memory (KB) |
|---------------------------|------------:|-------------:|------------:|
| `button_example`          |       1,780 |        75.86 |      21,312 |
| `counter_example`         |       1,628 |        99.82 |      18,464 |
| `custom_titlebar_example` |         385 |        78.46 |      34,260 |
| `image_example`           |       1,252 |        83.62 |      22,296 |
| `layout_example`          |         387 |        83.40 |      20,304 |
| `minimal_example`         |         374 |       128.55 |      20,224 |
| `scrollview_example`      |         394 |        87.48 |      20,248 |
| `text_example`            |       1,769 |       107.79 |      21,396 |
| `text_input_example`      |       1,640 |       100.35 |      19,152 |

The minimal blank-window binary is **369 KB**. Text rendering (`text` feature) adds ~1.4 MB. Image decoding (`image` feature) adds ~870 KB. All examples start in under 115 ms with ~33 MB working set.

Run `just benchmark` to reproduce these numbers on your machine.

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
├── aurora_macros     # Proc macros: #[composite_widget], #[derive(CompositeWidget)]
├── aurora_iconify    # Compile-time icon embedding from iconify.design
├── aurora_theme      # Theming system (planned)
├── aurora_animate    # Animation system (planned)
├── aurora_a11y       # Accessibility via AccessKit (planned)
└── aurora (facade)   # Public API — re-exports everything
```

## Custom Titlebar

Aurora provides first-class support for custom window chrome that respects
platform conventions:

**Windows 11** — DWM frame extension with `DWMWA_WINDOW_CORNER_PREFERENCE` for
native rounded corners, `WM_NCHITTEST` for snap layout support on the maximize
button, and automatic drop shadow via `DwmExtendFrameIntoClientArea`.

**macOS** — `titlebarAppearsTransparent` with `fullSizeContentView` style mask.
Traffic light buttons are automatically positioned within your custom titlebar
content.

**Linux** — Client-side decorations on Wayland (deferred).

## Custom Widgets

Aurora provides two ways to build reusable widgets:

### Composite Widgets (recommended)

Use `#[composite_widget]` to compose existing widgets with builder-pattern
ergonomics. Define a config struct, implement `CompositeBuilder`, and the
macro generates `new()`, setters, and the full `Widget` impl.

```rust
#[composite_widget]
pub struct MyCard {
    pub width: u32,
    pub background_color: Color,
    #[composite(skip)]
    pub on_click: Option<Box<dyn FnMut()>>,
}

impl CompositeBuilder for MyCard {
    fn build(&self) -> Box<dyn Widget> {
        Box::new(BoxWidget::new()
            .width(self.width)
            .background_color(self.background_color))
    }
}

// Usage:
MyCard::new().width(200).background_color(Color::RED)
```

Field attributes: `#[composite(rename = "bg")]`, `#[composite(skip)]`,
`#[composite(into)]`, `#[composite(with_types = "impl Into<String>")]`.

### Full Widgets

Implement the `Widget` trait directly for complete control over layout,
painting, and event handling. See `examples/custom_widget/` for a
side-by-side comparison.

## Button

The `Button` widget accepts any child content via `.child()` or the
`button!` macro:

```rust
// Text button (requires text feature):
button!("Click me").on_click(|e| println!("clicked"))

// Any widget as child:
button!(my_icon_widget).on_click(|e| { ... })

// Full builder:
Button::new()
    .child(my_widget)
    .width(200)
    .height(50)
    .background_color(Color::RED)
    .on_click(|e| { ... })
    .on_hover(|hovering| { ... })
```

## Iconify Icons

The `aurora_iconify` crate fetches icons from [iconify.design](https://iconify.design)
at compile time. Over 200,000 icons from 150+ icon sets are available.

```toml
[dependencies]
aurora_iconify = { path = "crates/aurora_iconify" }
```

```rust
// Declare which icon sets to include (fetched at compile time):
aurora_iconify::icon_sets!("mage", "lucide");

// Type-safe access — one method per icon:
let svg: &str = Icon::mage().calendar_2_fill();

// Dynamic access by name:
let svg: Option<&str> = Icon::mage().by_name("calendar-2-fill");

// Dynamic set + name:
let svg = Icon::from_set("mage").and_then(|s| s.by_name("calendar-2-fill"));
```

Icon data is cached locally for 7 days. All SVGs are embedded directly in
the binary as `&'static str`.

## Platform Support

| Platform        | Backend               | Status    |
|-----------------|-----------------------|-----------|
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
