use std::sync::Arc;
use wgpu::{self, util::DeviceExt, Buffer, BufferUsages, CommandEncoder};
use std::time::Instant;

pub struct ZeroSyncDispatch {
    device: Arc<wgpu::Device>,
    queue: wgpu::Queue,

    field_tensor_pipeline: wgpu::ComputePipeline,
    conservation_pipeline: wgpu::ComputePipeline,
    sparse_stream_pipeline: wgpu::ComputePipeline,
    indirect_build_pipeline: wgpu::ComputePipeline,

    indirect_dispatch: Buffer,

    stream_req_buffer: Buffer,
    sparse_active_count: Buffer,

    field_buffer: Buffer,
    gradient_buffer: Buffer,
    sparse_nodes: Buffer,
    spatial_hash: Buffer,

    conservation_state: Buffer,
    correction_log: Buffer,

    log_count_buffer: Buffer,
    cell_count_uniform: Buffer,

    field_bind_group: wgpu::BindGroup,
    conservation_bind_group: wgpu::BindGroup,
    sparse_bind_group: wgpu::BindGroup,

    field_pipeline_layout: wgpu::PipelineLayout,
    conservation_pipeline_layout: wgpu::PipelineLayout,
    sparse_stream_pipeline_layout: wgpu::PipelineLayout,

    frame_count: u64,
    total_cells: u32,
    active_cells: u32,
    mass_drift: f32,
}

impl ZeroSyncDispatch {
    pub fn new(device: Arc<wgpu::Device>, queue: wgpu::Queue, shader_modules: &ShaderModules) -> Self {
        let total_cells: u32 = 50_000_000;

        let field_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("field_buffer"),
            size: (total_cells as u64) * 16,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let gradient_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("gradient_buffer"),
            size: (total_cells as u64) * 16,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let conservation_state = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("conservation_state"),
            size: 256,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let meta_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("dispatch_meta"),
            contents: bytemuck::cast_slice(&[
                781250u32,   // tile_count: 50M / 64
                64u32,       // cells_per_tile
                0xFFFFFFFFu32, // active_mask: all active
                0x3F800000u32, // thermal_limit_pct: 1.0
                0x3F800000u32, // vram_pressure_pct: 1.0
            ]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let phase_diagram_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("phase_diagram"),
            size: 256 * 16,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let indirect_dispatch = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("indirect_dispatch"),
            contents: bytemuck::cast_slice(&[1u32, 1u32, 1u32]),
            usage: BufferUsages::STORAGE | BufferUsages::INDIRECT | BufferUsages::COPY_DST,
        });

