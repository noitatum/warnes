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
mod dma;
mod debug;

// Nes
use nes::Nes;
use debug::Debug;

// std
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() > 3 {
       println!("Usage: rnes rom_file or rnes rom_file dbgi/dbgc"); 
    } else if args.len() == 3 {
        if args[2] == "dbgi" || args[2] == "dbg" || args[2] == "debug" {
            let rom_file = &args[1];
            let mut debug = Debug::load_rom(rom_file).expect("RNES debug()");
            debug.run();
        }
    } else {
        let rom_file = &args[1];
        let mut nes = Nes::load_rom(rom_file).expect("RNES main()");
        nes.run();
    }
}

