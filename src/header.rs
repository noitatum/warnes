use std::io::prelude::*;
use std::io::{SeekFrom, Error};
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

const FLAGS_VMIRROR         : u8 = 0x01;
const FLAGS_BATTERY         : u8 = 0x02;
const FLAGS_TRAINER         : u8 = 0x04;
const FLAGS_4SCREEN         : u8 = 0x08;

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
    pub fn load_rom<P: AsRef<Path>>(path: P) -> Result<Header, String> {
        let mut rom = try_err!(File::open(path), "Couldn't open ROM file");
        let mut file_header = [0u8; INES_HEADER_SIZE];
        try_err!(rom.read_exact(&mut file_header), "Couldn't read ROM header");
        if &file_header[0..4] != &INES_SIGNATURE[..] {
            return err!("Invalid iNES Header");
        }
        let mapper = ((file_header[6] >> 4) | (file_header[7] & 0xF0)) as u16;
        let flags = (file_header[6] & 0xF) | (file_header[7] << 4);
        let prg_rom_size = file_header[4] as usize * INES_PRG_ROM_CHUNK;
        let mut prg_ram_size = file_header[8] as usize * INES_PRG_RAM_CHUNK;
        let mut prg_bat_size = 0;
        let chr_rom_size = file_header[5] as usize * INES_CHR_ROM_CHUNK;
        let mut chr_ram_size = 0;
        // Only one game is known to have battery backed chr ram
        let chr_bat_size = 0;
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
            println!("Warning: iNES 2.0 header detected and not parsed");
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

    pub fn get_mapper(&mut self) -> Result<Box<Mapper>, String> {
        let mem = try_err!(self.get_game_memory(), "Couldn't read ROM data");
        match self.mapper {
            0 => Ok(Nrom::new_boxed(mem)),
            3 => Ok(Cnrom::new_boxed(mem)),
            225 => Ok(Pirate225::new_boxed(mem)),
            _ => err!("Unrecognized Mapper {}", self.mapper)
        }
    }

    pub fn get_game_memory(&mut self) -> Result<GameMemory, Error> {
        let mut prg_rom = vec![0u8; self.prg_rom_size].into_boxed_slice();
        let mut chr_rom = vec![0u8; self.chr_rom_size].into_boxed_slice();
        let mut offset = INES_HEADER_SIZE;
        if is_flag_set!(self.flags, FLAGS_TRAINER) {
            offset += INES_TRAINER_SIZE;
        }
        self.rom_file.seek(SeekFrom::Start(offset as u64))?;
        self.rom_file.read_exact(&mut *prg_rom)?;
        self.rom_file.read_exact(&mut *chr_rom)?;
        Ok(
            GameMemory {
                prg_rom : prg_rom,
                prg_ram : vec![0; self.prg_ram_size].into_boxed_slice(),
                prg_bat : vec![0; self.prg_bat_size].into_boxed_slice(),
                chr_rom : chr_rom,
                chr_ram : vec![0; self.chr_ram_size].into_boxed_slice(),
                chr_bat : vec![0; self.chr_bat_size].into_boxed_slice(),
                vmirror : is_flag_set!(self.flags, FLAGS_VMIRROR),
                screen4 : is_flag_set!(self.flags, FLAGS_4SCREEN),
            }
        )
    }
}
