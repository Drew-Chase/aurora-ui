//! 2D rendering primitives for AuroraUI.
//!
//! Provides the [`Canvas`](canvas::Canvas) drawing API that operates on a raw
//! pixel buffer from a [`GpuContext`](aurora_gpu::gpu_context::GpuContext).

pub mod canvas;

/// Decoded raster image data (requires the `image` feature).
#[cfg(feature = "image")]
pub mod image_data;

/// Parsed SVG document for resolution-independent rendering (requires the `svg` feature).
#[cfg(feature = "svg")]
pub mod svg_data;