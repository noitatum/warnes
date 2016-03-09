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
mod enums;

// Nes
use nes::Nes;
use debug::Debug;

// std
use std::env;

// sdl2
use sdl2::render::Renderer;

const WIDTH  : u32 = 256;
const HEIGHT : u32 = 240;



fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() > 3 {
       println!("Usage: rnes rom_file or rnes rom_file dbgi/dbgc"); 
    } else{
        let sdl_context = sdl2::init().ok().expect("Sdl context init_sdl()");
        let video_subsystem = sdl_context.video().unwrap();
        let window = video_subsystem.window("RNES -----", WIDTH, HEIGHT)
                                    .position_centered()
                                    //.resizable() fullscreen lol
                                    .opengl()
                                    .build()
                                    .unwrap();

        let mut renderer : Renderer = window.renderer().build().unwrap();
        let mut event_pump = sdl_context.event_pump().unwrap(); 
        // get rom name
        let rom_file = &args[1];

        if args.len() == 3 {
            if args[2] == "dbgi" || args[2] == "dbg" || args[2] == "debug" {
                let mut debug = Debug::load_rom(rom_file, false).expect("RNES main() [dbg]");
                debug.run(&mut renderer, &mut event_pump);
            } else if args[2] == "dbgc" { // debug cycle per cycle
                let mut debug = Debug::load_rom(rom_file, true).expect("RNES main() [dbg cpc]");
                debug.run(&mut renderer, &mut event_pump);
            } else {
                panic!("dude");
            }
        } else {
            let mut nes = Nes::load_rom(rom_file).expect("RNES main() [nodbg]");
            nes.run(&mut renderer, &mut event_pump);
        }
    }
    println!("Exiting RNES.")
}


