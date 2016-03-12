extern crate sdl2;

// nes
use utils::print_mem;
use loadstore::LoadStore;
use mem::{Memory as Mem};
use enums::{MemState};

// std
use std::fmt;
use std::num::Wrapping as W;

// sdl2
use sdl2::pixels::Color;
use sdl2::rect::Point;

/*

// ppuctrl
// Const values to access the controller register bits.
const CTRL_BASE_TABLE           : u8 = 0x03;
 0 = 0x2000 e incrementa de a 0x400,
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

//ppu mask
const MASK_GRAYSCALE            : u8 = 0x01;
const MASK_SHOW_BACKGROUND_LEFT : u8 = 0x02; // set = show bacgrkound in leftmost 8 pixels of screen
const MASK_SHOW_SPRITES_LEFT    : u8 = 0x04; // set = show sprites in leftmost 8 pixels of screens
const MASK_SHOW_BACKGROUND      : u8 = 0x08;
const MASK_SHOW_SPRITES         : u8 = 0x10;
const MASK_EMPHASIZE_RED        : u8 = 0x20;
const MASK_EMPHASIZE_GREEN      : u8 = 0x40;
const MASK_EMPHASIZE_BLUE       : u8 = 0x80;

/*
// ppu status
const STATUS_SPRITE_OVERFLOW    : u8 = 0x20;
const STATUS_SPRITE_0_HIT       : u8 = 0x40;
const STATUS_VERTICAL_BLANK     : u8 = 0x80; // set = in vertical blank
*/

#[allow(dead_code)]
const SPRITE_INFO_CLEAN_UNIMPLEMENTED_BITS  : u8 = 0xE3;
#[allow(dead_code)]
const SPRITE_INFO_PRIORITY                  : u8 = 0x20;
#[allow(dead_code)]
const SPRITE_INFO_PALETTE                   : u8 = 0x3;
#[allow(dead_code)]
const SPRITE_INFO_HORIZONTALLY              : u8 = 0x40;
#[allow(dead_code)]
const SPRITE_INFO_VERTICALLY                : u8 = 0x80;

const PALETTE_SIZE          : usize = 0x20;
const PALETTE_ADDRESS       : usize = 0x3f00;

const PPU_ADDRESS_SPACE     : usize = 0x4000;
const VBLANK_END            : u32 = 88740; 
const VBLANK_END_NO_RENDER  : u32 = 27902;
// The tiles are fetched from
// chr ram
struct Tile {
    tile : u16,
    high : bool,
}

impl Tile {
    pub fn new () -> Tile {
        Tile {
            tile : 0,
            high : true,
        }
    }
}

impl Tile {
    pub fn set_tile_byte(&mut self, byte : u8) {
        if self.high {
            self.tile = (self.tile & 0) | ((byte as u16) << 8);
        } else {
            self.tile |= byte as u16 & 0xFF;
        }
    }

    pub fn get_tile(&mut self) -> u16 {
        return self.tile;
    }
}

pub struct Ppu {
    palette         : [u8; PALETTE_SIZE],
    oam             : Oam,

    // Registers
    ctrl            : u8,
    mask            : u8,
    status          : u8,
    scroll          : AddressLatch,
    addr            : AddressLatch, 
    oamaddr         : u8,

    
    // Scanline should count up until the total numbers of scanlines
    // which is 262
    scanline        : usize,
    // while scanline width goes up to 340 and visible pixels
    // ie drawn pixels start at 0 and go up to 256 width (240 scanlines)
    scanline_width  : usize,
    
    cycles          : u32,
    fps             : u32,

    // oam index for rendering

    oam_index       : W<u16>,

    // even/odd frame?
    frame_parity    : bool,

    name_table_byte : u8,
    attr_byte       : u8,
    tile            : Tile,
    
    sprite_unit     : [SpriteInfo; 0x08],

    ltile_sreg      : u16, // 2 byte shift register
    htile_sreg      : u16, // " "
    palette_sreg    : u8,
    tile_addr       : u16,

