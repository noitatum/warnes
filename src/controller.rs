extern crate sdl2;

// NES
use mem::Memory;
use enums::IoState;
use loadstore::LoadStore;
// std
use std::num::Wrapping as W;

pub struct GamePad {
    index       : usize,
    // A, B, Select, Start, Up, Down, Left, Right
    joykeys     : [u8; 8],
    key         : u8,
}

pub struct Controller {
    gamepad1    : GamePad,
    gamepad2    : GamePad,
}

impl Controller {
    pub fn new () -> Controller {
        Controller {
            gamepad1    : GamePad::new(0),
            gamepad2    : GamePad::new(1),
        }
    }

    pub fn cycle(&mut self, mem: &mut Memory, keys: &[[u8; 8]; 2]) {
        let status = mem.get_io_load_status();
        self.gamepad1.push_keys(mem, status);
        self.gamepad2.push_keys(mem, status);
        if mem.get_strobe() {
            self.gamepad1.set_keys(&keys[0]);
            self.gamepad2.set_keys(&keys[1]);
        }
    }
}

impl GamePad {
    pub fn new (index: usize) -> GamePad {
        GamePad {
            index   : index,
            joykeys : [0; 8],
            key     : 0,
        }
    }

    // Writes gamepad keys to memory accordingly.
    pub fn push_keys(&mut self, mem: &mut Memory, status: IoState) {
        if self.key < 8 {
            mem.set_joy_key(self.index, self.joykeys[self.key as usize]);
            if status == [IoState::GamePad1, IoState::GamePad2][self.index] {
                self.key += 1;
            }
        } else {
            // Nintendo controllers output 1 after all keys are read
            mem.set_joy_key(self.index, 1);
        }
    }

    pub fn set_keys(&mut self, keys: &[u8; 8]) {
        self.joykeys = *keys;
        self.key = 0;
    }
}
