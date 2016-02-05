use std::fmt;

#[allow(non_snake_case)]
pub struct Ram {
    lvariables  : [u8; 16],
    gvariables  : [u8; 240],
    dataNametbl : [u8; 160],
    stack       : [u8; 96],
    dataOAM     : [u8; 256],
    dataSound   : [u8; 256],
    arraysnGlbl : [u8; 1024],
}

impl Default for Ram {
    fn default () -> Ram {
        Ram {
            lvariables  : [0;  16],
            gvariables  : [0;  240],
            dataNametbl : [0;  160],
            stack       : [0;   96],
            dataOAM     : [0;  256],
            dataSound   : [0;  256],
            arraysnGlbl : [0; 1024],
        }
        /* todo */
    }
}
#[allow(non_snake_case)]
#[derive(Default)]
struct CPU{
    A : u8,  // Accumulator
    X : u8,  // Indexes
    Y : u8,  
    P : u8,  // Status
    SP: u8,  // Stack pointer
    PC: u16, // Program counter

    ram : Ram,
}

impl fmt::Display for CPU {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{{ A:{}, X:{}, Y:{}, P:{}, SP:{}, PC:{}  }}",
               self.A, self.X, self.Y, self.P, self.SP, self.PC)
    }
}

fn main() {
    let c = CPU::default();
    println!("{}", c);
}
