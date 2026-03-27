# Aurora UI Framework — Code Review

**Date:** 2026-03-26
**Scope:** Full codebase (10 crates, ~12,000 LOC, 100+ files)
**Verdict:** Early-stage framework. Solid foundations in core types and rendering, but multiple correctness and safety issues block production use.

| Severity | Count |
|----------|-------|
| Critical | 4 |
| High     | 5 |
| Medium   | 7 |
| Low      | 6 |

---

## Critical Issues

### C1. Negative hue wraps to garbage color in `from_hsla`

**File:** `crates/aurora_core/src/color.rs:104`

```rust
let h = hue.into() as f32 % 360.0;
// ...
let (r1, g1, b1) = match h as u32 {
    0..60 => (c, x, 0.0),
    // ...
};
```

Rust's `%` on floats preserves sign. A hue of `-10` produces `h = -10.0`. Casting `-10.0 as u32` saturates to `0` on current Rust (was previously undefined/implementation-defined). While saturation to 0 happens to land in the first match arm, this is accidental — the behavior is platform-dependent in older compiler versions and semantically wrong.

**Fix:**
```rust
let h = ((hue.into() as f32 % 360.0) + 360.0) % 360.0;
```

---

### C2. Panicking `.expect()` in `window_event`

**File:** `crates/aurora_platform/src/app.rs:867-870`

```rust
let window = self
    .window
    .as_mut()
    .expect("Window redraw request without a valid window");
```

If a `WindowEvent` arrives before `resumed()` creates the window (possible on some platforms or during rapid suspend/resume cycles), this panics with no recovery. The codebase already uses `if let Some(window)` elsewhere for similar checks.

**Fix:** Replace with `let Some(window) = self.window.as_mut() else { return; };`

---

### C3. NaN panic in keyframe sort

**File:** `crates/aurora_animate/src/keyframes.rs:76`

```rust
keyframes.sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap());
```

If any keyframe's `time` field is NaN (e.g., from a bad calculation upstream), `partial_cmp` returns `None` and `.unwrap()` panics. Animation input often comes from user-controlled values.

**Fix:**
```rust
keyframes.sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap_or(std::cmp::Ordering::Equal));
```

---

### C4. Image buffer index overflow in `draw_image`

**File:** `crates/aurora_render/src/canvas.rs:697`

```rust
let src_idx = ((sy * img_width + sx) * 4) as usize;
```

`sy`, `img_width`, and `sx` are all `u32`. The multiplication `sy * img_width` can silently wrap on large images (e.g., 65536x65536 would overflow). The subsequent bounds check on line 698 catches some cases but operates on the already-wrapped value.

**Fix:** Compute in `usize` to avoid u32 wrapping:
```rust
let src_idx = ((sy as usize) * (img_width as usize) + (sx as usize)) * 4;
```

---

## High Severity Issues

### H1. Softbuffer resize failure silently creates buffer/surface mismatch

**File:** `crates/aurora_gpu/src/backend/softbuffer.rs:40-49`

```rust
self.width = width;
self.height = height;
self.buffer.resize((width * height) as usize, 0);
// ...
if let Err(e) = self.surface.resize(w, h) {
    log::error!("Failed to resize softbuffer surface: {e}");
}
```

The internal buffer is resized unconditionally (line 42), but if the surface resize fails (line 47), the surface retains its old dimensions. On `present()`, the size mismatch is partially handled (line 78: `let len = surface_buffer.len().min(self.buffer.len())`) but results in a truncated or corrupted frame.

**Fix:** Only update internal state after surface resize succeeds:
```rust
fn resize(&mut self, width: u32, height: u32) {
    if width == 0 || height == 0 { return; }
    if let (Some(w), Some(h)) = (NonZeroU32::new(width), NonZeroU32::new(height)) {
        if let Err(e) = self.surface.resize(w, h) {
            log::error!("Failed to resize softbuffer surface: {e}");
            return; // Don't update internal state
        }
    }
    self.width = width;
    self.height = height;
    self.buffer.resize((width * height) as usize, 0);
}
```

---

### H2. Integer blend math has rounding bias (cumulative color darkening)

