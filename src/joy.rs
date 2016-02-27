extern crate sdl2;

//nes use
use mem::{Memory, MemState};
//sdl use
//use sdl2::event::Event;
use sdl2::keyboard::Scancode;
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
            if self.key != 8 {
                mem.store(JOY1, W(self.joykeys[self.key as usize]));
                self.key += 1;
            } else {
                self.key = 0;
                mem.write_status = MemState::NoState; 
            }
        }

        if let MemState::StartReadJoy1 = mem.write_status  { 
            let key_state = pump.keyboard_state();
            // Keyboard to joy Z = A, X = B, S = Select, Enter = Enter, arrows = dpad
            self.joykeys[0] = key_state.is_scancode_pressed(Scancode::Z) as u8;
            self.joykeys[1] = key_state.is_scancode_pressed(Scancode::X) as u8;
            self.joykeys[2] = key_state.is_scancode_pressed(Scancode::S) as u8;
            self.joykeys[3] = key_state.is_scancode_pressed(Scancode::KpEnter) as u8;
            self.joykeys[4] = key_state.is_scancode_pressed(Scancode::Up) as u8;
            self.joykeys[5] = key_state.is_scancode_pressed(Scancode::Down) as u8;  
            self.joykeys[6] = key_state.is_scancode_pressed(Scancode::Left) as u8;            
            self.joykeys[7] = key_state.is_scancode_pressed(Scancode::Right) as u8;   
            mem.write_status = MemState::ReadJoy1;
        }
    }
}
