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
            match address {
                0x4000 => 0,
                0x4001 => 0,
                0x4002 => 0,
                0x4003 => 0,
                0x4004 => 0,
                0x4005 => 0,
                0x4006 => 0,
                0x4007 => 0,
                0x4008 => 0,
                0x4009 => 0,
                0x400A => 0,
                0x400B => 0,
                0x400C => 0,
                0x400D => 0,
                0x400E => 0,
                0x400F => 0,
                0x4010 => 0,
                0x4011 => 0,
                0x4012 => 0,
                0x4013 => 0,
                0x4014 => self.ppu.load(W(address)).0,
                0x4015 => 0,
                0x4016 => 0,
                0x4017 => 0,
                0x4018 => 0,
                0x4019 => 0,
                0x401A => 0,
                0x401B => 0,
                0x401C => 0,
                0x401D => 0,
                0x401E => 0, 
                0x401F => 0,
                _      => 0,
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
            match address {
                0x4000 =>   (),
                0x4001 =>   (),
                0x4002 =>   (),
                0x4003 =>   (),
                0x4004 =>   (),
                0x4005 =>   (),
                0x4006 =>   (),
                0x4007 =>   (),
                0x4008 =>   (),
                0x4009 =>   (),
                0x400A =>   (),
                0x400B =>   (),
                0x400C =>   (),
                0x400D =>   (),
                0x400E =>   (),
                0x400F =>   (),
                0x4010 =>   (),
                0x4011 =>   (),
                0x4012 =>   (),
                0x4013 =>   (),
                0x4014 =>   {   
                                self.ppu.store(W(address), value);
                                // When oamdma is written to
                                // the cpu locks down and fills the
                                // the oam memory with the selected page.
                                // (value in oamdma).
                                for i in 0..256 {
                                    let byte = self.load((W16!(value) << 8) + W(i));
                                    self.store(W(0x2004), byte);
                                }       
                            },
                0x4015 =>   (),
                0x4016 =>   (),
                0x4017 =>   (),
                0x4018 =>   (),
                0x4019 =>   (),
                0x401A =>   (),
                0x401B =>   (),
                0x401C =>   (),
                0x401D =>   (),
                0x401E =>   (), 
                0x401F =>   (),
                _      =>   (),
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
