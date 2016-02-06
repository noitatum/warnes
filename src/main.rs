use std::fmt;
mod cpu;
mod mem;

fn main() {
    let memory = mem::Memory::default();
    let cpu = cpu::CPU::new(memory);
    println!("{}", cpu);
}
