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
                        $input.trim());
    );
}

macro_rules! rnl {
    ($input:expr) => ($input[0..$input.len()-1]);
}

const DEBUG_SPACE : &'static str = "                     ";

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
            input = String::new();
            stdin.read_line(&mut input).unwrap();
            let words : Vec<&str> = input.split(" ").collect();

            if words.len() > 0 {
                match words[0].trim() {
                    "l"|"list" => { self.print_list(renderer, event_pump); }
                    // alone just one step
                    // with a number steps several instrs
                    "s"|"step" => { println!("{} step", rdbg!()); },
                    // Since we only have 6502 assembly
                    // all these commands do the same
                    "n"|"nexti"|"ni"|"stepi"|"si"
                            => { 
                                 self.next(renderer, event_pump);
                               },
                    "c"|"continue"
                            => { println!("{} continue", rdbg!());
                                 self.nes_run(renderer, event_pump);
                                 break 'debug;
                               },
                    "p"     => { if words.len() == 1 {
                                     undefinedc!("No register or memory position given");
                                 } else {
                                     self.print_reg(words[1]);
                                 }
                               },
                    "pb"    => { println!("{} print", rdbg!());
                                 if words.len() == 1 {
                                     undefinedc!("No register or memory position given");
                                 } else {
                                     self.print_reg_binary(words[1]);
                                 }
                               }, 
                    "b"     => { println!("{} breakpoint", rdbg!());},
                    "q"|"quit"
                            => { print!("{} ", rdbg!());
                                break 'debug; },
                    "help"  => { self.help(); },
                    _       => { undefinedc!(words[0]); },
                }
            }
        }
    }
}

impl Debug {

    fn next(&mut self, renderer: &mut Renderer, event_pump: &mut EventPump) {
        self.print_next();
        self.nes.step(self.cpc, renderer, event_pump);
    }

    fn print_next(&mut self) {
        let operation = self.nes.next_operation();
        let inst = operation.inst;
        let operand = operation.operand.0;
        print!("{} {}", DEBUG_SPACE, inst.name);
        // TODO: Use constants from cpu.rs until we can cast enums to integers
        match inst.mode {
            0  => println!(""),
            1  => println!("#!{:02X}", operand),
            11 => println!("#{:04X}", operand),
            _  => println!("Invalid Mode"),
        }
    }

    fn print_list(&mut self, renderer: &mut Renderer, event_pump: &mut EventPump) {
        // FIXME
        for _ in 1..6 {
            self.print_next();
        }
    }

    fn print_reg(&mut self, word: &str) {
        println!("{} {}: {:x}", rdbg!(), word.trim(), self.get_reg(word));
    }

    fn print_reg_binary(&mut self, word: &str) {
        println!("{} {}: {:b}", rdbg!(), word.trim(), self.get_reg(word));
    }

    fn get_reg(&mut self, word: &str) -> u16 {
        return match word.trim() {
            "A"|"a"         => { self.nes.return_regs().0 },
            "X"|"x"         => { self.nes.return_regs().1 },
            "Y"|"y"         => { self.nes.return_regs().2 },
            "PC"|"pc"       => { self.nes.return_regs().3 },
            "FLAGS"|"flags" => { self.nes.return_regs().4 },
            _               => { println!("{} Error non register returning 0", rdbg!()); 0},
        }
    }

    fn nes_run(&mut self, renderer: &mut Renderer, event_pump: &mut EventPump) {
        self.nes.run(renderer, event_pump)
    }

    fn help(&self) {
        print!("{} Help commands\n", rdbg!());
        print!("{} 'c' or 'continue' to continue the execution.\n", rdbg!());
        print!("{} 'n', 'next', 'step' or 'step' to do execute the next instruction or do a single cpu cycle.\n", rdbg!());
        print!("{} 'b' or 'breakpoint' for breakpoints (NOT IMPLEMENTED YET).\n", rdbg!());
        print!("{} 'q' or 'quit' to quit.\n", rdbg!());
        print!("{} 'l' or 'list' to show the next instructions to be executed", rdbg!());
        print!("{} 'p' plus a register name to show the value of the register (ex: p A).\n", rdbg!());
        print!("{} 'pb' to show that value in binary (ex: pb A).\n", rdbg!());
    }

}