    next_ltile      : u8,
    next_htile      : u8,
}

macro_rules! in_render_range {
    ($scanline:expr) => ($scanline < 257 && $scanline > 1)
}

macro_rules! render_on {
    ($selfie:expr) => ($selfie.show_sprites() || $selfie.show_background())
}

macro_rules! sprite_pattern_base {
    ($selfie:expr) =>  (if $selfie.mask & CTRL_SPRITE_PATTERN == 0 {
                            0x0000
                        } else {
                            0x1000
                        })
}

impl Ppu {
    pub fn new () -> Ppu {
        Ppu {
            palette         : [0; PALETTE_SIZE], 
            oam             : Oam::default(), 

            ctrl            : 0,
            mask            : 0,
            status          : 0,
            scroll          : AddressLatch::default(),
            addr            : AddressLatch::default(),
            oamaddr         : 0,

            scanline        : 0,
            scanline_width  : 0,

            cycles          : 0,
            fps             : 0,

            // index

            oam_index       : W(0),

            frame_parity    : true,

            name_table_byte : 0,
            attr_byte       : 0,
            tile            : Tile::new(),

            sprite_unit     :[SpriteInfo::new(); 0x08],

            ltile_sreg      : 0,
            htile_sreg      : 0,
            palette_sreg    : 0,

            next_ltile      : 0,
            next_htile      : 0,

            tile_addr       : 0,
        }
    }
    
    pub fn cycle(&mut self, memory: &mut Mem, renderer: &mut sdl2::render::Renderer) {
        self.ls_latches(memory);

        // TODO: PPU CODE
        let val = self.load(memory);
        self.store(memory, val);
        
        self.oam.cycle(self.cycles, self.scanline);    // we let the oam prepare the next sprites

        // if on a visible scanline 
        // and width % 8 = 1 then we fetch nametable
        // if width % 8 = 3 we fetch attr
        // width % 5 fetch tile high (chr ram)
        // width % 7 fetch tile low (chr ram)
        if render_on!(self) && in_render_range!(self.scanline_width){
            self.draw(renderer, memory); // if rendering is off we only execute VBLANK_END cycles
        }

        self.scanline_width +=1;
        
        if self.scanline_width == 340 && self.scanline == 261 {
            self.scanline_width = 0;
            self.scanline = 0;
        } else if self.scanline_width == 340 {
            self.scanline += 1;
            self.scanline_width = 0;
        }
        self.cycles += 1;
        if !render_on!(self) && self.cycles == VBLANK_END_NO_RENDER ||
            render_on!(self) && self.cycles == VBLANK_END 
        {
            self.cycles = 0;
            self.fps += 1;
            self.frame_parity = !self.frame_parity;
       }
    }

    /* for now we dont use mem, remove warning, memory: &mut Mem*/
    fn draw(&mut self, renderer: &mut sdl2::render::Renderer, memory: &mut Mem) {
        let scanline_width = self.scanline_width;
        match scanline_width % 8 {
                    // fetch nametable address from the sprite unit
            0 => { self.fetch_nametable_addr(scanline_width); },            
                    // using that address fetch the tile
            1 => { self.next_ltile = self.fetch_nametable_tile(memory); }, 
            2 => {}, // fetch attribute
            3 => {}, // same as before
            4 => {}, // fetch low tile byte
            5 => {}, // as before
            6 => {}, // fetch high tile byte
            7 => {}, // as before
            _ => {}, // 
        }
        renderer.set_draw_color(Color::RGB(self.scanline as u8, self.scanline as u8, 20));
        renderer.draw_point(Point::new(self.scanline as i32, self.scanline as i32)).unwrap();
    }

    fn fetch_nametable_addr(&mut self, scanline_width: usize) {
        // we get the sprite unit idx 
        let sprite_idx = scanline_width % 8;
        // we get the tile index (offset on 0x0000 or 0x1000 of tiles)
        let tile_offset = self.sprite_unit[sprite_idx].tile_index() as u16;
        let base : u16 = sprite_pattern_base!(self) + tile_offset;
        self.addr.reset_address();
        self.addr.set_address(W((base >> 8) as u8));
        self.addr.set_address(W(base as u8));
    }

