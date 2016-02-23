use cpu::Cpu;
use ppu::Ppu;
use mem::Memory as Mem;

#[derive(Default)]
#[derive(Debug)]
pub struct Nes {
    cpu : Cpu,
    ppu : Ppu,
    mem : Mem,
}

impl Nes {
    pub fn new() -> Nes {
        Nes {
            cpu : Default::default(),
            ppu : Ppu::new(),
            mem : Mem::new(),
        }
    }
}

impl Nes {
    pub fn Run(&mut self) {
        self.cpu.cycle(&mut self.mem);
        self.ppu.cycle(&mut self.mem);
        self.ppu.cycle(&mut self.mem);
        self.ppu.cycle(&mut self.mem);
    }
}
