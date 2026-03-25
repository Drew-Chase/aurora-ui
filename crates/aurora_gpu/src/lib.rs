//! GPU surface abstraction with pluggable backends.
//!
//! Defines the [`GpuContext`](gpu_context::GpuContext) trait and feature-gated
//! backend implementations:
//! - `software` — CPU rendering via softbuffer (GDI on Windows)
//! - `opengl` — OpenGL 3.3 via glow + glutin
//! - `wgpu_backend` — Vulkan/Metal/DX12 via wgpu

pub mod gpu_context;
pub mod backend;