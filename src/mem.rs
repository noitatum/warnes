// nes
use mapper::Mapper;
use loadstore::LoadStore;
use utils::print_mem;
use enums::{MemState, IoState};
// std
use std::num::Wrapping as W;
use std::fmt;

const RAM_SIZE  : usize = 0x800;
const VRAM_SIZE : usize = 0x800;
const GAMEPAD1  : W<u16> = W(0x4016);
const GAMEPAD2  : W<u16> = W(0x4017);


pub struct Memory {
    ram                 : [u8; RAM_SIZE],
    vram                : [u8; VRAM_SIZE],
    mapper              : Box<Mapper>,
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
    pub fn new(mapper: Box<Mapper>) -> Memory {
        Memory {
            ram                 : [0; RAM_SIZE],
            vram                : [0; VRAM_SIZE], 

            mapper              : mapper,

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
    
    pub fn chr_load(&mut self, address: W<u16>) -> W<u8> {
        W(self.mapper.chr_load(&mut self.vram[..], address))
    }

    pub fn chr_store(&mut self, address: W<u16>, value: W<u8>){
        self.mapper.chr_store(&mut self.vram[..], address, value.0);
    }

    pub fn get_io_load_status(&mut self, gp : W<u16>) -> bool {
        if gp == GAMEPAD1 {
            return self.get_io_load_status_gp1();
        } else if gp == GAMEPAD2 {
            return self.get_io_load_status_gp2();
        }
        return false;
    }

    pub fn get_io_load_status_gp1(&mut self) -> bool {
        if let IoState::GamePad1 = self.io_load_status {
            true
        } else{
            false
        }
    } 

    pub fn get_io_load_status_gp2(&mut self) -> bool {
        if let IoState::GamePad2 = self.io_load_status {
            true
        } else{
            false
        }
    }

    pub fn set_io_store(&mut self, state: IoState) {
        self.io_store_status = state;
    }

    pub fn get_strobe(&self) -> u8 {
        self.joy1 
    }

}

impl LoadStore for Memory {
    fn load(&mut self, address: W<u16>) -> W<u8> {
        let addr = address.0; 
        let value = if addr < 0x2000 {
            self.mem_load_status = MemState::Memory;
            self.ram[(addr & 0x7ff) as usize]
        } else if addr < 0x4000 {
            // FIXME: This is broken now for status and oamdata
            self.mem_load_status = match addr & 0x7 {
                // Other registers are read only
                2 => MemState::PpuStatus,
                4 => MemState::OamData,
                7 => MemState::PpuData,
                _ => MemState::NoState,
            };
            self.latch.0
        } else if addr < 0x4020 {
            /* Apu AND IO TODO*/
            //self.mem_load_status = MemState::Io;
            match addr {
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
                            self.mem_load_status = MemState::Io;
                            self.joy1 
                          }
                0x4017 => { self.io_load_status = IoState::GamePad2;
                            self.mem_load_status = MemState::Io;
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
            self.mapper.prg_load(address)
        };
        W(value)
    }

    fn store (&mut self, address: W<u16>, value: W<u8>) {
        let addr = address.0; 
        let val = value.0;
        if addr < 0x2000 {
            self.mem_store_status = MemState::Memory;
            self.ram[(addr & 0x7ff) as usize] = val
        } else if addr < 0x4000 {
            self.mem_store_status = match addr & 0x7 {
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
        } else if addr < 0x4020 {
            /* Apu AND IO TODO*/
            self.mem_store_status = MemState::Io;
            match addr {
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
                0x4017 => { self.joy2 = val;
                            self.mem_store_status = MemState::Io;
                          },
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
            self.mapper.prg_store(address, val);
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