    fn fetch_nametable_tile(&mut self, memory: &mut Mem) -> u8 {
        return self.load(memory).0;
    }

    #[inline(always)]
    pub fn grayscale(&mut self) -> bool {
        return (self.mask & MASK_GRAYSCALE) > 0;
    }

    #[inline(always)]
    pub fn show_sprites(&mut self) -> bool {
        return (self.mask & MASK_SHOW_SPRITES) > 0;
    }

    #[inline(always)]
    pub fn show_background(&mut self) -> bool {
        return (self.mask & MASK_SHOW_BACKGROUND) > 0;
    }

    #[inline(always)]
    pub fn show_sprites_left(&mut self) -> bool {
        return (self.mask & MASK_SHOW_SPRITES_LEFT) > 0;
    }

    #[inline(always)]
    pub fn show_background_left(&mut self) -> bool {
        return (self.mask & MASK_SHOW_BACKGROUND_LEFT) > 0;
    }

    #[inline(always)]
    pub fn emphasize_red(&mut self) -> bool {
        return (self.mask & MASK_EMPHASIZE_RED) > 0;
    }

    #[inline(always)]
    pub fn emphasize_blue(&mut self) -> bool {
        return (self.mask & MASK_EMPHASIZE_BLUE) > 0;
    }

    #[inline(always)]
    pub fn emphasize_green(&mut self) -> bool {
        return (self.mask & MASK_EMPHASIZE_GREEN) > 0;
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
            MemState::PpuCtrl   => { self.ctrl = latch.0; }, 
            MemState::PpuMask   => { self.mask = latch.0; },
            MemState::OamAddr   => { self.oamaddr = latch.0; },
            MemState::OamData   => { self.oam.store_data(&mut self.oamaddr, latch); },
            MemState::PpuScroll => { self.scroll.set_address(latch); },
            MemState::PpuAddr   => { self.addr.set_address(latch); },
            MemState::PpuData   => { self.store(memory, latch);}, 
            _                   => (), 
        }

        let read_status = memory.get_mem_load_status();

        match read_status {
            MemState::PpuStatus => {
                self.addr.reset_address();
                self.scroll.reset_address();
                self.status &= 0x60;
            },
            MemState::PpuData   => { 
                let value = self.load(memory); 
                memory.set_latch(value);
            },
            _                   => {},
        }
    }

    fn palette_mirror(&mut self, address: usize) -> usize {
        let index = address & (PALETTE_SIZE - 1);
        // Mirroring 0x10/0x14/0x18/0x1C to lower address
        if (index & 0x3) == 0 {
            index & 0xF
        } else {
            index
        }
    }

    fn load(&mut self, memory: &mut Mem) -> W<u8> {
        let address = self.addr.get_address();
        let addr = address.0 as usize;
        if addr < PALETTE_ADDRESS {
            memory.chr_load(address)
        } else {
            if addr < PPU_ADDRESS_SPACE {
                W(self.palette[self.palette_mirror(addr)])
            } else {
                panic!("PPUADDR >= 0x4000");
            }
        }
    }

    fn store(&mut self, memory: &mut Mem, value: W<u8>) {
        let address = self.addr.get_address();
        let addr = address.0 as usize;
        if addr < PALETTE_ADDRESS {
            memory.chr_store(address, value);
        } else {
            if addr < PPU_ADDRESS_SPACE {
                self.palette[self.palette_mirror(addr)] = value.0;
            } else {
                panic!("PPUADDR >= 0x4000");
            }
        }
    }

    pub fn load_from_oam(&mut self) -> u8 {
        return self.oam.load(W(self.oamaddr as u16)).0;
    }
}

impl Default for Ppu {
    fn default () -> Ppu {
        Ppu::new()
    }
}


