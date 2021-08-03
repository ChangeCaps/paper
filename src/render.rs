use std::sync::Arc;

use winit::window::Window;

#[derive(Clone, Debug)]
pub struct RenderInstance {
    pub device: Arc<wgpu::Device>,
    pub queue: Arc<wgpu::Queue>,
}

impl RenderInstance {
    pub async fn new(window: &Window) -> anyhow::Result<(RenderInstance, Swapchain)> {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);
        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("render device"),
                    features: wgpu::Features::default(),
                    limits: wgpu::Limits::default(),
                },
                None,
            )
            .await?;

        let desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
            format: adapter.get_swap_chain_preferred_format(&surface).unwrap(),
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };
        let swapchain = device.create_swap_chain(&surface, &desc);

        Ok((
            RenderInstance {
                device: device.into(),
                queue: queue.into(),
            },
            Swapchain {
                surface,
                desc,
                swapchain,
            },
        ))
    }
}

#[derive(Debug)]
pub struct Swapchain {
    pub surface: wgpu::Surface,
    pub desc: wgpu::SwapChainDescriptor,
    pub swapchain: wgpu::SwapChain,
}

impl Swapchain {
    #[inline]
    pub fn current_frame(&self) -> Result<wgpu::SwapChainFrame, wgpu::SwapChainError> {
        self.swapchain.get_current_frame()
    }

    #[inline]
    pub fn resize(&mut self, instance: &RenderInstance, width: u32, height: u32) {
        self.desc.width = width;
        self.desc.height = height;
        self.swapchain = instance.device.create_swap_chain(&self.surface, &self.desc);
    }

    #[inline]
    pub fn recreate(&mut self, instance: &RenderInstance) {
        self.swapchain = instance.device.create_swap_chain(&self.surface, &self.desc);
    }

    #[inline]
    pub fn format(&self) -> wgpu::TextureFormat {
        self.desc.format
    }

    #[inline]
    pub fn width(&self) -> u32 {
        self.desc.width
    }

    #[inline]
    pub fn height(&self) -> u32 {
        self.desc.height
    }
}
