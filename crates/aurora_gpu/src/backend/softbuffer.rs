use crate::gpu_context::GpuContext;
use aurora_core::color::Color;
use std::sync::Arc;

pub struct SoftbufferBackend {
    context: softbuffer::Context<Arc<winit::window::Window>>,
    surface: softbuffer::Surface<Arc<winit::window::Window>, Arc<winit::window::Window>>,
    buffer: Vec<u32>,
    width: u32,
    height: u32,
}

impl SoftbufferBackend {
    pub fn new(window: Arc<winit::window::Window>) -> Result<Self, softbuffer::SoftBufferError> {
        let context = softbuffer::Context::new(window.clone())?;
        let surface = softbuffer::Surface::new(&context, window.clone())?;
        Ok(Self {
            context,
            surface,
            buffer: Vec::new(),
            width: 0,
            height: 0,
        })
    }
}

impl GpuContext for SoftbufferBackend {
    fn resize(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }
        self.width = width;
        self.height = height;
        self.buffer.resize((width * height) as usize, 0);
        if let (Some(w), Some(h)) = (
            std::num::NonZeroU32::new(width),
            std::num::NonZeroU32::new(height),
        ) {
            let _ = self.surface.resize(w, h);
        }
    }

    fn size(&self) -> (u32, u32) {
	    (self.width, self.height)
    }

    fn clear(&mut self, color: Color) {
        self.buffer.fill(color.to_rgb_u32());
    }

    fn buffer_mut(&mut self) -> &mut [u32] {
        &mut self.buffer
    }

    fn present(&mut self) {
        if self.width == 0 || self.height == 0 {
	        return;
        }

	    if let Ok(mut surface_buffer) = self.surface.buffer_mut()
	    {
		    let len = surface_buffer.len().min(self.buffer.len());
		    surface_buffer[..len].copy_from_slice(&self.buffer[..len]);
		    let _ = surface_buffer.present();
	    }
    }
}
