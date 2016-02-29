extern crate sdl2;

// nes 
use mem::{Memory, IoState};
use loadstore::LoadStore;

// sdl
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

// std stuff
use std::num::Wrapping as W;

const GAMEPAD1  : W<u16> = W(0x4016);
const GAMEPAD2  : W<u16> = W(0x4017);

pub struct GamePad {
    //a, b, select, start, up, down, left, right
    joykeys     : [u8; 8],
    key         : u8,
    strobe      : bool,
    reading     : bool,
}

impl GamePad {
    pub fn new () -> GamePad {
        GamePad {
            joykeys : [0; 8],
            key     : 0,
            strobe  : false,
            reading : false,
        }
    }
}

impl GamePad {
    // Reads the joystick (default to keyboard) and writes to memory accordingly.
    pub fn read_keys(&mut self, mem: &mut Memory, pump: &mut sdl2::EventPump) {
        if self.reading && mem.get_io_load_status() {
            if self.key != 8 {
                mem.store(GAMEPAD1, W(self.joykeys[self.key as usize]));
                mem.set_io_store(IoState::NoState);
                self.key += 1;
            } else {
                self.key = 0;
                self.reading = false;
            }
        } 

        if ((mem.get_joy1() & 1) > 0) && self.strobe == false{
            self.strobe = true;
        } else if ((mem.get_joy1() & 1) == 0) && self.strobe == true {
            self.get_state(pump);
            self.reading = true;
            self.strobe = false;
        } 
    }

    pub fn get_state (&mut self, pump: &mut sdl2::EventPump) {
        for event in pump.poll_iter() { 
            match event {
                // Keyboard to joy Z = A, X = B, S = Select, Enter = Enter, arrows = dpad
                Event::KeyDown { keycode: Some(key), .. } =>  {
                    match key {
                        Keycode::Z         => self.joykeys[0] = true as u8,
                        Keycode::X         => self.joykeys[1] = true as u8,
                        Keycode::S         => self.joykeys[2] = true as u8,
                        Keycode::Return    => self.joykeys[3] = true as u8,
                        Keycode::Up        => self.joykeys[4] = true as u8,
                        Keycode::Down      => self.joykeys[5] = true as u8,
                        Keycode::Left      => self.joykeys[6] = true as u8,
                        Keycode::Right     => self.joykeys[7] = true as u8,
                        _                  => {}, 
                    }
                }
                _                                         => {},
            }
        }
    }
}
                                                                          
