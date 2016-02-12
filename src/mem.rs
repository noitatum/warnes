use ppu::Ppu;
use std::num::Wrapping as W;

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
        if addr < 0x2000 {
            W(self.ram[(addr & 0x7ff) as usize])
        } else if addr < 0x4000 {
            W(match (addr % 0x2000) & 0x7 {
                0 => self.ppu.ppuctrl,
                1 => self.ppu.ppumask,
                2 => self.ppu.ppustatus,
                3 => self.ppu.oamaddr,
                4 => self.ppu.oamdata,
                5 => self.ppu.ppuscroll,
                6 => self.ppu.ppuaddr,
                7 => self.ppu.ppudata,
                _ => 0 // fuck you.
            })
        } else if addr < 0x4020 {
            /* Apu TODO*/
            W(0) 
        } else if addr < 0x6000 {
            /* Cartridge expansion ROM the f */
            W(0)
        } else if addr < 0x8000 {
            /* SRAM */
            W(0)
        } else /* 0x8000 <= address < 0xC000*/ {
            /* PRG-ROM */
            W(0)
        }
    }

    pub fn write (&mut self, address: W<u16>, value : W<u8>){
        let addr = address.0;
        let val = value.0;
        if addr < 0x2000 {
            self.ram[(addr & 0x7ff) as usize] = val
        } else if addr < 0x4000 {
            match (addr % 0x2000) & 0x7 {
                0 => self.ppu.ppuctrl = val,
                1 => self.ppu.ppumask = val, 
                2 => self.ppu.ppustatus = val,
                3 => self.ppu.oamaddr = val,
                4 => self.ppu.oamdata = val, 
                5 => self.ppu.ppuscroll = val,
                6 => self.ppu.ppuaddr = val,
                7 => self.ppu.ppudata = val,
                _ => self.ppu.ppuctrl = val  // epic.
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
}
