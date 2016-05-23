extern crate sdl2;
extern crate time;

// STD
use std::io::Error;
use std::path::Path;

// Custom stuff
use header::Header;
use cpu::{Cpu, Regs};
use ppu::Ppu;
use mem::Memory as Mem;
use controller::Controller;

// Time
use time::PreciseTime;

// SDL2
use sdl2::event::Event;
use sdl2::keyboard::{Keycode};
use sdl2::EventPump;
use sdl2::render::Renderer;

pub struct Nes  {
    cpu         : Cpu,
    ppu         : Ppu,
    mem         : Mem,
    //renderer    : Renderer,
    //event_pump  : EventPump,
    //sdl_context : Sdl,
    controller  : Controller,
}

impl Nes {
    pub fn load_rom<P: AsRef<Path>> (path: P) -> Result<Nes, Error>     {
        let mapper = try!(try!(Header::load_rom(path)).get_mapper());
        Ok (
            Nes {
                cpu         : Default::default(),
                ppu         : Ppu::new(),
                mem         : Mem::new(mapper),
                //renderer    : 
                //event_pump  : 
                controller  : Controller::new(),
            }
        )
    }
}


impl Nes  {
    pub fn run(&mut self, renderer: &mut Renderer, event_pump: &mut EventPump) {
/*      let video_subsystem = self.sdl_context.video().unwrap();
        let window  =  video_subsystem.window("RNES -----", WIDTH, HEIGHT)
                            .position_centered()
                            //.resizable() fullscreen lol
                            .opengl()
                            .build()
                            .unwrap();

        let mut renderer = window.renderer().build().unwrap();
        let mut event_pump = self.sdl_context.event_pump().unwrap();
*/
        let mut time = PreciseTime::now();

        self.cpu.reset(&mut self.mem);

        'nes: loop {
            if time.to(PreciseTime::now()) > time::Duration::seconds(1) {
                time = PreciseTime::now();
                self.ppu.print_fps();
                for event in event_pump.poll_iter() {
                    match event {
                        Event::Quit {..} | Event::KeyDown
                        { keycode: Some(Keycode::Escape), .. } =>  { break 'nes },
                        _                                      =>  { },
                    }
                }
            }
            // Does a full cpu cycle (includes 3 ppu cycles)
            self.cycle(renderer, event_pump);
        }
    }

    // This function does a complete CPU cycle
    // Including joy I/O and 3 PPU cycles.
    #[inline(always)]
    pub fn cycle(&mut self, renderer: &mut Renderer, event_pump: &mut EventPump) {
        self.controller.push_keys(&mut self.mem, event_pump);
        self.cpu.cycle(&mut self.mem);
        self.ppu.cycle(&mut self.mem, renderer);
        self.ppu.cycle(&mut self.mem, renderer);
        self.ppu.cycle(&mut self.mem, renderer);
    }

    pub fn reset(&mut self) {
        self.cpu.reset(&mut self.mem);
    }
}

// Debug stuff
impl Nes {
    pub fn cpu_registers(&self) -> Regs {
        self.cpu.registers()
    }

    pub fn memory(&mut self) -> &mut Mem {
        &mut self.mem
    } 
}
