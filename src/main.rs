extern crate sdl2;
extern crate time;

#[macro_use]
mod macros;
mod cpu;
mod mem;
mod ppu;
mod nes;
mod controller;
mod header;
mod loadstore;
mod utils;
mod mapper;

use nes::Nes;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
       println!("Usage: rnes rom_file"); 
    } else {
        let rom_file = &args[1];
        let mut nes = Nes::new_from_file(rom_file).expect("RNES main()");
        nes.run();
    }
}

