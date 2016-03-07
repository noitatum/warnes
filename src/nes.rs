extern crate sdl2;
extern crate time;

// STD
use std::io::Error;
use std::path::Path;

// Custom stuff
use header::Header;
use cpu::Cpu;
use ppu::Ppu;
use mem::Memory as Mem;
use controller::Controller;

// Time
use time::PreciseTime;

// SDL2
use sdl2::event::Event;
use sdl2::keyboard::{Keycode};
use sdl2::Sdl;

const WIDTH  : u32 = 256;
const HEIGHT : u32 = 240;


pub struct Nes {
    cpu         : Cpu,
    ppu         : Ppu,
    mem         : Mem,
    sdl_context : Sdl,
    controller  : Controller,
}

impl Nes {
    pub fn load_rom<P: AsRef<Path>>(path: P) -> Result<Nes, Error> {
        let mapper = try!(try!(Header::load_rom(path)).get_mapper());
        Ok (
            Nes {
                cpu         : Default::default(),
                ppu         : Ppu::new(),
                mem         : Mem::new(mapper),
                sdl_context : sdl2::init().unwrap(),
                controller  : Controller::new(),
            }
        )
    }
}


impl Nes {
    pub fn run(&mut self) {
        let video_subsystem = self.sdl_context.video().unwrap();
        let window  =  video_subsystem.window("RNES -----", WIDTH, HEIGHT)
                            .position_centered()
                            //.resizable() fullscreen lol
                            .opengl()
                            .build()
                            .unwrap();

        let mut renderer = window.renderer().build().unwrap();
        let mut event_pump = self.sdl_context.event_pump().unwrap();

        let mut echo = PreciseTime::now();

        'nes: loop {
            if echo.to(PreciseTime::now()) > time::Duration::seconds(1) {
                self.ppu.print_fps();
                echo = PreciseTime::now();
                for event in event_pump.poll_iter() {
                    match event {
                        Event::Quit {..} | Event::KeyDown
                        { keycode: Some(Keycode::Escape), .. } =>  { break 'nes },
                        _                                      =>  { },
                    }
                }
            }
            // Does a full cpu cycle (includes 3 ppu cycles)
            self.cycle(&mut renderer, &mut event_pump);
        }
    }

    // Runs a full instruction (all cycles needed)
    // Or executes a full CPU cycle (3ppu cycles),
    /*pub fn step(&mut self, complete_inst: bool) {
        if complete_inst {
            let cycle self.cpu.next_instr_cycles();
            // We do enough cycles to finish the instruction
            for _ in 0..cycle {
                self.cycle();  
            }
        } else{
            self.cycle();
        }
    }*/

    // This function does a complete CPU cycle
    // Including joy I/O and 3 PPU cycles.
    #[inline(always)]
    pub fn cycle(&mut self, renderer: &mut sdl2::render::Renderer, event_pump: &mut sdl2::EventPump) {
        self.controller.push_keys(&mut self.mem, event_pump);
        self.cpu.cycle(&mut self.mem);
        self.ppu.cycle(&mut self.mem, renderer);
        self.ppu.cycle(&mut self.mem, renderer);
        self.ppu.cycle(&mut self.mem, renderer);
    }
}
