extern crate sdl2;
extern crate time;

#[macro_use]
mod macros;
mod cpu;
mod mem;
mod ppu;
mod nes;
mod gamepad;

fn main() {
 
   let mut nes : nes::Nes = nes::Nes::new();

   nes.run();
}

