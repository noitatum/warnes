use loadstore::LoadStore;
use utils::print_mem;

use std::num::Wrapping as W;
use std::fmt;

const RAM_SIZE  : usize = 0x800;
const VRAM_SIZE : usize = 0x800;

#[derive(Clone, Copy)]
pub enum MemState {
    PpuCtrl,
    PpuMask,
    PpuStatus,
    OamAddr,
    OamData,
    PpuScroll,
    PpuAddr,
    PpuData,
    Io,
    Memory,
    NoState,
}

#[derive(Clone, Copy)]
pub enum IoState {
    GamePad1,
    GamePad2,
    NoState,
}

impl fmt::Display for MemState{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}",
            match *self {
                MemState::PpuCtrl       => "PpuCtrl",
                MemState::PpuMask       => "PpuMask",
                MemState::PpuStatus     => "PpuStatus",
                MemState::OamAddr       => "OamAddr",
                MemState::OamData       => "OamData",
                MemState::PpuScroll     => "PpuScroll",
                MemState::PpuAddr       => "PpuAddr",
                MemState::PpuData       => "PpuData",
                MemState::Memory        => "Memory",
                MemState::Io            => "Io",
                MemState::NoState       => "NoState",
            }
        )
    }
}

impl fmt::Display for IoState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}",
            match *self {
                IoState::GamePad1      => "GamePad1",
                IoState::GamePad2      => "GamePad2",
                IoState::NoState       => "NoState",
            }
        )
    }
}

pub struct Memory {
    ram             : [u8; RAM_SIZE],
    vram            : [u8; VRAM_SIZE],

    mem_load_status     : MemState,
    mem_store_status    : MemState,

    io_load_status      : IoState,
    io_store_status     : IoState,

    latch               : W<u8>,
    oamdma              : Option<W<u8>>,

    joy1                : u8,
    joy2                : u8,
}

impl Memory {
    pub fn new () -> Memory {
        Memory {
            ram             : [0; RAM_SIZE],
            vram            : [0; VRAM_SIZE], 

            mem_load_status     : MemState::NoState,
            mem_store_status    : MemState::NoState,            

            io_load_status      : IoState::NoState,
            io_store_status     : IoState::NoState,

            latch               : W(0),
            oamdma              : None,

            joy1                : 0,
            joy2                : 0,
        }
    }

    pub fn get_latch(&mut self) -> (W<u8>, MemState) {
        let status = (self.latch, self.mem_store_status);
        self.mem_store_status = MemState::NoState;
        return status;
    }

    pub fn set_latch(&mut self, value: W<u8>) {
        self.latch = value;
    }

    pub fn get_mem_load_status(&mut self) -> MemState {
        let status = self.mem_load_status;
        self.mem_load_status = MemState::NoState;
        return status;
    }

    pub fn get_oamdma(&mut self) -> Option<W<u8>> {
        let status = self.oamdma;
        self.oamdma = None;
        return status;
    }
    
    // FIXME Broken code, fix and move to mapper

    pub fn chr_load(&mut self, address: W<u16>) -> W<u8> {
        let value = if address.0 < 0x3000 {
            self.vram[address.0 as usize]
        } else if address.0 < 0x3F00 {
            self.vram[(address.0 - 0x1000) as usize]
        } else if address.0 < 0x3F20 {
            self.vram[address.0 as usize]
        } else if address.0 < 0x4000 {
            self.vram[(address.0 - 0x100) as usize]
        } else {
            self.vram[(address.0 % 0x4000) as usize]
        };
        W(value)
    }

    pub fn chr_store(&mut self, address: W<u16>, value: W<u8>){
        if address.0 < 0x3000 {
            self.vram[address.0 as usize] = value.0;
        } else if address.0 < 0x3F00 {
            self.vram[(address.0 - 0x1000) as usize] = value.0;
        } else if address.0 < 0x3F20 {
            self.vram[address.0 as usize] = value.0;
        } else if address.0 < 0x4000 {
            self.vram[(address.0 - 0x100) as usize] = value.0;
        } else {
            self.vram[(address.0 % 0x4000) as usize] = value.0;
        }
    }

    pub fn get_io_load_status(&mut self) -> bool {
        if let IoState::GamePad1 = self.io_load_status {
            true
        } else{
            false
        }
    } 

