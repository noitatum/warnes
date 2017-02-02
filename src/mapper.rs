use std::num::Wrapping as W;

const VRAM_SIZE : usize = 0x800;
const NT_SIZE   : usize = 0x400;

pub fn mirror(address: usize, horizontal: bool) -> usize {
    let high = (address - 0x2000) >> 10;
    ((if horizontal {high / 2} else {high % 2}) << 10) + (address & NT_SIZE - 1)
}

pub trait Mapper {
    fn chr_load(&mut self, vram: &mut[u8], address: W<u16>) -> u8;

    fn chr_store(&mut self, vram: &mut[u8], address: W<u16>, value: u8) {
        let addr = address.0 as usize;
        if addr >= 0x2000 {
            vram[addr & (VRAM_SIZE - 1)] = value;
        }
    }

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

    fn prg_load(&mut self, address: W<u16>) -> u8 {
        let addr = address.0 as usize;
        // Emulate NROM-128 Mirroring
        let mask = self.0.prg_rom.len() - 1;
        self.0.prg_rom[addr & mask]
    }

    fn prg_store(&mut self, _: W<u16>, _: u8) {}
}

pub struct Cnrom {
    mem: GameMemory,
    bank: u8,
}

impl Cnrom {
    pub fn new_boxed(mem: GameMemory) -> Box<Mapper> {
        Box::new(Cnrom {mem: mem, bank: 0})
    }
}

impl Mapper for Cnrom {
    fn chr_load(&mut self, vram: &mut[u8], address: W<u16>) -> u8 {
        let addr = address.0 as usize;
        if addr >= 0x2000 {
            vram[addr & (VRAM_SIZE - 1)]
        } else {
            let rom_addr = self.bank as usize * 0x2000 + addr;
            self.mem.chr_rom[rom_addr & (self.mem.chr_rom.len() - 1)]
        }
    }

    fn prg_load(&mut self, address: W<u16>) -> u8 {
        let addr = address.0 as usize;
        // Emulate Mirroring
        let mask = self.mem.prg_rom.len() - 1;
        self.mem.prg_rom[addr & mask]
    }

    fn prg_store(&mut self, address: W<u16>, value: u8) {
        if address >= W(0x8000) {
            self.bank = value & 0x3;
        }
    }
}

pub struct Pirate225 {
    mem: GameMemory,
    chr_bank: usize,
    prg_bank: usize,
    prg_small: usize,
    hmirror: bool,
}

impl Pirate225 {
    pub fn new_boxed(mem: GameMemory) -> Box<Mapper> {
        Box::new(Pirate225 {mem: mem, chr_bank: 0, prg_bank: 0,
                            prg_small: 0, hmirror: false})
    }
}

impl Mapper for Pirate225 {
    fn chr_load(&mut self, vram: &mut[u8], address: W<u16>) -> u8 {
        let addr = address.0 as usize;
        if addr >= 0x2000 {
            vram[mirror(addr, self.hmirror)]
        } else {
            self.mem.chr_rom[self.chr_bank + addr]
        }
    }

    fn chr_store(&mut self, vram: &mut[u8], address: W<u16>, value: u8) {
        let addr = address.0 as usize;
        if addr >= 0x2000 {
            vram[mirror(addr, self.hmirror)] = value;
        }
    }

    fn prg_load(&mut self, address: W<u16>) -> u8 {
        let addr = address.0 as usize;
        // Emulate mirroring
        let mask = 0x7FFF >> self.prg_small;
        self.mem.prg_rom[self.prg_bank + (addr & mask)]
    }

    fn prg_store(&mut self, address: W<u16>, value: u8) {
        if address >= W(0x8000) {
            // Select 8k bank
            let addr = address.0 as usize;
            self.prg_small = (addr & 0x1000) >> 12;
            self.chr_bank = (addr & 0x3F) << 13;
            self.prg_bank = ((addr >> 6) & 0x3F & !(1 - self.prg_small)) << 14;
            self.hmirror = addr & 0x2000 > 0;
        }
    }
}
