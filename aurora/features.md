# Feature flags

Aurora UI is feature-gated by design: every subsystem is opt-in so your binary
contains only what you use. This guide covers each feature flag in detail.

Aurora UI does not enable any features by default except `software` (the CPU
rendering backend). Below is a summary of available feature flags. You may also
notice above each function, struct, and trait there is listed one or more
feature flags required for that item to be used.

- `software`: CPU-based rendering via softbuffer (GDI on Windows). Default backend.
- `opengl`: OpenGL 3.3 rendering via glow. ~200 KB binary impact.
- `wgpu_backend`: Vulkan/Metal/DX12 rendering via wgpu. Best quality. ~2 MB binary impact.
- `text`: Font loading, text shaping, text widgets, buttons with labels, and clipboard.
- `image`: PNG/JPEG decoding and the `Image` widget.
- `svg`: SVG parsing, rasterization, and the `Svg` widget.
- `syntax`: Syntax highlighting for code display. Enables `text`.
- `animate`: Tweens, easing functions, keyframes, timelines, and animation presets.
- `dialogs`: Native file open/save/folder dialogs via rfd.
- `menu`: Native application menus (menu bar) via muda.
- `tray`: System tray icon with context menu via tray-icon. Enables `menu`.
- `a11y`: Screen reader support via AccessKit (Narrator, NVDA, VoiceOver, Orca).
- `i18n`: Locale detection, fluent-rs message bundles, and `lang!` compile-time TOML.
- `i18n-icu`: ICU4X number/date/currency formatting on top of `i18n`.
- `regex`: Regex-based validation patterns for input widgets.

## GPU Backends

At least one GPU backend must be enabled (enforced by `compile_error!`).
The `software` backend is enabled by default.

### `software` (default)

