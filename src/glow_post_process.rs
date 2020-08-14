use pixels::include_spv;
use pixels::wgpu::{self, *};

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
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Nearest,
            min_filter: FilterMode::Nearest,
            mipmap_filter: FilterMode::Nearest,
            lod_min_clamp: 0.0,
            lod_max_clamp: 100.0,
            compare: CompareFunction::Never,
        });
        let bind_group_layout1 = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            bindings: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStage::FRAGMENT,
                    ty: BindingType::Sampler { comparison: false },
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStage::FRAGMENT,
                    ty: BindingType::SampledTexture {
                        dimension: TextureViewDimension::D2,
                        component_type: TextureComponentType::Float,
                        multisampled: false,
                    },
                },
            ],
            label: None,
        });
        let bind_group_layout2 = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            bindings: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStage::FRAGMENT,
                    ty: BindingType::Sampler { comparison: false },
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStage::FRAGMENT,
                    ty: BindingType::SampledTexture {
                        dimension: TextureViewDimension::D2,
                        component_type: TextureComponentType::Float,
                        multisampled: false,
                    },
                },
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStage::FRAGMENT,
                    ty: BindingType::UniformBuffer { dynamic: false },
                },
            ],
            label: None,
        });
        let bind_group_layout3 = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            bindings: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStage::FRAGMENT,
                    ty: BindingType::Sampler { comparison: false },
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStage::FRAGMENT,
                    ty: BindingType::SampledTexture {
                        dimension: TextureViewDimension::D2,
                        component_type: TextureComponentType::Float,
                        multisampled: false,
                    },
                },
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStage::FRAGMENT,
                    ty: BindingType::SampledTexture {
                        dimension: TextureViewDimension::D2,
                        component_type: TextureComponentType::Float,
                        multisampled: false,
                    },
                },
            ],
            label: None,
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

        let rectangle_shader =
            device.create_shader_module(include_spv!("../shaders/rectangle.spv"));
        let copy_glowing_shader =
            device.create_shader_module(include_spv!("../shaders/copy_glowing.spv"));
        let vertical_blur_shader =
            device.create_shader_module(include_spv!("../shaders/vertical_blur.spv"));
        let horizontal_blur_shader =
            device.create_shader_module(include_spv!("../shaders/horizontal_blur.spv"));
        let combine_shader = device.create_shader_module(include_spv!("../shaders/combine.spv"));

        let pipeline_layout1 = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            bind_group_layouts: &[&bind_group_layout1],
        });
        let pipeline_layout2 = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            bind_group_layouts: &[&bind_group_layout2],
        });
        let pipeline_layout3 = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            bind_group_layouts: &[&bind_group_layout3],
        });
        let copy_glowing_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            layout: &pipeline_layout1,
            vertex_stage: ProgrammableStageDescriptor {
                module: &rectangle_shader,
                entry_point: "main",
            },
            fragment_stage: Some(ProgrammableStageDescriptor {
                module: &copy_glowing_shader,
                entry_point: "main",
            }),
            rasterization_state: Some(RasterizationStateDescriptor {
                front_face: FrontFace::Ccw,
                cull_mode: CullMode::None,
                depth_bias: 0,
                depth_bias_slope_scale: 0.0,
                depth_bias_clamp: 0.0,
            }),
            primitive_topology: PrimitiveTopology::TriangleList,
            color_states: &[ColorStateDescriptor {
                format: TextureFormat::Bgra8UnormSrgb,
                color_blend: BlendDescriptor::REPLACE,
                alpha_blend: BlendDescriptor::REPLACE,
                write_mask: ColorWrite::ALL,
            }],
            depth_stencil_state: None,
            vertex_state: VertexStateDescriptor {
                index_format: IndexFormat::Uint16,
                vertex_buffers: &[],
            },
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        });
        let vertical_blur_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            layout: &pipeline_layout2,
            vertex_stage: ProgrammableStageDescriptor {
                module: &rectangle_shader,
                entry_point: "main",
            },
            fragment_stage: Some(ProgrammableStageDescriptor {
                module: &vertical_blur_shader,
                entry_point: "main",
            }),
            rasterization_state: Some(RasterizationStateDescriptor {
                front_face: FrontFace::Ccw,
                cull_mode: CullMode::None,
                depth_bias: 0,
                depth_bias_slope_scale: 0.0,
                depth_bias_clamp: 0.0,
            }),
            primitive_topology: PrimitiveTopology::TriangleList,
            color_states: &[ColorStateDescriptor {
                format: TextureFormat::Bgra8UnormSrgb,
                color_blend: BlendDescriptor::REPLACE,
                alpha_blend: BlendDescriptor::REPLACE,
                write_mask: ColorWrite::ALL,
            }],
            depth_stencil_state: None,
            vertex_state: VertexStateDescriptor {
                index_format: IndexFormat::Uint16,
                vertex_buffers: &[],
            },
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        });
        let horizontal_blur_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            layout: &pipeline_layout2,
            vertex_stage: ProgrammableStageDescriptor {
                module: &rectangle_shader,
                entry_point: "main",
            },
            fragment_stage: Some(ProgrammableStageDescriptor {
                module: &horizontal_blur_shader,
                entry_point: "main",
            }),
            rasterization_state: Some(RasterizationStateDescriptor {
                front_face: FrontFace::Ccw,
                cull_mode: CullMode::None,
                depth_bias: 0,
                depth_bias_slope_scale: 0.0,
                depth_bias_clamp: 0.0,
            }),
            primitive_topology: PrimitiveTopology::TriangleList,
            color_states: &[ColorStateDescriptor {
                format: TextureFormat::Bgra8UnormSrgb,
                color_blend: BlendDescriptor::REPLACE,
                alpha_blend: BlendDescriptor::REPLACE,
                write_mask: ColorWrite::ALL,
            }],
            depth_stencil_state: None,
            vertex_state: VertexStateDescriptor {
                index_format: IndexFormat::Uint16,
                vertex_buffers: &[],
            },
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        });
        let combine_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            layout: &pipeline_layout3,
            vertex_stage: ProgrammableStageDescriptor {
                module: &rectangle_shader,
                entry_point: "main",
            },
            fragment_stage: Some(ProgrammableStageDescriptor {
                module: &combine_shader,
                entry_point: "main",
            }),
            rasterization_state: Some(RasterizationStateDescriptor {
                front_face: FrontFace::Ccw,
                cull_mode: CullMode::None,
                depth_bias: 0,
                depth_bias_slope_scale: 0.0,
                depth_bias_clamp: 0.0,
            }),
            primitive_topology: PrimitiveTopology::TriangleList,
            color_states: &[ColorStateDescriptor {
                format: TextureFormat::Bgra8UnormSrgb,
                color_blend: BlendDescriptor::REPLACE,
                alpha_blend: BlendDescriptor::REPLACE,
                write_mask: ColorWrite::ALL,
            }],
            depth_stencil_state: None,
            vertex_state: VertexStateDescriptor {
                index_format: IndexFormat::Uint16,
                vertex_buffers: &[],
            },
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
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
                color_attachments: &[RenderPassColorAttachmentDescriptor {
                    attachment: &self.texture2,
                    resolve_target: None,
                    load_op: LoadOp::Clear,
                    store_op: StoreOp::Store,
                    clear_color: Color::BLACK,
                }],
                depth_stencil_attachment: None,
            });
            pass.set_pipeline(&self.copy_glowing_pass.pipeline);
            pass.set_bind_group(0, &self.copy_glowing_pass.bind_group, &[]);
            pass.draw(0..6, 0..1);
        }
        {
            let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
                color_attachments: &[RenderPassColorAttachmentDescriptor {
                    attachment: &self.texture3,
                    resolve_target: None,
                    load_op: LoadOp::Clear,
                    store_op: StoreOp::Store,
                    clear_color: Color::BLACK,
                }],
                depth_stencil_attachment: None,
            });
            pass.set_pipeline(&self.vertical_blur_pass.pipeline);
            pass.set_bind_group(0, &self.vertical_blur_pass.bind_group, &[]);
            pass.draw(0..6, 0..1);
        }
        {
            let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
                color_attachments: &[RenderPassColorAttachmentDescriptor {
                    attachment: &self.texture2,
                    resolve_target: None,
                    load_op: LoadOp::Clear,
                    store_op: StoreOp::Store,
                    clear_color: Color::BLACK,
                }],
                depth_stencil_attachment: None,
            });
            pass.set_pipeline(&self.horizontal_blur_pass.pipeline);
            pass.set_bind_group(0, &self.horizontal_blur_pass.bind_group, &[]);
            pass.draw(0..6, 0..1);
        }
        {
            let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
                color_attachments: &[RenderPassColorAttachmentDescriptor {
                    attachment: render_texture,
                    resolve_target: None,
                    load_op: LoadOp::Clear,
                    store_op: StoreOp::Store,
                    clear_color: Color::BLACK,
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
    let texture_descriptor = TextureDescriptor {
        label: None,
        size: Extent3d {
            width: texture_width,
            height: texture_height,
            depth: 1,
        },
        array_layer_count: 1,
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: TextureFormat::Bgra8UnormSrgb,
        usage: TextureUsage::SAMPLED | TextureUsage::OUTPUT_ATTACHMENT,
    };
    let texture1 = device
        .create_texture(&texture_descriptor)
        .create_default_view();
    let texture2 = device
        .create_texture(&texture_descriptor)
        .create_default_view();
    let texture3 = device
        .create_texture(&texture_descriptor)
        .create_default_view();
    let first_half = texture_width.to_ne_bytes();
    let second_half = texture_height.to_ne_bytes();
    let texture_size_buffer = device.create_buffer_with_data(
        &[
            first_half[0],
            first_half[1],
            first_half[2],
            first_half[3],
            second_half[0],
            second_half[1],
            second_half[2],
            second_half[3],
        ],
        wgpu::BufferUsage::UNIFORM,
    );

    let copy_glowing_bind_group = device.create_bind_group(&BindGroupDescriptor {
        layout: bind_group_layout1,
        bindings: &[
            Binding {
                binding: 0,
                resource: BindingResource::Sampler(sampler),
            },
            Binding {
                binding: 1,
                resource: BindingResource::TextureView(&texture1),
            },
        ],
        label: None,
    });
    let vertical_blur_bind_group = device.create_bind_group(&BindGroupDescriptor {
        layout: bind_group_layout2,
        bindings: &[
            Binding {
                binding: 0,
                resource: BindingResource::Sampler(sampler),
            },
            Binding {
                binding: 1,
                resource: BindingResource::TextureView(&texture2),
            },
            Binding {
                binding: 2,
                resource: BindingResource::Buffer {
                    buffer: &texture_size_buffer,
                    range: 0..8,
                },
            },
        ],
        label: None,
    });
    let horizontal_blur_bind_group = device.create_bind_group(&BindGroupDescriptor {
        layout: bind_group_layout2,
        bindings: &[
            Binding {
                binding: 0,
                resource: BindingResource::Sampler(sampler),
            },
            Binding {
                binding: 1,
                resource: BindingResource::TextureView(&texture3),
            },
            Binding {
                binding: 2,
                resource: BindingResource::Buffer {
                    buffer: &texture_size_buffer,
                    range: 0..8,
                },
            },
        ],
        label: None,
    });
    let combine_bind_group = device.create_bind_group(&BindGroupDescriptor {
        layout: bind_group_layout3,
        bindings: &[
            Binding {
                binding: 0,
                resource: BindingResource::Sampler(sampler),
            },
            Binding {
                binding: 1,
                resource: BindingResource::TextureView(&texture1),
            },
            Binding {
                binding: 2,
                resource: BindingResource::TextureView(&texture2),
            },
        ],
        label: None,
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
