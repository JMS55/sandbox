use pixels::include_spv;
use pixels::wgpu::{self, *};

pub struct Render {
    screen_width: u32,
    screen_height: u32,

    glow_texture1: TextureView,
    glow_texture2: TextureView,
    glow_texture3: TextureView,
    pub screen_sized_texture: Texture,
    pub screen_sized_texture_view: TextureView,
    sampler: Sampler,

    render_bind_group_layout: BindGroupLayout,
    compute_bind_group_layout: BindGroupLayout,

    copy_to_texture1_bind_group: BindGroup,
    copy_glowing_bind_group: BindGroup,
    vertical_blur_bind_group: BindGroup,
    horizontal_blur_bind_group: BindGroup,
    add_glow_bind_group: BindGroup,
    copy_to_render_texture_bind_group: BindGroup,

    copy_to_texture1_pipeline: RenderPipeline,
    copy_glowing_pipeline: ComputePipeline,
    vertical_blur_pipeline: ComputePipeline,
    horizontal_blur_pipeline: ComputePipeline,
    add_glow_pipeline: ComputePipeline,
    copy_to_render_texture_pipeline: RenderPipeline,
}

impl Render {
    pub fn new(device: &Device, screen_width: u32, screen_height: u32) -> Self {
        // Resources
        let (
            [glow_texture1, glow_texture2, glow_texture3],
            screen_sized_texture,
            screen_sized_texture_view,
        ) = create_textures(device, screen_width, screen_height);
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

        // Bind Group Layouts
        let render_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                bindings: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStage::FRAGMENT,
                        ty: BindingType::SampledTexture {
                            dimension: TextureViewDimension::D2,
                            component_type: TextureComponentType::Float,
                            multisampled: false,
                        },
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStage::FRAGMENT,
                        ty: BindingType::Sampler { comparison: false },
                    },
                ],
                label: None,
            });
        let compute_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                bindings: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStage::COMPUTE,
                        ty: BindingType::StorageTexture {
                            dimension: TextureViewDimension::D2,
                            component_type: TextureComponentType::Float,
                            format: TextureFormat::Rgba16Float,
                            readonly: true,
                        },
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStage::COMPUTE,
                        ty: BindingType::StorageTexture {
                            dimension: TextureViewDimension::D2,
                            component_type: TextureComponentType::Float,
                            format: TextureFormat::Rgba16Float,
                            readonly: false,
                        },
                    },
                ],
                label: None,
            });

        // Bind Groups
        let [copy_to_texture1_bind_group, copy_glowing_bind_group, vertical_blur_bind_group, horizontal_blur_bind_group, add_glow_bind_group, copy_to_render_texture_bind_group] =
            create_bind_groups(
                device,
                [
                    &screen_sized_texture_view,
                    &glow_texture1,
                    &glow_texture2,
                    &glow_texture3,
                ],
                &sampler,
                &render_bind_group_layout,
                &compute_bind_group_layout,
            );

        // Shaders
        let rectangle_shader =
            device.create_shader_module(include_spv!("../shaders/rectangle.spv"));
        let copy_texture_shader =
            device.create_shader_module(include_spv!("../shaders/copy_texture.spv"));
        let copy_glowing_shader =
            device.create_shader_module(include_spv!("../shaders/copy_glowing.spv"));
        let vertical_blur_shader =
            device.create_shader_module(include_spv!("../shaders/vertical_blur.spv"));
        let horizontal_blur_shader =
            device.create_shader_module(include_spv!("../shaders/horizontal_blur.spv"));
        let add_glow_shader = device.create_shader_module(include_spv!("../shaders/add_glow.spv"));

        // Pipelines
        let pipeline_layout_render = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            bind_group_layouts: &[&render_bind_group_layout],
        });
        let pipeline_layout_compute = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            bind_group_layouts: &[&compute_bind_group_layout],
        });
        let copy_to_texture1_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            layout: &pipeline_layout_render,
            vertex_stage: ProgrammableStageDescriptor {
                module: &rectangle_shader,
                entry_point: "main",
            },
            fragment_stage: Some(ProgrammableStageDescriptor {
                module: &copy_texture_shader,
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
                format: TextureFormat::Rgba16Float,
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
        let copy_glowing_pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            layout: &pipeline_layout_compute,
            compute_stage: ProgrammableStageDescriptor {
                module: &copy_glowing_shader,
                entry_point: "main",
            },
        });
        let vertical_blur_pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            layout: &pipeline_layout_compute,
            compute_stage: ProgrammableStageDescriptor {
                module: &vertical_blur_shader,
                entry_point: "main",
            },
        });
        let horizontal_blur_pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            layout: &pipeline_layout_compute,
            compute_stage: ProgrammableStageDescriptor {
                module: &horizontal_blur_shader,
                entry_point: "main",
            },
        });
        let add_glow_pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            layout: &pipeline_layout_compute,
            compute_stage: ProgrammableStageDescriptor {
                module: &add_glow_shader,
                entry_point: "main",
            },
        });
        let copy_to_render_texture_pipeline =
            device.create_render_pipeline(&RenderPipelineDescriptor {
                layout: &pipeline_layout_render,
                vertex_stage: ProgrammableStageDescriptor {
                    module: &rectangle_shader,
                    entry_point: "main",
                },
                fragment_stage: Some(ProgrammableStageDescriptor {
                    module: &copy_texture_shader,
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
            screen_width,
            screen_height,

            glow_texture1,
            glow_texture2,
            glow_texture3,
            screen_sized_texture,
            screen_sized_texture_view,
            sampler,

            render_bind_group_layout,
            compute_bind_group_layout,

            copy_to_texture1_bind_group,
            copy_glowing_bind_group,
            vertical_blur_bind_group,
            horizontal_blur_bind_group,
            add_glow_bind_group,
            copy_to_render_texture_bind_group,

            copy_to_texture1_pipeline,
            copy_glowing_pipeline,
            vertical_blur_pipeline,
            horizontal_blur_pipeline,
            add_glow_pipeline,
            copy_to_render_texture_pipeline,
        }
    }

    pub fn resize(&mut self, device: &Device, screen_width: u32, screen_height: u32) {
        let (
            [glow_texture1, glow_texture2, glow_texture3],
            screen_sized_texture,
            screen_sized_texture_view,
        ) = create_textures(device, screen_width, screen_height);

        let [copy_to_texture1_bind_group, copy_glowing_bind_group, vertical_blur_bind_group, horizontal_blur_bind_group, add_glow_bind_group, copy_to_render_texture_bind_group] =
            create_bind_groups(
                device,
                [
                    &screen_sized_texture_view,
                    &glow_texture1,
                    &glow_texture2,
                    &glow_texture3,
                ],
                &self.sampler,
                &self.render_bind_group_layout,
                &self.compute_bind_group_layout,
            );

        self.screen_width = screen_width;
        self.screen_height = screen_height;

        self.glow_texture1 = glow_texture1;
        self.glow_texture2 = glow_texture2;
        self.glow_texture3 = glow_texture3;
        self.screen_sized_texture = screen_sized_texture;
        self.screen_sized_texture_view = screen_sized_texture_view;

        self.copy_to_texture1_bind_group = copy_to_texture1_bind_group;
        self.copy_glowing_bind_group = copy_glowing_bind_group;
        self.vertical_blur_bind_group = vertical_blur_bind_group;
        self.horizontal_blur_bind_group = horizontal_blur_bind_group;
        self.add_glow_bind_group = add_glow_bind_group;
        self.copy_to_render_texture_bind_group = copy_to_render_texture_bind_group;
    }

    pub fn glow_post_process(&self, encoder: &mut CommandEncoder) {
        {
            let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
                color_attachments: &[RenderPassColorAttachmentDescriptor {
                    attachment: &self.glow_texture1,
                    resolve_target: None,
                    load_op: LoadOp::Clear,
                    store_op: StoreOp::Store,
                    clear_color: Color::BLACK,
                }],
                depth_stencil_attachment: None,
            });
            pass.set_pipeline(&self.copy_to_texture1_pipeline);
            pass.set_bind_group(0, &self.copy_to_texture1_bind_group, &[]);
            pass.draw(0..6, 0..1);
        }
        {
            let mut pass = encoder.begin_compute_pass();
            pass.set_pipeline(&self.copy_glowing_pipeline);
            pass.set_bind_group(0, &self.copy_glowing_bind_group, &[]);
            pass.dispatch(
                (self.screen_width as u32 / 7) + 8,
                (self.screen_height as u32 / 7) + 8,
                1,
            );
        }
        {
            let mut pass = encoder.begin_compute_pass();
            pass.set_pipeline(&self.vertical_blur_pipeline);
            pass.set_bind_group(0, &self.vertical_blur_bind_group, &[]);
            pass.dispatch(
                (self.screen_width as u32 / 7) + 8,
                (self.screen_height as u32 / 7) + 8,
                1,
            );
        }
        {
            let mut pass = encoder.begin_compute_pass();
            pass.set_pipeline(&self.horizontal_blur_pipeline);
            pass.set_bind_group(0, &self.horizontal_blur_bind_group, &[]);
            pass.dispatch(
                (self.screen_width as u32 / 7) + 8,
                (self.screen_height as u32 / 7) + 8,
                1,
            );
        }
        {
            let mut pass = encoder.begin_compute_pass();
            pass.set_pipeline(&self.add_glow_pipeline);
            pass.set_bind_group(0, &self.add_glow_bind_group, &[]);
            pass.dispatch(
                (self.screen_width as u32 / 7) + 8,
                (self.screen_height as u32 / 7) + 8,
                1,
            );
        }
        {
            let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
                color_attachments: &[RenderPassColorAttachmentDescriptor {
                    attachment: &self.screen_sized_texture_view,
                    resolve_target: None,
                    load_op: LoadOp::Clear,
                    store_op: StoreOp::Store,
                    clear_color: Color::BLACK,
                }],
                depth_stencil_attachment: None,
            });
            pass.set_pipeline(&self.copy_to_render_texture_pipeline);
            pass.set_bind_group(0, &self.copy_to_render_texture_bind_group, &[]);
            pass.draw(0..6, 0..1);
        }
    }

    pub fn copy_screen_texture_to_swapchain(
        &self,
        encoder: &mut CommandEncoder,
        swapchain_texture: &TextureView,
    ) {
        let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
            color_attachments: &[RenderPassColorAttachmentDescriptor {
                attachment: swapchain_texture,
                resolve_target: None,
                load_op: LoadOp::Clear,
                store_op: StoreOp::Store,
                clear_color: Color::BLACK,
            }],
            depth_stencil_attachment: None,
        });
        pass.set_pipeline(&self.copy_to_render_texture_pipeline);
        pass.set_bind_group(0, &self.copy_to_render_texture_bind_group, &[]);
        pass.draw(0..6, 0..1);
    }
}

