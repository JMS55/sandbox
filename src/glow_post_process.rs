use crate::sandbox::{SANDBOX_HEIGHT, SANDBOX_WIDTH};
use pixels::wgpu::{self, *};
use pixels::{include_spv, BoxedRenderPass, Device, Queue, RenderPass};

pub struct GlowPostProcess {
    texture1: TextureView,
    copy_to_texture1_pass: (RenderPipeline, BindGroup),
    copy_glowing_pass: (ComputePipeline, BindGroup),
    vertical_blur_pass: (ComputePipeline, BindGroup),
    horizontal_blur_pass: (ComputePipeline, BindGroup),
    add_glow_pass: (ComputePipeline, BindGroup),
    copy_to_render_texture_pass: (RenderPipeline, BindGroup),
}

impl GlowPostProcess {
    pub fn new(
        device: Device,
        _: Queue,
        input_texture: &TextureView,
        texture_size: &Extent3d,
    ) -> BoxedRenderPass {
        // Resources
        let texture_descriptor_1 = TextureDescriptor {
            label: None,
            size: *texture_size,
            array_layer_count: 1,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8Unorm,
            usage: TextureUsage::STORAGE | TextureUsage::SAMPLED | TextureUsage::OUTPUT_ATTACHMENT,
        };
        let texture_descriptor_2_3 = TextureDescriptor {
            label: None,
            size: *texture_size,
            array_layer_count: 1,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8Unorm,
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

        // Bind groups
        let bind_group_layout_render =
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
        let bind_group_layout_compute =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                bindings: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStage::COMPUTE,
                        ty: BindingType::StorageTexture {
                            dimension: TextureViewDimension::D2,
                            component_type: TextureComponentType::Float,
                            format: TextureFormat::Rgba8Unorm,
                            readonly: true,
                        },
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStage::COMPUTE,
                        ty: BindingType::StorageTexture {
                            dimension: TextureViewDimension::D2,
                            component_type: TextureComponentType::Float,
                            format: TextureFormat::Rgba8Unorm,
                            readonly: false,
                        },
                    },
                ],
                label: None,
            });

        let bind_group_copy_to_texture1 = device.create_bind_group(&BindGroupDescriptor {
            layout: &bind_group_layout_render,
            bindings: &[
                Binding {
                    binding: 0,
                    resource: BindingResource::TextureView(&input_texture),
                },
                Binding {
                    binding: 1,
                    resource: BindingResource::Sampler(&sampler),
                },
            ],
            label: None,
        });
        let bind_group_copy_glowing = device.create_bind_group(&BindGroupDescriptor {
            layout: &bind_group_layout_compute,
            bindings: &[
                Binding {
                    binding: 0,
                    resource: BindingResource::TextureView(&texture1),
                },
                Binding {
                    binding: 1,
                    resource: BindingResource::TextureView(&texture2),
                },
            ],
            label: None,
        });
        let bind_group_vertical_blur = device.create_bind_group(&BindGroupDescriptor {
            layout: &bind_group_layout_compute,
            bindings: &[
                Binding {
                    binding: 0,
                    resource: BindingResource::TextureView(&texture2),
                },
                Binding {
                    binding: 1,
                    resource: BindingResource::TextureView(&texture3),
                },
            ],
            label: None,
        });
        let bind_group_horizontal_blur = device.create_bind_group(&BindGroupDescriptor {
            layout: &bind_group_layout_compute,
            bindings: &[
                Binding {
                    binding: 0,
                    resource: BindingResource::TextureView(&texture3),
                },
                Binding {
                    binding: 1,
                    resource: BindingResource::TextureView(&texture2),
                },
            ],
            label: None,
        });
        let bind_group_add_glow = device.create_bind_group(&BindGroupDescriptor {
            layout: &bind_group_layout_compute,
            bindings: &[
                Binding {
                    binding: 0,
                    resource: BindingResource::TextureView(&texture2),
                },
                Binding {
                    binding: 1,
                    resource: BindingResource::TextureView(&texture1),
                },
            ],
            label: None,
        });
        let bind_group_copy_to_render_texture = device.create_bind_group(&BindGroupDescriptor {
            layout: &bind_group_layout_render,
            bindings: &[
                Binding {
                    binding: 0,
                    resource: BindingResource::TextureView(&texture1),
                },
                Binding {
                    binding: 1,
                    resource: BindingResource::Sampler(&sampler),
                },
            ],
            label: None,
        });

        // Shaders
        let shader_rectangle =
            device.create_shader_module(include_spv!("../shaders/rectangle.spv"));
        let shader_copy_texture =
            device.create_shader_module(include_spv!("../shaders/copy_texture.spv"));
        let shader_copy_glowing =
            device.create_shader_module(include_spv!("../shaders/copy_glowing.spv"));
        let shader_vertical_blur =
            device.create_shader_module(include_spv!("../shaders/vertical_blur.spv"));
        let shader_horizontal_blur =
            device.create_shader_module(include_spv!("../shaders/horizontal_blur.spv"));
        let shader_add_glow = device.create_shader_module(include_spv!("../shaders/add_glow.spv"));

        // Pipelines
        let pipeline_layout_render = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            bind_group_layouts: &[&bind_group_layout_render],
        });
        let pipeline_layout_compute = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            bind_group_layouts: &[&bind_group_layout_compute],
        });

        let pipeline_copy_to_texture1 = device.create_render_pipeline(&RenderPipelineDescriptor {
            layout: &pipeline_layout_render,
            vertex_stage: ProgrammableStageDescriptor {
                module: &shader_rectangle,
                entry_point: "main",
            },
            fragment_stage: Some(ProgrammableStageDescriptor {
                module: &shader_copy_texture,
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
                format: TextureFormat::Rgba8Unorm,
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
        let pipeline_copy_glowing = device.create_compute_pipeline(&ComputePipelineDescriptor {
            layout: &pipeline_layout_compute,
            compute_stage: ProgrammableStageDescriptor {
                module: &shader_copy_glowing,
                entry_point: "main",
            },
        });
        let pipeline_vertical_blur = device.create_compute_pipeline(&ComputePipelineDescriptor {
            layout: &pipeline_layout_compute,
            compute_stage: ProgrammableStageDescriptor {
                module: &shader_vertical_blur,
                entry_point: "main",
            },
        });
        let pipeline_horizontal_blur = device.create_compute_pipeline(&ComputePipelineDescriptor {
            layout: &pipeline_layout_compute,
            compute_stage: ProgrammableStageDescriptor {
                module: &shader_horizontal_blur,
                entry_point: "main",
            },
        });
        let pipeline_add_glow = device.create_compute_pipeline(&ComputePipelineDescriptor {
            layout: &pipeline_layout_compute,
            compute_stage: ProgrammableStageDescriptor {
                module: &shader_add_glow,
                entry_point: "main",
            },
        });
        let pipeline_copy_to_render_texture =
            device.create_render_pipeline(&RenderPipelineDescriptor {
                layout: &pipeline_layout_render,
                vertex_stage: ProgrammableStageDescriptor {
                    module: &shader_rectangle,
                    entry_point: "main",
                },
                fragment_stage: Some(ProgrammableStageDescriptor {
                    module: &shader_copy_texture,
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

        Box::new(Self {
            texture1,
            copy_to_texture1_pass: (pipeline_copy_to_texture1, bind_group_copy_to_texture1),
            copy_glowing_pass: (pipeline_copy_glowing, bind_group_copy_glowing),
            vertical_blur_pass: (pipeline_vertical_blur, bind_group_vertical_blur),
            horizontal_blur_pass: (pipeline_horizontal_blur, bind_group_horizontal_blur),
            add_glow_pass: (pipeline_add_glow, bind_group_add_glow),
            copy_to_render_texture_pass: (
                pipeline_copy_to_render_texture,
                bind_group_copy_to_render_texture,
            ),
        })
    }
}

impl RenderPass for GlowPostProcess {
    fn render(&self, encoder: &mut CommandEncoder, render_texture: &TextureView) {
        {
            let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
                color_attachments: &[RenderPassColorAttachmentDescriptor {
                    attachment: &self.texture1,
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
                (SANDBOX_WIDTH as u32 / 7) + 8,
                (SANDBOX_HEIGHT as u32 / 7) + 8,
                1,
            );
        }
        {
            let mut pass = encoder.begin_compute_pass();
            pass.set_pipeline(&self.vertical_blur_pass.0);
            pass.set_bind_group(0, &self.vertical_blur_pass.1, &[]);
            pass.dispatch(
                (SANDBOX_WIDTH as u32 / 7) + 8,
                (SANDBOX_HEIGHT as u32 / 7) + 8,
                1,
            );
        }
        {
            let mut pass = encoder.begin_compute_pass();
            pass.set_pipeline(&self.horizontal_blur_pass.0);
            pass.set_bind_group(0, &self.horizontal_blur_pass.1, &[]);
            pass.dispatch(
                (SANDBOX_WIDTH as u32 / 7) + 8,
                (SANDBOX_HEIGHT as u32 / 7) + 8,
                1,
            );
        }
        {
            let mut pass = encoder.begin_compute_pass();
            pass.set_pipeline(&self.add_glow_pass.0);
            pass.set_bind_group(0, &self.add_glow_pass.1, &[]);
            pass.dispatch(
                (SANDBOX_WIDTH as u32 / 7) + 8,
                (SANDBOX_HEIGHT as u32 / 7) + 8,
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

    fn resize(&mut self, _: &mut CommandEncoder, _: u32, _: u32) {}

    fn update_bindings(&mut self, _: &TextureView, _: &Extent3d) {
        unreachable!()
    }
}
