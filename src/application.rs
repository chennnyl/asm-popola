use crate::render::context::RenderContext;
use crate::inter::mmio::{SCREEN_WIDTH, SCREEN_HEIGHT};
use crate::gfx::{Color, SpriteSize};

use pixels::{Pixels, SurfaceTexture};

use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{WindowEvent};
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

                let mut render_context = RenderContext::new(pixels);

                render_context.vrammodel.enable_sprite(0);
                render_context.vrammodel.enable_sprite(1);
                render_context.vrammodel.enable_sprite(2);
                render_context.vrammodel.enable_sprite(3);
                render_context.vrammodel.palettes[0].colors[0] = Color { r: 0, g: 0, b: 255 };

                render_context.vrammodel.sprites[1].location = (128, 0);
                render_context.vrammodel.sprites[1].properties.size = SpriteSize::X16;
                render_context.vrammodel.sprites[2].location = (0, 128);
                render_context.vrammodel.sprites[2].properties.size = SpriteSize::X32;
                render_context.vrammodel.sprites[3].location = (128, 128);
                render_context.vrammodel.sprites[3].properties.size = SpriteSize::X64;

                self.render_context = Some(render_context);
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