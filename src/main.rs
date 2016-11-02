extern crate sdl2;
extern crate time;

#[macro_use]
mod macros;
mod cpu;
mod mem;
mod ppu;
mod scroll;
mod nes;
mod controller;
mod header;
mod loadstore;
mod utils;
mod mapper;
mod debug;
mod enums;
mod render;
mod input;
mod test;

// std
use std::env;
// Nes
use nes::Nes;
// input
use input::get_keys;
// Time
use time::PreciseTime;
// Render
use render::render_frame;

const WIDTH  : u32 = 256;
const HEIGHT : u32 = 240;

fn rnes() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 || args.len() > 3 {
       return err!("Invalid parameter count");
    }
    let sdl_context = sdl2::init().expect("Sdl context init_sdl()");
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem.window("RNES -----", WIDTH, HEIGHT)
                                .position_centered().opengl().build().unwrap();
    let mut renderer = window.renderer().build().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();
    let mut nes = try!(Nes::new(&args[1]));
    if args.len() == 3 {
        if args[2] == "debug" {
            debug::run(&mut nes);
        } else {
            return err!("Invalid parameter {}", args[2]);
        }
    } else {
        let mut keys = [[0u8; 8]; 2];
        let (mut frame, mut last_frame) = (0u64, 0u64);
        let mut time = PreciseTime::now();
        nes.reset();
        'nes: loop {
            if time.to(PreciseTime::now()) > time::Duration::seconds(1) {
                time = PreciseTime::now();
                println!("FPS: {}", frame - last_frame);
                last_frame = frame;
            }
            {
                let (number, data) = nes.ppu().frame_data();
                if frame != number {
                    frame = number;
                    render_frame(&mut renderer, data);
                    if get_keys(&mut event_pump, &mut keys) {
                        break 'nes;
                    }
                }
            }
            nes.set_keys(&keys);
            // Does a full cpu cycle (includes 3 ppu cycles)
            nes.cycle();
        }
    }
    Ok(())
}

fn main() {
    match rnes() {
        Ok(()) => {
            println!("Exiting RNES.");
            std::process::exit(0);
        },
        Err(err) => {
            println!("Error: {}", err);
            println!("Usage: rnes ROM_FILE [debug]");
            std::process::exit(1);
        },
    };
}
