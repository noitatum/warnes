extern crate sdl2;

//nes
use utils::print_mem;
use loadstore::LoadStore;
use mem::{Memory as Mem};
use enums::{MemState};
use scroll::Scroll;

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
// set = show bacgrkound in leftmost 8 pixels of screen
const MASK_SHOW_BACKGROUND_LEFT : u8 = 0x02; 
// set = show sprites in leftmost 8 pixels of screens
const MASK_SHOW_SPRITES_LEFT    : u8 = 0x04; 
const MASK_SHOW_BACKGROUND      : u8 = 0x08;
const MASK_SHOW_SPRITES         : u8 = 0x10;
const MASK_EMPHASIZE_RED        : u8 = 0x20;
const MASK_EMPHASIZE_GREEN      : u8 = 0x40;
const MASK_EMPHASIZE_BLUE       : u8 = 0x80;


// ppu status
const STATUS_SPRITE_OVERFLOW    : u8 = 0x20;
const STATUS_SPRITE_0_HIT       : u8 = 0x40;
const STATUS_VERTICAL_BLANK     : u8 = 0x80; // set = in vertical blank

const SPRITE_INFO_CLEAN_UNIMPLEMENTED_BITS  : u8 = 0xE3;
const SPRITE_INFO_PRIORITY                  : u8 = 0x20;
const SPRITE_INFO_PALETTE                   : u8 = 0x3;
const SPRITE_INFO_HORIZONTALLY              : u8 = 0x40;
const SPRITE_INFO_VERTICALLY                : u8 = 0x80;

const PALETTE_SIZE          : usize = 0x20;
const PALETTE_ADDRESS       : usize = 0x3f00;

const PPU_ADDRESS_SPACE     : usize = 0x4000;
const VBLANK_END            : u32 = 88740; 
const VBLANK_END_NO_RENDER  : u32 = 27902;

const ATTR_BIT              : u8 = 0x80;
const TILE_BIT              : u16 = 0x8000;
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
    address         : Scroll,

    // Registers
    ctrl            : u8,
    mask            : u8,
    status          : u8,
    
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
    attr1_sreg      : u8,
    attr2_sreg      : u8,

    next_ltile      : W<u8>,
    next_htile      : W<u8>,
    next_attr       : W<u8>,
    next_name       : W<u8>,
}


impl Ppu {
    pub fn new () -> Ppu {
        Ppu {
            palette         : [0; PALETTE_SIZE], 
            oam             : Oam::default(), 
            address         : Scroll::default(),

            ctrl            : 0,
            mask            : 0,
            status          : 0,

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

            sprite_unit     : [SpriteInfo::new(); 0x08],

            ltile_sreg      : 0,
            htile_sreg      : 0,
            attr1_sreg      : 0,
            attr2_sreg      : 0,

            next_ltile      : W(0),
            next_htile      : W(0),
            next_attr       : W(0),
            next_name       : W(0),
        }
    }

    pub fn cycle(&mut self, memory: &mut Mem, 
                 renderer: &mut sdl2::render::Renderer) {
        self.ls_latches(memory);
        
        // we let the oam prepare the next sprites
        self.oam.cycle(self.cycles, self.scanline, &mut self.sprite_unit);

        // if on a visible scanline 
        // and width % 8 = 1 then we fetch nametable
        // if width % 8 = 3 we fetch attr
        // width % 5 fetch tile high (chr ram)
        // width % 7 fetch tile low (chr ram)
        if render_on!(self) && in_render_range!(self.scanline_width) {
            // if rendering is off we only execute VBLANK_END cycles
            self.draw(renderer); 
            self.evaluate_next_byte(memory);
        }

        self.scanline_width +=1;
        self.cycles += 1;

        // if we finished the current scanline we pass to the next one
        if self.scanline_width == 340 {
            self.scanline += 1;
            self.scanline_width = 0;
        }

        if !render_on!(self) && self.cycles == VBLANK_END_NO_RENDER ||
            scanline_end!(self) {
           // reset scanline values and qty of cycles
            self.scanline_width = 0;
            self.scanline = 0;
            self.cycles = 0;
            self.fps += 1;
            self.frame_parity = !self.frame_parity;
        }
        
        // we enable the vertical blank flag on ppuctrl
        if self.scanline_width == 1 && self.scanline == 240 {
            set_flag!(self.ctrl, STATUS_VERTICAL_BLANK);
        }
    }
    // gets the value for the next line of 8 pixels
    // ie bytes into tile and attr registers
    fn evaluate_next_byte(&mut self, memory: &mut Mem) {
        // First cycle is idle FIXME: Turbio workaround
        let scanline_width = self.scanline_width - 1;
        match scanline_width & 0x7 {
            1 => { let address = self.address.get_nametable_address(); 
                   self.next_name = memory.chr_load(address);
            },
            3 => { let address = self.address.get_attribute_address();
                   self.next_attr = memory.chr_load(address); 
            },
            5 => { let index = self.next_name;
                   let address = self.address.get_tile_address(index);
                   self.next_ltile = memory.chr_load(address);
            },
            7 => { let index = self.next_name;
                   let address = self.address.get_tile_address(index);
                   self.next_htile = memory.chr_load(address + W(8));
                   // load the next shift registers.
                   self.set_shift_regs();
            },        
            _ => {}, 
        }
    }

    fn set_shift_regs(&mut self) {
        self.ltile_sreg = (self.ltile_sreg & 0xFF00) | self.next_ltile.0 as u16;
        self.htile_sreg = (self.htile_sreg & 0xFF00) | self.next_htile.0 as u16;
        self.attr1_sreg = self.next_attr.0;
        self.attr2_sreg = self.next_attr.0;
    }

