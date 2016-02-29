extern crate sdl2;
extern crate time;

#[macro_use]
mod macros;
mod cpu;
mod mem;
mod ppu;
mod nes;
mod gamepad;
mod header;
mod loadstore;
mod utils;

fn main() {
 
   let mut nes : nes::Nes = nes::Nes::new();

   nes.run();
}

