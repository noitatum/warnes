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
        let addr = address.0; 
        W(if addr < 0x2000 {
            self.ram[(addr & 0x7ff) as usize]
        } else if addr < 0x4000 {
            match (addr % 0x2000) & 0x7 {
                // En teoria los registros comentados son read only
                // 0 => self.ppu.ppuctrl
                // 1 => self.ppu.ppumask,
                2 => self.ppu.ppustatus,
                // 3 => self.ppu.oamaddr,
                4 => self.ppu.oamdata,
                // 5 => self.ppu.ppuscroll,
                // 6 => self.ppu.ppuaddr,
                7 => self.ppu.ppudata,
                _ => 0 // fuck you.
            }
        } else if addr < 0x4020 {
            /* Apu TODO*/
            0 
        } else if addr < 0x6000 {
            /* Cartridge expansion ROM the f */
            0
        } else if addr < 0x8000 {
            /* SRAM */
            0
        } else /* 0x8000 <= addr < 0xC000*/ {
            /* PRG-ROM */
            0
        })
    }

    pub fn store (&mut self, address: W<u16>, value: W<u8>){
        let addr = address.0; 
        let val = value.0;
        if addr < 0x2000 {
            self.ram[(addr & 0x7ff) as usize] = val
        } else if addr < 0x4000 {
            match (addr % 0x2000) & 0x7 {
                0 => self.ppu.ppuctrl = val,
                1 => self.ppu.ppumask = val, 
                // 2 => self.ppu.ppustatus = value, Este registro es read only
                3 => self.ppu.oamaddr = val,
                4 => self.ppu.oamdata = val, 
                5 => self.ppu.ppuscroll = val,
                6 => self.ppu.ppuaddr = val,
                7 => self.ppu.ppudata = val,
                _ => self.ppu.ppuctrl = self.ppu.ppuctrl  // epic.
            }
        } else if addr < 0x4020 {
            /* Apu TODO*/
             
        } else if addr < 0x6000 {
            /* Cartridge expansion ROM the f */
            
        } else if addr < 0x8000 {
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
