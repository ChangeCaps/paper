use std::borrow::Cow;

use bytemuck::{bytes_of, cast_slice};
use glam::Mat4;
use scissor::{mesh::Mesh, Config, Shape};
use wgpu::util::DeviceExt;

use crate::{primary_pipeline::primary_pipeline, RenderInstance, Swapchain, Transform};

#[derive(Clone, Debug)]
pub enum ScaleMode {
    /// Scale camera to be (aspect * size, size).
    Aspect,
    /// Don't scale the camera.
    None,
}

#[derive(Clone, Debug)]
pub struct OrthographicCamera {
    pub left: f32,
    pub bottom: f32,
    pub right: f32,
    pub top: f32,
    pub near: f32,
    pub far: f32,
    pub size: f32,
    pub scale_mode: ScaleMode,
    pub transform: Transform,
}

impl Default for OrthographicCamera {
    #[inline]
    fn default() -> Self {
        Self {
            left: -1.0,
            bottom: -1.0,
            right: 1.0,
            top: 1.0,
            near: -500.0,
            far: 500.0,
            size: 2.0,
            scale_mode: ScaleMode::Aspect,
            transform: Transform::IDENTITY,
        }
    }
}

impl OrthographicCamera {
    #[inline]
    pub fn proj(&self, aspect: f32) -> Mat4 {
        let mut camera = self.clone();

        match self.scale_mode {
            ScaleMode::Aspect => {
                camera.left = -camera.size * aspect / 2.0;
                camera.right = camera.size * aspect / 2.0;
                camera.bottom = -camera.size / 2.0;
                camera.top = camera.size / 2.0;
            }
            ScaleMode::None => {}
        }

        Mat4::orthographic_rh(
            camera.left,
            camera.right,
            camera.bottom,
            camera.top,
            camera.near,
            camera.far,
        )
    }

    #[inline]
    pub fn view(&self) -> Mat4 {
        self.transform.matrix()
    }

    #[inline]
    pub fn view_proj(&self, aspect: f32) -> Mat4 {
        self.view().inverse() * self.proj(aspect)
    }
}

#[derive(Debug)]
pub struct Pipelines {
    pub primary: wgpu::RenderPipeline,
    pub primary_uniforms: wgpu::BindGroupLayout,
}

impl Pipelines {
    #[inline]
    pub fn new(instance: &RenderInstance, target_format: wgpu::TextureFormat) -> Self {
        let (primary, primary_uniforms) = primary_pipeline(instance, target_format);

        Self {
            primary,
            primary_uniforms,
        }
    }
}

#[derive(Debug)]
pub struct RenderTextures {
    pub primary_image: wgpu::TextureView,
    pub primary_depth: wgpu::TextureView,
}

impl RenderTextures {
    #[inline]
    pub fn new(
        instance: &RenderInstance,
        target_format: wgpu::TextureFormat,
        width: u32,
        height: u32,
    ) -> Self {
        fn texture(
            instance: &RenderInstance,
            format: wgpu::TextureFormat,
            sample_count: u32,
            width: u32,
            height: u32,
        ) -> wgpu::TextureView {
            let texture = instance.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("render texture"),
                format,
                dimension: wgpu::TextureDimension::D2,
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                sample_count,
                mip_level_count: 1,
                usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
            });

            texture.create_view(&wgpu::TextureViewDescriptor {
                label: None,
                aspect: wgpu::TextureAspect::All,
                format: None,
                dimension: None,
                array_layer_count: None,
                base_array_layer: 0,
                mip_level_count: None,
                base_mip_level: 0,
            })
        }

        Self {
            primary_image: texture(instance, target_format, 8, width, height),
            primary_depth: texture(instance, wgpu::TextureFormat::Depth24Plus, 8, width, height),
        }
    }
}

pub enum Renderable<'a> {
    Ui {
        mesh: Cow<'a, Mesh>,
        transform: Mat4,
        camera: Mat4,
    },
}

pub struct Frame<'a> {
    aspect: f32,
    pub config: Config,
    pub clear_color: [f32; 4],
    renderables: Vec<Renderable<'a>>,
}

