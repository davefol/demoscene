use std::sync::Arc;

use clap::Args;
use nalgebra::Matrix4;
use wgpu::{RenderPassDescriptor, util::DeviceExt, wgc::identity};

use crate::{egui_renderer::EguiRenderer, gpu_context::GpuContext};

#[derive(Args)]
pub(crate) struct Opts {}

pub struct App<'a> {
    window: Option<Arc<winit::window::Window>>,
    gpu_context: Option<GpuContext>,
    surface: Option<wgpu::Surface<'a>>,
    surface_config: Option<wgpu::SurfaceConfiguration>,
    render_pipeline: Option<wgpu::RenderPipeline>,
    vertex_buffer: Option<wgpu::Buffer>,
    index_buffer: Option<wgpu::Buffer>,
    egui_renderer: Option<EguiRenderer>,
    indices_len: u32,
    transforms: Vec<Matrix4<f32>>,
    transform_buffer: Option<wgpu::Buffer>,
    transform_buffer_bindgroup: Option<wgpu::BindGroup>,
}

impl<'a> App<'a> {
    pub fn new() -> Self {
        App {
            window: None,
            gpu_context: None,
            surface: None,
            surface_config: None,
            render_pipeline: None,
            vertex_buffer: None,
            index_buffer: None,
            egui_renderer: None,
            indices_len: 0,
            transforms: vec![Matrix4::identity()],
            transform_buffer: None,
            transform_buffer_bindgroup: None,
        }
    }

    fn render(&mut self) -> anyhow::Result<()> {
        if let (
            Some(gpu_context),
            Some(render_pipeline),
            Some(surface),
            Some(surface_config),
            Some(vertex_buffer),
            Some(index_buffer),
            Some(egui_renderer),
            Some(window),
            Some(transform_buffer),
            Some(transform_buffer_bindgroup),
        ) = (
            &self.gpu_context,
            &self.render_pipeline,
            &self.surface,
            &mut self.surface_config,
            &self.vertex_buffer,
            &self.index_buffer,
            &mut self.egui_renderer,
            &self.window,
            &self.transform_buffer,
            &self.transform_buffer_bindgroup,
        ) {
            let surface_texture = match surface.get_current_texture() {
                Ok(s) => s,
                Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                    let size = window.inner_size();
                    surface_config.height = size.height;
                    surface_config.width = size.width;
                    surface.configure(&gpu_context.device, &surface_config);
                    surface.get_current_texture().unwrap()
                }
                Err(_) => return Ok(()),
            };

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
            let mut t = Matrix4::identity();
            for transform in self.transforms.iter().rev() {
                t = transform * t;
            }
            gpu_context
                .queue
                .write_buffer(transform_buffer, 0, bytemuck::cast_slice(t.as_slice()));
            render_pass.set_bind_group(0, transform_buffer_bindgroup, &[]);
            render_pass.draw_indexed(0..self.indices_len, 0, 0..1);
            drop(render_pass);

            let mut deleted = None;
            egui_renderer.frame(window, &view, &mut encoder, &surface_config, |ui| {
                if ui.button("Add").clicked() {
                    self.transforms.push(Matrix4::identity());
                }
                ui.vertical(|ui| {
                    for (transform_i, transform) in self.transforms.iter_mut().enumerate() {
                        ui.vertical(|ui| {
                            ui.horizontal(|ui| {
                                ui.label(format!("{}", transform_i));
                                if ui.button("X").clicked() {
                                    deleted = Some(transform_i);
                                }
                            });
                            ui.columns(4, |columns| {
                                columns[0].add(egui::DragValue::new(&mut transform.m11).speed(0.1));
                                columns[0].add(egui::DragValue::new(&mut transform.m21).speed(0.1));
                                columns[0].add(egui::DragValue::new(&mut transform.m31).speed(0.1));
                                columns[0].add(egui::DragValue::new(&mut transform.m41).speed(0.1));

                                columns[1].add(egui::DragValue::new(&mut transform.m12).speed(0.1));
                                columns[1].add(egui::DragValue::new(&mut transform.m22).speed(0.1));
                                columns[1].add(egui::DragValue::new(&mut transform.m32).speed(0.1));
                                columns[1].add(egui::DragValue::new(&mut transform.m42).speed(0.1));

                                columns[2].add(egui::DragValue::new(&mut transform.m13).speed(0.1));
                                columns[2].add(egui::DragValue::new(&mut transform.m23).speed(0.1));
                                columns[2].add(egui::DragValue::new(&mut transform.m33).speed(0.1));
                                columns[2].add(egui::DragValue::new(&mut transform.m43).speed(0.1));

                                columns[3].add(egui::DragValue::new(&mut transform.m14).speed(0.1));
                                columns[3].add(egui::DragValue::new(&mut transform.m24).speed(0.1));
                                columns[3].add(egui::DragValue::new(&mut transform.m34).speed(0.1));
                                columns[3].add(egui::DragValue::new(&mut transform.m44).speed(0.1));
                            });
                        });
                    }
                });
            });

            if let Some(deleted) = deleted {
                self.transforms.remove(deleted);
            }

            gpu_context.queue.submit(std::iter::once(encoder.finish()));
            surface_texture.present();
            window.request_redraw();
        }
        Ok(())
    }

    fn resize(&mut self, size: winit::dpi::PhysicalSize<u32>) {
        if size.width == 0 {
            return;
        }
        if let (Some(surface), Some(surface_config), Some(gpu_context)) =
            (&self.surface, &mut self.surface_config, &self.gpu_context)
        {
            surface_config.height = size.height;
            surface_config.width = size.width;
            surface.configure(&gpu_context.device, &surface_config);
        }
    }
}