impl fmt::Debug for Ppu {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "PPU: \n OAM: {:?}, ctrl: {:?}, mask: {:?}, status: {:?}, scroll: {:?}, addr: {:?}", 
               self.oam, self.ctrl, self.mask, self.status, self.scroll, self.addr)
    }
}

#[derive(Default)]
struct AddressLatch {
    laddr   : W<u8>,
    haddr   : W<u8>,
    upper   : bool,
}


impl AddressLatch {
    pub fn reset_address(&mut self) {
        *self = AddressLatch::default();
    }

    pub fn get_address(&self) -> W<u16> {
        W16!(self.haddr) << 8 | W16!(self.laddr)
    }

    pub fn set_address(&mut self, value: W<u8>) {
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
        write!(f, "{:?}", self.get_address())
    }
}

struct Oam {
    mem                 : [u8; 0x100],
    mem_idx             : usize,
    secondary_mem       : [u8; 0x20],
    secondary_idx       : usize,
    copy_leftover_bytes : bool,
}

impl Default for Oam {
    fn default() -> Oam {
        Oam {
            mem                 : [0; 0x100],
            mem_idx             : 0,
            secondary_mem       : [0; 0x20], 
            secondary_idx       : 0,
            copy_leftover_bytes : false,
        }
    }
}

impl fmt::Debug for Oam {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut output = "OAM: mem: \n".to_string();
        print_mem(&mut output, &self.mem[..]);
        write!(f, "{}", output)
    }
}

impl Oam {

    #[inline]
    fn store_data(&mut self, address: &mut u8 , value: W<u8>) {
        self.mem[*address as usize] = value.0;
        *address += 1;
    }

    /*#[inline]
    fn set_addr(&mut self, value: W<u8>) {
        self.addr = value;
    }*/

    // cleans the secondary oam array
    // setting it to all FFs
    fn reset_sec_oam(&mut self) {
        for i in 0..64 {
            self.secondary_mem[i as usize] = 0xFF;
            self.secondary_idx = 0;
            self.mem_idx = 0;
        }
    }

    pub fn cycle(&mut self, cycles: u32, scanline: usize) {
        if cycles == 1 {
            self.reset_sec_oam();
        } else if cycles < 256 {
            // odd cycles
            if cycles % 2 == 1 {
                // TODO: Ignore odd cycles and do everything on even cycles? (reads).
            // even cycles
            } else {
                if self.secondary_idx != 64 && self.mem_idx != 256 {
                    // If we're on a y-pos byte and it fits with the scanline
                    // copy it to the current position of secondary oam memory
                    // else just add to the memory idx
                    if self.mem[self.mem_idx] as usize == scanline && (self.mem[self.mem_idx] % 4 == 0) {
                        self.secondary_mem[self.secondary_idx] = self.mem[self.mem_idx];
                        self.secondary_idx  += 1;
                    } else if self.copy_leftover_bytes {
                        self.secondary_mem[self.secondary_idx] = self.mem[self.mem_idx];
                        self.secondary_idx  += 1;
                        // If we copied the 4th byte we reset the copyleftover flags
                        // so we can evaluate the y-pos byte again.
                        if self.mem_idx % 4 == 3 {
                            self.copy_leftover_bytes = false;
                        }
                    }
                    self.mem_idx += 1;
                } else if self.secondary_idx < 64 {
                    self.reset_sec_oam();
                }
            }
        } else if cycles < 320 {
            // Copy to the sprite units??
        }
    }

