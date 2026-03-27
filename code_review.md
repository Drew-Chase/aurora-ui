# Aurora UI Framework — Code Review

**Date:** 2026-03-26
**Scope:** Full codebase (10+ crates, ~12,000 LOC, 100+ files)
**Verdict:** Strong foundations with correct geometry, good rendering pipeline, and clean architecture. A small number of correctness bugs remain that must be fixed before production use.

| Severity | Count |
|----------|-------|
| Critical | 2 |
| High     | 4 |
| Medium   | 6 |
| Low      | 5 |

---

## Critical Issues

### C1. `Corners::new()` initializes `bottom_left` and `bottom_right` in wrong order

**File:** `crates/aurora_core/src/geometry/corners.rs:38-44`

```rust
pub const fn new(top_left: f32, top_right: f32, bottom_right: f32, bottom_left: f32) -> Self {
    Self {
        top_left,
        top_right,
        bottom_left,    // parameter says bottom_right
        bottom_right,   // parameter says bottom_left
    }
}
```

The parameters follow CSS clockwise order (top_left, top_right, **bottom_right**, bottom_left), but the struct initializer has `bottom_left` and `bottom_right` swapped. Named field init in Rust binds by **field name**, not position -- so the 3rd parameter (`bottom_right`) gets assigned to the field `bottom_left`, and vice versa.

**Impact:** Any call like `Corners::new(10.0, 10.0, 30.0, 0.0)` silently swaps the bottom corners, producing incorrect rounded rectangles. The `all()`, `from_array()`, and individual setter methods are correct -- only `new()` is affected.

**Fix:**
```rust
Self {
    top_left,
    top_right,
    bottom_right,
    bottom_left,
}
```

---

### C2. Clip stack falls back to the new rect when intersection is empty

**File:** `crates/aurora_render/src/canvas.rs:168-169`

```rust
let clipped = match self.clip_stack.last() {
    Some(existing) => existing.intersection(&rect).unwrap_or(rect),
    None => rect,
};
```

When two clip rects don't overlap, `intersection()` returns `None`. The code falls back to `rect` (the new, non-overlapping rect), which **widens** the clip region instead of tightening it. This violates the documented invariant: "nested clips always tighten the region."

**Impact:** A child widget can escape its parent's clip boundary by pushing a non-overlapping clip rect. This breaks scrollview clipping and any layout that relies on nested clips to hide overflow.

**Fix:** Fall back to a zero-area rect (empty clip) when there's no intersection:
```rust
Some(existing) => existing.intersection(&rect).unwrap_or(Rect::new(
    existing.x1, existing.y1, existing.x1, existing.y1,
)),
```

---

## High Severity Issues

### H1. Text rendering blend math missing rounding correction

**File:** `crates/aurora_text/src/text_layout.rs:168-170` (render) and `275-277` (render_rich)

```rust
let r = (fg_r * a + bg_r * inv_a) / 255;
let g = (fg_g * a + bg_g * inv_a) / 255;
let b = (fg_b * a + bg_b * inv_a) / 255;
```

The canvas blend functions (`blend_pixel`, `blend_span`, `draw_image`) were corrected to use `+ 127` rounding, but the text rendering path still uses truncating division. This creates inconsistent blending between text and non-text elements -- text will appear slightly darker than identically-colored rectangles at the same alpha.

**Fix:** Add `+ 127` to match canvas blending:
```rust
let r = (fg_r * a + bg_r * inv_a + 127) / 255;
```
Apply to both `render()` (line 168-170) and `render_rich()` (line 275-277).

---

### H2. Stack, Position, and Button internals don't forward `paint_overlay`

**Files:**
- `crates/aurora_widgets/src/layout/stack.rs` — no `paint_overlay` impl
- `crates/aurora_widgets/src/layout/position.rs` — no `paint_overlay` impl
- `crates/aurora_widgets/src/interactables/button.rs` — internal widgets (AnimatedBg, ChildProxy, FlexProxy) have no `paint_overlay` impl

