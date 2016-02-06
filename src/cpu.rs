use std::fmt;
use mem;

#[allow(non_snake_case)]
pub struct CPU{
    A : u8,  // Accumulator
    X : u8,  // Indexes
    Y : u8,  
    P : u8,  // Status
    SP: u8,  // Stack pointer
    PC: u16, // Program counter

    mem : mem::Memory,
}

impl CPU {
    pub fn new(memory: mem::Memory) -> CPU {
        CPU {
            A : 0,
            X : 0,
            Y : 0,
            P : 0x24, 
            SP : 0xfd,
            PC : 0,
            
            mem : memory,  
        }
    } 
}

impl fmt::Display for CPU {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{{ A: {}, X: {}, Y: {}, P: {}, SP: {}, PC: {} }}",
               self.A, self.X, self.Y, self.P, self.SP, self.PC)
    }
}
