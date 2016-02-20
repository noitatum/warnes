#[macro_use]
mod macros;
mod cpu;
mod mem;
mod ppu;

fn main() {
    let mut cpu : cpu::CPU = Default::default();
    let mut ppu = ppu::Ppu::new();
    println!("{:?}", ppu);
    let mut memory = mem::Memory::new();
    println!("{:?}", memory);
    cpu.single_cycle(&mut memory);
    println!("{:?}", cpu);
}