The `Widget` trait requires containers to override `paint_overlay` and forward to children. The default implementation is a no-op. These container widgets silently swallow overlay paint calls, so any dropdown, popover, or tooltip rendered by a descendant widget inside a Stack, Positioned, or Button layout will not appear.

**Fix:** Implement `paint_overlay` in each container to iterate children and forward:
```rust
fn paint_overlay(&self, canvas: &mut Canvas, rect: Rect) {
    for (child, child_rect) in self.children.iter().zip(self.child_rects.iter()) {
        child.paint_overlay(canvas, child_rect.translate(&rect.origin()));
    }
}
```

---

### H3. Row/Column drop `focus_next`/`focus_prev` flags on mouse click events

**Files:**
- `crates/aurora_widgets/src/layout/row.rs:301-307`
- `crates/aurora_widgets/src/layout/column.rs:307-313`

```rust
} else if response.handled {
    return EventResponse {
        handled: true,
        cursor: response.cursor,
        request_focus: response.request_focus,
        ..Default::default()  // focus_next, focus_prev lost
    };
}
```

For non-mouse events (keyboard), focus flags are correctly propagated (lines 251-255). But for mouse click events, `..Default::default()` zeroes out `focus_next` and `focus_prev`. If a child widget sets these flags in response to a click (e.g., a custom "next field" button), the flags are silently dropped.

**Fix:** Preserve focus flags in the mouse click response:
```rust
return EventResponse {
    handled: true,
    cursor: response.cursor,
    request_focus: response.request_focus,
    focus_next: response.focus_next,
    focus_prev: response.focus_prev,
    ..Default::default()
};
```

---

### H4. Softbuffer `present()` silently discards frame errors

**File:** `crates/aurora_gpu/src/backend/softbuffer.rs:81`

```rust
let _ = surface_buffer.present();
```

If the surface present fails (minimized window, display disconnect, driver error), the frame is silently lost with no logging. The softbuffer backend already logs resize failures and buffer size mismatches, making this inconsistent.

**Fix:**
```rust
if let Err(e) = surface_buffer.present() {
    log::warn!("Failed to present softbuffer frame: {e}");
}
```

---

## Medium Severity Issues

### M1. `IntoColor` hex parsing silently defaults to black on invalid input

**File:** `crates/aurora_core/src/color.rs:287,294`

```rust
impl IntoColor for String {
    fn color(&self, has_alpha: bool) -> Color {
        let hex = u64::from_str_radix(self, 16).unwrap_or(0);
        Color::from_hex(hex, has_alpha)
    }
}
```

A typo in a theme file (e.g., `"not_a_hex"`) silently produces black. This makes theme configuration errors very hard to debug -- the user sees black elements with no error message.

**Fix:** Log a warning on parse failure:
```rust
let hex = match u64::from_str_radix(self, 16) {
    Ok(h) => h,
    Err(_) => {
        log::warn!("Invalid hex color '{self}', defaulting to black");
        0
    }
};
```

---

### M2. `icon_rgba` dimension multiplication can overflow before assert

**File:** `crates/aurora_platform/src/app.rs:295-300`

```rust
assert_eq!(
    rgba.len(),
    (width as usize) * (height as usize) * 4,
    "icon_rgba: buffer length must be width * height * 4"
);
```

If `width` and `height` are both very large (e.g., `u32::MAX`), the multiplication `(width as usize) * (height as usize) * 4` can wrap around on 64-bit systems, causing the assert to incorrectly pass. Also, zero-dimension icons (0x0) are not rejected.

**Fix:** Use checked arithmetic:
```rust
let expected = (width as usize)
    .checked_mul(height as usize)
    .and_then(|n| n.checked_mul(4))
    .expect("icon_rgba: dimensions overflow");
assert_eq!(rgba.len(), expected, "icon_rgba: buffer length must be width * height * 4");
assert!(width > 0 && height > 0, "icon_rgba: dimensions must be non-zero");
```

---

### M3. Glow/WGPU backends do redundant per-frame RGBA buffer conversion

**File:** `crates/aurora_gpu/src/backend/glow.rs:223-232`

