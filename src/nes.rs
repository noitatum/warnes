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

    // This function steps a single cpu instruction
    pub fn step(&mut self) {
        let next = self.cpu.instruction_count() + 1;
        while self.cpu.instruction_count() != next {
            self.cycle();
        }
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
}

#[cfg(test)]
mod test {
    // nes
    use nes::Nes;

    // std
    use std::io::BufReader;
    use std::io::prelude::*;
    use std::fs::File;

    macro_rules! assert_equal {
        ($a:expr, $b:expr, $format:expr) => (assert!($a == $b, $format, $a, $b))
    }

    #[test]
    pub fn test_cpu() {
        let mut nes = Nes::new("test/nestest_direct.nes").unwrap();
        let file = File::open("test/nestest_direct.out").unwrap();
        let log = BufReader::new(file);
        nes.reset();
        for buffer in log.lines() {
            let regs = nes.cpu().registers();
            let buffer = buffer.unwrap();
            let data : Vec<&str> = buffer.split_whitespace().collect();
            let r = data[1..6].iter()
                              .map(|s| u8::from_str_radix(s, 16).unwrap())
                              .collect::<Vec<u8>>();
            let pc = u16::from_str_radix(data[0], 16).unwrap();
            assert_equal!(regs.PC.0, pc, "PC {:04X} != {:04X}");
            assert_equal!(regs.A.0, r[0], "A {:02X} != {:02X}");
            assert_equal!(regs.X.0, r[1], "X {:02X} != {:02X}");
            assert_equal!(regs.Y.0, r[2], "Y {:02X} != {:02X}");
            assert_equal!(regs.P.0 | 0x20, r[3], "P {:02X} != {:02X}");
            assert_equal!(regs.SP.0, r[4], "SP {:02X} != {:02X}");
            assert_equal!((nes.cpu().cycle_count() * 3) % 341,
                          data[6].parse::<u64>().unwrap(), "Cycles {} != {}");
            nes.step();
        }
    }
}
