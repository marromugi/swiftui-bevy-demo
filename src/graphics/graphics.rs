use std::ffi::c_void;

use bevy_ecs::{
    prelude::*,
};
use wgpu::*;
use wgpu::util::DeviceExt;
use glam::{Mat4, Vec3};
use bytemuck::{Pod, Zeroable};

use crate::{
    graphics:: {
        CameraParams
    },
    ecs::{
        Transform3D,
        CubeMeshTag
    }
};

// 頂点1個ぶんのレイアウト (位置＋色)
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Vertex {
    pos: [f32; 3],
    color: [f32; 3],
}

// 小さなキューブメッシュをCPU側で用意する
fn make_unit_cube_vertices_and_indices() -> (Vec<Vertex>, Vec<u16>) {
    // 8頂点 (各面で色変えてるだけ)
    let verts: [Vertex; 8] = [
        // front (z+)
        Vertex { pos: [-0.5,-0.5, 0.5], color: [1.0,0.0,0.0] },
        Vertex { pos: [ 0.5,-0.5, 0.5], color: [1.0,0.0,0.0] },
        Vertex { pos: [ 0.5, 0.5, 0.5], color: [1.0,0.0,0.0] },
        Vertex { pos: [-0.5, 0.5, 0.5], color: [1.0,0.0,0.0] },

        // back (z-)
        Vertex { pos: [-0.5,-0.5,-0.5], color: [0.0,1.0,0.0] },
        Vertex { pos: [ 0.5,-0.5,-0.5], color: [0.0,1.0,0.0] },
        Vertex { pos: [ 0.5, 0.5,-0.5], color: [0.0,1.0,0.0] },
        Vertex { pos: [-0.5, 0.5,-0.5], color: [0.0,1.0,0.0] },
    ];

    // 12 triangles / 36 indices
    let idx: [u16; 36] = [
        // front
        0,1,2, 0,2,3,
        // right
        1,5,6, 1,6,2,
        // back
        5,4,7, 5,7,6,
        // left
        4,0,3, 4,3,7,
        // top
        3,2,6, 3,6,7,
        // bottom
        4,5,1, 4,1,0,
    ];

    (verts.to_vec(), idx.to_vec())
}

pub struct Graphics {
    surface: wgpu::Surface<'static>,
    surface_config: wgpu::SurfaceConfiguration,
    device: wgpu::Device,
    queue: wgpu::Queue,
    render_pipeline: wgpu::RenderPipeline,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    cube_vertex_buffer: wgpu::Buffer,
    cube_index_buffer: wgpu::Buffer,
    cube_index_count: u32,
    camera: CameraParams
}

impl Graphics {
    // CAMetalLayer から初期化する想定のファクトリ
    pub unsafe fn new_from_layer(
        layer_ptr: *mut c_void,
        width: u32,
        height: u32,
    ) -> Self {
        // 1. Instance
        let instance = Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        // 2. Surface (CAMetalLayerベース)
        //
        let surface = unsafe {
            instance
                .create_surface_unsafe(
                    wgpu::SurfaceTargetUnsafe::CoreAnimationLayer(layer_ptr)
                ) // ← 仮API名。要置き換え
                .expect("Failed to create surface from CAMetalLayer")
        };

        // 3. Adapter
        let adapter = pollster::block_on(instance.request_adapter(&RequestAdapterOptions {
            power_preference: PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
        }))
        .expect("no suitable GPU adapter found");

        // 4. Device / Queue
        let (device, queue) = pollster::block_on(adapter.request_device(
            &DeviceDescriptor {
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                label: None,
                memory_hints: wgpu::MemoryHints::Performance,
                trace: Trace::Off,
            }
        ))
        .expect("failed to create device");

        // 5. Surface config
        let caps = surface.get_capabilities(&adapter);
        let format = caps.formats[0];
        let surface_config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format,
            width: width.max(1),
            height: height.max(1),
            present_mode: PresentMode::Fifo,
            alpha_mode: CompositeAlphaMode::Opaque,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &surface_config);