```rust
fn convert_to_rgba(&mut self) {
    let rgba = &mut self.rgba_buffer;
    for (i, &pixel) in self.buffer.iter().enumerate() {
        let offset = i * 4;
        rgba[offset] = ((pixel >> 16) & 0xFF) as u8;
        rgba[offset + 1] = ((pixel >> 8) & 0xFF) as u8;
        rgba[offset + 2] = (pixel & 0xFF) as u8;
        rgba[offset + 3] = 255;
    }
}
```

Same pattern in `crates/aurora_gpu/src/backend/wgpu.rs`. Every frame converts the entire 0x00RRGGBB buffer to RGBA byte format before GPU upload. At 1080p, this is ~8MB of per-pixel work each frame.

**Fix:** Use OpenGL texture swizzle masks (`GL_TEXTURE_SWIZZLE_R/B`) to let the GPU handle channel reordering, or upload as `GL_BGRA` format.

---

### M4. Theme slot out-of-bounds silently returns black

**File:** `crates/aurora_theme/src/lib.rs:146-153`

```rust
pub fn color(slot: usize) -> Color {
    let profile = current_profile();
    USER_THEME
        .get()
        .and_then(|t| t.colors.get(profile))
        .and_then(|c| c.get(slot).copied())
        .unwrap_or_else(|| DEFAULT_COLORS.get(slot).copied().unwrap_or(Color::BLACK))
}
```

If a widget requests a slot beyond the 38 built-in defaults, it silently gets black. This makes theme configuration errors invisible.

**Fix:** Log a warning when falling through to the default:
```rust
.unwrap_or_else(|| {
    DEFAULT_COLORS.get(slot).copied().unwrap_or_else(|| {
        log::warn!("Theme color slot {slot} out of bounds, using black");
        Color::BLACK
    })
})
```

---

### M5. Row/Column mouse move early return drops focus flags for overlays

**File:** `crates/aurora_widgets/src/layout/row.rs:288-293`

```rust
if response.handled && !translated.contains(&pos) {
    return EventResponse {
        handled: true,
        cursor: response.cursor,
        ..Default::default()
    };
}
```

When a child handles a mouse move outside its layout rect (typical for overlays/popups), the response drops all flags. Same issue in Column at line 294-299.

**Fix:** Preserve `request_focus`, `focus_next`, `focus_prev` in this early return path.

---

### M6. Scroll delta uses undocumented magic number

**File:** `crates/aurora_platform/src/app.rs:940-942`

```rust
winit::event::MouseScrollDelta::PixelDelta(pos) => pos.y as f32 / 40.0,
```

The `40.0` divisor converts pixel scroll deltas to "line" deltas (standard is ~40px per line on most platforms). This should be a named constant with documentation.

**Fix:**
```rust
/// Approximate pixels per scroll "line" on most platforms.
const PIXELS_PER_SCROLL_LINE: f32 = 40.0;
// ...
winit::event::MouseScrollDelta::PixelDelta(pos) => pos.y as f32 / PIXELS_PER_SCROLL_LINE,
```

---

## Low Severity Issues

### L1. Missing SAFETY comments on macOS unsafe pointer cast

**File:** `crates/aurora_platform/src/window_controls/mod.rs:291-296`

```rust
unsafe {
    let ns_window: &objc2_app_kit::NSWindow =
        &*(h.ns_window.as_ptr() as *const objc2_app_kit::NSWindow);
    ns_window.performClose(None);
}
```

Dereferencing a raw pointer as a reference with no `// SAFETY:` comment documenting why the invariants are upheld.

---

### L2. Easing NaN guard returns 0.0 instead of propagating

**File:** `crates/aurora_animate/src/easing.rs:99-101`

```rust
if !t.is_finite() {
    return 0.0;
}
```

Returning 0.0 for NaN silently masks upstream bugs. Propagating NaN would make the source of bad animation timing more visible during development.

---

### L3. TextInput password bullet positioning assumes uniform glyph width

