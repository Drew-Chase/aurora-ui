# Aurora UI - Production Readiness Roadmap

## Current Status: ~55% Production Ready

Aurora UI is a well-architected, GPU-accelerated Rust UI framework with an impressive widget library (52+ components), excellent performance (374 KB minimal binary, sub-130ms startup), and solid theming/animation systems. However, several critical gaps prevent it from being a general-purpose desktop application framework.

This roadmap outlines everything needed to reach 100% production readiness.

---

## Phase 1: Critical Foundation (55% -> 70%)

These are blockers. No production app should ship without them.

### ~~1.1 Accessibility (a11y)~~ - _Complete_

The `aurora_a11y` crate exists but is empty. Accessibility is a legal requirement in many markets and a moral imperative everywhere.

**Required work:**

- Integrate [AccessKit](https://github.com/AccessKit/accesskit) into the widget tree
- Add semantic roles to all widgets (button, checkbox, textbox, slider, dialog, etc.)
- Expose widget labels, descriptions, and states to the accessibility tree
- Implement focus management that AccessKit can track
- Add live region announcements for dynamic content (toasts, alerts, progress)
- Ensure all interactive widgets are operable via keyboard alone
- Add `aria-expanded`, `aria-checked`, `aria-selected` equivalents to relevant widgets
- Test with NVDA (Windows), VoiceOver (macOS), and Orca (Linux)

**Components affected:** Every interactive widget in `aurora_widgets`

**Estimated scope:** Large - touches the entire widget trait and every component

--- 

### ~~1.2 Test Suite & CI/CD~~ - _Complete_

~~Zero tests exist today. Regressions will ship silently without a test infrastructure.~~

**Required work:**

- **Unit tests** for `aurora_core` (geometry math, color conversions, edge calculations)
- **Unit tests** for `aurora_render` (canvas clipping, pixel blending, shape rasterization)
- **Unit tests** for `aurora_text` (font loading, text layout, shaping)
- **Unit tests** for `aurora_animate` (tween interpolation, easing curves, timeline sequencing)
- **Widget tests** for `aurora_widgets` (layout calculations, event dispatch, state transitions)
- **Integration tests** for `aurora_platform` (window creation, frame lifecycle, event loop)
- **Snapshot/visual regression tests** for rendered output (pixel-compare reference images)
- **CI pipeline** (GitHub Actions) running `cargo clippy`, `cargo test`, and `cargo build` on Windows/macOS/Linux
- **Benchmarks** automated in CI to catch performance regressions

**New files needed:**

- `tests/` directories in each crate
- `.github/workflows/ci.yml`
- `benches/` for criterion benchmarks

---

### ~~1.3 File Dialogs~~ (Done)

Implemented in `aurora_platform::dialogs` behind the `dialogs` feature flag.

- [x] Integrated [rfd](https://github.com/PolyMeilex/rfd) v0.17
- [x] `FileDialog` builder API with `open_file`, `open_files`, `save_file`, `pick_folder`
- [x] `_with(window)` variants for modal parent window support
- [x] `FileFilter` for file type filtering
- [x] Feature-gated: `dialogs` feature on `aurora_platform` / `aurora_ui`
- [x] Example: `examples/file_dialog_example/`
- [x] Async variants that don't block the event loop (`*_async` and `*_with_async` methods returning `PendingDialog<T>`)

---

### 1.4 Drag and Drop

Required for file managers, kanban boards, list reordering, and desktop integration.

**Required work:**

- **Internal DnD** (widget-to-widget):
  - Add `DragEvent` variants to `WidgetEvent` (DragStart, DragMove, DragEnd, DragEnter, DragLeave, Drop)
  - Add `draggable(bool)` and `on_drop` to Widget trait or as a wrapper widget
  - Implement hit-testing during drag to find drop targets
  - Visual feedback: drag ghost/preview, drop zone highlighting
- **External DnD** (OS file drops):
  - Handle winit's `WindowEvent::DroppedFile` and `HoveredFile`
  - Surface file drop events to widgets
  - Support dragging files out of the app (platform-specific)

**Components affected:** `aurora_core` (new event types), `aurora_platform` (event dispatch), `aurora_widgets` (DnD wrapper)

---

## Phase 2: Essential Features (70% -> 85%)

These are needed for most real-world applications.

### 2.1 Internationalization (i18n/l10n)

**Required work:**

- Integrate or recommend [fluent-rs](https://github.com/projectfluent/fluent-rs) for message formatting
- Add locale-aware number/date/currency formatting utilities
- Implement RTL (right-to-left) layout support:
  - `Row` and `Column` must respect text direction
  - `Edges` (padding/margin) need logical properties (inline-start/end vs left/right)
  - Text alignment needs logical values
- Pass locale to `cosmic_text::FontSystem` instead of empty string
- Add bidirectional text support in text layout

**Components affected:** `aurora_text`, `aurora_widgets` (layout containers), `aurora_core` (Edges logical properties)

---

### 2.2 Multi-Window Support

**Required work:**

- Refactor `App` to manage multiple `AppWindow` instances
- Window spawning API: `app.open_window(config, root_widget)`
- Cross-window communication (shared state, message passing)
- Independent event loops or shared loop with window routing
- Window lifecycle events (opened, closing, closed, focus gained/lost)
- Parent-child window relationships (modal windows owned by parent)

**Components affected:** `aurora_platform` (major refactor of `App` and event loop)

---

### 2.3 System Tray & Native Menus

**Required work:**

- Integrate [tray-icon](https://github.com/nickelc/tray-icon) for system tray support
- Add tray icon API: icon, tooltip, context menu, click handler
- Integrate [muda](https://github.com/nickelc/muda) for native OS menu bars
- Menu bar API: items, submenus, separators, checkmarks, keyboard shortcuts
- Bridge native menu events into the Aurora event system

**New module:** `aurora_platform::tray`, `aurora_platform::menu`

---

### 2.4 Form Validation

**Required work:**

- Add validation trait/system for form fields:
  - `Validator` trait with `validate(value) -> Result<(), Vec<String>>`
  - Built-in validators: required, min/max length, regex, email, numeric range
  - Custom validator support
- Add error state rendering to form widgets (red border, error message below field)
- Add `Field` component enhancements: label, description, error message, required indicator
- Form-level validation: validate all fields, collect errors, prevent submit

**Components affected:** `TextInput`, `TextArea`, `Select`, `Combobox`, `Field`, `InputGroup`

---

### 2.5 Virtual Scrolling

**Required work:**

- Implement virtualized list/table rendering for large datasets
- Only render visible items + buffer zone
- Maintain scroll position and selection state
- Support variable-height items
- Apply to `DataTable`, `Select` dropdown, `Combobox` dropdown, and general lists

**New component:** `VirtualList` or integrate into `ScrollView`

---

### 2.6 Undo/Redo System

**Required work:**

- Add command pattern infrastructure for undoable actions
- Implement undo/redo in `TextInput` and `TextArea` (Ctrl+Z / Ctrl+Shift+Z)
- Optional app-level undo stack for custom actions
- Undo grouping (batch multiple edits into one undo step)

**Components affected:** `TextInput`, `TextArea`, new `aurora_core::undo` module

---

### 2.7 Multi-Threading Support

**Required work:**

- Evaluate migration from `Rc<RefCell<T>>` to `Arc<Mutex<T>>` or channel-based messaging
- Add async task spawning for background work (file I/O, network, computation)
- Thread-safe state updates that trigger UI rebuilds on the main thread
- Consider adopting a message-passing architecture (like Elm/Iced) for thread safety

**Components affected:** `aurora_widgets` (Composite, StateSetter), `aurora_platform` (event loop)

**Note:** This is a significant architectural decision. A message-passing model may be preferable to shared-state concurrency.

---

## Phase 3: Polish & Ecosystem (85% -> 95%)

Features that elevate the framework from functional to professional.

### 3.1 Missing Widgets

| Widget | Priority | Description |
|--------|----------|-------------|
| **TreeView** | High | Hierarchical data display with expand/collapse, selection, lazy loading |
| **ColorPicker** | Medium | Color selection with hex/RGB/HSL input, palette, eyedropper |
| **TimePicker** | Medium | Time selection (hours, minutes, AM/PM or 24h) to complement DatePicker |
| **DateTimePicker** | Medium | Combined date + time selection |
| **DateRangePicker** | Medium | Select a start and end date |
| **NumberInput** | Medium | Numeric input with increment/decrement buttons, min/max, step |
| **RichTextEditor** | Low | WYSIWYG text editing with formatting (bold, italic, links) |
| **SplitPane** | Medium | Resizable split view (horizontal/vertical) with drag handle |
| **Stepper** | Low | Step-by-step wizard/progress indicator |
| **SegmentedControl** | Low | iOS-style segmented toggle bar |
| **TagInput** | Medium | Multi-value input with removable tags |
| **AutoComplete** | Medium | Text input with suggestion dropdown (fuzzy search) |
| **Sheet/Drawer** | Medium | Slide-in panel from edge of screen |

---

### 3.2 Enhanced DataTable

**Required work:**

- Column resizing (drag column borders)
- Column reordering (drag column headers)
- Row selection (single, multi, range with Shift+click)
- Cell editing (inline edit on double-click)
- Column filtering (per-column filter inputs)
- Fixed/sticky columns and headers
- Row grouping and expandable groups
- Export support (CSV, clipboard)
- Virtual scrolling integration (from Phase 2.5)

---

### 3.3 Packaging & Distribution

**Required work:**

- Add build scripts or templates for platform-specific packaging:
  - **Windows:** WiX/NSIS installer templates, MSIX support
  - **macOS:** .app bundle creation, DMG packaging, notarization guide
  - **Linux:** AppImage, Flatpak, .deb/.rpm templates
- Code signing documentation and scripts
- Auto-update mechanism (integrate with [self_update](https://github.com/nickelc/self_update) or similar)
- Application icon embedding (`.ico`, `.icns`, embedded resources)

**New:** `tools/` directory with packaging scripts and templates, or a `cargo-aurora` CLI tool

---

### 3.4 Clipboard Enhancements

**Required work:**

- Rich clipboard support (HTML, RTF, images)
- Copy/paste in all text widgets by default
- Cut support (Ctrl+X)
- Clipboard change monitoring (optional)

---

### 3.5 Error Handling Improvements

**Required work:**

- Define typed error enums for each crate (not just strings)
- Add error recovery strategies (fallback rendering, graceful degradation)
- User-facing error display widget (error boundary concept)
- Panic handler that shows a crash dialog instead of silent exit
- Logging integration with structured logging (tracing crate)

---

### 3.6 Hot Reload / Dev Experience

**Required work:**

- Theme hot-reload (watch `themes.json` and rebuild at runtime)
- Widget inspector (dev-mode overlay showing widget bounds, IDs, state)
- Performance overlay (FPS, frame time, widget count, layout time)
- `cargo-aurora` CLI for scaffolding new projects

---

## Phase 4: Maturity (95% -> 100%)

The final stretch for a truly production-grade framework.

### 4.1 Comprehensive Documentation

- API reference for every public type and function (rustdoc)
- Migration guides for breaking changes between versions
- Architecture guide for contributors
- Cookbook with common patterns (forms, navigation, state management, theming)
- Performance tuning guide
- Accessibility compliance guide

---

### 4.2 Ecosystem & Community

- Publish all crates to crates.io
- Semantic versioning with a changelog
- Issue templates and contribution guidelines
- Example gallery deployed as a website (screenshots/GIFs)
- Template projects (starter apps for common patterns)

---

### 4.3 Advanced Rendering

- Text selection across multiple Text widgets
- Subpixel text rendering (ClearType on Windows, LCD on macOS)
- GPU-accelerated canvas operations (move rasterization to shaders)
- Retained-mode rendering option (scene graph diffing to minimize redraws)
- Custom shader support for advanced visual effects

---

### 4.4 Advanced Platform Integration

- Global keyboard shortcuts (register hotkeys even when app is not focused)
- Notification support (OS-native notifications, not just in-app toasts)
- Deep linking / URL scheme handling
- Single-instance enforcement (prevent duplicate app launches)
- Taskbar/dock badge counts
- Touch/pen input support

---

### 4.5 Hardening

- Fuzz testing for text input, layout edge cases, and event dispatch
- Memory leak detection in CI
- Security audit of unsafe code blocks
- Performance regression tracking with historical benchmarks
- Stress testing (thousands of widgets, rapid state changes, large text buffers)

---

## Summary

| Phase | Focus | Readiness |
|-------|-------|-----------|
| **Current** | Widgets, theming, animation, rendering | **55%** |
| **Phase 1** | Accessibility, testing, file dialogs, drag-and-drop | **70%** |
| **Phase 2** | i18n, multi-window, system tray, validation, virtual scroll, undo, threading | **85%** |
| **Phase 3** | Missing widgets, packaging, clipboard, error handling, dev tools | **95%** |
| **Phase 4** | Docs, ecosystem, advanced rendering, platform integration, hardening | **100%** |

---

## Priority Order (What to Build Next)

If resources are limited, this is the recommended order of implementation:

1. **Test suite + CI/CD** - Prevents regressions as you build everything else
2. **Accessibility** - Legal/ethical requirement, hardest to retrofit later
3. **File dialogs** - Small effort, massive impact on app capability
4. **Undo/redo in text inputs** - Expected by every user, jarring when missing
5. **Form validation** - Required for any data-entry application
6. **Drag and drop** - Enables a large class of applications
7. **Virtual scrolling** - Unblocks real-world data volumes
8. **Multi-window** - Required for many app architectures
9. **i18n** - Needed before any international release
10. **Everything else** - Iterate based on user demand
