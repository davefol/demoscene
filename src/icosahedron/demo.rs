use clap::Args;
use std::{sync::Arc, time::Instant};

use wgpu::{RenderPassDescriptor, util::DeviceExt};

use super::icosahedron::{Icosahedron, Vertex};
use crate::gpu_context::GpuContext;

#[derive(Args)]
pub(crate) struct Opts {}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct TimeUniform {
    time: f32,
}

pub struct App<'a> {
    window: Option<Arc<winit::window::Window>>,
    gpu_context: Option<GpuContext>,
    surface: Option<wgpu::Surface<'a>>,
    render_pipeline: Option<wgpu::RenderPipeline>,
    vertex_buffer: Option<wgpu::Buffer>,
    index_buffer: Option<wgpu::Buffer>,
    indices_len: u32,
    time_buffer: Option<wgpu::Buffer>,
    time_bind_group: Option<wgpu::BindGroup>,
    start_instant: Instant,
}

impl<'a> App<'a> {
    pub fn new() -> Self {
        App {
            window: None,
            gpu_context: None,
            surface: None,
            render_pipeline: None,
            vertex_buffer: None,
            index_buffer: None,
            indices_len: 0,
            time_buffer: None,
            time_bind_group: None,
            start_instant: Instant::now(),
        }
    }

    fn render(&self) -> anyhow::Result<()> {
        if let (
            Some(gpu_context),
            Some(render_pipeline),
            Some(surface),
            Some(vertex_buffer),
            Some(index_buffer),
            Some(time_buffer),
            Some(time_bind_group),
            Some(window)
        ) = (
            &self.gpu_context,
            &self.render_pipeline,
            &self.surface,
            &self.vertex_buffer,
            &self.index_buffer,
            &self.time_buffer,
            &self.time_bind_group,
            &self.window
        ) {
            let surface_texture = match surface.get_current_texture() {
                Ok(st) => Ok(st),
                Err(wgpu::SurfaceError::Outdated | wgpu::SurfaceError::Lost) => {
                    let window = self
                        .window
                        .as_ref()
                        .ok_or(anyhow::anyhow!("window not initialized"))?;
                    self.resize(window.inner_size());
                    surface.get_current_texture()
                }
                Err(e) => Err(e),
            }?;
            // let surface_texture = surface.get_current_texture().unwrap();
            let view = surface_texture.texture.create_view(&Default::default());
            let mut encoder = gpu_context
                .device
                .create_command_encoder(&Default::default());
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            render_pass.set_pipeline(render_pipeline);
            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);

            // set time uniform by:
            // 1. calculating the timestamp
            // 2. building the time struct
            // 3. moving the time struct to the GPU
            // 4. binding the GPU resident time struct to the right slot
            //    (setting the bind group)
            let timestamp = Instant::now()
                .duration_since(self.start_instant)
                .as_secs_f32();
            let time_uniform = TimeUniform { time: timestamp };
            gpu_context
                .queue
                .write_buffer(time_buffer, 0, bytemuck::bytes_of(&time_uniform));
            render_pass.set_bind_group(0, time_bind_group, &[]);

            render_pass.draw_indexed(0..self.indices_len, 0, 0..1);
            drop(render_pass);
            gpu_context.queue.submit(std::iter::once(encoder.finish()));
            surface_texture.present();
            window.request_redraw();
        }
        Ok(())
    }

    fn resize(&self, size: winit::dpi::PhysicalSize<u32>) {
        if size.width == 0 {
            return;
        }
        if let (Some(surface), Some(gpu_context), Some(_)) =
            (&self.surface, &self.gpu_context, &self.window)
        {
            let capabilities = surface.get_capabilities(&gpu_context.adapter);
            surface.configure(
                &gpu_context.device,
                &wgpu::SurfaceConfiguration {
                    alpha_mode: capabilities.alpha_modes[0],
                    desired_maximum_frame_latency: 2,
                    format: capabilities.formats[0],
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                    height: size.height,
                    width: size.width,
                    present_mode: capabilities.present_modes[0],
                    view_formats: vec![],
                },
            );
        }
    }
}

impl<'a> winit::application::ApplicationHandler for App<'a> {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if self.window.is_none() {
            let window = event_loop.create_window(Default::default()).unwrap();
            window.set_title("Icosahedron");
            let window = Arc::new(window);
            let gpu_context = GpuContext::new(wgpu::Features::POLYGON_MODE_LINE).unwrap();
            let surface = gpu_context.instance.create_surface(window.clone()).unwrap();
            let capabilities = surface.get_capabilities(&gpu_context.adapter);
            surface.configure(
                &gpu_context.device,
                &wgpu::SurfaceConfiguration {
                    alpha_mode: capabilities.alpha_modes[0],
                    desired_maximum_frame_latency: 2,
                    format: capabilities.formats[0],
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                    height: window.inner_size().height,
                    width: window.inner_size().width,
                    present_mode: capabilities.present_modes[0],
                    view_formats: vec![],
                },
            );

