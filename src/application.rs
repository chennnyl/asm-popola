use crate::inter::mmio;
use crate::render::context::RenderContext;
use mmio::{SCREEN_WIDTH, SCREEN_HEIGHT};

use pixels::{Error, Pixels, SurfaceTexture};

use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{Event, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowId};

const VIEW_SCALE: u32 = 2;

pub struct Application {
    window: Option<Window>,
    render_context: Option<RenderContext>
}

impl Application {
    pub fn new() -> Self {
        Self {
            window: None,
            render_context: None
        }
    }

    fn create_window(&mut self, event_loop: &ActiveEventLoop) {
        let size = LogicalSize::new(SCREEN_WIDTH, SCREEN_HEIGHT);
        let scaled_size = LogicalSize::new(VIEW_SCALE*SCREEN_WIDTH, VIEW_SCALE*SCREEN_HEIGHT);

        let window_attributes = Window::default_attributes()
            .with_title("Popola")
            .with_inner_size(scaled_size)
            .with_min_inner_size(size)
            .with_resizable(false);

        let window = event_loop.create_window(window_attributes).unwrap();

        self.window = Some(window);
    }
}

impl ApplicationHandler for Application {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // initialize window on first Resume event
        match self.window.as_ref() {
            Some(_) => return,
            None => {
                self.create_window(event_loop);

                let window = self.window.as_ref().unwrap();
                let inner_size = window.inner_size();
                let surface_texture = SurfaceTexture::new(inner_size.width, inner_size.height, window);

                let pixels = Pixels::new(SCREEN_WIDTH, SCREEN_HEIGHT, surface_texture).unwrap();

                self.render_context = Some(RenderContext::new(pixels));
            }
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent) {
        let window = match self.window.as_ref() {
            Some(window) => window,
            None => return
        };
        let render_context = match self.render_context.as_mut() {
            Some(render_context) => render_context,
            None => return
        };

        match event {
            WindowEvent::CloseRequested  => event_loop.exit(),
            WindowEvent::RedrawRequested => {
                render_context.render();
                window.request_redraw();
            },
            _ => ()
        }

    }
}