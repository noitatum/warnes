// nes
use mapper::Mapper;
use loadstore::LoadStore;
use utils::print_mem;
use enums::{MemState, IoState, Interrupt};
use ppu::PpuReadRegs;
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
    ppu_read_regs       : PpuReadRegs,
    latch               : W<u8>,
    oamdma              : Option<W<u8>>,
    interrupt           : Option<Interrupt>,
    io_strobe           : u8,
    joy_key             : [u8; 2],
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
            ppu_read_regs       : Default::default(),
            latch               : W(0),
            oamdma              : None,
            interrupt           : None,
            io_strobe           : 0,
            joy_key             : [0; 2],
        }
    }

    pub fn set_interrupt(&mut self, interrupt: Interrupt) {
        self.interrupt = Some(interrupt);
    }

    pub fn get_interrupt(&mut self) -> Option<Interrupt> {
        let interrupt = self.interrupt;
        self.interrupt = None;
        interrupt
    }

    pub fn get_latch(&mut self) -> (W<u8>, MemState) {
        let status = (self.latch, self.mem_store_status);
        self.mem_store_status = MemState::NoState;
        status
    }

    pub fn get_oamdma(&mut self) -> Option<W<u8>> {
        let status = self.oamdma;
        self.oamdma = None;
        status
    }

    pub fn ppu_load_status(&mut self) -> MemState {
        let status = self.mem_load_status;
        self.mem_load_status = MemState::NoState;
        status
    }

    pub fn set_ppu_read_regs(&mut self, regs: PpuReadRegs) {
        self.ppu_read_regs = regs;
    }

    pub fn chr_load(&mut self, address: W<u16>) -> W<u8> {
        W(self.mapper.chr_load(&mut self.vram[..], address))
    }

    pub fn chr_store(&mut self, address: W<u16>, value: W<u8>){
        self.mapper.chr_store(&mut self.vram[..], address, value.0);
    }

    pub fn get_io_load_status(&mut self) -> IoState {
        let status = self.io_load_status;
        self.io_load_status = IoState::NoState;
        status
    }

    pub fn get_strobe(&self) -> bool {
        self.io_strobe & 1 > 0
    }

    pub fn set_joy_key(&mut self, index: usize, key: u8) {
        self.joy_key[index] = key;
    }

    pub fn load_no_side_effect(&mut self, address: W<u16>) -> W<u8> {
        let mem_load_status  = self.mem_load_status;
        let mem_store_status = self.mem_store_status;
        let io_load_status   = self.io_load_status;
        let value = self.load(address);
        self.mem_load_status  = mem_load_status;
        self.mem_store_status = mem_store_status;
        self.io_load_status   = io_load_status;
        value
    }
}

impl LoadStore for Memory {
    fn load(&mut self, address: W<u16>) -> W<u8> {
        let addr = address.0;
        let value = if addr < 0x2000 {
            self.mem_load_status = MemState::Memory;
            self.ram[(addr & 0x7FF) as usize]
        } else if addr < 0x4000 {
            // FIXME: This is broken now for status and oamdata
            let (stat, data) = match addr & 0x7 {
                // Other registers are write only
                2 => (MemState::PpuStatus, self.ppu_read_regs.status),
                4 => (MemState::OamData, self.ppu_read_regs.oam),
                7 => (MemState::PpuData, self.ppu_read_regs.data),
                _ => (MemState::NoState, 0),
            };
            self.mem_load_status = stat;
            if stat != MemState::NoState {
                self.latch.0 = data;
            }
            data
        } else if addr < 0x4020 {
            /* Apu AND IO TODO*/
            match addr {
                // OAMDMA is Write only, TODO: Check what happens
                0x4014 => 0,
                0x4016 => {
                    self.io_load_status = IoState::GamePad1;
                    self.mem_load_status = MemState::Io;
                    self.joy_key[0]
                },
                0x4017 => {
                    self.io_load_status = IoState::GamePad2;
                    self.mem_load_status = MemState::Io;
                    self.joy_key[1]
                },
                _      => 0,
            }
        } else {
            self.mapper.prg_load(address)
        };
        W(value)
    }

    fn store(&mut self, address: W<u16>, value: W<u8>) {
        let addr = address.0;
        let val = value.0;
        if addr < 0x2000 {
            self.mem_store_status = MemState::Memory;
            self.ram[(addr & 0x7FF) as usize] = val
        } else if addr < 0x4000 {
            self.mem_store_status = match addr & 0x7 {
                // PpuStatus is read only
                0 => MemState::PpuCtrl,
                1 => MemState::PpuMask,
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
                // When OAMDMA is written to the cpu locks down and fills
                // the OAM memory with the selected page.
                0x4014 => {
                    self.oamdma = Some(value);
                },
                0x4016 => {
                    self.io_strobe = val;
                },
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
        write!(f, "{{ latch: {:#x}, oamdma: {:?}, mem_load_status: {:?}, mem_store_status: {:?}}}, \n {}",
               self.latch.0, self.oamdma, self.mem_load_status,
               self.mem_store_status, output)
    }
}
