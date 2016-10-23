extern crate sdl2;

// NES
use mem::Memory;
use enums::IoState;
use loadstore::LoadStore;
// std
use std::num::Wrapping as W;

pub struct GamePad {
    // a, b, select, start, up, down, left, right
    joykeys     : [u8; 8],
    key         : u8,
    reading     : bool,
    mem_pos     : W<u16>,
}

pub struct Controller {
    gamepad1    : GamePad,
    gamepad2    : GamePad,
    strobe      : bool,
}

impl Controller {
    pub fn new () -> Controller {
        Controller {
            gamepad1    : GamePad::new(W(0x4016)),
            gamepad2    : GamePad::new(W(0x4017)),
            strobe      : false,
        }
    }

    pub fn cycle(&mut self, mem: &mut Memory, keys: &[[u8; 8]; 2]) {
        self.gamepad1.push_keys(mem);
        self.gamepad2.push_keys(mem);
        if ((mem.get_strobe() & 1) > 0) && !self.strobe {
            self.strobe = true;
        } else if ((mem.get_strobe() & 1) == 0) && self.strobe {
            self.gamepad1.get_keys(&keys[0]);
            self.gamepad2.get_keys(&keys[1]);
            self.strobe = false;
        }
    }
}

impl GamePad {
    pub fn new (mem_pos : W<u16>) -> GamePad {
        GamePad {
            joykeys : [0; 8],
            key     : 0,
            mem_pos : mem_pos,
            reading : false,
        }
    }

    // Reads the joystick (default to keyboard) and writes to memory accordingly.
    pub fn push_keys(&mut self, mem: &mut Memory) {
        // If reading it means a write of 1/0 to 0x4016
        // we write to 0x4016 or 0x4017 the status of the key in gamepad
        if self.reading && mem.get_io_load_status(self.mem_pos) {
            if self.key != 8 {
                mem.store(self.mem_pos, W(self.joykeys[self.key as usize]));
                mem.set_io_store(IoState::NoState);
                self.key += 1;
            } else {
                self.key = 0;
                self.reading = false;
            }
        }
        // If we finish, reading will have a false status
        // This guarantees that if we read all the first gamepad
        // and wait to read the second we will still be able to with the original state
        // writing 1/0 again to 0x4016 will re-load all the gamepad-keys on gp1 and gp2
    }

    pub fn get_keys(&mut self, keys: &[u8; 8]) {
        self.joykeys = *keys;
        self.reading = true;
        self.key = 0;
    }
}
