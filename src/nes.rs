extern crate sdl2;

use cpu::Cpu;
use ppu::Ppu;
use mem::Memory as Mem;

//use sdl2::pixels::PixelFormatEnum;
//use sdl2::rect::Rect;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
//use sdl2::pixels::Color;
//use sdl2::video::{Window, WindowBuilder};
//use sdl2::rect::Point;
use sdl2::Sdl;


const WIDTH  : u32 = 256;
const HEIGHT : u32 = 240;


pub struct Nes {
    cpu         : Cpu,
    ppu         : Ppu,
    mem         : Mem,
    sdl_context : Sdl,
}

impl Nes {
    pub fn new () -> Nes {
        Nes {
            cpu         : Default::default(),
            ppu         : Ppu::new(),
            mem         : Mem::new(),
            sdl_context : sdl2::init().unwrap(),
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

        'running: loop {
            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit {..} 
                    | Event::KeyDown
                    { keycode: Some(Keycode::Escape), .. } =>  {
                                                                    break 'running
                                                                                },
                    _                                      =>  {}
                }
            }
        self.cpu.cycle(&mut self.mem);
        self.ppu.cycle(&mut self.mem, &mut renderer);
        self.ppu.cycle(&mut self.mem, &mut renderer);
        self.ppu.cycle(&mut self.mem, &mut renderer);
        }
    }
}
