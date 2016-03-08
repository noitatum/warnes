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
//use debug::Debug;

// std
use std::env;
use std::io::Error;

// sdl2
use sdl2::{EventPump, Sdl};
use sdl2::render::Renderer;

const WIDTH  : u32 = 256;
const HEIGHT : u32 = 240;

// hue
macro_rules! init_sdl { 
    ($msg:expr) => (
        let sdl_context = sdl2::init().ok().expect(&("Sdl context init_sdl()".to_string() + $msg)[..]);
        let video_subsystem = sdl_context.video().unwrap();
        let window = video_subsystem.window("RNES -----", WIDTH, HEIGHT)
                                    .position_centered()
                                    //.resizable() fullscreen lol
                                    .opengl()
                                    .build()
                                    .unwrap();

        let mut renderer : Renderer = window.renderer().build().unwrap();
        let mut event_pump = sdl_context.event_pump().unwrap();
    )
}


fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() > 3 {
       println!("Usage: rnes rom_file or rnes rom_file dbgi/dbgc"); 
    } else if args.len() == 3 {
        if args[2] == "dbgi" || args[2] == "dbg" || args[2] == "debug" {
            //let rom_file = &args[1];
            // let sdl_context = sdl2::init().ok().expect("Sdl context init_sdl() [dbg]");
            //let mut debug = Debug::load_rom(rom_file).expect("RNES main() [dbg]");
            //debug.run();
        }
    } else {
        let rom_file = &args[1];
        init_sdl!(" [nodbg]");
        // let (mut renderer, mut event_pump) = init_sdl().expect("SDL Init() in main() [nodbg]");
        let mut nes = Nes::load_rom(rom_file).expect("RNES main() [nodbg]");
        nes.run(&mut renderer, &mut event_pump);
    }

    println!("Exiting RNES.")
}


