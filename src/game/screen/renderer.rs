use bytemuck;
use crate::constants;
use crate::game::camera::Camera;
use crate::game::tile::Tile;
use wgpu::{
    util::DeviceExt,
    ComputePassDescriptor,
};

/* #region STRUCTS THAT SHOULD PROBABLY GO ELSEWHERE */
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 4],
    pub color: [f32; 4],
}
/* #endregion */

pub struct Renderer {
    vertex_bind_group_layout: wgpu::BindGroupLayout,

    light_buffer: wgpu::Buffer,
    light_bind_group: wgpu::BindGroup,

    g_buffer: wgpu::Buffer,
    g_buffer_bind_group: wgpu::BindGroup,

    screen_buffer: wgpu::Buffer,
    screen_bind_group: wgpu::BindGroup,

    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,

    lighting_pipeline: wgpu::ComputePipeline,
    g_buffer_pipeline: wgpu::ComputePipeline,

    render_pipeline: wgpu::RenderPipeline,
}

impl Renderer {
    pub fn new(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration) -> Renderer
    {
        let light_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Light Buffer"),
                contents: bytemuck::cast_slice(&[
                    Vertex {
                        position: [0.0, 0.0, 0.1, 1.0],
                        color: [1.0, 1.0, 1.0, 0.8],
                    },
                    Vertex {
                        position: [0.0, 0.5, 0.1, 1.11803398875],
                        color: [0.0, 1.0, 0.0, 0.3],
                    },
                ]),
                usage: wgpu::BufferUsages::STORAGE
                    | wgpu::BufferUsages::COPY_DST,
            }
        );
        let light_bind_group_layout = 
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage
                                { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });
        let light_bind_group = device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                label: Some("light_bind_group"),
                layout: &light_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::Buffer(
                            wgpu::BufferBinding {
                                buffer: &light_buffer,
                                offset: 0,
                                size: None,
                            })
                    }]
                });

        /* #region LAYOUT FOR VERTEX BIND GROUP */
        let vertex_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer{
                            ty: wgpu::BufferBindingType::Storage
                                { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer{
                            ty: wgpu::BufferBindingType::Storage
                                { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer{
                            ty: wgpu::BufferBindingType::Storage
                                { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });
        /* #endregion */
    
        /* #region CAMERA STUFF */
        let camera_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Camera Buffer"),
                contents: bytemuck::cast_slice(&[0.0f32; 32]),
                usage: wgpu::BufferUsages::UNIFORM
                    | wgpu::BufferUsages::COPY_DST,
            }
        );
        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }
                ],
            }
        );
        let camera_bind_group =
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &camera_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: camera_buffer.as_entire_binding(),
                    }
                ],
            });
        /* #endregion */

        /* #region THE SCREEN WE DRAW TO */
        let screen_buffer =
            device.create_buffer(&wgpu::BufferDescriptor {
                label: None,
                size: (constants::SCREEN_PIXELS.0
                    * constants::SCREEN_PIXELS.1
                    * 4) as u64,
                usage: wgpu::BufferUsages::COPY_DST
                    | wgpu::BufferUsages::STORAGE,
                mapped_at_creation: false,
            });
        let screen_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT
                            | wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });
        let screen_bind_group = device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                label: Some("compute output"),
                layout: &screen_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::Buffer(
                            wgpu::BufferBinding {
                                buffer: &screen_buffer,
                                offset: 0,
                                size: None
                            }
                        ),
                    },
                ],
            });
        /* #endregion */

        /* #region THE G-BUFFER */
        let g_buffer =
            device.create_buffer(&wgpu::BufferDescriptor {
                label: None,
                size: (constants::G_BUFFER_SIZE) as u64,
                usage: wgpu::BufferUsages::COPY_DST
                    | wgpu::BufferUsages::STORAGE,
                mapped_at_creation: false,
            });
        let g_buffer_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT
                            | wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });
        let g_buffer_bind_group = device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                label: Some("g-buffer"),
                layout: &screen_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::Buffer(
                            wgpu::BufferBinding {
                                buffer: &g_buffer,
                                offset: 0,
                                size: None
                            }
                        ),
                    },
                ],
            });
        /* #endregion */
        
        /* #region G-BUFFER PIPELINE SETUP */
        let g_buffer_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/g_buffer.wgsl").into()),
        });
        let g_buffer_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("g_buffer"),
                bind_group_layouts: &[
                    &g_buffer_bind_group_layout,
                    &camera_bind_group_layout,
                    &vertex_bind_group_layout,
                ],
                push_constant_ranges: &[wgpu::PushConstantRange{
                    stages: wgpu::ShaderStages::COMPUTE,
                    range: 0..32,
                }],
            });
        let g_buffer_pipeline =
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: None,
                layout: Some(&g_buffer_pipeline_layout),
                module: &g_buffer_shader,
                entry_point: "main",
            });
        /* #endregion */
        
        /* #region LIGHTING PIPELINE SETUP */

        let lighting_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/lighting.wgsl").into()),
        });

        let lighting_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("lighting"),
                bind_group_layouts: &[
                    &screen_bind_group_layout,
                    &g_buffer_bind_group_layout,
                    &camera_bind_group_layout,
                    &light_bind_group_layout,
                ],
                push_constant_ranges: &[wgpu::PushConstantRange{
                    stages: wgpu::ShaderStages::COMPUTE,
                    range: 0..32,
                }],
            });
        let lighting_pipeline =
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: None,
                layout: Some(&lighting_pipeline_layout),
                module: &lighting_shader,
                entry_point: "main",
            });
        /* #endregion */

        /* #region RENDER PIPELINE SETUP */
        let copy_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/copy.wgsl").into()),
        });
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("render"),
                bind_group_layouts: &[&g_buffer_bind_group_layout],
                push_constant_ranges: &[
                    wgpu::PushConstantRange {
                        stages: wgpu::ShaderStages::VERTEX_FRAGMENT,
                        range: 0..32,
                    }],
            });
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("render"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &copy_shader,
                entry_point: "vs_main",
                buffers: &[],
            },
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &copy_shader,
                entry_point: "fs_main",
                targets: &[Some(config.format.into())],
            }),
            multiview: None,
        });
        /* #endregion */

        Self {
            vertex_bind_group_layout,

            light_buffer,
            light_bind_group,

            g_buffer,
            g_buffer_bind_group,

            screen_buffer,
            screen_bind_group,

            camera_buffer,
            camera_bind_group,

            g_buffer_pipeline,
            lighting_pipeline,
            render_pipeline,
        }
    }

    pub fn render(&self,
        device: &wgpu::Device,
        surface: &wgpu::Surface,
        queue: &wgpu::Queue,
        camera: &Camera,
        tiles: &mut [Tile],
        borders: &[f32; 2]
    ) -> Result<(), wgpu::SurfaceError> {

        /* #region SETUP STUFF */
        let pc = &[(constants::SCREEN_PIXELS.0) as f32,
            (constants::SCREEN_PIXELS.1) as f32,
            borders[0],
            borders[1],
            constants::WORLD_SCALE,
            camera.get_position()[2],
            constants::NEAR_PLANE,
            constants::FAR_PLANE,
        ];
        let pc_bytes = bytemuck::cast_slice(pc);

        queue.write_buffer(&self.camera_buffer, 0,
            bytemuck::cast_slice(
                &[camera.world_to_camera(),
                camera.camera_to_screen(),]
            ));
        queue.write_buffer(&self.g_buffer, 0,
            &[0u8; (constants::SCREEN_PIXELS.0
                    * constants::SCREEN_PIXELS.1
                    * 4 * constants::G_BUFFER_NUMS) as usize]
            );
        queue.write_buffer(&self.screen_buffer, 0,
            &[0u8; (constants::SCREEN_PIXELS.0
                    * constants::SCREEN_PIXELS.1
                    * 4) as usize]
            );

        /* SDHFJKJSDKDHSFJKSD */
        queue.write_buffer(&self.light_buffer, 0,
            bytemuck::cast_slice(
                &[camera.get_position()[0],
                    camera.get_position()[1],
                    0.1,
                    camera.get_position()[3],
                    1.0, 1.0, 1.0, 0.5]
            ));
     
        let output = surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        let render_pass_descriptor = wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[
                Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(
                            wgpu::Color {
                                r: 0.0,
                                g: 0.0,
                                b: 0.0,
                                a: 1.0,
                            }
                        ),
                        store: true,
                    }
                })
            ],
            depth_stencil_attachment: None,
        };
        /* #endregion */

        /* #region COMPUTE PASS */
        {
            let mut cpass = 
                encoder.begin_compute_pass(&ComputePassDescriptor {
                    label: None,
                });
            cpass.set_pipeline(&self.g_buffer_pipeline);
            cpass.set_bind_group(0, &self.g_buffer_bind_group, &[]);
            cpass.set_bind_group(1, &self.camera_bind_group, &[]);
            cpass.set_push_constants(0, pc_bytes);
            
            for tile in tiles {
                match tile.get_vbuf() {
                    None => {self.make_buffers(tile, device)}
                    _ => {}
                }
                cpass.set_bind_group(2, tile.get_bind_group().unwrap(), &[]);
                cpass.dispatch_workgroups((tile.get_size() / 64) + 1, 1, 1);
            }
        }

        {
            let mut cpass = 
                encoder.begin_compute_pass(&ComputePassDescriptor {
                    label: None,
                });
            cpass.set_pipeline(&self.lighting_pipeline);
            cpass.set_push_constants(0, pc_bytes);

            cpass.set_bind_group(0, &self.screen_bind_group, &[]);
            cpass.set_bind_group(1, &self.g_buffer_bind_group, &[]);
            cpass.set_bind_group(2, &self.camera_bind_group, &[]);
            cpass.set_bind_group(3, &self.light_bind_group, &[]);
            
            cpass.dispatch_workgroups(
                (constants::SCREEN_PIXELS.0 + 15) / 16,
                (constants::SCREEN_PIXELS.1 + 15) / 16,
                3
            );
        }
        /* #endregion */

        /* #region RENDER PASS */
        {
            let mut rpass = encoder.begin_render_pass(&render_pass_descriptor);
            rpass.set_pipeline(&self.render_pipeline);
            rpass.set_push_constants(
                wgpu::ShaderStages::VERTEX_FRAGMENT,
                0,
                bytemuck::cast_slice(
                    &[(constants::SCREEN_PIXELS.0) as f32,
                    (constants::SCREEN_PIXELS.1) as f32,
                    borders[0],
                    borders[1],
                    (constants::WORLD_SCALE) as f32,
                    camera.get_position()[2],
                    constants::NEAR_PLANE,
                    constants::FAR_PLANE,
                ]
            ));
            rpass.set_bind_group(0, &self.screen_bind_group, &[]);
            rpass.draw(0..6, 0..1);
        }
        /* #endregion */

        queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    pub fn make_buffers(
        &self,
        tile: &mut Tile,
        device: &wgpu::Device,
    ) {
        let vbuf = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(tile.get_vertices()),
                usage: wgpu::BufferUsages::STORAGE,
            }
        );
        let ibuf = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(tile.get_indices()),
                usage: wgpu::BufferUsages::STORAGE,
            }
        );
        let pos = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Position Buffer"),
                contents: bytemuck::cast_slice(tile.get_mat().as_slice()),
                usage: wgpu::BufferUsages::STORAGE
                    | wgpu::BufferUsages::COPY_DST,
            }
        );
        let tile_bind_group = device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                label: Some("vbuf_bind_group"),
                layout: &self.vertex_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::Buffer(
                            wgpu::BufferBinding {
                                buffer: &vbuf,
                                offset: 0,
                                size: None,
                            })
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Buffer(
                            wgpu::BufferBinding {
                                buffer: &ibuf,
                                offset: 0,
                                size: None,
                            })
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::Buffer(
                            wgpu::BufferBinding {
                                buffer: &pos,
                                offset: 0,
                                size: None,
                            })
                    },
                ],
            });
        tile.set_vbuf(vbuf);
        tile.set_ibuf(ibuf);
        tile.set_pos(pos);
        tile.set_bind_group(tile_bind_group);
    }

}