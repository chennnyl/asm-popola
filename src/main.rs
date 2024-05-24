mod inter;
mod gfx;
mod render;

use inter::mmio;
use sdl2::event::Event;
use sdl2::pixels::Color;

const VIEW_SCALE: u32 = 2;

fn main() {
    let sdl = sdl2::init().unwrap();
    let video = sdl.video().unwrap();

    let window = video.window("Popola", mmio::SCREEN_WIDTH*VIEW_SCALE, mmio::SCREEN_HEIGHT*VIEW_SCALE)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    canvas.set_logical_size(mmio::SCREEN_WIDTH, mmio::SCREEN_HEIGHT).unwrap();

    canvas.set_draw_color(Color::BLACK);
    canvas.clear();
    canvas.present();
    let mut event_pump = sdl.event_pump().unwrap();
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} => { break 'running }
                _ => {}
            }
        }
        canvas.present();
    }
}