**File:** `crates/aurora_widgets/src/text_input.rs:304-334`

Password mode replaces text with Unicode bullets (U+2022). The character indexing logic assumes each bullet renders at the same width, which depends on the font. Non-monospace fonts could cause cursor drift in password fields.

---

### L4. Thread-local theme profile won't sync across threads

**File:** `crates/aurora_theme/src/lib.rs:22-34`

```rust
thread_local! {
    static CURRENT_PROFILE: Cell<usize> = const { Cell::new(0) };
}
```

Switching the theme profile on one thread has no effect on other threads. This is intentional (thread-local) but could surprise users if multi-threaded rendering is ever added.

---

### L5. `Corners` and `Edges` use f32 equality for `is_zero()`/`is_uniform()`

**File:** `crates/aurora_core/src/geometry/corners.rs:179-183`

```rust
pub fn is_uniform(&self) -> bool {
    self.top_left == self.top_right
        && self.top_right == self.bottom_right
        && self.bottom_right == self.bottom_left
}
```

Exact floating-point equality checks. Values like `7.9999999` and `8.0` from accumulated layout math will report as non-uniform. This is technically correct but fragile in practice. Consider an epsilon-based comparison for layout-facing APIs.

---

## Production Readiness by Crate

| Crate | Status | Key Issues |
|-------|--------|------------|
| `aurora_core` | **Needs fix** | Corners::new() field swap (C1), hex parse silent failure (M1) |
| `aurora_platform` | **Good** | Minor: icon overflow (M2), scroll magic number (M6) |
| `aurora_gpu` | **Good** | Present error logging (H4), RGBA conversion perf (M3) |
| `aurora_render` | **Needs fix** | Clip stack bug (C2). Blend math and SDF rounding already fixed. |
| `aurora_text` | **Needs fix** | Blend rounding inconsistency with canvas (H1) |
| `aurora_animate` | **Good** | Well-tested. NaN handling in place. Timeline optimized. |
| `aurora_widgets` | **Needs fix** | Missing paint_overlay (H2), focus flag propagation (H3) |
| `aurora_theme` | **Good** | Minor: slot bounds logging (M4) |
| `aurora_layout` | Not started | Placeholder only |
| `aurora_a11y` | Not started | Placeholder only |

---

## Strengths

- **Geometry types are well-designed.** `Rect` with min/max corners, proper `inset()` clamping, comprehensive operator overloads. The immutable/mutable method pairs pattern is clean.
- **SDF-based anti-aliasing** produces smooth rounded rects without per-scanline complexity. The `rounded_rect_sdf` function handles non-uniform corner radii correctly.
- **Canvas blend math is now correct** with proper `+ 127` rounding in all three blend sites (blend_pixel, blend_span, draw_image).
- **Animation system is solid.** Tweens, timelines, and keyframes all have good test coverage. Easing functions handle edge cases (zero-duration, elastic overshoot). Timeline tick was optimized to delta-based advancing.
- **Feature gating is sound.** Text, image, SVG, syntax highlighting, and animation are all independently gatable. The softbuffer/glow/wgpu backends are cleanly feature-gated.
- **Focus/Tab navigation works correctly** for keyboard events, with proper wrap-around and blur/focus sequencing.
- **Error handling avoids `thiserror`/`anyhow`** as specified -- manual `Display` + `Error` impls keep the dependency graph lean.
- **Crate separation** provides fast incremental builds (~1-3s for widget changes).
- **The proc macro `#[composite_widget]`** cleanly generates builder setters from config structs.

---

## Recommended Fix Order

1. **C1** — Corners::new() field swap (silent rendering corruption)
2. **C2** — Clip stack intersection fallback (breaks scrollview/overflow clipping)
3. **H1** — Text blend rounding (visual inconsistency with canvas)
4. **H2** — Missing paint_overlay forwarding (overlays invisible in containers)
5. **H3** — Focus flag propagation in Row/Column mouse events
6. **H4** — Softbuffer present error logging
7. **M1-M6** — Medium issues as time permits
8. **L1-L5** — Low issues for polish