        // 6. キューブのジオメトリをGPUにアップロード
        let (cube_vertices, cube_indices) = make_unit_cube_vertices_and_indices();

        let cube_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("cube-vertex-buffer"),
            contents: bytemuck::cast_slice(&cube_vertices),
            usage: BufferUsages::VERTEX,
        });

        let cube_index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("cube-index-buffer"),
            contents: bytemuck::cast_slice(&cube_indices),
            usage: BufferUsages::INDEX,
        });

        let cube_index_count = cube_indices.len() as u32;

        // 7. Uniform buffer (model_view_proj 行列)
        let identity = Mat4::IDENTITY;
        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("mvp-buffer"),
            contents: bytemuck::cast_slice(&identity.to_cols_array()),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("mvp-bgl"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(16 * 4),
                    },
                    count: None,
                }],
            });

        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("mvp-bg"),
            layout: &uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        // 8. Render pipeline
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("cube-shader"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!("shader.wgsl"))),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("pipeline-layout"),
            bind_group_layouts: &[&uniform_bind_group_layout],
            push_constant_ranges: &[],
        });

        let vertex_buffers = &[wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    shader_location: 0,
                    offset: 0,
                    format: wgpu::VertexFormat::Float32x3, // pos
                },
                wgpu::VertexAttribute {
                    shader_location: 1,
                    offset: 12,
                    format: wgpu::VertexFormat::Float32x3, // color
                },
            ],
        }];

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("cube-pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: vertex_buffers,
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(format.into())],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let camera = CameraParams {
            eye: glam::Vec3::new(0.0, 4.0, -4.5),
            target: glam::Vec3::new(0.0, 0.0, 0.0),
            up: glam::Vec3::Y,
            fov_y_radians: 45f32.to_radians(),
        };

        Self {
            surface,
            surface_config,
            device,
            queue,
            render_pipeline,
            uniform_buffer,
            uniform_bind_group,
            cube_vertex_buffer,
            cube_index_buffer,
            cube_index_count,
            camera
        }
    }

    fn proj_matrix(&self) -> Mat4 {
        let aspect = self.surface_config.width as f32 / self.surface_config.height as f32;
        Mat4::perspective_rh_gl(self.camera.fov_y_radians, aspect, 0.1, 100.0)
    }

    fn view_matrix(&self) -> Mat4 {
       Mat4::look_at_rh(self.camera.eye, self.camera.target, self.camera.up)
    }
    
    fn model_matrix(tf: &Transform3D) -> Mat4 {
        Mat4::from_scale_rotation_translation(tf.scale, tf.rotation, tf.translation)
    }

    fn write_mvp(&self, mvp: Mat4) {
        self.queue.write_buffer(
            &self.uniform_buffer,
            0,
            bytemuck::cast_slice(&mvp.to_cols_array()),
        );
    }

    pub fn draw_world(&mut self, world: &mut World) {
        // 次のフレームのテクスチャを取得
        let frame = self
            .surface
            .get_current_texture()
            .expect("Failed to acquire next swap chain texture.");

        let view_tex = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("main-encoder"),
            });

        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("main-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view_tex,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.05,
                            g: 0.05,
                            b: 0.08,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            rpass.set_pipeline(&self.render_pipeline);
            rpass.set_vertex_buffer(0, self.cube_vertex_buffer.slice(..));
            rpass.set_index_buffer(self.cube_index_buffer.slice(..), wgpu::IndexFormat::Uint16);

            // ECSから「描画対象のキューブ」を全部拾って描く
            let mut query = world.query_filtered::<&Transform3D, With<CubeMeshTag>>();

            for tf in query.iter(world) {
                // per-object MVP
                let model = Self::model_matrix(tf);
                let view = self.view_matrix();
                let proj = self.proj_matrix();
                let mvp = proj * view * model;

                self.write_mvp(mvp);
                rpass.set_bind_group(0, &self.uniform_bind_group, &[]);
                rpass.draw_indexed(0..self.cube_index_count, 0, 0..1);
            }
        }

        self.queue.submit(Some(encoder.finish()));
        frame.present();
    }
}
