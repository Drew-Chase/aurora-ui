# Aurora UI Framework — Code Review (Round 3)

**Date:** 2026-03-26
**Scope:** Full codebase (10+ crates, ~12,000 LOC, 100+ files)
**Prior work:** 27 commits across 2 review rounds fixed all prior critical/high issues
**Verdict:** ~95% production-ready. No critical correctness bugs remain. Remaining issues are defensive hardening and silent failure logging.

| Severity | Count |
|----------|-------|
| Critical | 0 |
| High     | 2 |
| Medium   | 3 |
| Low      | 3 |

---

## High Severity Issues

### H1. Glow/WGPU backends use unchecked `width * height` in buffer allocation

**Files:**
- `crates/aurora_gpu/src/backend/glow.rs:245`
- `crates/aurora_gpu/src/backend/wgpu.rs:200,281`

```rust
let pixel_count = (width * height) as usize;
self.buffer.resize(pixel_count, 0);
self.rgba_buffer.resize(pixel_count * 4, 0);
```

`width * height` is u32 arithmetic that silently wraps on overflow (e.g., 65536x65536 = 0). The subsequent `as usize` cast operates on the wrapped value, allocating a buffer far too small. Drawing into it causes an out-of-bounds panic or memory corruption.

Note: The softbuffer backend has the same pattern but is protected by the surface resize failing first (fixed in a prior commit). Glow and WGPU are not protected.

**Fix:** Use checked arithmetic in usize:
```rust
let pixel_count = (width as usize).checked_mul(height as usize)
    .expect("pixel dimensions overflow");
```

---

### H2. Text layout divides by zero when width is 0

**File:** `crates/aurora_text/src/text_layout.rs:101` (render) and `~198` (render_rich)

```rust
let height = buffer.len() as i32 / width as i32;
```

If `width` is 0 (e.g., during initial layout before the first resize, or a zero-width canvas), this panics with division by zero. The width parameter comes directly from the caller.

**Fix:** Early return when width is 0:
```rust
if width == 0 { return; }
```

---

## Medium Severity Issues

### M1. Font loading returns None silently on parse failure

**File:** `crates/aurora_text/src/font_manager.rs:38-45`

```rust
pub fn load_from_bytes(&mut self, bytes: &[u8]) -> Option<String> {
    let db = self.font_system.db_mut();
    let bytes = Arc::new(bytes.to_vec());
    let font_source = Source::Binary(bytes);
    let id = db.load_font_source(font_source);
    id.first()
        .and_then(|face_id| db.face(*face_id).map(|face| face.families[0].0.clone()))
}
```

If cosmic-text fails to parse the font bytes, this returns `None` with no logging. The caller has no visibility into why text isn't rendering. In production, this makes font configuration errors invisible.

**Fix:** Log a warning before returning None.

---

### M2. SVG render returns empty Vec silently on allocation failure

**File:** `crates/aurora_render/src/svg_data.rs:41-42`

```rust
let Some(mut pixmap) = tiny_skia::Pixmap::new(width, height) else {
    return Vec::new();
};
```

If pixmap allocation fails (OOM or invalid dimensions), returns empty with no indication. The caller renders nothing with no error.

**Fix:** Log a warning before returning.

---

### M3. Row/Column center alignment can produce negative offsets

**File:** `crates/aurora_widgets/src/layout/row.rs:194`

```rust
Align::Center => (align_height - child_size.height) / 2.0,
Align::End => align_height - child_size.height,
```

If a child is taller than the alignment height (e.g., a large image in a small row), the subtraction produces a negative value, positioning the child above the container's top edge. Same issue in Column for width alignment.

**Fix:** Clamp to zero: `(align_height - child_size.height).max(0.0) / 2.0`

---

## Low Severity Issues

### L1. Glow viewport casts u32 to i32 without overflow check

**File:** `crates/aurora_gpu/src/backend/glow.rs:254`

```rust
self.gl.viewport(0, 0, width as i32, height as i32);
```

If width exceeds `i32::MAX` (~2.1 billion), the cast wraps to a negative value. OpenGL may interpret this as an error or produce corruption.

**Fix:** Clamp: `width.min(i32::MAX as u32) as i32`

---

### L2. FontOptions accepts NaN/negative font sizes

**File:** `crates/aurora_text/src/font_options.rs:45`

```rust
pub fn size(mut self, size: f32) -> Self {
    self.size = Some(size);
    self
}
```

No validation. NaN or negative sizes could cause unexpected behavior in cosmic-text.

**Fix:** Add `debug_assert!(size.is_finite() && size > 0.0)`

---

### L3. Tween duration accepts NaN

**File:** `crates/aurora_animate/src/tween.rs:54`

```rust
pub fn duration(mut self, seconds: f32) -> Self {
    self.duration = seconds.max(0.0);
    self
}
```

`NaN.max(0.0)` returns `0.0` in Rust (since 1.70), but this is implementation-defined in older versions. NaN durations indicate upstream bugs that should be caught early.

**Fix:** Add `debug_assert!(seconds.is_finite())`

---

## Production Readiness by Crate

| Crate | Status | Notes |
|-------|--------|-------|
| `aurora_core` | **Production-ready** | Solid geometry, color math, zero deps |
| `aurora_platform` | **Production-ready** | Robust event loop, proper error handling |
| `aurora_gpu` | **Needs hardening** | Buffer overflow in glow/wgpu resize (H1) |
| `aurora_render` | **Production-ready** | Correct blend math, SDF AA, solid clipping |
| `aurora_text` | **Needs hardening** | Division by zero (H2), silent font failure (M1) |
| `aurora_animate` | **Production-ready** | Well-tested, NaN-safe, optimized timeline |
| `aurora_widgets` | **Near-ready** | Minor alignment clamping (M3) |
| `aurora_theme` | **Production-ready** | Slot bounds logging in place |

---

## Strengths

- **All prior correctness bugs are fixed.** Corners field order, clip stack, blend rounding, SDF symmetry, focus propagation, paint_overlay forwarding — all resolved.
- **Error handling is consistent.** Platform errors use proper Display/Error impls, GPU backends log failures, theme system warns on bounds.
- **Architecture is clean.** Zero-dep core, feature-gated backends, clean trait abstractions, fast incremental builds.
- **Animation system is production-grade.** Comprehensive easing functions, delta-based timeline, NaN guards, good test coverage.
- **Widget system works correctly.** Layout algorithms are sound, event propagation preserves all flags, overlay rendering chains through all containers.
- **The proc macro `#[composite_widget]`** generates correct builder patterns with type-safe configuration.

---

## Recommended Fix Order

1. **H1** — Checked arithmetic in Glow/WGPU resize (memory safety)
2. **H2** — Zero-width guard in text layout (panic prevention)
3. **M1** — Font load failure logging (debuggability)
4. **M2** — SVG allocation failure logging (debuggability)
5. **M3** — Alignment offset clamping (visual correctness)
6. **L1-L3** — Viewport clamp, input validation (defensive hardening)
