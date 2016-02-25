extern crate sdl2;

#[macro_use]
mod macros;
mod cpu;
mod mem;
mod ppu;
mod nes;
mod joy;

fn main() {
 
   let mut nes : nes::Nes = nes::Nes::new();

   nes.run();
}