    pub fn store_to_secondary_oam(&mut self, address: u8) {
        let address = address as usize;

        self.secondary_idx += 1;
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

#[derive(Copy, Clone)]
struct SpriteInfo {
    bytes           : [u8; 4],
}

impl SpriteInfo {
    #[allow(dead_code)]
    pub fn new (/*ppu: &mut Ppu*/) -> SpriteInfo {
        SpriteInfo {
            bytes : [0; 4], //bytes,
        }
    }

    pub fn reset(&mut self) {
        for i in 0..4 {
            self.bytes[i] = 0xFF;
        }
    }

    pub fn set(&mut self, arr : &[u8]) {
        for i in 0..4 {
            self.bytes[i] = arr[i];
        }
        self.bytes[2] = self.bytes[2] & SPRITE_INFO_CLEAN_UNIMPLEMENTED_BITS;
    }
}

impl SpriteInfo {
    #[allow(dead_code)]
    #[inline]
    pub fn y_position(&mut self) -> u8 {
        return self.bytes[0];
    }

    #[allow(dead_code)]
    #[inline]
    pub fn tile_index(&mut self) -> u8 {
        return self.bytes[1];
    }

    #[allow(dead_code)]
    #[inline]
    pub fn x_position(&mut self) -> u8 {
        return self.bytes[3];
    }

    // true = in front of background 
    // false = behind background
    #[allow(dead_code)]
    #[inline]
    pub fn sprite_priority(&mut self) -> bool {
        return (self.bytes[2] & SPRITE_INFO_PRIORITY) != 0;
    }

    #[allow(dead_code)]
    #[inline]
    pub fn palette(&mut self) -> u8 {
        return self.bytes[2] & SPRITE_INFO_PALETTE;
    }

    #[allow(dead_code)]
    #[inline]
    pub fn flip_horizontally(&mut self) -> bool {
        return (self.bytes[2] & SPRITE_INFO_HORIZONTALLY) > 1;
    }

    #[allow(dead_code)]
    #[inline]
    pub fn flip_vertically(&mut self) -> bool {
        return (self.bytes[2] & SPRITE_INFO_VERTICALLY) > 1;
    }
}

macro_rules! to_RGB {
    ($r:expr, $g:expr, $b:expr) => { 
        Color::RGB($r, $g, $b) 
    }
}
const PALLETE : [Color; 0x40] = [
    to_RGB!(3,3,3), to_RGB!(0,1,4), to_RGB!(0,0,6), to_RGB!(3,2,6), 
    to_RGB!(4,0,3), to_RGB!(5,0,3), to_RGB!(5,1,0), to_RGB!(4,2,0), 
    to_RGB!(3,2,0), to_RGB!(1,2,0), to_RGB!(0,3,1), to_RGB!(0,4,0), 
    to_RGB!(0,2,2), to_RGB!(0,0,0), to_RGB!(0,0,0), to_RGB!(0,0,0), 
    to_RGB!(5,5,5), to_RGB!(0,3,6), to_RGB!(0,2,7), to_RGB!(4,0,7), 
    to_RGB!(5,0,7), to_RGB!(7,0,4), to_RGB!(7,0,0), to_RGB!(6,3,0), 
    to_RGB!(4,3,0), to_RGB!(1,4,0), to_RGB!(0,4,0), to_RGB!(0,5,3), 
    to_RGB!(0,4,4), to_RGB!(0,0,0), to_RGB!(0,0,0), to_RGB!(0,0,0), 
    to_RGB!(7,7,7), to_RGB!(3,5,7), to_RGB!(4,4,7), to_RGB!(6,3,7), 
    to_RGB!(7,0,7), to_RGB!(7,3,7), to_RGB!(7,4,0), to_RGB!(7,5,0), 
    to_RGB!(6,6,0), to_RGB!(3,6,0), to_RGB!(0,7,0), to_RGB!(2,7,6), 
    to_RGB!(0,7,7), to_RGB!(0,0,0), to_RGB!(0,0,0), to_RGB!(0,0,0), 
    to_RGB!(7,7,7), to_RGB!(5,6,7), to_RGB!(6,5,7), to_RGB!(7,5,7), 
    to_RGB!(7,4,7), to_RGB!(7,5,5), to_RGB!(7,6,4), to_RGB!(7,7,2), 
    to_RGB!(7,7,3), to_RGB!(5,7,2), to_RGB!(4,7,3), to_RGB!(2,7,6), 
    to_RGB!(4,6,7), to_RGB!(0,0,0), to_RGB!(0,0,0), to_RGB!(0,0,0),
];
