use std::fmt;
use mem;

// TODO: Separate into multiple arrays
const opcode_space : usize = 2;
const opcode_table : [fn(&CPU, &mem::Memory) -> (); opcode_space] = [
   CPU::brk,
   CPU::invalid,
  ];

#[allow(non_snake_case)]
pub struct CPU {
    A : u8,  // Accumulator
    X : u8,  // Indexes
    Y : u8,  
    P : u8,  // Status
    SP: u8,  // Stack pointer
    PC: u16, // Program counter
}

impl CPU {
    pub fn new() -> CPU {
        CPU {
            A : 0,
            X : 0,
            Y : 0,
            P : 0x24, 
            SP : 0xfd,
            PC : 0,
        }
    }

    pub fn execute(&self, memory: &mem::Memory) -> () {

    }

    fn invalid(&self, memory: &mem::Memory) -> () {

    }

    fn brk(&self, memory: &mem::Memory) -> () {
        
    }

}

impl fmt::Display for CPU {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{{ A: {}, X: {}, Y: {}, P: {}, SP: {}, PC: {} }}",
               self.A, self.X, self.Y, self.P, self.SP, self.PC)
    }
}