            let icosahedron = Icosahedron::new();
            self.indices_len = icosahedron.indices.len() as u32;

            self.vertex_buffer = Some(gpu_context.device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("vertices"),
                    usage: wgpu::BufferUsages::VERTEX,
                    contents: bytemuck::cast_slice(&icosahedron.vertices),
                },
            ));

            self.index_buffer = Some(gpu_context.device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("indices"),
                    usage: wgpu::BufferUsages::INDEX,
                    contents: bytemuck::cast_slice(&icosahedron.indices),
                },
            ));

            // to make a uniform available to the shaders we need to
            // create a bind_group_layout which represents a slot for our
            // uniform and add it to the pipeline layout which represents
            // all the binding groups and then make sure the pipeline is
            // using that layout.

            // at render time, we need to have a bind group which references
            // a buffer that contains our uniform data. each pass we bind
            // the bind group into the appropriate slot.

            // we can make our buffer and its bind group once, then update
            // it as needed.
            let time_buffer =
                gpu_context
                    .device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("time"),
                        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                        contents: bytemuck::bytes_of(&TimeUniform { time: 0.0 }),
                    });

            let time_bind_group_layout =
                gpu_context
                    .device
                    .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                        label: Some("time bind group layout"),
                        entries: &[wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        }],
                    });

            self.time_bind_group = Some(gpu_context.device.create_bind_group(
                &wgpu::BindGroupDescriptor {
                    label: Some("time bind group"),
                    layout: &time_bind_group_layout,
                    entries: &[wgpu::BindGroupEntry {
                        binding: 0,
                        resource: time_buffer.as_entire_binding(),
                    }],
                },
            ));
            self.time_buffer = Some(time_buffer);

            let shader_module = gpu_context
                .device
                .create_shader_module(wgpu::include_wgsl!("demo.wgsl"));

            let pipeline_layout =
                gpu_context
                    .device
                    .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                        label: Some("pipeline layout"),
                        bind_group_layouts: &[&time_bind_group_layout],
                        push_constant_ranges: &[],
                    });
            let render_pipeline = gpu_context.device.create_render_pipeline(
                &wgpu::RenderPipelineDescriptor {
                    label: Some("render pipeline"),
                    layout: Some(&pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &shader_module,
                        entry_point: Some("vs_main"),
                        compilation_options: Default::default(),
                        buffers: &[wgpu::VertexBufferLayout {
                            array_stride: size_of::<Vertex>() as u64,
                            step_mode: wgpu::VertexStepMode::Vertex,
                            attributes: &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3],
                        }],
                    },
                    primitive: wgpu::PrimitiveState {
                        topology: wgpu::PrimitiveTopology::TriangleList,
                        strip_index_format: None,
                        front_face: wgpu::FrontFace::Ccw,
                        cull_mode: None,
                        unclipped_depth: false,
                        polygon_mode: wgpu::PolygonMode::Line,
                        conservative: false,
                    },
                    depth_stencil: None,
                    multisample: Default::default(),
                    fragment: Some(wgpu::FragmentState {
                        module: &shader_module,
                        entry_point: Some("fs_main"),
                        compilation_options: Default::default(),
                        targets: &[Some(wgpu::ColorTargetState {
                            format: capabilities.formats[0],
                            blend: Some(wgpu::BlendState::REPLACE),
                            write_mask: wgpu::ColorWrites::ALL,
                        })],
                    }),
                    multiview: None,
                    cache: None,
                },
            );

            self.gpu_context = Some(gpu_context);
            self.window = Some(window);
            self.surface = Some(surface);
            self.render_pipeline = Some(render_pipeline);
            if let Some(window) = &self.window {
                window.request_redraw();
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            winit::event::WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            winit::event::WindowEvent::RedrawRequested => {
                self.render().unwrap();
            }
            winit::event::WindowEvent::Resized(size) => {
                self.resize(size);
            }
            _ => {}
        }
    }
}

pub fn demo() -> anyhow::Result<()> {
    let event_loop = winit::event_loop::EventLoop::new()?;
    let mut app = App::new();
    event_loop.run_app(&mut app)?;
    Ok(())
}
