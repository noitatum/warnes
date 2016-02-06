use std::fmt;
mod cpu;
mod mem;

fn main() {
    let mut cpu = cpu::CPU::new();
    let mut memory = mem::Memory::default();
    cpu.execute(&memory);
    println!("{}", cpu);
}
