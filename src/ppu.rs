use std::num::Wrapping as W;
use std::fmt;

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

 

pub struct Ppu {
    pub ppuctrl         : u8,
    pub ppumask         : u8,
    pub ppustatus       : u8,
    pub oamaddr         : u8,
    pub oamdata         : u8,
    pub ppuscroll       : u8,
    pub ppuaddr         : u8,
    pub ppudata         : u8,

    pub oamdma          : u8,
    
    pub oam             : [u8; 256],    // Object atribute memory 
    pub vram            : [u8; 0x4000], // 16kb

    // status
    pub oam_writable    : bool,
    pub oam_write_bytes : u8,
}

impl Ppu {
    pub fn new () -> Ppu {
        Ppu {
            ppuctrl         : 0,
            ppumask         : 0,
            ppustatus       : 0,
            oamaddr         : 0,
            oamdata         : 0,
            ppuscroll       : 0,
            ppuaddr         : 0,
            ppudata         : 0,
            
            oamdma          : 0,

            oam             : [0; 256],
            vram            : [0;  0x4000],

            oam_writable    : false,
            oam_write_bytes : 0,
        }
    }

    pub fn load (&self, address: W<u16>) -> W<u8> {
       let address = address.0;
       W(if address != 0x4014 {
           match (address % 0x2000) & 0x7 {
                // En teoria los registros comentados son read only
                // 0 => self.ppuctrl
                // 1 => self.ppumask,
                2 => self.ppustatus,
                // 3 => self.oamaddr,
                4 => self.oamdata,
                // 5 => self.ppuscroll,
                // 6 => self.ppuaddr,
                7 => self.ppudata,
                _ => 0 // fuck you.
            }
       } else {
            0
       })
    }

    pub fn store (&mut self, address: W<u16>, value: W<u8> ){
        let address = address.0;
        let val = value.0;
        if address != 0x4014 {
            match (address % 0x2000) & 0x7 {
                0 =>    self.ppuctrl = val,
                1 =>    self.ppumask = val, 
                // 2 => self.ppustatus = value, Este registro es read only
                3 =>    self.oamaddr = val,
                4 =>    {  if self.oam_writable {
                                self.oam[self.oamdma as usize] = val;
                                self.oam_write_bytes += 1;
                                if self.oam_write_bytes == 255 { 
                                    self.oam_writable = false;
                                    self.oam_write_bytes = 0;
                                }
                            } else {
                                self.oamdata = val;
                            }
                        },
                5 =>    self.ppuscroll = val,
                6 =>    self.ppuaddr = val,
                7 =>    self.ppudata = val,
                _ =>    self.ppuctrl = self.ppuctrl  // epic.
            };
        } else {
            self.oam_writable = true;
            self.oamdma = val;
        };
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
        let word : W<u16> = W16!(self.load(address));
        word | W16!(self.load(address + W(1)) << 8)
    }

    pub fn store_word_vram (&mut self, address: W<u16>, word: W<u16>) {
        self.store(address, W8!(word >> 8));
        self.store(address + W(1), W8!(word))
    }

}

impl fmt::Debug for Ppu {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut output  = "oam: [".to_string();
        for i in 0..255 {
            output.push_str(&format!("{:#x}|", self.oam[i]));
        }
        output.push_str(&format!("{:#x}]", self.oam[255]));
        write!(f, "{}", output)
    }
}
