use pixels::include_spv;
use pixels::wgpu::{self, *};

pub struct GlowPostProcess {
    texture_width: u32,
    texture_height: u32,

    copy_to_texture1_pass: (RenderPipeline, BindGroup),
    copy_glowing_pass: (ComputePipeline, BindGroup),
    vertical_blur_pass: (ComputePipeline, BindGroup),
    horizontal_blur_pass: (ComputePipeline, BindGroup),
    add_glow_pass: (ComputePipeline, BindGroup),
    copy_to_render_texture_pass: (RenderPipeline, BindGroup),

    textures: [TextureView; 3],
    sampler: Sampler,
    render_bind_group_layout: BindGroupLayout,
    compute_bind_group_layout: BindGroupLayout,
}

impl GlowPostProcess {
    pub fn new(
        device: &Device,
        input_texture: &TextureView,
        texture_width: u32,
        texture_height: u32,
    ) -> Self {
        // Resources
        let [texture1, texture2, texture3] = create_textures(device, texture_width, texture_height);
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

        // Bind Groups
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
        let [copy_to_texture1_bind_group, copy_glowing_bind_group, vertical_blur_bind_group, horizontal_blur_bind_group, add_glow_bind_group, copy_to_render_texture_bind_group] =
            create_bind_groups(
                device,
                [input_texture, &texture1, &texture2, &texture3],
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
            texture_width,
            texture_height,

            copy_to_texture1_pass: (copy_to_texture1_pipeline, copy_to_texture1_bind_group),
            copy_glowing_pass: (copy_glowing_pipeline, copy_glowing_bind_group),
            vertical_blur_pass: (vertical_blur_pipeline, vertical_blur_bind_group),
            horizontal_blur_pass: (horizontal_blur_pipeline, horizontal_blur_bind_group),
            add_glow_pass: (add_glow_pipeline, add_glow_bind_group),
            copy_to_render_texture_pass: (
                copy_to_render_texture_pipeline,
                copy_to_render_texture_bind_group,
            ),

            textures: [texture1, texture2, texture3],
            sampler,
            render_bind_group_layout,
            compute_bind_group_layout,
        }
    }

    pub fn resize(
        &mut self,
        device: &Device,
        input_texture: &TextureView,
        texture_width: u32,
        texture_height: u32,
    ) {
        let [texture1, texture2, texture3] = create_textures(device, texture_width, texture_height);
        let [copy_to_texture1_bind_group, copy_glowing_bind_group, vertical_blur_bind_group, horizontal_blur_bind_group, add_glow_bind_group, copy_to_render_texture_bind_group] =
            create_bind_groups(
                device,
                [input_texture, &texture1, &texture2, &texture3],
                &self.sampler,
                &self.render_bind_group_layout,
                &self.compute_bind_group_layout,
            );

        self.texture_width = texture_width;
        self.texture_height = texture_height;

        self.textures = [texture1, texture2, texture3];
        self.copy_to_texture1_pass.1 = copy_to_texture1_bind_group;
        self.copy_glowing_pass.1 = copy_glowing_bind_group;
        self.vertical_blur_pass.1 = vertical_blur_bind_group;
        self.horizontal_blur_pass.1 = horizontal_blur_bind_group;
        self.add_glow_pass.1 = add_glow_bind_group;
        self.copy_to_render_texture_pass.1 = copy_to_render_texture_bind_group;
    }

    pub fn render(&self, encoder: &mut CommandEncoder, render_texture: &TextureView) {
        {
            let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
                color_attachments: &[RenderPassColorAttachmentDescriptor {
                    attachment: &self.textures[0],
                    resolve_target: None,
                    load_op: LoadOp::Clear,
                    store_op: StoreOp::Store,
                    clear_color: Color::BLACK,
                }],
                depth_stencil_attachment: None,
            });
            pass.set_pipeline(&self.copy_to_texture1_pass.0);
            pass.set_bind_group(0, &self.copy_to_texture1_pass.1, &[]);
            pass.draw(0..6, 0..1);
        }
        {
            let mut pass = encoder.begin_compute_pass();
            pass.set_pipeline(&self.copy_glowing_pass.0);
            pass.set_bind_group(0, &self.copy_glowing_pass.1, &[]);
            pass.dispatch(
                (self.texture_width as u32 / 7) + 8,
                (self.texture_height as u32 / 7) + 8,
                1,
            );
        }
        {
            let mut pass = encoder.begin_compute_pass();
            pass.set_pipeline(&self.vertical_blur_pass.0);
            pass.set_bind_group(0, &self.vertical_blur_pass.1, &[]);
            pass.dispatch(
                (self.texture_width as u32 / 7) + 8,
                (self.texture_height as u32 / 7) + 8,
                1,
            );
        }
        {
            let mut pass = encoder.begin_compute_pass();
            pass.set_pipeline(&self.horizontal_blur_pass.0);
            pass.set_bind_group(0, &self.horizontal_blur_pass.1, &[]);
            pass.dispatch(
                (self.texture_width as u32 / 7) + 8,
                (self.texture_height as u32 / 7) + 8,
                1,
            );
        }
        {
            let mut pass = encoder.begin_compute_pass();
            pass.set_pipeline(&self.add_glow_pass.0);
            pass.set_bind_group(0, &self.add_glow_pass.1, &[]);
            pass.dispatch(
                (self.texture_width as u32 / 7) + 8,
                (self.texture_height as u32 / 7) + 8,
                1,
            );
        }
        {
            let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
                color_attachments: &[RenderPassColorAttachmentDescriptor {
                    attachment: &render_texture,
                    resolve_target: None,
                    load_op: LoadOp::Clear,
                    store_op: StoreOp::Store,
                    clear_color: Color::BLACK,
                }],
                depth_stencil_attachment: None,
            });
            pass.set_pipeline(&self.copy_to_render_texture_pass.0);
            pass.set_bind_group(0, &self.copy_to_render_texture_pass.1, &[]);
            pass.draw(0..6, 0..1);
        }
    }
}

fn create_textures(device: &Device, texture_width: u32, texture_height: u32) -> [TextureView; 3] {
    let texture_descriptor_1 = TextureDescriptor {
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
        format: TextureFormat::Rgba16Float,
        usage: TextureUsage::STORAGE | TextureUsage::SAMPLED | TextureUsage::OUTPUT_ATTACHMENT,
    };
    let texture_descriptor_2_3 = TextureDescriptor {
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
        format: TextureFormat::Rgba16Float,
        usage: TextureUsage::STORAGE,
    };
    let texture1 = device
        .create_texture(&texture_descriptor_1)
        .create_default_view();
    let texture2 = device
        .create_texture(&texture_descriptor_2_3)
        .create_default_view();
    let texture3 = device
        .create_texture(&texture_descriptor_2_3)
        .create_default_view();
    [texture1, texture2, texture3]
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
