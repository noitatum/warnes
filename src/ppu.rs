extern crate sdl2;

// NES
use utils::print_mem;
use mem::{Memory as Mem};
use enums::{MemState, Interrupt};
use scroll::Scroll;

// std
use std::fmt;
use std::num::Wrapping as W;
use std::ops::{Index, IndexMut};

macro_rules! attr_bit {
    ($attr:expr, $fine_x:expr) => (($attr & (0x80 - $fine_x)) >> 7)
}

macro_rules! tile_bit {
    ($tile:expr, $fine_x:expr) =>
        (($tile & (0x8000 >> $fine_x)) >> (15 - $fine_x))
}

const CTRL_SPRITE_PATTERN       : u8 = 0x08;
const CTRL_BACKGROUND_PATTERN   : u8 = 0x10;
const CTRL_SPRITE_SIZE          : u8 = 0x20;
// trigger warning
const CTRL_PPU_SLAVE_MASTER     : u8 = 0x40;
const CTRL_NMI                  : u8 = 0x80;

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

const STATUS_SPRITE_OVERFLOW    : u8 = 0x20;
const STATUS_SPRITE_0_HIT       : u8 = 0x40;
const STATUS_VBLANK             : u8 = 0x80;

const SPRITE_INFO_PRIORITY      : u8 = 0x20;
const SPRITE_INFO_PALETTE       : u8 = 0x3;
const SPRITE_INFO_HORIZONTALLY  : u8 = 0x40;
const SPRITE_INFO_VERTICALLY    : u8 = 0x80;

const PALETTE_SIZE          : usize = 0x20;
const PALETTE_ADDRESS       : usize = 0x3f00;

const PPU_ADDRESS_SPACE     : usize = 0x4000;
const VBLANK_END            : u32 = 88740;
const VBLANK_END_NO_RENDER  : u32 = 27902;

// Resolution
pub const SCANLINE_WIDTH        : usize = 256;
pub const SCANLINE_COUNT        : usize = 240;

// TODO: Wait for arbitrary size array default impls to remove Scanline
pub struct Scanline(pub [u8; SCANLINE_WIDTH]);

impl Scanline {
    fn new() -> Scanline {
        Scanline([0u8; SCANLINE_WIDTH])
    }
}

impl Clone for Scanline {
    fn clone(&self) -> Scanline {
        Scanline(self.0)
    }
}

impl Index<usize> for Scanline {
    type Output = u8;

    fn index(&self, index: usize) -> &u8 {
        &self.0[index]
    }
}

impl IndexMut<usize> for Scanline {
    fn index_mut(&mut self, index: usize) -> &mut u8 {
        &mut self.0[index]
    }
}

#[derive(Copy, Clone, Default)]
pub struct PpuReadRegs {
    pub data    : u8,
    pub oam     : u8,
    pub status  : u8,
}

pub struct Ppu {
    palette         : [u8; PALETTE_SIZE],
    oam             : Oam,
    address         : Scroll,

    // Registers
    ctrl            : u8,
    mask            : u8,
    status          : u8,
    data_buffer     : u8,

    // Scanline should count up until the total numbers of scanlines
    // which is 262
    scanline        : usize,
    // while scanline width goes up to 340 and visible pixels
    // ie drawn pixels start at 0 and go up to 256 width (240 scanlines)
    scycle          : usize,
    cycles          : u32,

    // oam index for rendering
    oam_index       : W<u16>,
    // for sprite rendering
    sprite_unit     : [SpriteInfo; 0x08],

    // Shift registers
    ltile_sreg      : u16,
    htile_sreg      : u16,
    attr1_sreg      : u8,
    attr2_sreg      : u8,

    next_ltile      : W<u8>,
    next_htile      : W<u8>,
    next_attr       : W<u8>,
    next_name       : W<u8>,

    frames          : u64,
    frame_data      : Box<[Scanline]>,
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
            data_buffer     : 0,

            scanline        : 0,
            scycle          : 0,
            cycles          : 0,

