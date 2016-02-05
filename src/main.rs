#[allow(non_snake_case)]
#[derive(Default, Debug)]
struct CPU{
    A : u8,  // Accumulator
    X : u8,  // Indexes
    Y : u8,  
    P : u8,  // Status
    SP: u8,  // Stack pointer
    PC: u16, // Program counter
}


fn main() {
    let c = CPU::default();
    println!("CPU, {:?},", c);
}
