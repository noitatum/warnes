use std::fmt;
use mem::Memory as Mem;
use loadstore::LoadStore;
use std::num::Wrapping as W;

const OAMDATA           : W<u16> = W(0x2004);
const DMA_CYCLES        : u32 = 512;

#[derive(Default)]
pub struct DMA {
    cycles_left : u32,
    address     : W<u16>,
    value       : W<u8>,
}

impl fmt::Debug for DMA {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{{DMA: cycles_left: {}, address: {:#x}, value: {:#x}}}",
               self.cycles_left, self.address.0, self.value.0)
    }
}

impl DMA {
    // Returns true if DMA is active
    pub fn cycle(&mut self, memory: &mut Mem, cycles: u64) -> bool {
        if self.cycles_left > 0 {
            self.execute(memory);
            true
        } else if let Some(page) = memory.get_oamdma() {
            self.start(page, cycles as u32);
            true
        } else {
            false
        }
    }

    fn start(&mut self, page: W<u8>, cycles: u32) {
        self.address = W16!(page) << 8; 
        // Additional cycle if on odd cycle
        self.cycles_left = DMA_CYCLES + cycles & 1;
    }

    fn execute(&mut self, memory: &mut Mem) {
        self.cycles_left -= 1;
        // Simulate idle cycles
        if self.cycles_left < DMA_CYCLES {
            // Read on odd cycles and write on even cycles
            if self.cycles_left & 1 == 1 {
                self.value = memory.load(self.address);
                self.address = self.address + W(1);
            } else {
                memory.store(OAMDATA, self.value);
            }
        }
    }
}