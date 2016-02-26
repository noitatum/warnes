extern crate sdl2;

//nes use
use mem::{Memory, MemState};
//sdl use
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
//sdl stuff
use std::num::Wrapping as W;


const JOY1  : W<u16> = W(0x4016);
const JOY2  : W<u16> = W(0x4017);

pub struct JoyStick {
    //a, b, select, start, up, down, left, right
    joykeys : [u8; 8],
    key     : u8,
}

impl JoyStick {
    pub fn new () -> JoyStick {
        JoyStick {
            joykeys : [0; 8],
            key     : 0,
        }
    }
}

impl JoyStick {
    // Reads the joystick (default to keyboard) and writes to memory accordingly.
    pub fn read_keys(&mut self, mem: &mut Memory, pump: &mut sdl2::EventPump){
        if let MemState::ReadJoy1 = mem.write_status { 
            // GET CURRENT STATE OF KEYBOARD
            // TODO
            // WRITE IT TO MEMORY IN ORDER
            mem.store(JOY1, W(self.joykeys[self.key as usize]));
            self.key += 1;
            if self.key == 8 {
                self.key = 0;
            }
        }
    }
}
