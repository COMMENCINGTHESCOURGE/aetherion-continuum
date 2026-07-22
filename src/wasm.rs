use wasm_bindgen::prelude::*;
use std::sync::Arc;
use crate::pipeline::zero_sync_dispatch::{ZeroSyncDispatch, ShaderModules};

#[wasm_bindgen]
pub struct AetherionEngine {
    engine: ZeroSyncDispatch,
}

#[wasm_bindgen]
impl AetherionEngine {
    #[wasm_bindgen(constructor)]
    pub async fn new() -> Result<AetherionEngine, JsValue> {
        console_error_panic_hook::set_once();
        
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await
            .ok_or_else(|| JsValue::from_str("No GPU adapter found"))?;

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Aetherion Continuum"),
                    required_features: wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES
                        | wgpu::Features::INDIRECT_FIRST_INSTANCE,
                    required_limits: wgpu::Limits {
                        max_storage_buffer_binding_size: 1 << 29, 
                        max_compute_workgroup_storage_size: 16384,
                        ..wgpu::Limits::downlevel_defaults()
                    },
                    ..Default::default()
                },
                None,
            )
            .await
            .map_err(|e| JsValue::from_str(&format!("Failed to create device: {}", e)))?;

        let device = Arc::new(device);

        let field_tensor_src = include_str!("../core/field_tensor.wgsl");
        let conservation_src = include_str!("../core/conservation_enforce.wgsl");
        let sparse_stream_src = include_str!("../core/sparse_stream.wgsl");
        let indirect_build_src = include_str!("../core/indirect_build.wgsl");
        let gpu_kompress_src = include_str!("../core/gpu_kompress.wgsl");

        let shaders = ShaderModules {
            field_tensor: device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("field_tensor"),
                source: wgpu::ShaderSource::Wgsl(field_tensor_src.into()),
            }),
            conservation: device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("conservation"),
                source: wgpu::ShaderSource::Wgsl(conservation_src.into()),
            }),
            sparse_stream: device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("sparse_stream"),
                source: wgpu::ShaderSource::Wgsl(sparse_stream_src.into()),
            }),
            indirect_build: device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("indirect_build"),
                source: wgpu::ShaderSource::Wgsl(indirect_build_src.into()),
            }),
            gpu_kompress: device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("gpu_kompress"),
                source: wgpu::ShaderSource::Wgsl(gpu_kompress_src.into()),
            }),
        };

        let engine = ZeroSyncDispatch::new(device.clone(), queue, &shaders);

        Ok(AetherionEngine {
            engine,
        })
    }

    pub fn tick(&mut self) {
        let mut encoder = self.engine.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("frame_encoder"),
        });
        self.engine.dispatch_frame(&mut encoder);
        self.engine.queue.submit(Some(encoder.finish()));
    }

    pub fn load_landscape(&mut self, payload: &[f32]) {
        self.engine.load_landscape_payload(payload);
    }
}
