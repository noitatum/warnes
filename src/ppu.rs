extern crate sdl2;

use std::fmt;
use mem::{Memory as Mem, MemState};
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


const VBLANK_END                : u32 = 6819; 

pub struct Ppu {

    oam             : [u8; 256],    // Object atribute memory 
    vram            : [u8; 0x4000], // 16kb    
    cycles          : u32,

    // status
    vram_address    : u16,
    scroll_address  : u16,
    upper_vram      : bool,         // The CPU writes two bytes consecutively to ppuaddr to write a 16b address for addressing the vram
    upper_scroll    : bool,         // If upper is set this mean we get the higher byte.

    ppuctrl         : u8,
    ppumask         : u8,
    ppustatus       : u8,
    oamaddr         : u8,
    ppuscroll       : u8,
    ppuaddr         : u8,
    ppudata         : u8,
    oamdma          : u8,

    px_height       : usize,
    px_width        : usize,
}

impl Ppu {
    pub fn new () -> Ppu {
        Ppu {
            oam             : [0; 256],
            vram            : [0;  0x4000],
            cycles          : 0,

            vram_address    : 0,
            scroll_address  : 0,
            upper_vram      : true,
            upper_scroll    : true,

            // Registers, some may be removed later.
            ppuctrl         : 0,
            ppumask         : 0,
            ppustatus       : 0,
            oamaddr         : 0,
            ppuscroll       : 0,
            ppuaddr         : 0,
            ppudata         : 0,
            oamdma          : 0,

            px_height       : 0,
            px_width        : 0,
        }
    }
    
    #[inline(always)]
    pub fn cycle(&mut self, memory: &mut Mem, renderer: &mut sdl2::render::Renderer ) {
        self.ls_latches(memory);

        if self.cycles == 0 {
            self.draw(memory, renderer);
        } else {
            self.cycles += 1;
        }

        if self.cycles == VBLANK_END {
            self.cycles = 0;
            println!("f");
        } 
    }

    fn draw(&mut self, memory: &mut Mem, renderer: &mut sdl2::render::Renderer) {
        // buffers the points and their color in a 256x240 matrix
        //
        // Point = (x, y) = (width, height) !!.
        //self.buffer[self.px_height][self.px_width] = (Point::new(self.px_width as i32, self.px_height as i32), Color::RGB(self.px_height as u8, self.px_width as u8, 20));
        renderer.set_draw_color(Color::RGB(self.px_height as u8, self.px_width as u8, 20));
        renderer.draw_point(Point::new(self.px_width as i32, self.px_height as i32));
        if self.px_width == 255 && self.px_height < 239 {
            self.px_width = 0;
            self.px_height+= 1;
        } else if self.px_width == 255 && self.px_height == 239 {
            renderer.present();
            self.px_width = 0;
            self.px_height = 0;
            self.cycles += 1;
        } else {
            self.px_width += 1;
        }
    }
    
    /* load store latches */
    fn ls_latches(&mut self, memory: &mut Mem){
        match memory.write_status {
            MemState::PpuMask   =>  {   self.ppumask = memory.ppumask;
                                        memory.write_status = MemState::NoState;
                                    },

            MemState::OamAddr   =>  {   self.oamaddr = memory.oamaddr;
                                        memory.write_status = MemState::NoState;
                                    },

            MemState::OamData   =>  {   self.oam[self.oamaddr as usize] = memory.oamdata;
                                        self.oamaddr += 1;
                                        memory.write_status = MemState::NoState;
                                    },

            MemState::PpuScroll =>  {   if self.upper_scroll {
                                            self.upper_scroll   = !self.upper_scroll;
                                            self.scroll_address = (memory.ppuscroll as u16) << 8;
                                        } else {
                                            self.upper_scroll   = !self.upper_scroll;
                                            self.scroll_address = memory.ppuaddr as u16;
                                        }
                                        memory.write_status = MemState::NoState;
                                    },

            MemState::PpuAddr   =>  {   if self.upper_vram {
                                            self.upper_vram = false;
                                            self.vram_address = (memory.ppuaddr as u16) << 8;
                                        } else {
                                            self.upper_vram = true;
                                            self.vram_address |= memory.ppuaddr as u16;
                                        }
                                        memory.write_status = MemState::NoState;
                                    },

            MemState::PpuData   =>  {   self.vram[self.vram_address as usize] = memory.ppudata;
                                        memory.write_status = MemState::NoState;
                                    },
            MemState::NoState =>    (),
            _ => (), // do something probably update internal registers.
        }

        match memory.read_status {
            MemState::PpuStatus =>  {   memory.read_status  = MemState::NoState;
                                    },

            MemState::PpuData   =>  {   memory.ppudata = self.vram[self.vram_address as usize];
                                        memory.read_status  = MemState::NoState;
                                    },

            MemState::NoState   =>  (),
            _ => (),
        }

    }

    pub fn load_oam (&self, address: W<u16>) -> W<u8> {
       W(self.oam[address.0 as usize])
    }

    pub fn store_oam (&mut self, address: W<u16>, value: W<u8> ){ 
       self.oam[address.0 as usize] = value.0;
    }

    pub fn load_word_oam (&self, address: W<u16>) -> W<u16> {
       let low : W<u16> = W16!(self.load_oam(address));
       low | W16!(self.load_oam(address + W(1)))
    }

    pub fn store_word_oam (&mut self, address: W<u16>,  word: W<u16>){ 
        self.store_oam(address, W8!(word >> 8));
        self.store_oam(address + W(1), W8!(word));
    }


    pub fn load_vram (&self, address: W<u16>) -> W<u8> {
        W(if address.0 < 0x3000 {
            self.vram[address.0 as usize]
        }else if address.0 < 0x3F00 {
            self.vram[(address.0 - 0x1000) as usize]
        }else if address.0 < 0x3F20 {
            self.vram[address.0 as usize]
        }else if address.0 < 0x4000 {
            self.vram[(address.0 - 0x100) as usize]
        }else {
            self.vram[(address.0 % 0x4000) as usize]
        })
    }

    pub fn store_vram (&mut self, address: W<u16>, value: W<u8>){
        if address.0 < 0x3000 {
            self.vram[address.0 as usize] = value.0;
        }else if address.0 < 0x3F00 {
            self.vram[(address.0 - 0x1000) as usize] = value.0;
        }else if address.0 < 0x3F20 {
            self.vram[address.0 as usize] = value.0;
        }else if address.0 < 0x4000 {
            self.vram[(address.0 - 0x100) as usize] = value.0;
        }else {
            self.vram[(address.0 % 0x4000) as usize] = value.0;
        }
    }

    pub fn load_word_vram (&mut self, address: W<u16>) -> W<u16> {
        let word : W<u16> = W16!(self.load_vram(address));
        word | W16!(self.load_vram(address + W(1)) << 8)
    }

    pub fn store_word_vram (&mut self, address: W<u16>, word: W<u16>) {
        self.store_vram(address, W8!(word >> 8));
        self.store_vram(address + W(1), W8!(word));
    }

}

impl fmt::Debug for Ppu {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut output  = "oam: [".to_string();
        for i in 0..255 {
            output.push_str(&format!("{:02x}|", self.oam[i]));
        }
        output.push_str(&format!("{:#x}]", self.oam[255]));
        write!(f, "Vram addr: {:#x} \n {}", self.vram_address, output)
    }
}

impl Default for Ppu {
    fn default () -> Ppu {
        Ppu::new()
    }
}
