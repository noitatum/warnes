use ppu::Ppu;

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

    pub fn load (&self, address: u16) -> u8 {
        if address < 0x2000 {
            return self.ram[ (address & 0x7ff) as usize]
        } else if address < 0x4000 {
            return match (address % 0x2000) & 0x7 {
                //0 => self.ppu.ppuctrl, En teoria los registros comentados son write only
                //1 => self.ppu.ppumask,
                2 => self.ppu.ppustatus,
                //3 => self.ppu.oamaddr,
                4 => self.ppu.oamdata,
                //5 => self.ppu.ppuscroll,
                //6 => self.ppu.ppuaddr,
                7 => self.ppu.ppudata,
                _ => 0 // fuck you.
            }
        } else if address < 0x4020 {
            /* Apu TODO*/
            return 0 
        } else if address < 0x6000 {
            /* Cartridge expansion ROM the f */
            return 0
        } else if address < 0x8000 {
            /* SRAM */
            return 0
        } else /* 0x8000 <= address < 0xC000*/ {
            /* PRG-ROM */
            return 0
        }
    }

    pub fn write (&mut self, address: u16, value : u8){
        if address < 0x2000 {
            self.ram[ (address & 0x7ff) as usize] = value
        } else if address < 0x4000 {
            match (address % 0x2000) & 0x7 {
                0 => self.ppu.ppuctrl = value,
                1 => self.ppu.ppumask = value, 
                //2 => self.ppu.ppustatus = value, Este registro es read only
                3 => self.ppu.oamaddr = value,
                4 => self.ppu.oamdata = value, 
                5 => self.ppu.ppuscroll = value,
                6 => self.ppu.ppuaddr = value,
                7 => self.ppu.ppudata = value,
                _ => self.ppu.ppuctrl = self.ppu.ppuctrl  // epic.
            }
        } else if address < 0x4020 {
            /* Apu TODO*/
             
        } else if address < 0x6000 {
            /* Cartridge expansion ROM the f */
            
        } else if address < 0x8000 {
            /* SRAM */
           
        } else /* 0x8000 <= address < 0xC000*/ {
            /* PRG-ROM */
           
        }
    }
}