**File:** `crates/aurora_render/src/canvas.rs:100-102`

```rust
let r = (fg_r * a + bg_r * inv_a) / 255;
let g = (fg_g * a + bg_g * inv_a) / 255;
let b = (fg_b * a + bg_b * inv_a) / 255;
```

Integer division truncates toward zero. For semi-transparent layers, each blend operation loses up to 1 unit per channel. In complex UIs with many overlapping semi-transparent elements, this produces visibly darker-than-expected results.

**Fix:** Add half-divisor for rounding:
```rust
let r = (fg_r * a + bg_r * inv_a + 127) / 255;
```

---

### H3. Asymmetric rounding in SDF rounded rect bounds

**File:** `crates/aurora_render/src/canvas.rs:332-335`

```rust
let mut x0 = (rect.x1.max(0.0) as u32).min(self.width);
let mut y0 = (rect.y1.max(0.0) as u32).min(self.height);
let mut x1 = (rect.x2.ceil().max(0.0) as u32).min(self.width);
let mut y1 = (rect.y2.ceil().max(0.0) as u32).min(self.height);
```

`x0`/`y0` use truncation (`as u32` truncates) while `x1`/`y1` use `.ceil()`. For fractional coordinates, this creates a 1px expansion on the right/bottom but not on the left/top. SDF anti-aliasing depends on pixel-center sampling; inconsistent bounds cause asymmetric coverage at edges.

**Fix:** Use `.floor()` for the start coordinates:
```rust
let mut x0 = (rect.x1.floor().max(0.0) as u32).min(self.width);
let mut y0 = (rect.y1.floor().max(0.0) as u32).min(self.height);
```

---

### H4. `debug_assert` in `ImageData::from_raw` stripped in release builds

**File:** `crates/aurora_render/src/image_data.rs:41-45`

```rust
debug_assert_eq!(
    pixels.len(),
    (width as usize) * (height as usize) * 4,
    "pixel buffer length must match width * height * 4"
);
```

In release builds, `debug_assert` is removed. A caller passing mismatched dimensions gets an `ImageData` that causes out-of-bounds reads in `draw_image()`. The bounds check in `draw_image` (canvas.rs:698) catches some cases but not all due to the u32 overflow issue (C4).

**Fix:** Use a proper `assert!` or return `Result`:
```rust
pub fn from_raw(pixels: Vec<u8>, width: u32, height: u32) -> Option<Self> {
    if pixels.len() != (width as usize) * (height as usize) * 4 {
        return None;
    }
    Some(Self { pixels, width, height })
}
```

---

### H5. Windows titlebar API failures silently swallowed

**File:** `crates/aurora_platform/src/windows_titlebar.rs:23-31`

```rust
pub fn apply(window: &winit::window::Window) {
    let Some(hwnd) = get_hwnd(window) else {
        log::warn!("Failed to extract HWND from winit window");
        return;
    };
    apply_rounded_corners(hwnd);
    apply_drop_shadow(hwnd);
    install_custom_frame(hwnd);
}
```

`apply_rounded_corners`, `apply_drop_shadow`, and `install_custom_frame` each call Windows APIs (`DwmSetWindowAttribute`, `DwmExtendFrameIntoClientArea`, `SetWindowSubclass`) that can fail. Errors are not logged or returned. On older Windows versions or remote desktop sessions, these fail silently and the window renders without expected decorations.

**Fix:** Log per-call results at minimum, or return `Result` so the caller can decide:
```rust
pub fn apply(window: &winit::window::Window) -> Result<(), Vec<String>> {
    // ... collect and return errors
}
```

---

## Medium Severity Issues

### M1. `winit` is an unconditional dependency in `aurora_gpu`

**File:** `crates/aurora_gpu/Cargo.toml:11`

```toml
winit = "0.30"
```

Per the project's architecture principles, `aurora_gpu` should receive window handles through its constructor, not depend on winit directly. This couples the GPU abstraction to a specific windowing library, preventing use with other windowing solutions.

**Fix:** Feature-gate winit behind each backend that needs it, or accept `impl HasRawWindowHandle` instead of `Arc<winit::window::Window>`.

---

