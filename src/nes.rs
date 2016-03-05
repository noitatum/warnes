extern crate sdl2;
extern crate time;


// Custom stuff
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
    pub fn new () -> Nes {
        Nes {
            cpu         : Default::default(),
            ppu         : Ppu::new(),
            mem         : Mem::new(),
            sdl_context : sdl2::init().unwrap(),
            controller  : Controller::new(),
        }
    }
}


impl Nes {
    pub fn run(&mut self) {
    
        let video_subsystem = self.sdl_context.video().unwrap();
        let  window  =  video_subsystem.window("RNES -----", WIDTH, HEIGHT)
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

            self.controller.push_keys(&mut self.mem, &mut event_pump);
            self.cpu.cycle(&mut self.mem);
            self.ppu.cycle(&mut self.mem, &mut renderer);
            self.ppu.cycle(&mut self.mem, &mut renderer);
            self.ppu.cycle(&mut self.mem, &mut renderer);
        }
    }
}
