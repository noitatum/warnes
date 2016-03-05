use std::io::prelude::*;
use std::io::{Error, ErrorKind};
use std::fs::File;
use std::path::Path;


// its not dead!!
#[allow(dead_code)]
const INES_SIGNATURE : [u8; 4] = [0x4E, 0x45, 0x53, 0x1A];

#[allow(dead_code)]
pub struct Header {
    rom_file     : File,
    mapper       : u8,
    flags        : u8,
    prg_rom_size : u8,
    chr_rom_size : u8,
    prg_ram_size : u8,
}

impl Header {
    #[allow(dead_code)]
    pub fn new_from_file<P: AsRef<Path>>(path: P) -> Result<Header, Error> {
        let mut rom = try!(File::open(path));
        let mut file_header : [u8; 16] = [0; 16];
        try!(rom.read_exact(&mut file_header));
        let mapper = (file_header[6] >> 4) | (file_header[7] & 0xF0);
        let flags = (file_header[6] & 0xF) | (file_header[7] << 4);
        let mut prg_ram_size = file_header[8];
        // There is always prg_ram on iNES 1.0
        if prg_ram_size == 0 {
            prg_ram_size = 1;
        }
        if &file_header[0..4] == &INES_SIGNATURE[..] {
            Ok(Header {
                rom_file     : rom,
                mapper       : mapper,
                prg_rom_size : file_header[4],
                chr_rom_size : file_header[5],
                prg_ram_size : prg_ram_size,
                flags        : flags,
            })
        } else {
            Err(Error::new(ErrorKind::Other, "Invalid iNES Header"))
        }
    } 
}
