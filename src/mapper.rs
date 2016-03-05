use std::num::Wrapping as W;
use header::Header;

const VRAM_SIZE : usize = 0x800;

pub trait Mapper {
    fn chr_load(&mut self, vram: &mut[u8], address: W<u16>) -> u8;
    fn chr_store(&mut self, vram: &mut[u8], address: W<u16>, value: u8);
    fn prg_load(&mut self, address: W<u16>) -> u8;
    fn prg_store(&mut self, address: W<u16>, value: u8);
}

pub struct GameMemory {
    pub prg_rom     : Box<[u8]>,
    pub prg_ram     : Box<[u8]>,
    pub prg_bat     : Box<[u8]>,
    pub chr_rom     : Box<[u8]>,
    pub chr_ram     : Box<[u8]>,
    pub chr_bat     : Box<[u8]>,
}

pub struct Nrom(GameMemory);

impl Nrom {
    pub fn new_boxed(mem: GameMemory) -> Box<Mapper> {
        Box::new(Nrom(mem))
    }
}

impl Mapper for Nrom {

    fn chr_load(&mut self, vram: &mut[u8], address: W<u16>) -> u8 {
        let addr = address.0 as usize;
        if addr >= 0x2000 {
            vram[addr & (VRAM_SIZE - 1)]
        } else {
            self.0.chr_rom[addr]
        }
    }

    fn chr_store(&mut self, vram: &mut[u8], address: W<u16>, value: u8) {
        let addr = address.0 as usize;
        if addr >= 0x2000 {
            vram[addr & (VRAM_SIZE - 1)] = value;
        }
    }

    fn prg_load(&mut self, address: W<u16>) -> u8 {
        let addr = address.0 as usize;
        // Emulate NROM-128 Mirroring
        let mask = self.0.prg_rom.len() - 1;
        self.0.prg_rom[addr & mask]
    }

    fn prg_store(&mut self, address: W<u16>, value: u8) {

    }
}
