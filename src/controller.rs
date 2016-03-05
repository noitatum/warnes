extern crate sdl2;

// nes 
use mem::{Memory, IoState};
use loadstore::LoadStore;

// sdl
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

// std stuff
use std::num::Wrapping as W;


pub struct GamePad {
    //a, b, select, start, up, down, left, right
    joykeys     : [u8; 8],
    key         : u8,
    mem_pos  : W<u16>
}

pub struct Controller {
    gamepad1    : GamePad,
    gamepad2    : GamePad,
    strobe      : bool,
    reading_gp1 : bool,
    reading_gp2 : bool,
}

impl Controller {
    pub fn new () -> Controller {
        Controller {
            gamepad1    : GamePad::new(W(0x4016)),
            gamepad2    : GamePad::new(W(0x4017)),
            strobe      : false,
            reading_gp1 : false,
            reading_gp2 : false,
        }
    }
}

impl GamePad {
    pub fn new (mem_pos : W<u16>) -> GamePad {
        GamePad {
            joykeys : [0; 8],
            key     : 0,
            mem_pos : mem_pos,
        }
    }
}

impl Controller {
    pub fn push_keys(&mut self, mem: &mut Memory, pump: &mut sdl2::EventPump) {
        self.reading_gp1 = self.gamepad1.push_keys(self.reading_gp1, mem);
        self.reading_gp2 = self.gamepad2.push_keys(self.reading_gp2, mem);

        if ((mem.get_strobe() & 1) > 0) && self.strobe == false{
            self.strobe = true;
        } else if ((mem.get_strobe() & 1) == 0) && self.strobe == true {
            self.gamepad1.get_keys(pump);
            self.gamepad2.get_keys(pump);
            self.reading_gp1 = true;
            self.reading_gp2 = true;
            self.strobe = false;
        } 
    }
}

impl GamePad {
    // Reads the joystick (default to keyboard) and writes to memory accordingly.
    pub fn push_keys(&mut self, mut reading : bool, mem: &mut Memory) -> bool {
        // If reading it means a write of 1/0 to 0x4016 
        // we write to 0x4016 or 0x4017 the status of the key in gamepad
        if reading && mem.get_io_load_status(self.mem_pos) {
            if self.key != 8 {
                mem.store(self.mem_pos, W(self.joykeys[self.key as usize]));
                mem.set_io_store(IoState::NoState);
                self.key += 1;
            } else {
                self.key = 0;
                reading = false;
            }
        }
        // If we finish reading reading will return a false status
        // This guarantees that if we read all the first gamepad
        // and wait to read the second we will still be able to with the original state
        // writing 1/0 again to 0x4016 will re-load all the gamepad-keys on gp1 and gp2
        return reading;
    }

    pub fn get_keys (&mut self, pump: &mut sdl2::EventPump) {
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
        self.key = 0;
    }
}
                                                                          
