#![doc = include_str!("../../../.wiki/GpuContext.md")]

use aurora_core::color::Color;

pub trait GpuContext {
    fn resize(&mut self, width: u32, height: u32);
    fn size(&self) -> (u32, u32);
    fn clear(&mut self, color: Color);
	fn buffer_mut(&mut self) -> &mut [u32];
	fn present(&mut self);
}
