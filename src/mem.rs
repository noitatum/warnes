use ppu::Ppu;
use std::num::Wrapping as W;

const PAGE_MASK         : W<u16> = W(0xFF00 as u16);

pub struct Memory {
    ram : [u8; 2048],
    ppu : Ppu,
}

impl Memory {
    pub fn new (ppu : Ppu) -> Memory {
        Memory {
            ram : [0;  2048],
            ppu : ppu,
        }
    }

    pub fn load (&self, address: W<u16>) -> W<u8> {
        let address = address.0; 
        W(if address < 0x2000 {
            self.ram[(address & 0x7ff) as usize]
        } else if address < 0x4000 {
            self.ppu.load(W(address)).0
        } else if address < 0x4020 {
            /* Apu AND IO TODO*/
            if address == 0x4014 {
                self.ppu.load(W(address)).0
            } else{
                0
            }
            
        } else if address < 0x6000 {
            /* Cartridge expansion ROM the f */
            0
        } else if address < 0x8000 {
            /* SRAM */
            0
        } else /* 0x8000 <= addr < 0xC000*/ {
            /* PRG-ROM */
            0
        })
    }

    pub fn store (&mut self, address: W<u16>, value: W<u8>){
        let address = address.0; 
        let val = value.0;
        if address < 0x2000 {
            self.ram[(address & 0x7ff) as usize] = val
        } else if address < 0x4000 {
            self.ppu.store(W(address), value)
        } else if address < 0x4020 {
            /* Apu AND IO TODO*/
            if address == 0x4014 {
                self.ppu.store(W(address), value)
            }
        } else if address < 0x6000 {
            /* Cartridge expansion ROM the f */
            
        } else if address < 0x8000 {
            /* SRAM */
           
        } else /* 0x8000 <= address < 0xC000*/ {
            /* PRG-ROM */
           
        }
    }

    pub fn load_word(&mut self, address: W<u16>) -> W<u16> {
        let low = W16!(self.load(address));
        (W16!(self.load(address + W(1))) << 8) | low
    }

    pub fn load_word_page_wrap(&mut self, address: W<u16>) -> W<u16> {
        let low = self.load(address);
        let high = self.load((address & PAGE_MASK) | W16!(W8!(address) + W(1)));
        (W16!(high) << 8) | W16!(low)
    }

    pub fn store_word(&mut self, address: W<u16>, word: W<u16>) {
        self.store(address, W8!(word >> 8));
        self.store(address + W(1), W8!(word));
    }
}
