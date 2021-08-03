use futures::executor::block_on;
use winit::{
    dpi::PhysicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use crate::{Frame, RenderInstance, Renderer};

#[allow(unused)]
pub trait State {
    fn draw<'a>(&'a mut self, frame: &mut Frame<'a>) {}
}

pub struct App {
    pub size: Option<(u32, u32)>,
    pub title: String,
}

impl App {
    #[inline]
    pub fn new() -> Self {
        Self {
            size: None,
            title: String::from("Paper Application"),
        }
    }

    #[inline]
    pub fn window_size(mut self, width: u32, height: u32) -> Self {
        self.size = Some((width, height));
        self
    }

    #[inline]
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    #[inline]
    pub fn run(self, mut state: impl State + 'static) -> ! {
        simple_logger::SimpleLogger::new()
            .with_level(log::LevelFilter::Debug)
            .with_module_level("wgpu", log::LevelFilter::Error)
            .with_module_level("winit", log::LevelFilter::Warn)
            .with_module_level("naga", log::LevelFilter::Warn)
            .with_module_level("gfx", log::LevelFilter::Warn)
            .init()
            .unwrap();

        let event_loop = EventLoop::new();
        let mut window_builder = WindowBuilder::new().with_title(self.title);

        if let Some((width, height)) = self.size {
            window_builder = window_builder.with_inner_size(PhysicalSize::new(width, height));
        }

        let window = window_builder.build(&event_loop).unwrap();

        let (instance, swapchain) = block_on(RenderInstance::new(&window)).unwrap();

        let mut renderer = Renderer::new(&instance, swapchain);

        event_loop.run(move |event, _, control_flow| match event {
            Event::RedrawRequested(_) => {
                let mut render_frame = Frame::new(renderer.aspect());

                state.draw(&mut render_frame);

                match renderer.render(render_frame) {
                    Ok(_) => {}
                    Err(wgpu::SwapChainError::Lost) => renderer.recreate(),
                    Err(wgpu::SwapChainError::OutOfMemory) => {
                        log::error!("Out of gpu memory");

                        *control_flow = ControlFlow::Exit;
                    }
                    Err(e) => log::error!("{}", e),
                }
            }
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                }
                WindowEvent::Resized(size) => {
                    renderer.resize(size.width, size.height);
                }
                WindowEvent::ScaleFactorChanged {
                    new_inner_size: size,
                    ..
                } => {
                    renderer.resize(size.width, size.height);
                }
                _ => {}
            },
            _ => {}
        })
    }
}