    pub fn set_io_store(&mut self, state: IoState) {
        self.io_store_status = state;
    }

    pub fn get_joy1(&self) -> u8 {
        self.joy1 
    }

    pub fn get_joy2(&self) -> u8 {
        self.joy2
    }

}

impl LoadStore for Memory {
    fn load(&mut self, address: W<u16>) -> W<u8> {
        let address = address.0; 
        let value = if address < 0x2000 {
            self.mem_load_status = MemState::Memory;
            self.ram[(address & 0x7ff) as usize]
        } else if address < 0x4000 {
            // FIXME: This is broken now for status and oamdata
            self.mem_load_status = match address & 0x7 {
                // Other registers are read only
                2 => MemState::PpuStatus,
                4 => MemState::OamData,
                7 => MemState::PpuData,
                _ => MemState::NoState,
            };
            self.latch.0
        } else if address < 0x4020 {
            /* Apu AND IO TODO*/
            //self.mem_load_status = MemState::Io;
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
                0x4014 => 0, // OAMDMA is Write only, TODO: Check what happens
                0x4015 => 0,
                0x4016 => { self.io_load_status = IoState::GamePad1;
                            self.joy1 
                          }
                0x4017 => { self.io_load_status = IoState::GamePad2;
                            self.joy2 
                          }
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
        } else {
            //self.mapper.load()
            0
        };
        W(value)
    }

    fn store (&mut self, address: W<u16>, value: W<u8>) {
        let address = address.0; 
        let val = value.0;
        if address < 0x2000 {
            self.mem_store_status = MemState::Memory;
            self.ram[(address & 0x7ff) as usize] = val
        } else if address < 0x4000 {
            self.mem_store_status = match address & 0x7 {
                0 => MemState::PpuCtrl,
                1 => MemState::PpuMask,
             // 2 => MemState::PpuStatus Read Only Register 
                3 => MemState::OamAddr,
                4 => MemState::OamData,
                5 => MemState::PpuScroll,
                6 => MemState::PpuAddr,
                7 => MemState::PpuData,
                _ => MemState::NoState, 
            };
            self.latch = value;
        } else if address < 0x4020 {
            /* Apu AND IO TODO*/
            self.mem_store_status = MemState::Io;
            match address {
                0x4000 => (),
                0x4001 => (),
                0x4002 => (),
                0x4003 => (),
                0x4004 => (),
                0x4005 => (),
                0x4006 => (),
                0x4007 => (),
                0x4008 => (),
                0x4009 => (),
                0x400A => (),
                0x400B => (),
                0x400C => (),
                0x400D => (),
                0x400E => (),
                0x400F => (),
                0x4010 => (),
                0x4011 => (),
                0x4012 => (),
                0x4013 => (),
                // When oamdma is written to
                // the cpu locks down and fills the
                // the oam memory with the selected page.
                0x4014 => { 
                    self.oamdma = Some(value);
                },
                0x4015 => (),
                0x4016 => { self.joy1 = val;
                            self.io_store_status = IoState::GamePad1;
                          },
                0x4017 => { /*
                    if let IoState::GamePad2 = self.io_load_status {
                        self.joy2 = val;
                    }
                    if self.keystrobe2 && ((self.joy2 & 1) == 0) {
                        self.io_load_status = IoState::StartGamePad2;
                        self.keystrobe2 = false;
                    } else if self.joy2 & 1 > 0 {
                        self.keystrobe2 = true;
                    }
                */},
                0x4018 => (),
                0x4019 => (),
                0x401A => (),
                0x401B => (),
                0x401C => (),
                0x401D => (),
                0x401E => (), 
                0x401F => (),
                _      => (),
            }
        } else {
            //self.mapper.store()
        }
    }
}

impl fmt::Debug for Memory {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut output = "RAM: \n".to_string();
        output.push_str("RAM:\n");
        print_mem(&mut output, &self.ram[..]);
        output.push_str("VRAM:\n");
        print_mem(&mut output, &self.vram[..]);
        write!(f, "{{ latch: {:#x}, oamdma: {:?}, mem_load_status: {}, mem_store_status: {}}}, \n {}", self.latch.0, self.oamdma, self.mem_load_status, self.mem_store_status, output)
    }
}

impl Default for Memory {
    fn default () -> Memory {
        Memory::new()
    }
}
