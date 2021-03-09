use pixels::wgpu::util::{BufferInitDescriptor, DeviceExt};
use pixels::wgpu::*;

pub struct GlowPostProcess {
    pub texture1: TextureView,
    texture2: TextureView,
    texture3: TextureView,

    sampler: Sampler,
    bind_group_layout1: BindGroupLayout,
    bind_group_layout2: BindGroupLayout,
    bind_group_layout3: BindGroupLayout,

    copy_glowing_pass: RenderPass,
    vertical_blur_pass: RenderPass,
    horizontal_blur_pass: RenderPass,
    combine_pass: RenderPass,
}

struct RenderPass {
    pipeline: RenderPipeline,
    bind_group: BindGroup,
}

impl GlowPostProcess {
    pub fn new(device: &Device, texture_width: u32, texture_height: u32) -> Self {
        let sampler = device.create_sampler(&SamplerDescriptor {
            label: Some("glow_post_process_sampler"),
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Nearest,
            min_filter: FilterMode::Nearest,
            mipmap_filter: FilterMode::Nearest,
            lod_min_clamp: 0.0,
            lod_max_clamp: 100.0,
            compare: None,
            anisotropy_clamp: None,
            border_color: None,
        });
        let bind_group_layout1 = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("glow_post_process_bind_group_layout1"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStage::FRAGMENT,
                    ty: BindingType::Sampler {
                        filtering: true,
                        comparison: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStage::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
            ],
        });
        let bind_group_layout2 = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("glow_post_process_bind_group_layout2"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStage::FRAGMENT,
                    ty: BindingType::Sampler {
                        filtering: true,
                        comparison: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStage::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStage::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });
        let bind_group_layout3 = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("glow_post_process_bind_group_layout3"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStage::FRAGMENT,
                    ty: BindingType::Sampler {
                        filtering: true,
                        comparison: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStage::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStage::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
            ],
        });

        let (
            [texture1, texture2, texture3],
            [copy_glowing_bind_group, vertical_blur_bind_group, horizontal_blur_bind_group, combine_bind_group],
        ) = create_resources(
            device,
            texture_width,
            texture_height,
            &sampler,
            &bind_group_layout1,
            &bind_group_layout2,
            &bind_group_layout3,
        );

        let fullscreen_shader =
            device.create_shader_module(&include_spirv!("../shaders/fullscreen.spv"));
        let copy_glowing_shader =
            device.create_shader_module(&include_spirv!("../shaders/copy_glowing.spv"));
        let vertical_blur_shader =
            device.create_shader_module(&include_spirv!("../shaders/vertical_blur.spv"));
        let horizontal_blur_shader =
            device.create_shader_module(&include_spirv!("../shaders/horizontal_blur.spv"));
        let combine_shader = device.create_shader_module(&include_spirv!("../shaders/combine.spv"));

        let pipeline_layout1 = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("glow_post_process_pipeline_layout1"),
            bind_group_layouts: &[&bind_group_layout1],
            push_constant_ranges: &[],
        });
        let pipeline_layout2 = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("glow_post_process_pipeline_layout2"),
            bind_group_layouts: &[&bind_group_layout2],
            push_constant_ranges: &[],
        });
        let pipeline_layout3 = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("glow_post_process_pipeline_layout3"),
            bind_group_layouts: &[&bind_group_layout3],
            push_constant_ranges: &[],
        });
        let copy_glowing_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("glow_post_process_copy_glowing_pipeline"),
            layout: Some(&pipeline_layout1),
            vertex: VertexState {
                module: &fullscreen_shader,
                entry_point: "main",
                buffers: &[],
            },
            primitive: PrimitiveState::default(),
            depth_stencil: None,
            multisample: MultisampleState::default(),
            fragment: Some(FragmentState {
                module: &copy_glowing_shader,
                entry_point: "main",
                targets: &[ColorTargetState {
                    format: TextureFormat::Bgra8UnormSrgb,
                    color_blend: BlendState::REPLACE,
                    alpha_blend: BlendState::REPLACE,
                    write_mask: ColorWrite::ALL,
                }],
            }),
        });
        let vertical_blur_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("glow_post_process_vertical_blur_pipeline"),
            layout: Some(&pipeline_layout2),
            vertex: VertexState {
                module: &fullscreen_shader,
                entry_point: "main",
                buffers: &[],
            },
            primitive: PrimitiveState::default(),
            depth_stencil: None,
            multisample: MultisampleState::default(),
            fragment: Some(FragmentState {
                module: &vertical_blur_shader,
                entry_point: "main",
                targets: &[ColorTargetState {
                    format: TextureFormat::Bgra8UnormSrgb,
                    color_blend: BlendState::REPLACE,
                    alpha_blend: BlendState::REPLACE,
                    write_mask: ColorWrite::ALL,
                }],
            }),
        });
        let horizontal_blur_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("glow_post_process_horizontal_blur_pipeline"),
            layout: Some(&pipeline_layout2),
            vertex: VertexState {
                module: &fullscreen_shader,
                entry_point: "main",
                buffers: &[],
            },
            primitive: PrimitiveState::default(),
            depth_stencil: None,
            multisample: MultisampleState::default(),
            fragment: Some(FragmentState {
                module: &horizontal_blur_shader,
                entry_point: "main",
                targets: &[ColorTargetState {
                    format: TextureFormat::Bgra8UnormSrgb,
                    color_blend: BlendState::REPLACE,
                    alpha_blend: BlendState::REPLACE,
                    write_mask: ColorWrite::ALL,
                }],
            }),
        });
        let combine_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("glow_post_process_combine_pipeline"),
            layout: Some(&pipeline_layout3),
            vertex: VertexState {
                module: &fullscreen_shader,
                entry_point: "main",
                buffers: &[],
            },
            primitive: PrimitiveState::default(),
            depth_stencil: None,
            multisample: MultisampleState::default(),
            fragment: Some(FragmentState {
                module: &combine_shader,
                entry_point: "main",
                targets: &[ColorTargetState {
                    format: TextureFormat::Bgra8UnormSrgb,
                    color_blend: BlendState::REPLACE,
                    alpha_blend: BlendState::REPLACE,
                    write_mask: ColorWrite::ALL,
                }],
            }),
        });

        Self {
            texture1,
            texture2,
            texture3,

            sampler,
            bind_group_layout1,
            bind_group_layout2,
            bind_group_layout3,

            copy_glowing_pass: RenderPass {
                pipeline: copy_glowing_pipeline,
                bind_group: copy_glowing_bind_group,
            },
            vertical_blur_pass: RenderPass {
                pipeline: vertical_blur_pipeline,
                bind_group: vertical_blur_bind_group,
            },
            horizontal_blur_pass: RenderPass {
                pipeline: horizontal_blur_pipeline,
                bind_group: horizontal_blur_bind_group,
            },
            combine_pass: RenderPass {
                pipeline: combine_pipeline,
                bind_group: combine_bind_group,
            },
        }
    }

    pub fn resize(&mut self, device: &Device, texture_width: u32, texture_height: u32) {
        let (
            [texture1, texture2, texture3],
            [copy_glowing_bind_group, vertical_blur_bind_group, horizontal_blur_bind_group, combine_bind_group],
        ) = create_resources(
            device,
            texture_width,
            texture_height,
            &self.sampler,
            &self.bind_group_layout1,
            &self.bind_group_layout2,
            &self.bind_group_layout3,
        );
        self.texture1 = texture1;
        self.texture2 = texture2;
        self.texture3 = texture3;
        self.copy_glowing_pass.bind_group = copy_glowing_bind_group;
        self.vertical_blur_pass.bind_group = vertical_blur_bind_group;
        self.horizontal_blur_pass.bind_group = horizontal_blur_bind_group;
        self.combine_pass.bind_group = combine_bind_group;
    }

    pub fn render(&self, encoder: &mut CommandEncoder, render_texture: &TextureView) {
        {
            let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("glow_post_process_copy_glowing_render_pass"),
                color_attachments: &[RenderPassColorAttachmentDescriptor {
                    attachment: &self.texture2,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color::BLACK),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });
            pass.set_pipeline(&self.copy_glowing_pass.pipeline);
            pass.set_bind_group(0, &self.copy_glowing_pass.bind_group, &[]);
            pass.draw(0..6, 0..1);
        }
        {
            let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("glow_post_process_vertical_blur_render_pass"),
                color_attachments: &[RenderPassColorAttachmentDescriptor {
                    attachment: &self.texture3,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color::BLACK),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });
            pass.set_pipeline(&self.vertical_blur_pass.pipeline);
            pass.set_bind_group(0, &self.vertical_blur_pass.bind_group, &[]);
            pass.draw(0..6, 0..1);
        }
        {
            let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("glow_post_process_horizontal_blur_render_pass"),
                color_attachments: &[RenderPassColorAttachmentDescriptor {
                    attachment: &self.texture2,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color::BLACK),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });
            pass.set_pipeline(&self.horizontal_blur_pass.pipeline);
            pass.set_bind_group(0, &self.horizontal_blur_pass.bind_group, &[]);
            pass.draw(0..6, 0..1);
        }
        {
            let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("glow_post_process_combine_render_pass"),
                color_attachments: &[RenderPassColorAttachmentDescriptor {
                    attachment: render_texture,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color::BLACK),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });
            pass.set_pipeline(&self.combine_pass.pipeline);
            pass.set_bind_group(0, &self.combine_pass.bind_group, &[]);
            pass.draw(0..6, 0..1);
        }
    }
}

