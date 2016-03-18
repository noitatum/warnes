// nes
use nes::Nes;
use cpu::{Operation, DebugRegs};
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

const DEBUG_SPACE       : &'static str = "                     ";
const DEBUG_LIST_SIZE   : u32 = 5;

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
            if words.len() == 0 {
                continue 'debug;
            }
            match words[0].trim() {
                "l"|"list" => self.print_list(DEBUG_LIST_SIZE),
                // alone just one step
                // with a number steps several instrs
                "s"|"step" => println!("{} step", rdbg!()),
                // Since we only have 6502 assembly
                // all these commands do the same
                "n"|"nexti"|"ni"|"stepi"|"si" => { 
                    self.next(renderer, event_pump);
                },
                "c"|"continue" => { 
                    println!("{} continue", rdbg!());
                    self.nes_run(renderer, event_pump);
                    break 'debug;
                },
                "p" => { 
                    if words.len() == 1 {
                        undefinedc!("No register or memory position given");
                    } else {
                        self.print_reg(words[1]);
                    }
                },
                "pb" => { 
                    println!("{} print", rdbg!());
                    if words.len() == 1 {
                        undefinedc!("No register or memory position given");
                    } else {
                        self.print_reg_binary(words[1]);
                    }
                }, 
                "b" => println!("{} breakpoint", rdbg!()),
                "q"|"quit" => { 
                    print!("{} ", rdbg!());
                    break 'debug; 
                },
                "help"  => self.help(),
                _       => undefinedc!(words[0]),
            }
        }
    }
}

impl Debug {

    fn next_operation(&mut self) -> Operation {
        let pc = self.nes.cpu_registers().PC;
        let mem = self.nes.memory();
        Operation::from_address(mem, pc)
    }

    fn next(&mut self, renderer: &mut Renderer, event_pump: &mut EventPump) {
        let operation = self.next_operation();
        self.print_operation(&operation);
        let cycles = if self.cpc {1} else {operation.inst.cycles};
        for _ in 0..cycles {
            self.nes.cycle(renderer, event_pump);
        }
    }

    fn print_operation(&self, operation: &Operation) {
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

    fn print_list(&mut self, count: u32) {
        let mut pc = self.nes.cpu_registers().PC;
        for _ in 0..count {
            let operation = Operation::from_address(self.nes.memory(), pc);
            self.print_operation(&operation);
            pc = pc + operation.mode.size;
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
            "A"|"a"         => self.nes.cpu_registers().A.0 as u16,
            "X"|"x"         => self.nes.cpu_registers().X.0 as u16,
            "Y"|"y"         => self.nes.cpu_registers().Y.0 as u16,
            "PC"|"pc"       => self.nes.cpu_registers().PC.0,
            "FLAGS"|"flags" => self.nes.cpu_registers().P.0 as u16,
            _               => {println!("{} Error non register returning 0", rdbg!()); 0},
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
