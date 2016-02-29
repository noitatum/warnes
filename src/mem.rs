use loadstore::LoadStore;

use std::num::Wrapping as W;
use std::fmt;

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
    ReadGamePad1,
    ReadGamePad2,
    StartReadGamePad1,
    StartReadGamePad2,
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
                IoState::ReadGamePad1      => "ReadGamePad1",
                IoState::ReadGamePad2      => "ReadGamePad2",
                IoState::StartReadGamePad1 => "StartReadGamePad1",
                IoState::StartReadGamePad2 => "StartReadGamePad2",
                IoState::NoState           => "NoState",
            }
        )
    }
}

pub struct Memory {
    ram : [u8; 2048],

    read_status     : MemState,
    write_status    : MemState,

    io_read_status  : IoState,
    pub io_write_status : IoState,

    latch           : W<u8>,
    oamdma          : Option<W<u8>>,

    keystrobe1      : bool,
    keystrobe2      : bool,
    joy1            : u8,
    joy2            : u8,
}

impl Memory {
    pub fn new () -> Memory {
        Memory {
            ram : [0;  2048],
            read_status     : MemState::NoState,
            write_status    : MemState::NoState,            

            io_read_status  : IoState::NoState,
            io_write_status : IoState::NoState,

            latch           : W(0),
            oamdma          : None,

            keystrobe1      : false,
            keystrobe2      : false,
            joy1            : 0,
            joy2            : 0,
        }
    }

    pub fn get_latch(&mut self) -> (W<u8>, MemState) {
        let res = (self.latch, self.write_status);
        self.write_status = MemState::NoState;
        res
    }

    pub fn set_latch(&mut self, value: W<u8>) {
        self.latch = value;
    }

    pub fn get_read_status(&mut self) -> MemState {
        let res = self.read_status;
        self.read_status = MemState::NoState;
        res
    }

    pub fn get_oamdma(&mut self) -> Option<W<u8>> {
        let res = self.oamdma;
        self.oamdma = None;
        res
    }
}

impl LoadStore for Memory {
    fn load(&mut self, address: W<u16>) -> W<u8> {
        let address = address.0; 
        let value = if address < 0x2000 {
            self.read_status = MemState::Memory;
            self.ram[(address & 0x7ff) as usize]
        } else if address < 0x4000 {
            // FIXME: This is broken now for status and oamdata
            self.read_status = match (address % 0x2000) & 0x7 {
                // Other registers are read only
                2 => MemState::PpuStatus,
                4 => MemState::OamData,
                7 => MemState::PpuData,
                _ => MemState::NoState,
            };
            self.latch.0
        } else if address < 0x4020 {
            /* Apu AND IO TODO*/
            //self.read_status = MemState::Io;
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
                0x4016 => {   
                    if let IoState::ReadGamePad1 = self.io_read_status {
                        self.joy1
                    } else {
                        0
                    }
                }
                0x4017 => {
                    if let IoState::ReadGamePad2 = self.io_read_status {
                        self.joy1
                    } else {
                        0
                    }
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
            self.write_status = MemState::Memory;
            self.ram[(address & 0x7ff) as usize] = val
        } else if address < 0x4000 {
            self.write_status = match address & 0x7 {
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
            self.write_status = MemState::Io;
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
                0x4016 => {   
                    if let IoState::ReadGamePad1 = self.io_read_status {
                        self.joy1 = val;
                    }
                    if self.keystrobe1 && ((self.joy1 & 1) == 0) {
                        self.io_read_status = IoState::StartReadGamePad1;
                        self.keystrobe2 = false;
                    } else if self.joy1 & 1 > 0 {
                        self.keystrobe1 = true;
                    }
                },
                0x4017 => {
                    if let IoState::ReadGamePad2 = self.io_read_status {
                        self.joy2 = val;
                    }
                    if self.keystrobe2 && ((self.joy2 & 1) == 0) {
                        self.io_read_status = IoState::StartReadGamePad2;
                        self.keystrobe2 = false;
                    } else if self.joy2 & 1 > 0 {
                        self.keystrobe2 = true;
                    }
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
            //self.mapper.store()
        }
    }
}

impl fmt::Debug for Memory {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut output = "ram: [".to_string();
        for i in 0..2048 {
            output.push_str(&format!("|{:02x}", self.ram[i]));
        }
        write!(f, "{{ latch: {:#x}, oamdma: {:?}, read_status: {}, write_status: {}}}, \n {}", self.latch.0, self.oamdma, self.read_status, self.write_status, output)
    }
}

impl Default for Memory {
    fn default () -> Memory {
        Memory::new()
    }
}