impl<'a> Frame<'a> {
    #[inline]
    pub fn new(aspect: f32) -> Self {
        Self {
            aspect,
            config: Config::default(),
            clear_color: [1.0; 4],
            renderables: Vec::new(),
        }
    }

    #[inline]
    pub fn aspect(&self) -> f32 {
        self.aspect
    }

    #[inline]
    pub fn draw_ui(
        &mut self,
        mesh: &'a Mesh,
        transform: impl Into<Mat4>,
        camera: &OrthographicCamera,
    ) {
        self.renderables.push(Renderable::Ui {
            mesh: Cow::Borrowed(mesh),
            transform: transform.into(),
            camera: camera.view_proj(self.aspect),
        });
    }

    #[inline]
    pub fn draw_shape(
        &mut self,
        shape: &impl Shape<Input = (), Output = Mesh>,
        transform: impl Into<Mat4>,
        camera: &OrthographicCamera,
    ) {
        self.renderables.push(Renderable::Ui {
            mesh: Cow::Owned(shape.generate(&self.config, ())),
            transform: transform.into(),
            camera: camera.view_proj(self.aspect),
        });
    }
}

#[derive(Debug)]
pub struct PrimaryData {
    pub uniform_buffer: wgpu::Buffer,
    pub uniform_bind_group: wgpu::BindGroup,
    pub vertex_buffer: wgpu::Buffer,
    pub vertex_buffer_size: usize,
    pub index_buffer: wgpu::Buffer,
    pub index_buffer_size: usize,
    pub indices: u32,
}

#[derive(Debug)]
pub struct Renderer {
    instance: RenderInstance,
    swapchain: Swapchain,
    textures: RenderTextures,
    pipelines: Pipelines,
    /// # Layout
    /// 0. Transform matrix
    /// 64. Camera matrix
    primary_uniforms: Vec<PrimaryData>,
}

impl Renderer {
    #[inline]
    pub fn new(instance: &RenderInstance, swapchain: Swapchain) -> Self {
        Self {
            instance: instance.clone(),
            textures: RenderTextures::new(
                instance,
                swapchain.format(),
                swapchain.width(),
                swapchain.height(),
            ),
            pipelines: Pipelines::new(instance, swapchain.format()),
            swapchain,
            primary_uniforms: Vec::new(),
        }
    }

