use std::io::prelude::*;
use std::io::{Error, ErrorKind, SeekFrom};
use std::fs::File;
use std::path::Path;

use mapper::*;

const INES_SIGNATURE        : [u8; 4] = [0x4E, 0x45, 0x53, 0x1A];
const INES_HEADER_SIZE      : usize = 0x10;
const INES_TRAINER_SIZE     : usize = 0x0200;
const INES_PRG_ROM_CHUNK    : usize = 0x4000;
const INES_PRG_RAM_CHUNK    : usize = 0x2000;
const INES_CHR_ROM_CHUNK    : usize = 0x2000;
const INES_CHR_RAM_SIZE     : usize = 0x2000;
const INES_BAT_RAM_SIZE     : usize = 0x2000;

const NES2_SIGN_MASK        : u8 = 0xC0;
const NES2_SIGNATURE        : u8 = 0x80;

const FLAGS_BATTERY         : u8 = 0x02;
const FLAGS_TRAINER         : u8 = 0x04;



pub struct Header {
    rom_file     : File,
    mapper       : u16,
    flags        : u8,
    prg_rom_size : usize,
    prg_ram_size : usize,
    prg_bat_size : usize,
    chr_rom_size : usize,
    chr_ram_size : usize,
    chr_bat_size : usize,
}

impl Header {
    pub fn new_from_file<P: AsRef<Path>>(path: P) -> Result<Header, Error> {
        let mut rom = try!(File::open(path));
        let mut file_header : [u8; INES_HEADER_SIZE] = [0; INES_HEADER_SIZE];
        try!(rom.read_exact(&mut file_header));
        if &file_header[0..4] != &INES_SIGNATURE[..] {
            return Err(Error::new(ErrorKind::Other, "Invalid iNES Header"));
        } 
        let mapper = ((file_header[6] >> 4) | (file_header[7] & 0xF0)) as u16;
        let flags = (file_header[6] & 0xF) | (file_header[7] << 4);
        let mut prg_rom_size = file_header[4] as usize * INES_PRG_ROM_CHUNK;
        let mut prg_ram_size = file_header[8] as usize * INES_PRG_RAM_CHUNK;
        let mut prg_bat_size = 0;
        let mut chr_rom_size = file_header[5] as usize * INES_CHR_ROM_CHUNK; 
        let mut chr_ram_size = 0;
        // Only one game is known to have battery backed chr ram
        let mut chr_bat_size = 0;
        if is_flag_set!(flags, FLAGS_BATTERY) {
            prg_bat_size = INES_BAT_RAM_SIZE; 
        }
        // PRG RAM size was later added on iNES, it was 8KiB by default
        if prg_ram_size == 0 {
            prg_ram_size = INES_PRG_RAM_CHUNK;
        }
        // iNES doesn't specify CHR RAM size, we asume 8KiB if there is no ROM
        if chr_rom_size == 0 {
            chr_ram_size = INES_CHR_RAM_SIZE;
        }
        if flags & NES2_SIGN_MASK == NES2_SIGNATURE {
            // TODO: NES 2.0 parsing
        }
        Ok(
            Header {
                rom_file     : rom,
                mapper       : mapper,
                flags        : flags,
                prg_rom_size : prg_rom_size, 
                prg_ram_size : prg_ram_size,
                prg_bat_size : prg_bat_size,
                chr_rom_size : chr_rom_size, 
                chr_ram_size : chr_ram_size, 
                chr_bat_size : chr_bat_size, 
            }
        )
    } 

    pub fn get_mapper(&mut self) -> Option<Box<Mapper>> {
        let mem = self.get_game_memory();
        match self.mapper {
            0 => Some(Nrom::new_boxed(mem)),
            _ => None,
        }
    } 

    pub fn get_game_memory(&mut self) -> GameMemory {
        let mut prg_rom = vec![0u8; self.prg_rom_size].into_boxed_slice();
        let mut chr_rom = vec![0u8; self.chr_rom_size].into_boxed_slice();
        let mut offset = INES_HEADER_SIZE;
        if is_flag_set!(self.flags, FLAGS_TRAINER) { 
            offset += INES_TRAINER_SIZE;
        }
        self.rom_file.seek(SeekFrom::Start(offset as u64));
        self.rom_file.read_exact(&mut *prg_rom);
        self.rom_file.read_exact(&mut *chr_rom);
        GameMemory {
            prg_rom : prg_rom, 
            prg_ram : vec![0; self.prg_ram_size].into_boxed_slice(),
            prg_bat : vec![0; self.prg_bat_size].into_boxed_slice(),
            chr_rom : chr_rom, 
            chr_ram : vec![0; self.chr_ram_size].into_boxed_slice(),
            chr_bat : vec![0; self.chr_bat_size].into_boxed_slice(),
        }

    }
}
