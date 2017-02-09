use std::num::Wrapping as W;

const VRAM_SIZE : usize = 0x800;
const NT_SIZE   : usize = 0x400;

pub fn hmirror(address: usize) -> usize {
    ((address >> 1) & NT_SIZE) + (address & NT_SIZE - 1)
}

pub fn vmirror(address: usize) -> usize {
    address & VRAM_SIZE - 1
}

pub struct GameMemory {
    pub prg_rom     : Box<[u8]>,
    pub prg_ram     : Box<[u8]>,
    pub prg_bat     : Box<[u8]>,
    pub chr_rom     : Box<[u8]>,
    pub chr_ram     : Box<[u8]>,
    pub chr_bat     : Box<[u8]>,
    pub vmirror     : bool,
    pub screen4     : bool,
}

impl GameMemory {
    fn chr_load(&mut self, vram: &mut[u8], addr: W<u16>, bank: usize) -> u8 {
        let addr = addr.0 as usize;
        if addr >= 0x2000 {
            vram[if self.vmirror {vmirror(addr)} else {hmirror(addr)}]
        } else {
            self.chr_rom[bank + addr]
        }
    }

    fn chr_store(&mut self, vram: &mut[u8], addr: W<u16>, value: u8) {
        let addr = addr.0 as usize;
        if addr >= 0x2000 {
            vram[if self.vmirror {vmirror(addr)} else {hmirror(addr)}] = value;
        }
    }

    fn prg_load(&mut self, addr: W<u16>, bank: usize) -> u8 {
        // Emulate mirroring
        let mask = self.prg_rom.len() - 1;
        self.prg_rom[bank + (addr.0 as usize & mask)]
    }
}

pub trait Mapper {
    fn chr_load(&mut self, vram: &mut[u8], address: W<u16>) -> u8;
    fn chr_store(&mut self, vram: &mut[u8], address: W<u16>, value: u8);
    fn prg_load(&mut self, address: W<u16>) -> u8;
    fn prg_store(&mut self, address: W<u16>, value: u8);
}

pub struct Nrom(GameMemory);

impl Nrom {
    pub fn new_boxed(mem: GameMemory) -> Box<Mapper> {
        Box::new(Nrom(mem))
    }
}

impl Mapper for Nrom {
    fn chr_load(&mut self, vram: &mut[u8], address: W<u16>) -> u8 {
        self.0.chr_load(vram, address, 0)
    }

    fn chr_store(&mut self, vram: &mut[u8], address: W<u16>, value: u8) {
        self.0.chr_store(vram, address, value);
    }

    fn prg_load(&mut self, address: W<u16>) -> u8 {
        self.0.prg_load(address, 0)
    }

    fn prg_store(&mut self, _: W<u16>, _: u8) {}
}

pub struct Cnrom {
    mem: GameMemory,
    bank: usize,
}

impl Cnrom {
    pub fn new_boxed(mem: GameMemory) -> Box<Mapper> {
        Box::new(Cnrom {mem: mem, bank: 0})
    }
}

impl Mapper for Cnrom {
    fn chr_load(&mut self, vram: &mut[u8], address: W<u16>) -> u8 {
        self.mem.chr_load(vram, address, self.bank)
    }

    fn chr_store(&mut self, vram: &mut[u8], address: W<u16>, value: u8) {
        self.mem.chr_store(vram, address, value);
    }

    fn prg_load(&mut self, address: W<u16>) -> u8 {
        self.mem.prg_load(address, 0)
    }

    fn prg_store(&mut self, address: W<u16>, value: u8) {
        if address >= W(0x8000) {
            self.bank = (value as usize & 0x3) * 0x2000;
        }
    }
}

pub struct Pirate225 {
    mem: GameMemory,
    chr_bank: usize,
    prg_bank: usize,
    prg_small: usize,
}

impl Pirate225 {
    pub fn new_boxed(mem: GameMemory) -> Box<Mapper> {
        Box::new(Pirate225 {mem: mem, chr_bank: 0, prg_bank: 0, prg_small: 0})
    }
}

impl Mapper for Pirate225 {
    fn chr_load(&mut self, vram: &mut[u8], address: W<u16>) -> u8 {
        self.mem.chr_load(vram, address, self.chr_bank)
    }

    fn chr_store(&mut self, vram: &mut[u8], address: W<u16>, value: u8) {
        self.mem.chr_store(vram, address, value);
    }

    fn prg_load(&mut self, address: W<u16>) -> u8 {
        // Emulate mirroring
        let addr = (address.0 as usize) & (0x7FFF >> self.prg_small);
        self.mem.prg_rom[self.prg_bank + addr]
    }

    fn prg_store(&mut self, address: W<u16>, _: u8) {
        if address >= W(0x8000) {
            // Select PPU 8k bank and 32k or 16k
            let addr = address.0 as usize;
            self.prg_small = (addr & 0x1000) >> 12;
            self.chr_bank = (addr & 0x3F) << 13;
            self.prg_bank = ((addr >> 6) & 0x3F & !(1 - self.prg_small)) << 14;
            self.mem.vmirror = addr & 0x2000 == 0;
        }
    }
}