fn create_textures(
    device: &Device,
    screen_width: u32,
    screen_height: u32,
) -> ([TextureView; 3], Texture, TextureView) {
    let glow_texture_descriptor_1 = TextureDescriptor {
        label: None,
        size: Extent3d {
            width: screen_width,
            height: screen_height,
            depth: 1,
        },
        array_layer_count: 1,
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: TextureFormat::Rgba16Float,
        usage: TextureUsage::STORAGE | TextureUsage::SAMPLED | TextureUsage::OUTPUT_ATTACHMENT,
    };
    let glow_texture_descriptor_2_3 = TextureDescriptor {
        label: None,
        size: Extent3d {
            width: screen_width,
            height: screen_height,
            depth: 1,
        },
        array_layer_count: 1,
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: TextureFormat::Rgba16Float,
        usage: TextureUsage::STORAGE,
    };
    let screen_sized_texture_descriptor = TextureDescriptor {
        label: None,
        size: Extent3d {
            width: screen_width,
            height: screen_height,
            depth: 1,
        },
        array_layer_count: 1,
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: TextureFormat::Bgra8UnormSrgb,
        usage: TextureUsage::SAMPLED | TextureUsage::OUTPUT_ATTACHMENT | TextureUsage::COPY_SRC,
    };

    let glow_texture1 = device
        .create_texture(&glow_texture_descriptor_1)
        .create_default_view();
    let glow_texture2 = device
        .create_texture(&glow_texture_descriptor_2_3)
        .create_default_view();
    let glow_texture3 = device
        .create_texture(&glow_texture_descriptor_2_3)
        .create_default_view();
    let screen_sized_texture = device.create_texture(&screen_sized_texture_descriptor);
    let screen_sized_texture_view = screen_sized_texture.create_default_view();

    (
        [glow_texture1, glow_texture2, glow_texture3],
        screen_sized_texture,
        screen_sized_texture_view,
    )
}

