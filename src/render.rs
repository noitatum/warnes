use sdl2::render::Renderer;
use sdl2::surface::Surface;
use sdl2::pixels::PixelFormatEnum;
use ppu::{Scanline, SCANLINE_WIDTH, SCANLINE_COUNT};

const PIXEL_BYTES : usize = 3;
// RGB NES Palette
const PALETTE : [[u8; 3]; 0x40] = [
    [ 84, 84, 84], [  0, 30,116], [  8, 16,144], [ 48,  0,136],
    [ 68,  0,100], [ 92,  0, 48], [ 84,  4,  0], [ 60, 24,  0],
    [ 32, 42,  0], [  8, 58,  0], [  0, 64,  0], [  0, 60,  0],
    [  0, 50, 60], [  0,  0,  0], [  0,  0,  0], [  0,  0,  0],
    [152,150,152], [  8, 76,196], [ 48, 50,236], [ 92, 30,228],
    [136, 20,176], [160, 20,100], [152, 34, 32], [120, 60,  0],
    [ 84, 90,  0], [ 40,114,  0], [  8,124,  0], [  0,118, 40],
    [  0,102,120], [  0,  0,  0], [  0,  0,  0], [  0,  0,  0],
    [236,238,236], [ 76,154,236], [120,124,236], [176, 98,236],
    [228, 84,236], [236, 88,180], [236,106,100], [212,136, 32],
    [160,170,  0], [116,196,  0], [ 76,208, 32], [ 56,204,108],
    [ 56,180,204], [ 60, 60, 60], [  0,  0,  0], [  0,  0,  0],
    [236,238,236], [168,204,236], [188,188,236], [212,178,236],
    [236,174,236], [236,174,212], [236,180,176], [228,196,144],
    [204,210,120], [180,222,120], [168,226,144], [152,226,180],
    [160,214,228], [160,162,160], [  0,  0,  0], [  0,  0,  0],
];

pub fn render_frame(renderer: &mut Renderer, frame: &[Scanline]) {
    let (w, h) = (SCANLINE_WIDTH as u32, SCANLINE_COUNT as u32);
    let mut pixels = [0u8; SCANLINE_WIDTH * SCANLINE_COUNT * PIXEL_BYTES];
    for y in 0..SCANLINE_COUNT {
        for x in 0..SCANLINE_WIDTH {
            let index = (y * SCANLINE_WIDTH + x) * PIXEL_BYTES;
            let color = PALETTE[frame[y][x] as usize];
            pixels[index + 0] = color[0];
            pixels[index + 1] = color[1];
            pixels[index + 2] = color[2];
        }
    }
    let surface = Surface::from_data(&mut pixels, w, h, w * PIXEL_BYTES as u32,
                                     PixelFormatEnum::RGB24).unwrap();
    let texture = renderer.create_texture_from_surface(surface).unwrap();
    renderer.copy(&texture, None, None).unwrap();
    renderer.present();
}