    /* for now we dont use mem, remove warning, memory: &mut Mem*/
    fn draw(&mut self, renderer: &mut sdl2::render::Renderer) {
        let color_idx = join_bits!(attr_bit!(self.attr1_sreg),
                                   attr_bit!(self.attr2_sreg),
                                   tile_bit!(self.ltile_sreg),
                                   tile_bit!(self.htile_sreg));
        renderer.set_draw_color(PALETTE[color_idx as usize]);
        renderer.draw_point(Point::new(self.scanline as i32, 
                                       self.scanline as i32)).unwrap();
        shift_bits!(self);
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
            MemState::PpuCtrl   => { 
                self.ctrl = latch.0; 
                self.address.set_ppuctrl(latch);
            }, 
            MemState::PpuMask   => { self.mask = latch.0; },
            MemState::OamAddr   => { self.oam.set_address(latch); },
            MemState::OamData   => { self.oam.store_data(latch); },
            MemState::PpuScroll => { self.address.set_scroll(latch); },
            MemState::PpuAddr   => { self.address.set_address(latch); },
            MemState::PpuData   => { self.store(memory, latch);}, 
            _                   => (), 
        }

        let read_status = memory.get_mem_load_status();

        match read_status {
            MemState::PpuStatus => {
                self.address.reset();
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
        let address = self.address.get_address();
        let addr = address.0 as usize;
        if addr < PALETTE_ADDRESS {
            memory.chr_load(address)
        } else {
            W(self.palette[self.palette_mirror(addr)])
        }
    }

    fn store(&mut self, memory: &mut Mem, value: W<u8>) {
        let address = self.address.get_address();
        let addr = address.0 as usize;
        if addr < PALETTE_ADDRESS {
            memory.chr_store(address, value);
        } else {
            self.palette[self.palette_mirror(addr)] = value.0;
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
        write!(f, "PPU: \n OAM: {:?}, ctrl: {:?}, mask: {:?}, status: {:?}, \
                   address: {:?}", 
               self.oam, self.ctrl, self.mask, self.status, self.address)
    }
}

struct Oam {
    mem                 : [u8; 0x100],
    secondary_mem       : [u8; 0x20],
    address             : W<u8>,
    mem_idx             : usize,
    secondary_idx       : usize,
    copy_leftover_bytes : bool,
}

impl Default for Oam {
    fn default() -> Oam {
        Oam {
            mem                 : [0; 0x100],
            secondary_mem       : [0; 0x20],    
            address             : W(0),
            mem_idx             : 0,
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

    fn store_data(&mut self, value: W<u8>) {
        self.mem[self.address.0 as usize];
        self.address = self.address + W(1);
    }

    fn set_address(&mut self, addr: W<u8>) {
        self.address = addr;
    }

    fn reset_sec_oam(&mut self, idx: usize) {
        self.secondary_mem[idx] = 0xFF;
        self.secondary_idx = 0;
        self.mem_idx = 0;
    }

    fn reset_sec_oam_tot(&mut self) {
        for idx in 0..64 {
            self.secondary_mem[idx as usize] = 0xFF;
            self.secondary_idx = 0;
            self.mem_idx = 0;
        }
    }

    pub fn cycle(&mut self, cycles: u32, scanline: usize, spr_units: &mut [SpriteInfo]) {
        if cycles <= 64 {
            self.reset_sec_oam(cycles as usize);
        } else if cycles < 257 {
            // odd cycles
            if cycles % 2 == 1 {
                // TODO: Ignore odd cycles and do everything on even cycles? (reads).
            // even cycles
            } else {
                if self.secondary_idx < 64 && self.mem_idx != 256 {
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
                    self.reset_sec_oam_tot();
                }
            }
        } else if cycles < 320 {
            // set index to 0 so we can copy to the sprite units.
            if cycles == 257 { self.secondary_idx = 0; }
            // cycle 257, 265, 273
            if cycles % 8 < 5 {
                let idx = self.secondary_idx;
                spr_units[idx/8 - 1]
                    .set_sprite_info((idx % 8) - 1, self.secondary_mem[idx]);
            }
            self.secondary_idx += 1;
        }
    }

    pub fn store_to_secondary_oam(&mut self, address: u8) {
        let address = address as usize;
        self.secondary_idx += 1;
    }
}

#[derive(Copy, Clone, Default)]
struct SpriteInfo {
    pub y_pos       : u8,
    pub tile_idx    : u8,
    pub attributes  : u8,
    pub x_pos       : u8,
}

impl SpriteInfo {
    #[allow(dead_code)]
    pub fn new (/*ppu: &mut Ppu*/) -> SpriteInfo {
        SpriteInfo {
            y_pos       : 0,
            tile_idx    : 0,
            attributes  : 0,
            x_pos       : 0,
        }
    }

    pub fn reset(&mut self) {
        for i in 0..4 {
            self.set_sprite_info(i, 0xFF);
        }
    }

    pub fn set_sprite_info(&mut self, idx: usize, value: u8) {
        match idx {
            0 => { self.y_pos = value; },
            1 => { self.tile_idx = value; },
            2 => { self.attributes = value; },
            3 => { self.x_pos = value; },
            _ => { panic!("wrong sprite unit index!"); }
        }
    }
}

macro_rules! get_sprite_priority {
    ($attr:expr) => (($attr & SPRITE_INFO_PRIORITY) != 0)
}

macro_rules! get_palette {
    ($attr:expr) => ($attr & SPRITE_INFO_PALETTE)
}

macro_rules! flip_horizontally {
    ($attr:expr) => (($attr & SPRITE_INFO_HORIZONTALLY) > 1)
}

macro_rules! flip_vertically {
    ($attr:expr) => (($attr & SPRITE_INFO_VERTICALLY) > 1)
}

const PALETTE : [Color; 0x40] = [
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
