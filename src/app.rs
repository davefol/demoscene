use winit::event;

pub struct App {
    window: Option<winit::window::Window>,
}

impl App {
    pub fn new() -> Self {
        App {
            window: None,
        }
    }
}

impl winit::application::ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if self.window.is_none() {
            let window = event_loop.create_window(Default::default()).unwrap();
            self.window = Some(window);
        }
    }

    fn window_event(
            &mut self,
            event_loop: &winit::event_loop::ActiveEventLoop,
            window_id: winit::window::WindowId,
            event: event::WindowEvent,
        ) {

            match event {
                winit::event::WindowEvent::CloseRequested => {
                    event_loop.exit();
                },
                _ => {}
            }
        
    }
}
