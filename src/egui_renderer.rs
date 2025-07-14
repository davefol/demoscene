use crate::gpu_context::GpuContext;

pub(crate) struct EguiRenderer {
    state: egui_winit::State,
    renderer: egui_wgpu::Renderer,
    gpu_context: GpuContext,
    frame_started: bool,
    title: &'static str,
}

impl EguiRenderer {
    pub fn egui_context(&self) -> &egui::Context {
        self.state.egui_ctx()
    }

    pub fn new(
        gpu_context: GpuContext,
        output_color_format: wgpu::TextureFormat,
        output_depth_format: Option<wgpu::TextureFormat>,
        msaa_samples: u32,
        window: &winit::window::Window,
        title: &'static str,
    ) -> Self {
        let egui_context = egui::Context::default();
        let egui_state = egui_winit::State::new(
            egui_context,
            egui::viewport::ViewportId::ROOT,
            &window,
            Some(window.scale_factor() as f32),
            None,
            Some(2 * 1024),
        );
        let egui_renderer = egui_wgpu::Renderer::new(
            &gpu_context.device,
            output_color_format,
            output_depth_format,
            msaa_samples,
            true,
        );

        Self {
            state: egui_state,
            renderer: egui_renderer,
            gpu_context,
            frame_started: false,
            title,
        }
    }

    pub fn handle_input(
        &mut self,
        window: &winit::window::Window,
        event: &winit::event::WindowEvent,
    ) {
        let _ = self.state.on_window_event(window, event);
    }

    pub fn ppp(&mut self, v: f32) {
        self.egui_context().set_pixels_per_point(v);
    }

    pub fn begin_frame(&mut self, window: &winit::window::Window) {
        let raw_input = self.state.take_egui_input(window);
        self.state.egui_ctx().begin_pass(raw_input);
        self.frame_started = true;
    }

    pub fn end_frame_and_draw(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        window: &winit::window::Window,
        window_surface_view: &wgpu::TextureView,
        screen_descriptor: egui_wgpu::ScreenDescriptor,
    ) {
        if !self.frame_started {
            panic!("frame not started");
        }

        self.ppp(screen_descriptor.pixels_per_point);

        let full_output = self.state.egui_ctx().end_pass();
        self.state
            .handle_platform_output(window, full_output.platform_output);

        let tris = self
            .state
            .egui_ctx()
            .tessellate(full_output.shapes, self.state.egui_ctx().pixels_per_point());
        for (id, image_delta) in &full_output.textures_delta.set {
            self.renderer.update_texture(
                &self.gpu_context.device,
                &self.gpu_context.queue,
                *id,
                image_delta,
            );
        }
        self.renderer.update_buffers(
            &self.gpu_context.device,
            &self.gpu_context.queue,
            encoder,
            &tris,
            &screen_descriptor,
        );
        let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("egui main render pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: window_surface_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        self.renderer.render(
            &mut render_pass.forget_lifetime(),
            &tris,
            &screen_descriptor,
        );
        for x in &full_output.textures_delta.free {
            self.renderer.free_texture(x);
        }
        self.frame_started = false;
    }

    pub fn frame<R>(
        &mut self,
        window: &winit::window::Window,
        view: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
        surface_config: &wgpu::SurfaceConfiguration,
        show_fn: impl FnOnce(&mut egui::Ui) -> R,
    ) {
        self.begin_frame(window);
        let screen_descriptor = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [surface_config.width, surface_config.height],
            pixels_per_point: window.scale_factor() as f32,
        };
        egui::Window::new(self.title)
            .resizable(true)
            .default_open(true)
            .show(self.egui_context(),show_fn);
        self.end_frame_and_draw(encoder, window, &view, screen_descriptor);
    }
}
