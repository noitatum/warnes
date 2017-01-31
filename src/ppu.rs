extern crate sdl2;

// NES
use utils::*;
use mem::{Memory as Mem};
use enums::{MemState, Interrupt};
use scroll::Scroll;

// std
use std::fmt;
use std::num::Wrapping as W;
use std::ops::{Index, IndexMut};

macro_rules! attr_bit {
    ($tile:expr, $fine_x:expr) =>
        (($tile & (0xC0000000 >> ($fine_x * 2))) >> ((15 - $fine_x) * 2))
}

macro_rules! tile_bit {
    ($tile:expr, $fine_x:expr) =>
        (($tile & (0x8000 >> $fine_x)) >> (15 - $fine_x))
}

const CTRL_SPRITE_PATTERN       : u8 = 0x08;
const CTRL_BACKGROUND_PATTERN   : u8 = 0x10;
const CTRL_PPU_SLAVE_MASTER     : u8 = 0x40;
const CTRL_NMI                  : u8 = 0x80;

const MASK_GRAYSCALE            : u8 = 0x01;
// set = show bacgrkound in leftmost 8 pixels of screen
const MASK_SHOW_BACKGROUND_LEFT : u8 = 0x02;
// set = show sprites in leftmost 8 pixels of screens
const MASK_SHOW_SPRITES_LEFT    : u8 = 0x04;

