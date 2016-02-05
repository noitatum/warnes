use std::fmt;
mod cpu;
mod mem;

fn main() {
    let c = cpu::CPU::default();
    println!("{}", c);
}
