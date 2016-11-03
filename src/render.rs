use sdl2::pixels::Color;
use sdl2::pixels::Color::RGB;
use sdl2::rect::Point;
use sdl2::render::Renderer;
use ppu::{Scanline, SCANLINE_WIDTH};

const PALETTE : [Color; 0x40] = [
    RGB( 84, 84, 84), RGB(  0, 30,116), RGB(  8, 16,144), RGB( 48,  0,136),
    RGB( 68,  0,100), RGB( 92,  0, 48), RGB( 84,  4,  0), RGB( 60, 24,  0),
    RGB( 32, 42,  0), RGB(  8, 58,  0), RGB(  0, 64,  0), RGB(  0, 60,  0),
    RGB(  0, 50, 60), RGB(  0,  0,  0), RGB(  0,  0,  0), RGB(  0,  0,  0),
    RGB(152,150,152), RGB(  8, 76,196), RGB( 48, 50,236), RGB( 92, 30,228),
    RGB(136, 20,176), RGB(160, 20,100), RGB(152, 34, 32), RGB(120, 60,  0),
    RGB( 84, 90,  0), RGB( 40,114,  0), RGB(  8,124,  0), RGB(  0,118, 40),
    RGB(  0,102,120), RGB(  0,  0,  0), RGB(  0,  0,  0), RGB(  0,  0,  0),
    RGB(236,238,236), RGB( 76,154,236), RGB(120,124,236), RGB(176, 98,236),
    RGB(228, 84,236), RGB(236, 88,180), RGB(236,106,100), RGB(212,136, 32),
    RGB(160,170,  0), RGB(116,196,  0), RGB( 76,208, 32), RGB( 56,204,108),
    RGB( 56,180,204), RGB( 60, 60, 60), RGB(  0,  0,  0), RGB(  0,  0,  0),
    RGB(236,238,236), RGB(168,204,236), RGB(188,188,236), RGB(212,178,236),
    RGB(236,174,236), RGB(236,174,212), RGB(236,180,176), RGB(228,196,144),
    RGB(204,210,120), RGB(180,222,120), RGB(168,226,144), RGB(152,226,180),
    RGB(160,214,228), RGB(160,162,160), RGB(  0,  0,  0), RGB(  0,  0,  0),
];

pub fn render_frame(renderer: &mut Renderer, frame: &[Scanline]) {
    for (y, scanline) in frame.iter().enumerate() {
        for x in 0..SCANLINE_WIDTH {
            renderer.set_draw_color(PALETTE[scanline[x] as usize]);
            renderer.draw_point(Point::new(x as i32, y as i32)).unwrap();
        }
    }
    renderer.present();
}
