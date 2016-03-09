// nes
use nes::Nes;

// std
use std::io;
use std::io::Error;
use std::path::Path;
use std::io::prelude::*;

// SDL2
use sdl2::EventPump;
use sdl2::render::Renderer;

macro_rules! rdbg {
    () => { "(rdbg)" }
}

// Macro to print undefined commands. It removes the newline.
macro_rules! undefinedc {
    ($input:expr) => (
        println!("{} Undefined command: {}. Try 'help'", rdbg!(),
                        $input[0..$input.len()-1].to_string());
    );
}

macro_rules! rnl {
    ($input:expr) => ($input[0..$input.len()-1]);
}

pub struct Debug  {
    nes: Nes,
    // cycle per cycle
    cpc: bool,
}

impl Debug {
    pub fn load_rom<P: AsRef<Path>>(rom_path: P, cpc: bool) -> Result<Debug, Error>   {
        let rnes = try!(Nes::load_rom(rom_path));
        Ok (
            Debug {
                nes : rnes,
                cpc : cpc,
            }
        )
    }
}

impl Debug {
    pub fn run(&mut self, renderer: &mut Renderer, event_pump: &mut EventPump) {
        // Reset nes.
        self.nes.reset();

        let mut input : String;
        let stdin = io::stdin();
        'debug: loop {
            print!("{} ", rdbg!());
            io::stdout().flush().ok().expect("io flush");
            input = "".to_string();
            stdin.read_line(&mut input).unwrap();
            let words : Vec<&str> = input.split(" ").collect();
            if words.len() > 0 {
                match &rnl!(words[0]) {
                    "l" => { println!("{} list", rdbg!()); }
                    // alone just one step
                    // with a number steps several instrs
                    "s" => { println!("{} step", rdbg!()); },
                    // Since we only have 6502 assembly
                    // all these commands are the same
                    "n"|"nexti"|"ni"|"stepi"|"si"
                        => { println!("{} next", rdbg!());
                             println!("next instr: {}, cycles: {}",
                                self.nes.next_instr().0,
                                self.nes.next_instr().1);
                             self.nes.step(self.cpc, renderer, event_pump);
                             // Print executed instruction
                            },
                    "c" => { println!("{} continue", rdbg!());
                             self.nes.run(renderer, event_pump);
                             break 'debug;
                            },
                    "p" => { println!("{} print", rdbg!()); },
                    "b" => { println!("{} breakpoint", rdbg!());},
                    "q" => { print!("{} ", rdbg!());
                             break 'debug; },
                    _   => { undefinedc!(words[0]); },
                }
            }
        }
    }
}