            oam_index       : W(0),
            sprite_unit     : [SpriteInfo::default(); 0x08],

            ltile_sreg      : 0,
            htile_sreg      : 0,
            attr1_sreg      : 0,
            attr2_sreg      : 0,

            next_ltile      : W(0),
            next_htile      : W(0),
            next_attr       : W(0),
            next_name       : W(0),

            frames          : 0,
            frame_data      : vec![Scanline::new(); SCANLINE_COUNT]
                                  .into_boxed_slice(),
        }
    }

    pub fn cycle(&mut self, memory: &mut Mem) {
        self.ls_latches(memory);

        // we let the oam prepare the next sprites
        self.oam.cycle(self.cycles, self.scanline, &mut self.sprite_unit);

        if self.render_on() {
            if self.scanline < 240 {
                if self.scycle < 257 && self.scycle > 0 {
                    // if rendering is off we only execute VBLANK_END cycles
                    self.draw();
                    self.evaluate_next_byte(memory);
                    if self.scycle == 256 {
                        self.address.increment_y();
                    }
                } else if self.scycle == 257 {
                    self.address.copy_horizontal();
                }
            } else if self.scanline == 261 {
                if self.scycle > 279 && self.scycle < 305 {
                    self.address.copy_vertical();
                }
            }
        }

        // VBLANK
        if self.scycle == 1 && self.scanline == 241 {
            set_flag!(self.status, STATUS_VBLANK);
            if is_flag_set!(self.ctrl, CTRL_NMI) {
                memory.set_interrupt(Interrupt::NMI);
            }
        } else if self.scycle == 1 && self.scanline == 261 {
            unset_flag!(self.status, STATUS_VBLANK);
        }
        // TODO
        if !self.render_on() && self.cycles == VBLANK_END_NO_RENDER {}
        // When render is not activated the loop is shorter
        if self.scycle == 340 && self.scanline == 261 {
            // reset scanline values and qty of cycles
            // TODO: Skip a cycle on odd frames and background on
            self.scycle = 0;
            self.scanline = 0;
            self.cycles = 0;
            self.frames += 1;
        } else if self.scycle == 340 {
            // if we finished the current scanline we pass to the next one
            self.scanline += 1;
            self.scycle = 0;
        } else {
            self.scycle += 1;
            self.cycles += 1;
        }

        let read_regs = PpuReadRegs {
                data    : self.data_buffer,
                oam     : self.oam.load_data(),
                status  : self.status,
        };
        memory.set_ppu_read_regs(read_regs);
    }
    // gets the value for the next line of 8 pixels
    // ie bytes into tile and attr registers
    fn evaluate_next_byte(&mut self, memory: &mut Mem) {
        // First cycle is idle
        match (self.scycle - 1) & 0x7 {
            // if on a visible scanline
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
                   // Increment horizontal scroll
                   self.address.increment_coarse_x();
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
    fn draw(&mut self) {
        let fine_x = self.address.get_fine_x();
        let palette_idx = tile_bit!(self.ltile_sreg, fine_x) |
                          tile_bit!(self.htile_sreg, fine_x) << 1;
        let color_idx = self.palette[0] >> (palette_idx * 2);
        self.frame_data[self.scanline][self.scycle - 1] = color_idx;
        self.ltile_sreg <<= 1;
        self.htile_sreg <<= 1;
        self.attr1_sreg <<= 1;
        self.attr2_sreg <<= 1;
    }

    pub fn frame_data(&self) -> (u64, &[Scanline]) {
        (self.frames, &self.frame_data)
    }

    fn render_on(&self) -> bool {
        self.show_sprites() || self.show_background()
    }

    fn rendering(&self) -> bool {
        self.render_on() && (self.scanline < 240 || self.scanline == 261)
    }

    fn show_sprites(&self) -> bool {
        self.mask & MASK_SHOW_SPRITES > 0
    }

    fn show_background(&self) -> bool {
        self.mask & MASK_SHOW_BACKGROUND > 0
    }

    #[allow(dead_code)]
    #[inline(always)]
    pub fn grayscale(&mut self) -> bool {
        return (self.mask & MASK_GRAYSCALE) > 0;
    }

    #[allow(dead_code)]
    #[inline(always)]
    pub fn show_sprites_left(&mut self) -> bool {
        return (self.mask & MASK_SHOW_SPRITES_LEFT) > 0;
    }

    #[allow(dead_code)]
    #[inline(always)]
    pub fn show_background_left(&mut self) -> bool {
        return (self.mask & MASK_SHOW_BACKGROUND_LEFT) > 0;
    }

    #[allow(dead_code)]
    #[inline(always)]
    pub fn emphasize_red(&mut self) -> bool {
        return (self.mask & MASK_EMPHASIZE_RED) > 0;
    }

    #[allow(dead_code)]
    #[inline(always)]
    pub fn emphasize_blue(&mut self) -> bool {
        return (self.mask & MASK_EMPHASIZE_BLUE) > 0;
    }

    #[allow(dead_code)]
    #[inline(always)]
    pub fn emphasize_green(&mut self) -> bool {
        return (self.mask & MASK_EMPHASIZE_GREEN) > 0;
    }

    /* load store latches */
    fn ls_latches(&mut self, memory: &mut Mem) {
        let (latch, status) = memory.get_latch();
        match status {
            MemState::PpuCtrl   => {
                if !is_flag_set!(self.ctrl, CTRL_NMI) &&
                    is_flag_set!(latch.0, CTRL_NMI) &&
                    is_flag_set!(self.status, STATUS_VBLANK) {
                    memory.set_interrupt(Interrupt::NMI);
                }
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

        let read_status = memory.ppu_load_status();

        match read_status {
            MemState::PpuStatus => {
                self.address.reset();
                unset_flag!(self.status, STATUS_VBLANK);
            },
            MemState::PpuData => {
                self.data_buffer = self.load(memory).0;
            }
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
        let rendering = self.rendering();
        let address = self.address.get_address(rendering);
        let addr = address.0 as usize;
        if addr < PALETTE_ADDRESS {
            memory.chr_load(address)
        } else {
            W(self.palette[self.palette_mirror(addr)])
        }
    }

    fn store(&mut self, memory: &mut Mem, value: W<u8>) {
        let rendering = self.rendering();
        let address = self.address.get_address(rendering);
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
    secondary_mem       : [u8; 0x40],
    address             : W<u8>,
    mem_idx             : usize,
    secondary_idx       : usize,
    copy_leftover_bytes : bool,
}

impl Default for Oam {
    fn default() -> Oam {
        Oam {
            mem                 : [0; 0x100],
            secondary_mem       : [0; 0x40],
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

    fn load_data(&mut self) -> u8 {
        self.mem[self.address.0 as usize]
    }

    fn store_data(&mut self, value: W<u8>) {
        self.mem[self.address.0 as usize] = value.0;
        self.address += W(1);
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
        if cycles <= 64 && cycles != 0 {
            self.reset_sec_oam((cycles - 1) as usize);
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
            let idx = self.secondary_idx;
            match cycles % 0x7 {
                1 => { spr_units[idx / 8].
                        set_sprite_info(0 , self.secondary_mem[idx]); },
                3 => { spr_units[idx / 8].
                        set_sprite_info(1 , self.secondary_mem[idx]); },
                5 => { spr_units[idx / 8].
                        set_sprite_info(2 , self.secondary_mem[idx]); },
                7 => { spr_units[idx / 8].
                        set_sprite_info(3 , self.secondary_mem[idx]); },
                _ => {},
            }
            self.secondary_idx += 1;
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

#[derive(Copy, Clone, Default)]
struct SpriteInfo {
    pub y_pos       : u8,
    pub tile_idx    : u8,
    pub attributes  : u8,
    pub x_pos       : u8,
}

impl SpriteInfo {
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