fn create_bind_groups(
    device: &Device,
    textures: [&TextureView; 4],
    sampler: &Sampler,
    render_bind_group_layout: &BindGroupLayout,
    compute_bind_group_layout: &BindGroupLayout,
) -> [BindGroup; 6] {
    let copy_to_texture1_bind_group = device.create_bind_group(&BindGroupDescriptor {
        layout: &render_bind_group_layout,
        bindings: &[
            Binding {
                binding: 0,
                resource: BindingResource::TextureView(textures[0]),
            },
            Binding {
                binding: 1,
                resource: BindingResource::Sampler(sampler),
            },
        ],
        label: None,
    });
    let copy_glowing_bind_group = device.create_bind_group(&BindGroupDescriptor {
        layout: &compute_bind_group_layout,
        bindings: &[
            Binding {
                binding: 0,
                resource: BindingResource::TextureView(textures[1]),
            },
            Binding {
                binding: 1,
                resource: BindingResource::TextureView(textures[2]),
            },
        ],
        label: None,
    });
    let vertical_blur_bind_group = device.create_bind_group(&BindGroupDescriptor {
        layout: &compute_bind_group_layout,
        bindings: &[
            Binding {
                binding: 0,
                resource: BindingResource::TextureView(textures[2]),
            },
            Binding {
                binding: 1,
                resource: BindingResource::TextureView(textures[3]),
            },
        ],
        label: None,
    });
    let horizontal_blur_bind_group = device.create_bind_group(&BindGroupDescriptor {
        layout: &compute_bind_group_layout,
        bindings: &[
            Binding {
                binding: 0,
                resource: BindingResource::TextureView(textures[3]),
            },
            Binding {
                binding: 1,
                resource: BindingResource::TextureView(textures[2]),
            },
        ],
        label: None,
    });
    let add_glow_bind_group = device.create_bind_group(&BindGroupDescriptor {
        layout: &compute_bind_group_layout,
        bindings: &[
            Binding {
                binding: 0,
                resource: BindingResource::TextureView(textures[2]),
            },
            Binding {
                binding: 1,
                resource: BindingResource::TextureView(textures[1]),
            },
        ],
        label: None,
    });
    let copy_to_render_texture_bind_group = device.create_bind_group(&BindGroupDescriptor {
        layout: &render_bind_group_layout,
        bindings: &[
            Binding {
                binding: 0,
                resource: BindingResource::TextureView(textures[1]),
            },
            Binding {
                binding: 1,
                resource: BindingResource::Sampler(sampler),
            },
        ],
        label: None,
    });
    [
        copy_to_texture1_bind_group,
        copy_glowing_bind_group,
        vertical_blur_bind_group,
        horizontal_blur_bind_group,
        add_glow_bind_group,
        copy_to_render_texture_bind_group,
    ]
}
