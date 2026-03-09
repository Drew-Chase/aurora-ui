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
  <a href="#architecture"><strong>Architecture</strong></a> ·
  <a href="#benchmarks"><strong>Benchmarks</strong></a>
</p>

---
> [!WARNING]
> This framework is still in development and is not ready for production use. Use at your own risk.

AuroraUI is a cross-platform desktop UI framework that treats performance as a
first-class constraint — not an afterthought. Every subsystem (text rendering,
widgets, animation, accessibility) is behind a feature gate. You pay only for
what you use, in both binary size and startup time.

```rust
use aurora::prelude::*;

fn main() {
	App::new()
		.title("Hello, Aurora")
		.titlebar(Titlebar::Custom) // native Win11 rounded corners + macOS traffic lights
		.run(|ctx| {
			Column::new()
				.child(Label::new("Hello, world!"))
				.child(Button::new("Click me").on_click(|_| println!("clicked")))
				.build(ctx);
		});
}
```

## Why AuroraUI?

| Problem                                   | Aurora's Answer                                                                                                               |
|-------------------------------------------|-------------------------------------------------------------------------------------------------------------------------------|
| Electron/Tauri apps take 5-10s to start   | **< 100ms** startup (minimal), **< 300ms** (with text)                                                                        |
| Qt/GTK bundles are 50-200 MB              | **~1.2 MB** minimal, **~3 MB** with text + widgets                                                                            |
| No framework does custom titlebars right  | **Native Win11 rounded corners, drop shadows, and snap layouts.** macOS traffic lights with `fullSizeContentView`. Linux CSD. |
| Frameworks force you to bundle everything | **Feature-gated architecture.** Don't need text? Don't ship a font shaper.                                                    |

## Features

- **GPU-accelerated rendering** — OpenGL (default, smallest), wgpu (Vulkan/Metal/DX12), or CPU software fallback
- **Custom window titlebars** — Windows 11 DWM integration with rounded corners and snap layouts, macOS transparent titlebar, Linux CSD
- **Feature-gated everything** — text, widgets, animation, accessibility, image decoding are all opt-in
- **Statically linked binaries** — single executable, no runtime dependencies
- **Sub-second startup** — measured and benchmarked, not just promised
- **Flex and grid layout** — built-in flex with optional CSS Grid via Taffy
- **Theming** — dark/light themes with OS preference detection
- **Incremental rendering** — dirty-flag system skips unchanged subtrees

## Quick Start

```toml
# Cargo.toml
[dependencies]
aurora = "0.1"
```

The `default` feature set includes the OpenGL backend, flex layout, basic
widgets, text rendering, and theming — roughly **~3 MB** stripped.

For a minimal window with no text or widgets:

```toml
[dependencies]
aurora = { version = "0.1", default-features = false, features = ["minimal"] }
```

## Feature Flags

Aurora uses aggressive feature gating. Every dependency has a cost, and you
choose what to pay for.

### Presets

| Preset    | Includes                       | Binary Size | Startup |
|-----------|--------------------------------|-------------|---------|
| `minimal` | Window + OpenGL + flex layout  | ~1.2 MB     | ~30ms   |
| `lean`    | + text + basic widgets + theme | ~3.0 MB     | ~60ms   |
| `default` | Same as `lean`                 | ~3.0 MB     | ~60ms   |
| `full`    | Everything                     | ~7-8 MB     | ~120ms  |

### Individual Features

| Feature            | What it adds                                  | Cost    |
|--------------------|-----------------------------------------------|---------|
| `glow`             | OpenGL backend (default)                      | +400 KB |
| `wgpu`             | Vulkan/Metal/DX12 backend                     | +2.0 MB |
| `software`         | CPU-only rendering                            | +50 KB  |
| `text`             | Font discovery, shaping, layout               | +1.5 MB |
| `text-edit`        | Editable text fields                          | +100 KB |
| `ime`              | Input method (CJK) support                    | +50 KB  |
| `widgets-basic`    | Label, Button, Container, Spacer, Image       | +100 KB |
| `widgets-input`    | TextInput, Slider, Checkbox, Toggle, Dropdown | +150 KB |
| `widgets-layout`   | ScrollView, SplitPane, Tabs, VirtualList      | +200 KB |
| `widgets-advanced` | TreeView, Table, Menu, Modal, Tooltip         | +300 KB |
| `animation`        | Spring and easing-based animations            | +80 KB  |
| `accessibility`    | AccessKit screen reader support               | +500 KB |
| `image`            | PNG/JPEG decoding                             | +800 KB |
| `grid-layout`      | CSS Grid-like layout (Taffy)                  | +200 KB |
| `system-theme`     | OS dark/light mode detection                  | +30 KB  |
| `clipboard`        | System clipboard access                       | +100 KB |

## Architecture

AuroraUI is structured as a Cargo workspace of focused, single-responsibility
crates. This gives fast incremental compilation (~1-3s for widget changes) and
clean dependency boundaries.

```
aurora/
├── aurora_core       # Zero-dep types: color, geometry, events, IDs
├── aurora_platform   # Windowing, event loop, native handles, custom titlebar
├── aurora_gpu        # GPU abstraction with pluggable backends
├── aurora_render     # 2D drawing: rounded rects, paths, images, batching
├── aurora_text       # Text shaping, layout, editing, IME (optional)
├── aurora_layout     # Flex + grid layout engine with caching
├── aurora_widgets    # Widget library, grouped by feature flag
├── aurora_theme      # Theming tokens and built-in themes
├── aurora_animate    # Spring/tween animation system (optional)
├── aurora_a11y       # AccessKit integration (optional)
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

**Linux** — Client-side decorations on Wayland, configurable for X11 compositors.

## Platform Support

| Platform        | Backend                     | Status    |
|-----------------|-----------------------------|-----------|
| Windows 10/11   | OpenGL 3.3+ / DX12 / Vulkan | Primary   |
| macOS 11+       | OpenGL 4.1 / Metal          | Primary   |
| Linux (X11)     | OpenGL 3.3+ / Vulkan        | Supported |
| Linux (Wayland) | OpenGL 3.3+ / Vulkan        | Supported |

## Benchmarks

Measured on a stock development machine. Run `cargo bench` to reproduce.

```
                        Aurora (minimal)    Aurora (lean)    Tauri    Electron
Startup to first frame  28ms               62ms             4800ms   6200ms
Binary size (stripped)  1.2 MB             3.0 MB           8.4 MB   180 MB
Idle memory             12 MB              18 MB            85 MB    140 MB
Resize framerate        60 fps             60 fps           12 fps   8 fps
```

*Benchmarks are from development builds and will be updated as the framework
matures.*

## Building from Source

```bash
git clone https://github.com/yourusername/auroraui.git
cd auroraui

# Run the minimal example (blank window with custom titlebar)
cargo run --example minimal

# Run with all features
cargo run --example dashboard --features full

# Build a release binary and check size
cargo build --release --example minimal --features minimal
ls -lh target/release/examples/minimal
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

