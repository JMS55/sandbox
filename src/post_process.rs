use pixels::wgpu::util::{BufferInitDescriptor, DeviceExt};
use pixels::wgpu::*;

pub struct PostProcess {
    pub texture1: TextureView,
    texture2: TextureView,
    texture3: TextureView,

    texture_width: u32,
    texture_height: u32,

    sampler: Sampler,
    bind_group_layout1: BindGroupLayout,
    bind_group_layout2: BindGroupLayout,
    bind_group_layout3: BindGroupLayout,

    copy_glowing_pass: RenderPass,
    vertical_blur_pass: RenderPass,
    horizontal_blur_pass: RenderPass,
    final_pass: RenderPass,
}

struct RenderPass {
    pipeline: RenderPipeline,
    bind_group: BindGroup,
}

impl PostProcess {
    pub fn new(device: &Device, texture_width: u32, texture_height: u32) -> Self {
        let sampler = device.create_sampler(&SamplerDescriptor {
            label: Some("post_process_sampler"),
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
            label: Some("post_process_bind_group_layout1"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
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
            label: Some("post_process_bind_group_layout2"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::FRAGMENT,
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
            label: Some("post_process_bind_group_layout3"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::FRAGMENT,
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
            [copy_glowing_bind_group, vertical_blur_bind_group, horizontal_blur_bind_group, final_pass_bind_group],
        ) = create_resources(
            device,
            texture_width,
            texture_height,
            &sampler,
            &bind_group_layout1,
            &bind_group_layout2,
            &bind_group_layout3,
        );

        let shader = device.create_shader_module(include_wgsl!("../post_process.wgsl"));

        let pipeline_layout1 = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("post_process_pipeline_layout1"),
            bind_group_layouts: &[&bind_group_layout1],
            push_constant_ranges: &[],
        });
        let pipeline_layout2 = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("post_process_pipeline_layout2"),
            bind_group_layouts: &[&bind_group_layout2],
            push_constant_ranges: &[],
        });
        let pipeline_layout3 = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("post_process_pipeline_layout3"),
            bind_group_layouts: &[&bind_group_layout3],
            push_constant_ranges: &[],
        });

        let create_pipeline = |entry_point, layout, label| {
            device.create_render_pipeline(&RenderPipelineDescriptor {
                label: Some(label),
                layout: Some(layout),
                vertex: VertexState {
                    module: &shader,
                    entry_point: "fullscreen_main",
                    buffers: &[],
                },
                primitive: PrimitiveState::default(),
                depth_stencil: None,
                multisample: MultisampleState::default(),
                fragment: Some(FragmentState {
                    module: &shader,
                    entry_point,
                    targets: &[Some(ColorTargetState {
                        format: TextureFormat::Bgra8UnormSrgb,
                        blend: Some(BlendState::REPLACE),
                        write_mask: ColorWrites::ALL,
                    })],
                }),
                multiview: None,
            })
        };
        let copy_glowing_pipeline = create_pipeline(
            "copy_glowing_main",
            &pipeline_layout1,
            "post_process_copy_glowing_pipeline",
        );
        let vertical_blur_pipeline = create_pipeline(
            "blur_vertical_main",
            &pipeline_layout2,
            "post_process_vertical_blur_pipeline",
        );
        let horizontal_blur_pipeline = create_pipeline(
            "blur_horizontal_main",
            &pipeline_layout2,
            "post_process_horizontal_blur_pipeline",
        );
        let final_pass_pipeline = create_pipeline(
            "final_pass_main",
            &pipeline_layout3,
            "post_process_final_pass_pipeline",
        );

        Self {
            texture1,
            texture2,
            texture3,

            texture_width,
            texture_height,

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
            final_pass: RenderPass {
                pipeline: final_pass_pipeline,
                bind_group: final_pass_bind_group,
            },
        }
    }

    pub fn resize(&mut self, device: &Device, texture_width: u32, texture_height: u32) {
        let (
            [texture1, texture2, texture3],
            [copy_glowing_bind_group, vertical_blur_bind_group, horizontal_blur_bind_group, final_pass_bind_group],
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
        self.texture_width = texture_width;
        self.texture_height = texture_height;
        self.copy_glowing_pass.bind_group = copy_glowing_bind_group;
        self.vertical_blur_pass.bind_group = vertical_blur_bind_group;
        self.horizontal_blur_pass.bind_group = horizontal_blur_bind_group;
        self.final_pass.bind_group = final_pass_bind_group;
    }

    pub fn render(&self, encoder: &mut CommandEncoder, render_texture: &TextureView) {
        let mut create_pass = |pass_desc: &RenderPass, texture, label| {
            let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some(label),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: texture,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color::BLACK),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });
            pass.set_pipeline(&pass_desc.pipeline);
            pass.set_bind_group(0, &pass_desc.bind_group, &[]);
            pass.set_scissor_rect(0, 0, self.texture_width, self.texture_height);
            pass.draw(0..3, 0..1);
        };

        create_pass(
            &self.copy_glowing_pass,
            &self.texture2,
            "post_process_copy_glowing_render_pass",
        );
        create_pass(
            &self.vertical_blur_pass,
            &self.texture3,
            "post_process_vertical_blur_render_pass",
        );
        create_pass(
            &self.horizontal_blur_pass,
            &self.texture2,
            "post_process_horizontal_blur_render_pass",
        );
        create_pass(
            &self.final_pass,
            &render_texture,
            "post_process_final_pass_render_pass",
        );
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
        label: Some("post_process_texture1"),
        size: Extent3d {
            width: texture_width,
            height: texture_height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: TextureFormat::Bgra8UnormSrgb,
        usage: TextureUsages::TEXTURE_BINDING | TextureUsages::RENDER_ATTACHMENT,
    };
    let texture1 = device
        .create_texture(&texture_descriptor)
        .create_view(&TextureViewDescriptor::default());
    texture_descriptor.label = Some("post_process_texture2");
    let texture2 = device
        .create_texture(&texture_descriptor)
        .create_view(&TextureViewDescriptor::default());
    texture_descriptor.label = Some("post_process_texture3");
    let texture3 = device
        .create_texture(&texture_descriptor)
        .create_view(&TextureViewDescriptor::default());
    let texture_size_buffer = device.create_buffer_init(&BufferInitDescriptor {
        label: Some("post_process_texture_size_buffer"),
        contents: bytemuck::cast_slice(&[texture_width as f32, texture_height as f32]),
        usage: BufferUsages::UNIFORM,
    });

    let copy_glowing_bind_group = device.create_bind_group(&BindGroupDescriptor {
        label: Some("post_process_copy_glowing_bind_group"),
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
        label: Some("post_process_vertical_blur_bind_group"),
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
                resource: texture_size_buffer.as_entire_binding(),
            },
        ],
    });
    let horizontal_blur_bind_group = device.create_bind_group(&BindGroupDescriptor {
        label: Some("post_process_horizontal_blur_bind_group"),
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
                resource: texture_size_buffer.as_entire_binding(),
            },
        ],
    });
    let final_pass_bind_group = device.create_bind_group(&BindGroupDescriptor {
        label: Some("post_process_final_pass_bind_group"),
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
            final_pass_bind_group,
        ],
    )
}