### M2. Timeline resets and re-ticks all tracks every frame

**File:** `crates/aurora_animate/src/timeline.rs:64-71`

```rust
for track in &mut self.tracks {
    let track_time = self.elapsed - track.offset;
    if track_time > 0.0 {
        track.tween.reset();
        track.tween.tick(track_time);
    }
}
```

Every `tick()` call resets every active track to zero and replays from the start. For N tracks, this is O(N) work per frame regardless of whether tracks have changed. With many overlapping animations (e.g., staggered list item transitions), this adds unnecessary CPU work.

**Fix:** Track delta time per-track instead of replaying from zero:
```rust
for track in &mut self.tracks {
    let track_time = self.elapsed - track.offset;
    if track_time > 0.0 && !track.tween.finished() {
        track.tween.tick(dt); // Just advance by delta
    }
}
```

---

### M3. Column/Row width defaults to full available width instead of shrink-wrapping

**File:** `crates/aurora_widgets/src/layout/column.rs:213-216`

```rust
let final_width = match self.width {
    Some(w) => w as f32,
    None => available.width,
};
```

Same pattern in `crates/aurora_widgets/src/layout/row.rs:211-214`.

When no explicit width is set, Column and Row report `available.width` as their width. Height correctly computes from children. This asymmetry means auto-width containers unexpectedly stretch to fill their parent, forcing users to set explicit widths for basic alignment.

**Fix:** Compute width from widest child when no explicit width is set.

---

### M4. Inconsistent `inner_size`/`outer_size` documentation

**File:** `crates/aurora_platform/src/app.rs:639-649`

```rust
/// Returns the inner (client-area) size in logical pixels.
pub fn inner_size(&self) -> Size { ... }

/// Returns the outer (including decorations) size in physical pixels.
pub fn outer_size(&self) -> Size { ... }
```

`inner_size` claims logical pixels, `outer_size` claims physical pixels. Both call winit methods that return `PhysicalSize<u32>`. The documentation is inconsistent — both actually return physical pixels. Additionally, `outer_size()` uses a local variable named `inner_size` (line 647) which adds confusion.

**Fix:** Correct the docs to both say "physical pixels" and rename the variable in `outer_size`.

---

### M5. `icon_rgba` has no validation

**File:** `crates/aurora_platform/src/app.rs:292-298`

```rust
pub fn icon_rgba(mut self, rgba: Vec<u8>, width: u32, height: u32) -> Self {
    self.icon = Some(IconData { rgba, width, height });
    self
}
```

No check that `rgba.len() == width * height * 4`. Mismatched dimensions are only caught later when winit tries to create the icon, at which point the error is logged but not returned to the caller.

**Fix:** Validate at the builder call site and panic or return `Result`.

---

### M6. Glow backend does redundant per-frame RGBA conversion

**File:** `crates/aurora_gpu/src/backend/glow.rs:223-232`

```rust
fn convert_to_rgba(&mut self) {
    let rgba = &mut self.rgba_buffer;
    for (i, &pixel) in self.buffer.iter().enumerate() {
        let offset = i * 4;
        rgba[offset] = ((pixel >> 16) & 0xFF) as u8;
        // ...
    }
}
```

Every frame converts the entire `0x00RRGGBB` buffer to RGBA byte format before uploading to the GPU. For 1080p, this is ~8MB of conversion work per frame. The same pattern exists in the wgpu backend.

**Fix:** Use OpenGL texture swizzle masks to handle the channel reorder on the GPU, or store the buffer in RGBA format natively.

---

### M7. Button animation state lost on rebuild

**File:** `crates/aurora_widgets/src/interactables/button.rs:197-202`

```rust
let anim = Rc::new(Cell::new(AnimData { ... }));
```

`build()` creates fresh animation state each time. When a `Composite` parent rebuilds (e.g., on state change), all child buttons lose their hover/press transition progress, causing visual pops.

**Fix:** Accept an external `Rc<Cell<AnimData>>` or use widget-ID-based state persistence.

---

## Low Severity Issues

### L1. Placeholder crates in default-members

**File:** `Cargo.toml:35-48`

