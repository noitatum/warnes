#[derive(Default, Debug)]
struct CPU{
    A : u8,  // Accumulator
    X : u8,  // Indexes
    Y : u8,  
    S : u8,  // Stack pointer
    P : u8,  // Status
    PC: u8,  // Program counter
}


fn main() {
    let c = CPU::default();
    println!("CPU, {:?},", c);
}
