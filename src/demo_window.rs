use winit::event_loop::EventLoop;

struct App {
    window: Option<winit::window::Window>
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
            _window_id: winit::window::WindowId,
            event: winit::event::WindowEvent,
        ) {
            match event {
                winit::event::WindowEvent::CloseRequested => {
                    event_loop.exit();
                },
                _ => {}
            }
        
    }

}

pub fn demo() -> anyhow::Result<()> {
    let event_loop = EventLoop::new()?;
    let mut app = App {
        window: None,
    };
    event_loop.run_app(&mut app)?;
    Ok(())
}