CPU-based presentation via [softbuffer](https://github.com/rust-windowing/softbuffer).
On Windows this uses GDI, on macOS it uses Core Graphics, and on Linux it uses
X11 shared memory or Wayland shared memory.

**When to use:** Development, CI, headless testing, or when GPU drivers are
unreliable.

```toml
aurora_ui = { version = "0.1", features = ["software"] }
```

### `opengl`

OpenGL 3.3 rendering via [glow](https://github.com/grovesNL/glow).
Adds approximately 200 KB to the release binary.

**When to use:** Applications that need GPU acceleration with broad
compatibility across older hardware and drivers.

```toml
aurora_ui = { version = "0.1", features = ["opengl"] }
```

### `wgpu_backend`

Vulkan/Metal/DX12 rendering via [wgpu](https://wgpu.rs/).
Adds approximately 2 MB to the release binary.

**When to use:** Applications that need the best rendering quality and
per-platform native graphics API support (DX12 on Windows, Metal on macOS,
Vulkan on Linux).

```toml
aurora_ui = { version = "0.1", features = ["wgpu_backend"] }
```

> **Priority:** When multiple backends are enabled, the runtime selects:
> wgpu > opengl > software.

---

## Content Features

### `text`

Enables font loading, text shaping via
[cosmic-text](https://github.com/pop-os/cosmic-text), and all text-dependent
widgets: `Text`, `TextInput`, `Button` with labels, `Label`, `Field`,
`TextArea`, `Combobox`, `Select`, `InputGroup`. Also enables clipboard
support via [arboard](https://github.com/1Password/arboard).

**Key types:** `FontOptions`, `FontWeight`, `FontStyle`, `FontStretch`,
`FontManager`, `TextLayout`

```toml
aurora_ui = { version = "0.1", features = ["software", "text"] }
```

### `image`

PNG and JPEG decoding via the [`image`](https://github.com/image-rs/image)
crate. Provides the `Image` widget for displaying raster graphics.

**Key types:** `ImageData`, `Image`, `ImageFit`

```toml
aurora_ui = { version = "0.1", features = ["software", "image"] }
```

### `svg`

SVG parsing via [usvg](https://github.com/RazrFalcon/resvg) and
rasterization via [tiny-skia](https://github.com/nickel-org/tiny-skia).
Provides the `Svg` widget for resolution-independent vector graphics.

**Key types:** `SvgData`, `Svg`

```toml
aurora_ui = { version = "0.1", features = ["software", "svg"] }
```

### `syntax`

Syntax highlighting for code display. Automatically enables `text`
(since highlighted code requires text rendering).

```toml
aurora_ui = { version = "0.1", features = ["software", "syntax"] }
```

---

## Animation

### `animate`

Provides a full animation system with tweens, easing functions, keyframes,
timelines, loop modes, and built-in presets. Fully independent of other
features.

**Key types:** `Tween`, `Easing`, `Keyframe`, `KeyframeAnimation`,
`Timeline`, `LoopMode`, `Preset`, `Animatable`

```toml
aurora_ui = { version = "0.1", features = ["software", "animate"] }
```

---

## Platform Integration

### `dialogs`

Native file open, file save, and folder picker dialogs via
[rfd](https://github.com/PolyMeilex/rfd).

**Key types:** `FileDialog`, `FileFilter`, `PendingDialog`

```toml
aurora_ui = { version = "0.1", features = ["software", "dialogs"] }
```

### `menu`

Native application menus (menu bar with File, Edit, View, etc.) via
[muda](https://github.com/nickel-org/muda).

**Key types:** `NativeMenu`, `MenuItemBuilder`, `SubmenuBuilder`,
`CheckMenuItemBuilder`, `MenuAccelerator`, `MenuModifiers`

```toml
aurora_ui = { version = "0.1", features = ["software", "menu"] }
```

### `tray`

System tray icon with context menus via
[tray-icon](https://github.com/nickel-org/tray-icon). Automatically
enables `menu` (tray context menus use the same menu API).

**Key types:** `TrayConfig`, `TrayInteraction`

```toml
aurora_ui = { version = "0.1", features = ["software", "tray"] }
```

---

## Accessibility & Internationalization

### `a11y`

Exposes the widget tree to platform accessibility APIs via
[AccessKit](https://github.com/AccessKit/accesskit). Supports Windows
Narrator/NVDA, macOS VoiceOver, and Linux Orca.

```toml
aurora_ui = { version = "0.1", features = ["software", "text", "a11y"] }
```

### `i18n`

Locale detection, message bundles via
[fluent-rs](https://github.com/projectfluent/fluent-rs), and compile-time
TOML locale files via the `lang!` macro (aurora_lang).

```toml
aurora_ui = { version = "0.1", features = ["software", "text", "i18n"] }
```

### `i18n-icu`

Adds [ICU4X](https://github.com/unicode-org/icu4x) formatting to the `i18n`
feature for comprehensive locale, number, date, and currency formatting.
Automatically enables `i18n`.

```toml
aurora_ui = { version = "0.1", features = ["software", "text", "i18n-icu"] }
```

---

## Input Validation

### `regex`

Adds the `Pattern` validator for regex-based input validation. Useful for
phone numbers, postal codes, and other structured input formats.

**Key types:** `Pattern`

```toml
aurora_ui = { version = "0.1", features = ["software", "text", "regex"] }
```

---

## Feature Dependency Graph

```text
i18n-icu --> i18n
tray     --> menu
syntax   --> text
```

All other features are independent and can be combined freely.

## Recommended Feature Sets

| Use case                  | Features                                              |
|---------------------------|-------------------------------------------------------|
| Blank window / embedding  | `software`                                            |
| Basic desktop app         | `software`, `text`, `image`                           |
| Full desktop app          | `software`, `text`, `image`, `svg`, `dialogs`, `menu` |
| Accessible app            | above + `a11y`                                        |
| Internationalized app     | above + `i18n` (or `i18n-icu`)                        |
| GPU-accelerated           | Replace `software` with `opengl` or `wgpu_backend`   |