impl<'a> winit::application::ApplicationHandler for App<'a> {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if self.window.is_none() {
            let window = event_loop.create_window(Default::default()).unwrap();
            let window = Arc::new(window);
            let gpu_context = GpuContext::new(wgpu::Features::empty()).unwrap();
            let surface = gpu_context.instance.create_surface(window.clone()).unwrap();
            let capabilities = surface.get_capabilities(&gpu_context.adapter);
            let surface_config = wgpu::SurfaceConfiguration {
                alpha_mode: capabilities.alpha_modes[0],
                desired_maximum_frame_latency: 2,
                format: capabilities.formats[0],
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                height: window.inner_size().height,
                width: window.inner_size().width,
                present_mode: capabilities.present_modes[0],
                view_formats: vec![],
            };
            surface.configure(&gpu_context.device, &surface_config);

            let cone = super::cone::Cone::default();
            let vertices = &cone.vertices;
            let indices = &cone.indices;
            self.indices_len = indices.len() as u32;

            self.vertex_buffer = Some(gpu_context.device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("vertices"),
                    usage: wgpu::BufferUsages::VERTEX,
                    contents: bytemuck::cast_slice(&vertices),
                },
            ));

            self.index_buffer = Some(gpu_context.device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("indices"),
                    usage: wgpu::BufferUsages::INDEX,
                    contents: bytemuck::cast_slice(&indices),
                },
            ));

            let transform_buffer = gpu_context.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("transforms"),
                size: (size_of::<f32>() * 4 * 4) as u64,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });

            let transform_bindgroup_layout =
                gpu_context
                    .device
                    .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                        label: Some("transform bindgroup layout"),
                        entries: &[wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::VERTEX,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        }],
                    });

            let transform_buffer_bindgroup =
                gpu_context
                    .device
                    .create_bind_group(&wgpu::BindGroupDescriptor {
                        label: Some("transforms bind group"),
                        layout: &transform_bindgroup_layout,
                        entries: &[wgpu::BindGroupEntry {
                            binding: 0,
                            resource: transform_buffer.as_entire_binding(),
                        }],
                    });

            let shader_module = gpu_context
                .device
                .create_shader_module(wgpu::include_wgsl!("demo.wgsl"));

            let pipeline_layout =
                gpu_context
                    .device
                    .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                        label: Some("pipeline layout"),
                        bind_group_layouts: &[&transform_bindgroup_layout],
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
                            array_stride: size_of::<super::cone::Vertex>() as u64,
                            step_mode: wgpu::VertexStepMode::Vertex,
                            attributes: &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3],
                        }],
                    },
                    primitive: wgpu::PrimitiveState {
                        topology: wgpu::PrimitiveTopology::TriangleList,
                        strip_index_format: None,
                        front_face: wgpu::FrontFace::Ccw,
                        cull_mode: Some(wgpu::Face::Back),
                        unclipped_depth: false,
                        polygon_mode: wgpu::PolygonMode::Fill,
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

            let egui_renderer = EguiRenderer::new(
                gpu_context.clone(),
                surface_config.format,
                None,
                1,
                &window,
                "Transforms",
            );

            self.gpu_context = Some(gpu_context);
            self.window = Some(window);
            self.surface = Some(surface);
            self.surface_config = Some(surface_config);
            self.render_pipeline = Some(render_pipeline);
            self.egui_renderer = Some(egui_renderer);
            self.transform_buffer = Some(transform_buffer);
            self.transform_buffer_bindgroup = Some(transform_buffer_bindgroup);

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
        if let (Some(egui_renderer), Some(window)) = (&mut self.egui_renderer, &self.window) {
            egui_renderer.handle_input(&window, &event);
        }
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
