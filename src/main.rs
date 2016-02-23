extern crate sdl2;

#[macro_use]
mod macros;
mod cpu;
mod mem;
mod ppu;
mod nes;

use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::video::{Window, WindowBuilder};
use sdl2::rect::Point;

const WIDTH  : u32 = 256;
const HEIGHT : u32 = 240;


fn main() {
 
   let mut nes : nes::Nes = nes::Nes::new();

   nes.run();
}

