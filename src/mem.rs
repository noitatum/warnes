pub struct Memory {
    ram : [u8; 2048],
}

impl Memory {
    pub fn new () -> Memory {
        Memory {
            ram  : [0;  2048],
        }
    }

    pub fn load (&mut self, address: u16) -> u8 {
        self.ram[address as usize]
    }

    pub fn store (&mut self, address: u16, val : u8){
        self.ram[address as usize] = val;
    }
}
