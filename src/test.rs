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
    let log = BufReader::new(File::open("test/nestest_direct.out").unwrap());
    let mut nes = Nes::new("test/nestest_direct.nes").unwrap();
    nes.reset();
    for buffer in log.lines() {
        let regs = nes.cpu().registers();
        let buffer = buffer.unwrap();
        let data : Vec<&str> = buffer.split_whitespace().collect();
        let r : Vec<u8> = data[1..6].iter()
                                    .map(|s| u8::from_str_radix(s, 16).unwrap())
                                    .collect();
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
