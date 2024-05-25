mod inter;
mod gfx;
// mod render;
mod application;

use inter::mmio;

use pixels::{Error, Pixels, SurfaceTexture};
use winit::event_loop::{ControlFlow, EventLoop, ActiveEventLoop};

const VIEW_SCALE: u32 = 2;

fn main() {
    let mut application = application::Application::new();
    let event_loop = EventLoop::new().unwrap();

    event_loop.run_app(&mut application).unwrap();
}