const STATUS_SPRITE_OVERFLOW    : u8 = 0x20;
const STATUS_SPRITE_0_HIT       : u8 = 0x40;
const STATUS_VBLANK             : u8 = 0x80;

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

    // Scanline should count up until the total numbers of scanlines (262)
    scanline        : usize,
    // while scanline width goes up to 340 and visible pixels
    // ie drawn pixels start at 0 and go up to 256 width (240 scanlines)
    scycle          : usize,
    cycles          : u32,

    // oam index for rendering
    oam_index       : W<u16>,
    // for sprite rendering
    sprites         : [Sprite; 0x08],

    // Shift registers
    ltile_sreg      : u16,
    htile_sreg      : u16,
    attr_sreg       : u32,

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
            sprites         : [Sprite::default(); 8],

            ltile_sreg      : 0,
            htile_sreg      : 0,
            attr_sreg       : 0,

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
        // Update PPU with what the CPU hay have sent to memory latch
        self.ls_latches(memory);
        if self.render_on() {
            match (self.scycle, self.scanline) {
                // Idle scanlines
                (_, 240...260) => {},
                // Last scanline, updates vertical scroll
                (280...304, 261) => {
                    self.address.copy_vertical();
                },
                // Dot 257 updates horizontal scroll
                (257, _) => {
                    self.address.copy_horizontal();
                }
                // Overlaps with above but nothing really happens in 257
                (257...320, _) => {
                    // This syncs with sprite evaluation in oam
                    self.fetch_sprite(memory);
                }
                // At prerender scanline we have to reset the sprite 0 hit
                (1, 261) => {
                    self.status &= !(STATUS_SPRITE_0_HIT |
                                     STATUS_SPRITE_OVERFLOW);
                    println!("{:?}", self.oam);
                }
                // Idle cycles
                (0, _) | (337...340, _) => {},
                _ => {
                    // (1...256 + 321..336, 0...239 + 261)
                    if self.scycle < 257 && self.scanline != 261 {
                        self.draw_dot();
                    }
                    self.fetch_background(memory);
                    if self.scycle == 256 {
                        self.address.increment_y();
                    }
                }
            }
            if self.scanline < 240 && self.scanline > 0 &&
               self.oam.cycle(self.scycle, self.scanline as u8,
                              &mut self.sprites) {
                set_flag!(self.status, STATUS_SPRITE_OVERFLOW);
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
        // Update memory PPU registers copy
        memory.set_ppu_read_regs(read_regs);
    }

    fn fetch_background(&mut self, memory: &mut Mem) {
        self.ltile_sreg <<= 1;
        self.htile_sreg <<= 1;
        self.attr_sreg <<= 2;
        // First cycle is idle
        match (self.scycle - 1) & 0x7 {
            // if on a visible scanline
            1 => {
                let address = self.address.get_nametable_address();
                self.next_name = memory.chr_load(address);
            },
            3 => {
                let address = self.address.get_attribute_address();
                self.next_attr = memory.chr_load(address);
            },
            5 => {
                let index = self.next_name;
                let address = self.address.get_tile_address(index);
                self.next_ltile = memory.chr_load(address);
            },
            7 => {
                let index = self.next_name;
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

    fn fetch_sprite(&mut self, memory: &mut Mem) {
        let big_sprites = self.sprite_big();
        let table = self.sprite_table();
        let sprite = &mut self.sprites[((self.scycle - 1) / 8) % 8];
        // Get fine Y position
        let y_offset = W16!(W(self.scanline as u8) - sprite.y_pos);
        // Compose the table, and the tile address with the fine Y position
        let address = if big_sprites {
            (W16!(W(sprite.tile.0.rotate_right(1))) << 4) | y_offset
        } else {
            table | (W16!(sprite.tile) << 4) | y_offset
        };
        //let address = W(0x300) | fine_y;
        match (self.scycle - 1) % 8 {
            3 => sprite.latch = sprite.attributes.0,
            4 => sprite.counter = sprite.x_pos.0,
            5 => {
                sprite.lshift = memory.chr_load(address).0;
                if !sprite.flip_horizontally() {
                    sprite.lshift = reverse_byte(sprite.lshift);
                }
            },
            7 => {
                sprite.hshift = memory.chr_load(address + W(8)).0;
                if !sprite.flip_horizontally() {
                    sprite.hshift = reverse_byte(sprite.hshift);
                }
            },
            _ => {},
        }
    }

    fn set_shift_regs(&mut self) {
        self.ltile_sreg = self.ltile_sreg & 0xFF00 | self.next_ltile.0 as u16;
        self.htile_sreg = self.htile_sreg & 0xFF00 | self.next_htile.0 as u16;
        let next = self.next_attr;
        let attr = self.address.get_tile_attribute(next).0 as u32;
        // attr is a 2 bit palette index, broadcast that into 16 bits
        self.attr_sreg = self.attr_sreg & 0xFFFF0000 | (attr * 0x5555);
    }

    fn draw_dot(&mut self) {
        let fine_x = self.address.get_fine_x();
        let back_index = (tile_bit!(self.ltile_sreg, fine_x) |
                          tile_bit!(self.htile_sreg, fine_x) << 1) as usize;
        let back_palette = attr_bit!(self.attr_sreg, fine_x) as usize;
        let mut color_id = self.palette[back_palette * 4 + back_index];
        let mut sprite_index = 0;
        let mut sprite_palette = 0;
        let mut sprite_front = false;
        // Amount of sprites in this scanline
        let count = self.oam.count();
        // Look for the first sprite that has a pixel to draw
        let index = self.sprites[..count].iter().position(Sprite::has_pixel);
        if let Some(index) = index {
            // TODO: Use Latch
            let sprite = &self.sprites[index];
            sprite_index = (sprite.lshift & 1) | ((sprite.hshift & 1) << 1);
            sprite_palette = sprite.get_palette() + 4;
            sprite_front = !sprite.get_priority();
        }
        // Choose which pixel to prioritize
        if back_index == 0 && sprite_index == 0 {
            color_id = self.palette[0];
        } else if sprite_index != 0 && (back_index == 0 || sprite_front) {
            let full_index = sprite_palette * 4 + sprite_index as usize;
            color_id = self.palette[full_index];
            if back_index != 0 && sprite_front && index == Some(0) {
                self.status |= STATUS_SPRITE_0_HIT;
            }
        }
        self.frame_data[self.scanline][self.scycle - 1] = color_id;
        // Decrement the sprite counters or shift their tile data
        for sprite in self.sprites.iter_mut() {
            if sprite.counter > 0 {
                sprite.counter -= 1;
            } else {
                sprite.lshift >>= 1;
                sprite.hshift >>= 1;
            }
        }
    }

    fn sprite_big(&self) -> bool {
        is_flag_set!(self.ctrl, 0x20)
    }

    fn sprite_table(&self) -> W<u16> {
        if is_flag_set!(self.ctrl, CTRL_SPRITE_PATTERN) {
            W(0x1000)
        } else {
            W(0)
        }
    }

    fn render_on(&self) -> bool {
        self.show_sprites() || self.show_background()
    }

    fn rendering(&self) -> bool {
        self.render_on() && (self.scanline < 240 || self.scanline == 261)
    }

    fn show_sprites(&self) -> bool {
        is_flag_set!(self.mask, 0x10)
    }

    fn show_background(&self) -> bool {
        is_flag_set!(self.mask, 0x08)
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
            self.palette[self.palette_mirror(addr)] = value.0 & 0x3F;
        }
    }

    pub fn frame_data(&self) -> (u64, &[Scanline]) {
        (self.frames, &self.frame_data)
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

#[derive(Copy, Clone, Default)]
struct Sprite {
    pub y_pos       : W<u8>,
    pub tile        : W<u8>,
    pub attributes  : W<u8>,
    pub x_pos       : W<u8>,
    pub counter     : u8,
    pub latch       : u8,
    pub lshift      : u8,
    pub hshift      : u8,
}

impl Sprite {

    pub fn set_sprite_info(&mut self, index: usize, value: W<u8>) {
        match index {
            0 => self.y_pos = value,
            1 => self.tile = value,
            2 => self.attributes = value,
            3 => self.x_pos = value,
            _ => unreachable!(),
        }
    }

    pub fn get_priority(&self) -> bool {
        is_flag_set!(self.attributes.0, 0x20)
    }

    pub fn get_palette(&self) -> usize {
        (self.attributes.0 & 3) as usize
    }

    pub fn flip_horizontally(&self) -> bool {
        is_flag_set!(self.attributes.0, 0x40)
    }

    pub fn flip_vertically(&self) -> bool {
        is_flag_set!(self.attributes.0, 0x80)
    }

    pub fn has_pixel(&self) -> bool {
        self.counter == 0 && (self.lshift & 1 != 0 || self.hshift & 1 != 0)
    }
}

struct Oam {
    mem         : [u8; 0x100],
    smem        : [u8; 0x20],
    sprite      : W<u8>,
    ssprite     : usize,
    count       : usize,
    address     : W<u8>,
    read        : u8,
    next_sprite : bool,
}

impl Default for Oam {
    fn default() -> Oam {
        Oam {
            mem         : [0; 0x100],
            smem        : [0; 0x20],
            sprite      : W(0),
            ssprite     : 0,
            count       : 0,
            address     : W(0),
            read        : 0,
            next_sprite : false,
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

    // The amount of sprites we found
    fn count(&self) -> usize {
        self.count
    }

    pub fn cycle(&mut self, cycles: usize, scanline: u8,
                 spr_units: &mut [Sprite]) -> bool {
        if cycles == 0 {
            self.sprite = W(0);
            self.ssprite = 0;
            return false;
        }
        let cycles = cycles - 1;
        if cycles < 64 {
            // Fill OAM
            if cycles % 2 == 0 {
                self.read = 0xFF;
            } else {
                self.smem[cycles >> 1] = self.read;
            }
        } else if cycles < 256 {
            // Read on even cycles
            if cycles % 2 == 0 {
                self.read = self.mem[self.sprite.0 as usize];
                return false;
            }
            if self.ssprite % 4 != 0 {
                // Copy the rest of the sprite data when previous was in range
                self.smem[self.ssprite] = self.read;
                self.ssprite += 1;
                self.sprite += W(1);
            } else if self.ssprite < 0x20 {
                // Copy the Y coordinate and test if in range
                self.smem[self.ssprite] = self.read;
                // If sprite is in range copy the rest, else go to the next one
                if self.in_range(scanline) {
                    self.ssprite += 1;
                    self.sprite += W(1);
                } else {
                    self.sprite += W(4);
                }
            } else {
                // 8 sprite limit reached, look for sprite overflow
                if self.sprite.0 % 4 != 0 && !self.next_sprite {
                    self.sprite += W(1);
                } else if self.in_range(scanline) {
                    self.sprite += W(1);
                    self.next_sprite = false;
                    // Sprite overflow
                    return true;
                } else {
                    // Emulate hardware bug, add 5 instead of 4
                    // FIXME: I think there shouldn't be a carry from bit 1 to 2
                    self.sprite += W(5);
                    self.next_sprite = true;
                }
            }
        } else if cycles < 320 {
            // Set index to 0 at start so we can copy to the sprite units.
            // Set also the count to the amount of sprites we have found
            if cycles == 256 {
                self.sprite = W(0);
                self.count = self.ssprite / 4;
            }
            // Fill up to eight sprite units with data
            if cycles & 4 == 0 && (self.sprite.0 as usize) < self.ssprite {
                let data = W(self.smem[self.sprite.0 as usize]);
                let sprite = &mut spr_units[(self.sprite.0 / 4) as usize];
                sprite.set_sprite_info(cycles % 4, data);
                self.sprite += W(1);
            }
        }
        return false;
    }

    fn in_range(&self, scanline: u8) -> bool {
        self.read < 0xF0 && self.read + 8 > scanline && self.read <= scanline
    }
}