fn create_resources(
    device: &Device,
    texture_width: u32,
    texture_height: u32,
    sampler: &Sampler,
    bind_group_layout1: &BindGroupLayout,
    bind_group_layout2: &BindGroupLayout,
    bind_group_layout3: &BindGroupLayout,
) -> ([TextureView; 3], [BindGroup; 4]) {
    let mut texture_descriptor = TextureDescriptor {
        label: Some("glow_post_process_texture1"),
        size: Extent3d {
            width: texture_width,
            height: texture_height,
            depth: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: TextureFormat::Bgra8UnormSrgb,
        usage: TextureUsage::SAMPLED | TextureUsage::RENDER_ATTACHMENT,
    };
    let texture1 = device
        .create_texture(&texture_descriptor)
        .create_view(&TextureViewDescriptor::default());
    texture_descriptor.label = Some("glow_post_process_texture2");
    let texture2 = device
        .create_texture(&texture_descriptor)
        .create_view(&TextureViewDescriptor::default());
    texture_descriptor.label = Some("glow_post_process_texture3");
    let texture3 = device
        .create_texture(&texture_descriptor)
        .create_view(&TextureViewDescriptor::default());
    let texture_size_buffer = device.create_buffer_init(&BufferInitDescriptor {
        label: Some("glow_post_process_texture_size_buffer"),
        contents: bytemuck::cast_slice(&[texture_width as f32, texture_height as f32]),
        usage: BufferUsage::UNIFORM,
    });

    let copy_glowing_bind_group = device.create_bind_group(&BindGroupDescriptor {
        label: Some("glow_post_process_copy_glowing_bind_group"),
        layout: bind_group_layout1,
        entries: &[
            BindGroupEntry {
                binding: 0,
                resource: BindingResource::Sampler(sampler),
            },
            BindGroupEntry {
                binding: 1,
                resource: BindingResource::TextureView(&texture1),
            },
        ],
    });
    let vertical_blur_bind_group = device.create_bind_group(&BindGroupDescriptor {
        label: Some("glow_post_process_vertical_blur_bind_group"),
        layout: bind_group_layout2,
        entries: &[
            BindGroupEntry {
                binding: 0,
                resource: BindingResource::Sampler(sampler),
            },
            BindGroupEntry {
                binding: 1,
                resource: BindingResource::TextureView(&texture2),
            },
            BindGroupEntry {
                binding: 2,
                resource: BindingResource::Buffer {
                    buffer: &texture_size_buffer,
                    offset: 0,
                    size: None,
                },
            },
        ],
    });
    let horizontal_blur_bind_group = device.create_bind_group(&BindGroupDescriptor {
        label: Some("glow_post_process_horizontal_blur_bind_group"),
        layout: bind_group_layout2,
        entries: &[
            BindGroupEntry {
                binding: 0,
                resource: BindingResource::Sampler(sampler),
            },
            BindGroupEntry {
                binding: 1,
                resource: BindingResource::TextureView(&texture3),
            },
            BindGroupEntry {
                binding: 2,
                resource: BindingResource::Buffer {
                    buffer: &texture_size_buffer,
                    offset: 0,
                    size: None,
                },
            },
        ],
    });
    let combine_bind_group = device.create_bind_group(&BindGroupDescriptor {
        label: Some("glow_post_process_combine_bind_group"),
        layout: bind_group_layout3,
        entries: &[
            BindGroupEntry {
                binding: 0,
                resource: BindingResource::Sampler(sampler),
            },
            BindGroupEntry {
                binding: 1,
                resource: BindingResource::TextureView(&texture1),
            },
            BindGroupEntry {
                binding: 2,
                resource: BindingResource::TextureView(&texture2),
            },
        ],
    });

    (
        [texture1, texture2, texture3],
        [
            copy_glowing_bind_group,
            vertical_blur_bind_group,
            horizontal_blur_bind_group,
            combine_bind_group,
        ],
    )
}
