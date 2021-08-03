use crate::RenderInstance;

pub fn primary_pipeline(
    instance: &RenderInstance,
    format: wgpu::TextureFormat,
) -> (wgpu::RenderPipeline, wgpu::BindGroupLayout) {
    let module = instance
        .device
        .create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: Some("primary shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/primary.wgsl").into()),
            flags: wgpu::ShaderFlags::all(),
        });

    let uniforms = instance
        .device
        .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("primary uniforms"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                visibility: wgpu::ShaderStage::VERTEX_FRAGMENT,
                count: None,
            }],
        });

    let layout = instance
        .device
        .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("primary layout"),
            bind_group_layouts: &[&uniforms],
            push_constant_ranges: &[],
        });

    let pipeline = instance
        .device
        .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("primary pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &module,
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: 12 + 16,
                    step_mode: wgpu::InputStepMode::Vertex,
                    attributes: &[
                        wgpu::VertexAttribute {
                            offset: 0,
                            format: wgpu::VertexFormat::Float32x3,
                            shader_location: 0,
                        },
                        wgpu::VertexAttribute {
                            offset: 12,
                            format: wgpu::VertexFormat::Float32x4,
                            shader_location: 1,
                        },
                    ],
                }],
                entry_point: "main",
            },
            fragment: Some(wgpu::FragmentState {
                module: &module,
                targets: &[wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrite::ALL,
                }],
                entry_point: "main",
            }),
            primitive: wgpu::PrimitiveState::default(),
            multisample: wgpu::MultisampleState {
                count: 8,
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth24Plus,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::LessEqual,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
        });

    (pipeline, uniforms)
}