`aurora_layout` and `aurora_a11y` contain only placeholder `add()` functions but are listed in `default-members`. This adds unnecessary build time for crates that provide no functionality.

**Fix:** Remove from `default-members` until they have real implementations.

---

### L2. `lerp_many` panics on empty input

**File:** `crates/aurora_core/src/color.rs:228`

```rust
assert!(!colors.is_empty(), "lerp_many requires at least one color");
```

Utility functions that panic on edge-case inputs create fragile call sites. A default return (e.g., `Color::TRANSPARENT`) would be more robust.

---

### L3. Easing functions propagate NaN silently

**File:** `crates/aurora_animate/src/easing.rs:98-99`

```rust
let t = t.clamp(0.0, 1.0);
```

`f32::clamp` returns NaN when the input is NaN. All downstream easing math then produces NaN, which propagates to rendered positions/colors as invisible or glitched elements.

**Fix:** Guard against non-finite input: `if !t.is_finite() { return 0.0; }`

---

### L4. Missing SAFETY comments on unsafe Windows callbacks

**File:** `crates/aurora_platform/src/windows_titlebar.rs:73-80`

The `custom_frame_proc` unsafe extern function has no `// SAFETY:` comment documenting why the invariants are upheld. This makes auditing harder.

---

### L5. Profile settings in example `Cargo.toml` are ignored

**File:** `examples/widget_gallery/Cargo.toml:16-21`

`[profile.release]` in a non-root workspace member is silently ignored by Cargo. These settings only take effect when defined in the workspace root.

**Fix:** Remove the profile section or move it to the workspace root `Cargo.toml`.

---

### L6. `AtomicU64` ID counter in TextInput could theoretically overflow

**File:** `crates/aurora_widgets/src/text_input.rs:15-17`

```rust
static NEXT_ID: AtomicU64 = AtomicU64::new(1);
```

At one ID per nanosecond, overflow takes ~584 years. Practically impossible, but worth documenting the assumption.

---

## Production Readiness by Crate

| Crate | Status | Summary |
|-------|--------|---------|
| `aurora_core` | Ready | Solid types, good test coverage, zero deps. Minor edge case in HSL (C1). |
| `aurora_platform` | Needs Work | Panic in event handler (C2), silent API failures (H5), inconsistent docs (M4). |
| `aurora_gpu` | Needs Work | Resize error handling (H1), architecture violation (M1), redundant conversions (M6). |
| `aurora_render` | Needs Work | Index overflow (C4), blend rounding (H2), SDF asymmetry (H3), ImageData validation (H4). |
| `aurora_text` | Needs Work | Bounds check ordering could be cleaner, error handling gaps in font loading. |
| `aurora_animate` | Needs Work | NaN panic (C3), timeline performance (M2), easing NaN propagation (L3). |
| `aurora_widgets` | Fair | Event propagation works, layout width defaults are surprising (M3), animation state (M7). |
| `aurora_theme` | Early Stage | Basic theme loading works, no critical issues found. |
| `aurora_layout` | Not Started | Placeholder only. |
| `aurora_a11y` | Not Started | Placeholder only. |

---

## Strengths

- **Core types are well-designed.** `Rect`, `Point`, `Size`, `Edges`, `Corners` all have correct immutable/mutable pairs, proper clamping in `inset()`/`outset()`, and comprehensive tests.
- **SDF-based anti-aliasing** is a good architectural choice — it produces smooth edges without per-scanline complexity.
- **Clipping system is thorough.** Canvas operations consistently clip to both canvas bounds and the clip stack.
- **Feature gating philosophy** is sound — keeping text rendering optional avoids bloating apps that don't need it.
- **Crate separation** provides fast incremental compile times.

---

## Recommended Fix Order

1. **C1-C4** — Fix all critical issues (crashes and memory safety)
2. **H1, H4** — Fix silent data corruption paths (resize mismatch, release-mode validation)
3. **H2-H3** — Fix visual correctness (blend rounding, SDF symmetry)
4. **H5, M4-M5** — Fix error handling and documentation consistency
5. **M1-M3, M6-M7** — Address architecture and performance issues
6. **L1-L6** — Clean up minor issues as time permits
