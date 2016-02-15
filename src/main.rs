#[macro_use]
mod macros;
mod cpu;
mod mem;
mod ppu;

fn main() {
    let mut cpu = cpu::CPU::new();
    let mut ppu = ppu::Ppu::new();
    println!("{:?}", ppu);
    let mut memory = mem::Memory::new(ppu);
    println!("{:?}", memory);
    cpu.execute(&mut memory);
    println!("{}", cpu);
}