    #[inline]
    pub fn render(&mut self, frame: Frame<'_>) -> Result<(), wgpu::SwapChainError> {
        let swapchain_frame = self.swapchain.current_frame()?;

        let mut encoder =
            self.instance
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("render encoder"),
                });

        let mut primary_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("primary pass"),
            color_attachments: &[wgpu::RenderPassColorAttachment {
                view: &self.textures.primary_image,
                resolve_target: Some(&swapchain_frame.output.view),
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: frame.clear_color[0] as f64,
                        g: frame.clear_color[1] as f64,
                        b: frame.clear_color[2] as f64,
                        a: frame.clear_color[3] as f64,
                    }),
                    store: true,
                },
            }],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.textures.primary_depth,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: true,
                }),
                stencil_ops: None,
            }),
        });

        primary_pass.set_pipeline(&self.pipelines.primary);

        let mut idx = 0;

        for renderable in &frame.renderables {
            match renderable {
                Renderable::Ui {
                    mesh,
                    transform,
                    camera,
                } => {
                    if let Some(data) = self.primary_uniforms.get_mut(idx) {
                        self.instance.queue.write_buffer(
                            &data.uniform_buffer,
                            0,
                            bytes_of(transform),
                        );
                        self.instance.queue.write_buffer(
                            &data.uniform_buffer,
                            64,
                            bytes_of(camera),
                        );

                        let index_data: &[u8] = cast_slice(&mesh.indices);

                        if data.index_buffer_size == index_data.len() {
                            self.instance
                                .queue
                                .write_buffer(&data.index_buffer, 0, index_data);
                        } else {
                            let index_buffer = self.instance.device.create_buffer_init(
                                &wgpu::util::BufferInitDescriptor {
                                    label: Some("primary index buffer"),
                                    contents: index_data,
                                    usage: wgpu::BufferUsage::COPY_DST | wgpu::BufferUsage::INDEX,
                                },
                            );

                            data.index_buffer = index_buffer;
                            data.index_buffer_size = index_data.len();
                            data.indices = mesh.indices.len() as u32;
                        }

                        let vertex_data: &[u8] = cast_slice(&mesh.vertices);

                        if data.vertex_buffer_size == vertex_data.len() {
                            self.instance
                                .queue
                                .write_buffer(&data.vertex_buffer, 0, vertex_data);
                        } else {
                            let vertex_buffer = self.instance.device.create_buffer_init(
                                &wgpu::util::BufferInitDescriptor {
                                    label: Some("primary vertex buffer"),
                                    contents: vertex_data,
                                    usage: wgpu::BufferUsage::COPY_DST | wgpu::BufferUsage::VERTEX,
                                },
                            );

                            data.vertex_buffer = vertex_buffer;
                            data.vertex_buffer_size = vertex_data.len();
                        }
                    } else {
                        let uniform_buffer = self.instance.device.create_buffer_init(
                            &wgpu::util::BufferInitDescriptor {
                                label: Some("primary uniforms"),
                                contents: cast_slice(&[*transform, *camera]),
                                usage: wgpu::BufferUsage::COPY_DST | wgpu::BufferUsage::UNIFORM,
                            },
                        );

                        let uniform_bind_group =
                            self.instance
                                .device
                                .create_bind_group(&wgpu::BindGroupDescriptor {
                                    label: Some("primary uniforms"),
                                    layout: &self.pipelines.primary_uniforms,
                                    entries: &[wgpu::BindGroupEntry {
                                        binding: 0,
                                        resource: uniform_buffer.as_entire_binding(),
                                    }],
                                });

                        let vertex_data = cast_slice(&mesh.vertices);

                        let vertex_buffer = self.instance.device.create_buffer_init(
                            &wgpu::util::BufferInitDescriptor {
                                label: Some("primary vertex buffer"),
                                contents: vertex_data,
                                usage: wgpu::BufferUsage::COPY_DST | wgpu::BufferUsage::VERTEX,
                            },
                        );

                        let index_data = cast_slice(&mesh.indices);

                        let index_buffer = self.instance.device.create_buffer_init(
                            &wgpu::util::BufferInitDescriptor {
                                label: Some("primary index buffer"),
                                contents: index_data,
                                usage: wgpu::BufferUsage::COPY_DST | wgpu::BufferUsage::INDEX,
                            },
                        );

                        self.primary_uniforms.push(PrimaryData {
                            uniform_buffer,
                            uniform_bind_group,
                            vertex_buffer,
                            vertex_buffer_size: vertex_data.len(),
                            index_buffer,
                            index_buffer_size: index_data.len(),
                            indices: mesh.indices.len() as u32,
                        });
                    }

                    idx += 1;
                }
            }
        }

        idx = 0;

        for renderable in &frame.renderables {
            match renderable {
                Renderable::Ui { .. } => {
                    let data = &self.primary_uniforms[idx];

                    primary_pass.set_bind_group(0, &data.uniform_bind_group, &[]);
                    primary_pass.set_vertex_buffer(0, data.vertex_buffer.slice(..));
                    primary_pass
                        .set_index_buffer(data.index_buffer.slice(..), wgpu::IndexFormat::Uint32);

                    primary_pass.draw_indexed(0..data.indices, 0, 0..1);

                    idx += 1;
                }
            }
        }

        drop(primary_pass);

        self.instance
            .queue
            .submit(std::iter::once(encoder.finish()));

        Ok(())
    }

    #[inline]
    pub fn aspect(&self) -> f32 {
        self.swapchain.desc.width as f32 / self.swapchain.desc.height as f32
    }

    #[inline]
    pub fn resize(&mut self, width: u32, height: u32) {
        self.swapchain.resize(&self.instance, width, height);
        self.textures = RenderTextures::new(&self.instance, self.swapchain.format(), width, height);
    }

    #[inline]
    pub fn recreate(&mut self) {
        self.swapchain.recreate(&self.instance);
    }
}
