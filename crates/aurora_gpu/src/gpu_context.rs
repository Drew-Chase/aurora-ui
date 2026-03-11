#![doc = include_str!("../../../.wiki/GpuContext.md")]

use aurora_core::color::Color;

/// Abstraction over a GPU rendering surface.
///
/// Backends implement this trait behind feature flags. The trait provides a
/// retained pixel buffer that persists between frames — drawing writes to the
/// buffer via [`buffer_mut`](Self::buffer_mut), then [`present`](Self::present)
/// copies to the display surface.
pub trait GpuContext {
    /// Resizes the internal buffer and surface to the given physical pixel dimensions.
    ///
    /// Implementations should no-op when width or height is zero.
    fn resize(&mut self, width: u32, height: u32);

    /// Returns the current buffer dimensions as `(width, height)` in physical pixels.
    fn size(&self) -> (u32, u32);

    /// Fills the entire buffer with the given color.
    fn clear(&mut self, color: Color);

    /// Returns a mutable reference to the raw pixel buffer.
    ///
    /// Each `u32` is a pixel in `0x00RRGGBB` format. The buffer is row-major:
    /// pixel at `(x, y)` is at index `y * width + x`.
    fn buffer_mut(&mut self) -> &mut [u32];

    /// Copies the internal buffer to the display surface and presents the frame.
    fn present(&mut self);
}
