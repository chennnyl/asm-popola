use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop, ActiveEventLoop};
use winit::window::{Window, WindowId};
use winit_input_helper::WinitInputHelper;

pub struct Application {
    window: Option<Window>
}

impl Application {
    pub fn new() -> Self {
        Self {
            window: None
        }
    }

    fn create_window(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = Window::default_attributes()
            .with_title("Popola")
            .with_resizable(false);

        let window = event_loop.create_window(window_attributes).unwrap();

        self.window = Some(window);
    }
}

impl ApplicationHandler for Application {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.create_window(event_loop);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, window_id: WindowId, event: WindowEvent) {
        println!("{event:?}");

        let window = match self.window.as_ref() {
            Some(window) => window,
            None => return
        };

        match event {
            WindowEvent::CloseRequested  => event_loop.exit(),
            WindowEvent::RedrawRequested => {
                window.request_redraw();
            },
            _ => ()
        }

    }
}