        let stream_req_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("stream_req"),
            contents: bytemuck::cast_slice(&[
                -1.0f32, -1.0f32, -1.0f32,
                0.0f32,
                1.0f32, 1.0f32, 1.0f32,
                0.0f32,
                1.0f32,
                16.0f32,
            ]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let sparse_active_count = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("sparse_active_count"),
            size: 4,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let log_count_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("log_count"),
            size: 4,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let cell_count_uniform = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("cell_count_uniform"),
            contents: bytemuck::cast_slice(&[total_cells]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let bind_group_layout_field = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("field_tensor_bgl"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0, visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Storage { read_only: false }, has_dynamic_offset: false, min_binding_size: None },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1, visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Storage { read_only: false }, has_dynamic_offset: false, min_binding_size: None },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2, visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3, visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Storage { read_only: true }, has_dynamic_offset: false, min_binding_size: None },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 4, visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Storage { read_only: false }, has_dynamic_offset: false, min_binding_size: None },
                    count: None,
                },
            ],
        });

        let field_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("field_tensor_bg"),
            layout: &bind_group_layout_field,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: field_buffer.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: conservation_state.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 2, resource: meta_buffer.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 3, resource: phase_diagram_buffer.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 4, resource: gradient_buffer.as_entire_binding() },
            ],
        });

        let field_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("field_tensor_pl"),
            bind_group_layouts: &[&bind_group_layout_field],
            push_constant_ranges: &[],
        });

        let field_tensor_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("field_tensor_update"),
            layout: Some(&field_pipeline_layout),
            module: &shader_modules.field_tensor,
            entry_point: "field_tensor_update",
        });

        // Explicit pipeline layout for conservation to match explicit BGL
        let conservation_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("conservation_pl"),
            bind_group_layouts: &[&bgl_conservation],
            push_constant_ranges: &[],
        });

        let conservation_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("conservation_enforce"),
            layout: Some(&conservation_pipeline_layout),
            module: &shader_modules.conservation,
            entry_point: "enforce_conservation",
        });

        // Explicit pipeline layout for sparse_stream to match explicit BGL
        let sparse_stream_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("sparse_stream_pl"),
            bind_group_layouts: &[&bgl_sparse],
            push_constant_ranges: &[],
        });

        let sparse_stream_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("sparse_stream"),
            layout: Some(&sparse_stream_pipeline_layout),
            module: &shader_modules.sparse_stream,
            entry_point: "sparse_stream_activate",
        });

        let indirect_build_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("indirect_build"),
            layout: Some(&sparse_stream_pipeline_layout),
            module: &shader_modules.indirect_build,
            entry_point: "build_indirect",
        });

        ZeroSyncDispatch {
            device,
            queue,
            field_tensor_pipeline,
            conservation_pipeline,
            sparse_stream_pipeline,
            indirect_build_pipeline,
            indirect_dispatch,
            stream_req_buffer,
            sparse_active_count,
            field_buffer,
            gradient_buffer,
            sparse_nodes: device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("sparse_nodes"),
                size: 1024 * 1024 * 64,
                usage: BufferUsages::STORAGE | BufferUsages::INDIRECT,
                mapped_at_creation: false,
            }),
            spatial_hash: device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("spatial_hash"),
                size: 1024 * 1024 * 4,
                usage: BufferUsages::STORAGE,
                mapped_at_creation: false,
            }),
            conservation_state,
            correction_log: device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("correction_log"),
                size: 1024 * 4,
                usage: BufferUsages::STORAGE | BufferUsages::COPY_SRC,
                mapped_at_creation: false,
            }),
            log_count_buffer,
            cell_count_uniform,
            field_bind_group,
            conservation_bind_group: {
                let bgl_conservation = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("conservation_bgl"),
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0, visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Storage { read_only: false }, has_dynamic_offset: false, min_binding_size: None },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1, visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Storage { read_only: false }, has_dynamic_offset: false, min_binding_size: None },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 2, visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Storage { read_only: false }, has_dynamic_offset: false, min_binding_size: None },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 3, visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Storage { read_only: false }, has_dynamic_offset: false, min_binding_size: None },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 4, visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Storage { read_only: false }, has_dynamic_offset: false, min_binding_size: None },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 5, visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None },
                            count: None,
                        },
                    ],
                });
                device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("conservation_bg"),
                    layout: &bgl_conservation,
                    entries: &[
                        wgpu::BindGroupEntry { binding: 0, resource: field_buffer.as_entire_binding() },
                        wgpu::BindGroupEntry { binding: 1, resource: gradient_buffer.as_entire_binding() },
                        wgpu::BindGroupEntry { binding: 2, resource: conservation_state.as_entire_binding() },
                        wgpu::BindGroupEntry { binding: 3, resource: correction_log.as_entire_binding() },
                        wgpu::BindGroupEntry { binding: 4, resource: log_count_buffer.as_entire_binding() },
                        wgpu::BindGroupEntry { binding: 5, resource: cell_count_uniform.as_entire_binding() },
                    ],
                })
            },
            sparse_bind_group: {
                let bgl_sparse = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("sparse_stream_bgl"),
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0, visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Storage { read_only: false }, has_dynamic_offset: false, min_binding_size: None },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1, visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Storage { read_only: false }, has_dynamic_offset: false, min_binding_size: None },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 2, visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 3, visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Storage { read_only: false }, has_dynamic_offset: false, min_binding_size: None },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 4, visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Storage { read_only: false }, has_dynamic_offset: false, min_binding_size: None },
                            count: None,
                        },
                    ],
                });
                device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("sparse_bg"),
                    layout: &bgl_sparse,
                    entries: &[
                        wgpu::BindGroupEntry { binding: 0, resource: sparse_nodes.as_entire_binding() },
                        wgpu::BindGroupEntry { binding: 1, resource: spatial_hash.as_entire_binding() },
                        wgpu::BindGroupEntry { binding: 2, resource: stream_req_buffer.as_entire_binding() },
                        wgpu::BindGroupEntry { binding: 3, resource: sparse_active_count.as_entire_binding() },
                        wgpu::BindGroupEntry { binding: 4, resource: indirect_dispatch.as_entire_binding() },
                    ],
                })
            },
            field_pipeline_layout,
            conservation_pipeline_layout,
            sparse_stream_pipeline_layout,
            frame_count: 0,
            total_cells,
            active_cells: total_cells,
            mass_drift: 0.0,
        }
    }

    pub fn dispatch_frame(&mut self, encoder: &mut CommandEncoder) {
        let frame_start = Instant::now();

        {
            let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("field_tensor"),
                timestamp_writes: None,
            });
            cpass.set_pipeline(&self.field_tensor_pipeline);
            cpass.set_bind_group(0, &self.field_bind_group, &[]);
            cpass.dispatch_workgroups_indirect(&self.indirect_dispatch, 0);
        }

        {
            let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("conservation"),
                timestamp_writes: None,
            });
            cpass.set_pipeline(&self.conservation_pipeline);
            cpass.set_bind_group(0, &self.conservation_bind_group, &[]);
            cpass.dispatch_workgroups_indirect(&self.indirect_dispatch, 0);
        }

        {
            let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("sparse_stream"),
                timestamp_writes: None,
            });
            cpass.set_pipeline(&self.sparse_stream_pipeline);
            cpass.set_bind_group(0, &self.sparse_bind_group, &[]);
            cpass.dispatch_workgroups(128, 1, 1);
        }

        {
            let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("indirect_build"),
                timestamp_writes: None,
            });
            cpass.set_pipeline(&self.indirect_build_pipeline);
            cpass.set_bind_group(0, &self.sparse_bind_group, &[]);
            cpass.dispatch_workgroups(1, 1, 1);
        }

        self.frame_count += 1;
        let frame_us = frame_start.elapsed().as_micros();
        if self.frame_count % 60 == 0 {
            println!("Frame {} | dispatch submitted in {}µs | {:.1} FPS",
                self.frame_count, frame_us,
                1_000_000.0 / frame_us as f64);
        }
    }

    pub fn hot_reload_field_tensor(&mut self, new_wgsl: &str) -> Result<(), String> {
        let module = self.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("field_tensor_reloaded"),
            source: wgpu::ShaderSource::Wgsl(new_wgsl.into()),
        });
        self.field_tensor_pipeline = self.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("field_tensor_update"),
            layout: Some(&self.field_pipeline_layout),
            module: &module,
            entry_point: "field_tensor_update",
        });
        println!("Field tensor shader hot-reloaded ({} bytes)", new_wgsl.len());
        Ok(())
    }
}

