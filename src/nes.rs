extern crate sdl2;
extern crate time;

// STD
use std::path::Path;

// Custom stuff
use header::Header;
use cpu::Cpu;
use ppu::Ppu;
use mem::Memory as Mem;
use controller::Controller as Pad;

pub struct Nes {
    cpu : Cpu,
    ppu : Ppu,
    mem : Mem,
    pad : Pad,
    keys : [[u8; 8]; 2],
}

impl Nes {
    pub fn new<P: AsRef<Path>> (rom_path: P) -> Result<Nes, String> {
        let mapper = Header::load_rom(rom_path)?.get_mapper()?;
        Ok (
            Nes {
                cpu : Default::default(),
                ppu : Ppu::new(),
                mem : Mem::new(mapper),
                pad : Pad::new(),
                keys : [[0u8; 8]; 2],
            }
        )
    }

    // This function does a complete CPU cycle
    // Including joy I/O and 3 PPU cycles.
    pub fn cycle(&mut self) {
        self.pad.cycle(&mut self.mem, &self.keys);
        self.cpu.cycle(&mut self.mem);
        self.ppu.cycle(&mut self.mem);
        self.ppu.cycle(&mut self.mem);
        self.ppu.cycle(&mut self.mem);
    }

    pub fn set_keys(&mut self, keys: &[[u8; 8]; 2]){
        self.keys = *keys;
    }

    pub fn reset(&mut self) {
        self.cpu.reset(&mut self.mem);
    }

    pub fn cpu(&self) -> &Cpu {
        &self.cpu
    }

    pub fn ppu(&self) -> &Ppu {
        &self.ppu
    }

    pub fn memory(&mut self) -> &mut Mem {
        &mut self.mem
    }
}
