#[macro_use]
mod macros;
mod cpu;
mod mem;
mod ppu;

fn main() {
    let mut cpu = cpu::CPU::new();
    let mut ppu = ppu::Ppu::new();
    let mut memory = mem::Memory::new(ppu);
    cpu.execute(&mut memory);
    println!("{}", cpu);
}