pub struct ShaderModules {
    pub field_tensor: wgpu::ShaderModule,
    pub conservation: wgpu::ShaderModule,
    pub sparse_stream: wgpu::ShaderModule,
    pub indirect_build: wgpu::ShaderModule,
}

pub async fn run() {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());
    let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: None,
        force_fallback_adapter: false,
    }).await.expect("No GPU adapter found");

    let (device, queue) = adapter.request_device(
        &wgpu::DeviceDescriptor {
            label: Some("Aetherion Continuum"),
            required_features: wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES
                | wgpu::Features::INDIRECT_FIRST_INSTANCE,
            required_limits: wgpu::Limits {
                max_storage_buffer_binding_size: 1 << 30,
                max_compute_workgroup_storage_size: 65536,
                ..Default::default()
            },
            ..Default::default()
        }, None,
    ).await.expect("Failed to create device");

    let device = Arc::new(device);

    let field_tensor_src = include_str!("../../core/field_tensor.wgsl");
    let conservation_src = include_str!("../../core/conservation_enforce.wgsl");
    let sparse_stream_src = include_str!("../../core/sparse_stream.wgsl");
    let indirect_build_src = include_str!("../../core/indirect_build.wgsl");

    let shaders = ShaderModules {
        field_tensor: device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("field_tensor"), source: wgpu::ShaderSource::Wgsl(field_tensor_src.into()),
        }),
        conservation: device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("conservation"), source: wgpu::ShaderSource::Wgsl(conservation_src.into()),
        }),
        sparse_stream: device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("sparse_stream"), source: wgpu::ShaderSource::Wgsl(sparse_stream_src.into()),
        }),
        indirect_build: device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("indirect_build"), source: wgpu::ShaderSource::Wgsl(indirect_build_src.into()),
        }),
    };

    let mut engine = ZeroSyncDispatch::new(device.clone(), queue, &shaders);

    println!("╔══════════════════════════════════════╗");
    println!("║  AETHERION-CONTINUUM — BOOT OK      ║");
    println!("║  Field cells: {}              ║", engine.total_cells);
    println!("║  Mode: Zero-Sync GPU Compute        ║");
    println!("║  Conservation: Enforced             ║");
    println!("╚══════════════════════════════════════╝");

    loop {
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("frame_encoder"),
        });
        engine.dispatch_frame(&mut encoder);
        device.queue().submit(Some(encoder.finish()));
        device.poll(wgpu::Maintain::Wait);
    }
}
