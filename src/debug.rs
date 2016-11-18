// nes
use nes::Nes;
use cpu::Operation;
// std
use std::io;
use std::io::Write;

// Macro to print undefined commands. It removes the newline.
macro_rules! undefinedc {
    ($input:expr) => (
        println!("Undefined command: {}. Try 'help'", $input.trim());
    );
}

macro_rules! rnl {
    ($input:expr) => ($input[0..$input.len()-1]);
}

const DEBUG_LIST_SIZE   : u32 = 5;

pub fn run(nes: &mut Nes) {
    nes.reset();
    'debug: loop {
        let mut input = String::new();
        print!("(rdbg) ");
        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut input).unwrap();
        let words : Vec<&str> = input.split(" ").collect();
        if words.len() == 0 {
            continue;
        }
        match words[0].trim() {
            // "l"|"list" => print_list(nes, DEBUG_LIST_SIZE),
            // Over function calls
            "n"|"next" => {
                next(nes);
            },
            // Single step
            "s"|"step" =>  {
                step(nes);
            },
            // Single cycle
            "si"|"stepi"|"ni"|"nexti" => {
                step_cycle(nes);
            },
            "c"|"continue" => {
                println!("continue");
                // TODO: nes.run();
                break 'debug;
            },
            "u"|"until" => {
                until(nes);
            }
            "p" => {
                if words.len() == 1 {
                    println!("No register or memory position given");
                } else {
                    print_reg(nes, words[1]);
                }
            },
            "pb" => {
                println!("print");
                if words.len() == 1 {
                    println!("No register or memory position given");
                } else {
                    print_reg_binary(nes, words[1]);
                }
            },
            "b" => println!("breakpoint"),
            "q"|"quit" => {
                break 'debug;
            },
            "help"  => help(),
            _       => undefinedc!(words[0]),
        }
    }
}

fn step_cycle(nes: &mut Nes) {
    nes.cycle();
    print_current_operation(nes);
}

pub fn step(nes: &mut Nes) {
    let next = nes.cpu().instruction_count() + 1;
    while nes.cpu().instruction_count() != next {
        nes.cycle()
    }
    print_current_operation(nes);
}

fn next(nes: &mut Nes) {
    // TODO
    step(nes);
    print_current_operation(nes);
}

fn until(nes: &mut Nes) {
    let pc = nes.cpu().registers().PC;
    while nes.cpu().registers().PC <= pc {
        nes.cycle();
    }
    print_current_operation(nes);
}

fn print_current_operation(nes: &Nes) {
    print!("{:04X} ", nes.cpu().registers().PC);
    let execution = &nes.cpu().execution();
    print_operation(&execution.operation, execution.address.0);
    println!("");
}

fn print_operation(operation: &Operation, address: u16) {
    let inst = operation.inst;
    let operand = operation.operand.0;
    print!("{} ", inst.name);
    match inst.mode.name {
        "imp" => print!(""),
        "imm" => print!("#{:02X}", operand),
        "rel" => print!("{:04X}", address),
        "abs" => print!("{:04X}", operand),
        "abx" => print!("{:04X},X", operand),
        "aby" => print!("{:04X},Y", operand),
        "ind" => print!("({:04X})", operand),
        "idx" => print!("({:02X},X)", operand),
        "idy" => print!("({:02X}),Y", operand),
        "zpg" => print!("{:02X}", operand),
        "zpx" => print!("{:02X},X", operand),
        "zpy" => print!("{:02X},Y", operand),
        _  => print!("Invalid Mode"),
    }
    match inst.mode.name {
        "imp" | "imm" | "rel" | "abs" => {},
        _ => {print!(" @ {:04X}", address)},
    }
}

/* TODO: Rethink this function, from_address modifies nes.memory()
         And we also need the execution
fn print_list(nes: &mut Nes, count: u32) {
    let mut pc = nes.cpu().registers().PC;
    for _ in 0..count {
        let operation = Operation::from_address(nes.memory(), pc);
        print_operation(&operation, 0);
        pc = pc + operation.inst.mode.size;
    }
}
*/

fn print_reg(nes: &Nes, word: &str) {
    println!("{}: {:x}", word.trim(), get_reg(nes, word));
}

fn print_reg_binary(nes: &Nes, word: &str) {
    println!("{}: {:b}", word.trim(), get_reg(nes, word));
}

fn get_reg(nes: &Nes, word: &str) -> u16 {
    match word.trim().to_string().to_uppercase().as_ref() {
        "A"     => nes.cpu().registers().A.0 as u16,
        "X"     => nes.cpu().registers().X.0 as u16,
        "Y"     => nes.cpu().registers().Y.0 as u16,
        "P"     => nes.cpu().registers().P.0 as u16,
        "SP"    => nes.cpu().registers().SP.0 as u16,
        "PC"    => nes.cpu().registers().PC.0,
        _       => {println!("Error non register returning 0"); 0},
    }
}

fn help() {
    println!("Help commands");
    println!("'c' or 'continue' to continue the execution.");
    println!("'n', 'next', 'step' or 'step' to do execute the next instruction or do a single cpu cycle.");
    println!("'b' or 'breakpoint' for breakpoints (NOT IMPLEMENTED YET).");
    println!("'q' or 'quit' to quit.");
    println!("'l' or 'list' to show the next instructions to be executed");
    println!("'p' plus a register name to show the value of the register (ex: p A).");
    println!("'pb' to show that value in binary (ex: pb A).");
}
