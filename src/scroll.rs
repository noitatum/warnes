use std::num::Wrapping as W;
use std::fmt;

const ATTRIBUTE_OFFSET : W<u16> = W(0x03C0);
const NAMETABLE_OFFSET : W<u16> = W(0x2000);
const NAMETABLE_X_BIT  : W<u16> = W(0x0400);
const NAMETABLE_Y_BIT  : W<u16> = W(0x0800);
const NAMETABLE_MASK   : W<u16> = W(0x0C00); 
const COARSE_X_MASK    : W<u16> = W(0x001F);
const COARSE_Y_MASK    : W<u16> = W(0x03E0);
const FINE_Y_MASK      : W<u16> = W(0x7000);
const BG_OFFSET_FLAG   : W<u8>  = W(0x10);
const INCREMENT_FLAG   : W<u8>  = W(0x40);

/* Coarse is 5 upper bits of a scroll (Byte selection) 
 * Fine is 3 lower bits of a scroll (Pixel selection inside byte)
 * Nametable selection is represented by 2 bits
 * Address and temporal are 15 bit wide and composed by:
 * fine_y | nametable | coarse_y | coarse_x
 * fine_x has its own separate register
 */

pub struct Scroll { 
    address     : W<u16>,
    temporal    : W<u16>,
    fine_x      : W<u8>,
    write_flag  : bool,
    bg_offset   : W<u16>,
    increment   : W<u16>,
}

impl Default for Scroll {
    fn default() -> Scroll {
        Scroll {
            address     : W(0),
            temporal    : W(0),
            fine_x      : W(0),
            write_flag  : false,
            bg_offset   : W(0),
            increment   : W(1),
        }
    }
}

impl Scroll { 
    pub fn reset(&mut self) {
        self.write_flag = false;
    }

    pub fn get_address(&self) -> W<u16> {
        // The lower 14 bits compose a full address
        self.address & W(0x3FFF)
    }

    pub fn get_nametable_address(&self) -> W<u16> {
        // The lower 12 bits are the position in the nametables
        NAMETABLE_OFFSET | self.address & W(0xFFF)
    }

    pub fn get_attribute_address(&self) -> W<u16> {
        // Nametable | High 3 bits of scroll_y | High 3 bits of scroll_x
        NAMETABLE_OFFSET | ATTRIBUTE_OFFSET    |
            self.address      & NAMETABLE_MASK |
            self.address >> 4 & W(0x0038)      |
            self.address >> 2 & W(0x0007)
    }

    pub fn get_tile_address(&self, index: W<u8>) -> W<u16> {
        self.bg_offset | W16!(index) << 4 | (self.address & FINE_Y_MASK) >> 12
    }

    pub fn set_ppuctrl(&mut self, value: W<u8>) {
        self.temporal = self.temporal & !NAMETABLE_MASK |
                        W16!(value & W(0x3)) << 10;
        self.bg_offset = W16!(value & BG_OFFSET_FLAG) << 8; 
        self.increment = if value & INCREMENT_FLAG > W(0) {W(1)} else {W(32)};
    }

    pub fn set_address(&mut self, value: W<u8>) {
        if self.write_flag {
            set_low_byte!(self.temporal, value);
            self.address = self.temporal;
        } else {
            set_high_byte!(self.temporal, value & W(0x3F));
        }
        self.write_flag = !self.write_flag;
    }

    pub fn set_scroll(&mut self, value: W<u8>) {
        if self.write_flag {
            self.set_scroll_y(value);
        } else {
            self.set_scroll_x(value);
        }
        self.write_flag = !self.write_flag;
    }

    pub fn set_scroll_x(&mut self, value: W<u8>) {
        self.fine_x = value & W(0x7);
        let coarse_x = W16!(value) >> 3;    
        self.temporal = self.temporal & !COARSE_X_MASK | coarse_x; 
    }

    pub fn get_scroll_x(&mut self) -> u8 {
        return self.fine_x.0;
    }

    pub fn set_scroll_y(&mut self, value: W<u8>) {
        let fine_y = W16!(value & W(0x07)) << 12; 
        let coarse_y = W16!(value & W(0xF8)) << 2; 
        self.temporal = self.temporal & !COARSE_Y_MASK | fine_y | coarse_y; 
    }

    pub fn get_scroll_y(&mut self) -> W<u8> {
        W8!((self.address & COARSE_Y_MASK) >> 2 | 
            (self.address & FINE_Y_MASK) >> 12)
    }

    pub fn increment_coarse_x(&mut self) {
        // If coarse_x is about to overflow
        if self.address & COARSE_X_MASK == COARSE_X_MASK {
            // Wrap coarse_x to 0 and go to next nametable
            self.address = self.address & !COARSE_X_MASK;
            self.address = self.address ^ NAMETABLE_X_BIT;
        } else {
            self.address = self.address + W(1);
        } 
    }
    
    pub fn increment_y(&mut self) {
        let mut scroll_y = self.get_scroll_y() + W(1);
        // If coarse_y overflowed into the attribute table
        if scroll_y == W(0xF0) { 
            // Wrap coarse_y to 0 and go to next nametable
            scroll_y = W(0);
            self.address = self.address ^ NAMETABLE_Y_BIT;
        } 
        self.set_scroll_y(scroll_y);
    }
}

impl fmt::Debug for Scroll {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "address: {:#X}, temporal: {:#X}, \
                   fine_x: {:#X}, write_flag: {:?}", 
               self.address.0, self.temporal.0, self.fine_x.0, self.write_flag)
    }
}
