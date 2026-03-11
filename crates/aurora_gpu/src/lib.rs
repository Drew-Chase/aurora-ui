//! GPU surface abstraction with pluggable backends.
//!
//! Defines the [`GpuContext`](gpu_context::GpuContext) trait and feature-gated
//! backend implementations. Currently available: `softbuffer` (CPU software rendering).

pub mod gpu_context;
pub mod backend;