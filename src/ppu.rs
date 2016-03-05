extern crate sdl2;

use utils::print_mem;
use loadstore::LoadStore;
use mem::{Memory as Mem, MemState};

use std::fmt;
use std::num::Wrapping as W;


use sdl2::pixels::PixelFormatEnum;
//use sdl2::rect::Rect;
//use sdl2::event::Event;
//use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
//use sdl2::video::{Window, WindowBuilder};
use sdl2::rect::{Point, Rect};


// ppuctrl
// Const values to access the controller register bits.
const CTRL_BASE_TABLE           : u8 = 0x03;
/* 0 = 0x2000 e incrementa de a 0x400,
 1 = 0x2400 etc. */
const CTRL_INCREMENT            : u8 = 0x04;
const CTRL_SPRITE_PATTERN       : u8 = 0x08;
const CTRL_BACKGROUND_PATTERN   : u8 = 0x10;
const CTRL_SPRITE_SIZE          : u8 = 0x20;
// trigger warning
const CTRL_PPU_SLAVE_MASTER     : u8 = 0x40;
const CTRL_GEN_NMI              : u8 = 0x80;

// ppu scroll coordinates
const COORDINATE_X              : u8 = 0x01;
const COORDINATE_Y              : u8 = 0x02;

// ppu mask
const MASK_GRAYSCALE            : u8 = 0x01;
const MASK_SHOW_BACKGROUND_LEFT : u8 = 0x02; // set = show bacgrkound in leftmost 8 pixels of screen
const MASK_SHOW_SPRITES_LEF     : u8 = 0x04; // set = show sprites in leftmost 8 pixels of screens
const MASK_SHOW_BACKGROUND      : u8 = 0x08;
const MASK_SHOW_SPRITES         : u8 = 0x10;
const MASK_EMPHASIZE_RED        : u8 = 0x20;
const MASK_EMPHASIZE_GREEN      : u8 = 0x40;
const MASK_EMPHASIZE_BLUE       : u8 = 0x80;

// ppu status
const STATUS_SPRITE_OVERFLOW    : u8 = 0x20;
const STATUS_SPRITE_0_HIT       : u8 = 0x40;
const STATUS_VERTICAL_BLANK     : u8 = 0x80; // set = in vertical blank


const VBLANK_END                : u32 = 27901; 

pub struct Ppu {
    oam             : Oam,
    vram            : AddressLatch, 
    scroll          : AddressLatch,
    cycles          : u32,

    ctrl            : W<u8>,
    mask            : W<u8>,
    status          : u8,

    px_height       : usize,
    px_width        : usize,
    
    fps             : u32,
}

impl Ppu {
    pub fn new () -> Ppu {
        Ppu {
            oam             : Oam::default(), 
            vram            : AddressLatch::default(),
            scroll          : AddressLatch::default(),
            cycles          : 0,


            // Registers, some may be removed later.
            ctrl            : W(0),
            mask            : W(0),
            status          : 0,

            px_height       : 0,
            px_width        : 0,

            fps             : 0,
        }
    }
    
    pub fn cycle(&mut self, memory: &mut Mem, renderer: &mut sdl2::render::Renderer) {
        self.ls_latches(memory);

        if self.cycles == 0 {
            self.draw(memory, renderer);
        } else {
            self.cycles += 1;
        }

        if self.cycles == VBLANK_END {
            self.cycles = 0;
            self.fps += 1;
        } 
    }

    fn draw(&mut self, memory: &mut Mem, renderer: &mut sdl2::render::Renderer) {
        renderer.set_draw_color(Color::RGB(self.px_height as u8, self.px_width as u8, 20));
        renderer.draw_point(Point::new(self.px_width as i32, self.px_height as i32)).unwrap();
        if self.px_width == 255 && self.px_height < 239 {
            self.px_width = 0;
            self.px_height += 1;
        } else if self.px_width == 255 && self.px_height == 239 {
            // Once entire image is draw we present the result and start counting until the next
            // vblank
            renderer.present();
            self.px_width = 0;
            self.px_height = 0;
            self.cycles += 1;
        } else {
            self.px_width += 1;
        }
    }

    #[inline(always)]
    pub fn print_fps(&mut self) {
        println!("fps: {}", self.fps);
        self.fps = 0;
    }

    /* load store latches */
    fn ls_latches(&mut self, memory: &mut Mem){
        let (latch, status) = memory.get_latch();
        match status {
            MemState::PpuCtrl   => { self.ctrl = latch; }, 
            MemState::PpuMask   => { self.mask = latch; },
            MemState::OamAddr   => { self.oam.set_addr(latch); },
            MemState::OamData   => { self.oam.store_data(latch); },
            MemState::PpuScroll => { self.scroll.set(latch); },
            MemState::PpuAddr   => { self.vram.set(latch); },
            MemState::PpuData   => { memory.chr_store(self.vram.get(), latch);},
            _                   => {}, 
        }

        let read_status = memory.get_mem_load_status();

        match read_status {
            MemState::PpuStatus => {
                self.vram.reset();
                self.scroll.reset();
                self.status &= 0x60;
            },
            MemState::PpuData   => { 
                let value = memory.chr_load(self.vram.get()); 
                memory.set_latch(value);
            },
            _                   => {},
        }
    }
}

impl Default for Ppu {
    fn default () -> Ppu {
        Ppu::new()
    }
}


impl fmt::Debug for Ppu {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "PPU: \n OAM: {:?}, scroll: {:?}, vram: {:?}", 
               self.oam, self.scroll, self.vram)
    }
}

#[derive(Default)]
struct AddressLatch {
    laddr   : W<u8>,
    haddr   : W<u8>,
    upper   : bool,
}


impl AddressLatch {
    pub fn reset(&mut self) {
        *self = AddressLatch::default();
    }

    pub fn get(&self) -> W<u16> {
        W16!(self.haddr) << 8 | W16!(self.laddr)
    }

    pub fn set(&mut self, value: W<u8>) {
        if self.upper {
            self.haddr = value;
        } else {
            self.laddr = value;
        }
        self.upper = !self.upper;
    }
}

impl fmt::Debug for AddressLatch {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.get())
    }
}

struct Oam {
    mem     : [u8; 0x100],
    addr    : W<u8>,
}

impl Default for Oam {
    fn default() -> Oam {
        Oam {
            mem  : [0; 0x100],
            addr : W(0),
        }
    }
}

impl fmt::Debug for Oam {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut output = "OAM: mem: \n".to_string();
        print_mem(&mut output, &self.mem[..]);
        write!(f, "{}, addr: {:#x}", output, self.addr.0)
    }
}

impl Oam {

    #[inline]
    fn store_data(&mut self, value: W<u8>) {
        self.mem[self.addr.0 as usize] = value.0;
        self.addr = self.addr + W(1);
    }
    
    #[inline]
    fn set_addr(&mut self, value: W<u8>) {
        self.addr = value;
    }
}

impl LoadStore for Oam {

    #[inline]
    fn load(&mut self, address: W<u16>) -> W<u8> {
       W(self.mem[address.0 as usize])
    }

    #[inline]
    fn store(&mut self, address: W<u16>, value: W<u8>) { 
       self.mem[address.0 as usize] = value.0;
    }
}
