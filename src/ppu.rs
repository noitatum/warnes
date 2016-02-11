
pub struct Ppu {
    pub ppuctrl     : u8,
    pub ppumask     : u8,
    pub ppustatus   : u8,
    pub oamaddr     : u8,
    pub oamdata     : u8,
    pub ppuscroll   : u8,
    pub ppuaddr     : u8,
    pub ppudata     : u8,

    pub oam         : [u8; 256],    /* Object atribute memory */
    pub vram        : [u8; 0x4000], //16kb
}

impl Ppu {
    pub fn new () -> Ppu {
        Ppu {
            ppuctrl     : 0,
            ppumask     : 0,
            ppustatus   : 0,
            oamaddr     : 0,
            oamdata     : 0,
            ppuscroll   : 0,
            ppuaddr     : 0,
            ppudata     : 0,

            oam         : [0; 256],
            vram        : [0;  0x4000],
        }
    }

    pub fn load (self, address: u16) -> u8 {
        if address < 0x3000 {
            self.vram[address as usize];
        }else if address < 0x3F00 {
            self.vram[(address - 0x1000) as usize];
        }else if address < 0x3F20 {
            self.vram[address as usize];
        }else if address < 0x4000 {
            self.vram[(address - 0x100) as usize];
        }else {
            self.vram[(address % 0x4000) as usize];
        }
    }

    /* 
        pub fn execute(&mut self, memory: &mut Mem) -> u32 {
        let op = memory.load(self.PC.0);
        match op {
            _ if op & OP_JUMP_MASK == OP_JUMP => self.do_jump(memory, op),
            _ if op & OP_SPECIAL_MASK == OP_SPECIAL => self.do_special(memory, op),
            _ if op & OP_BRANCH_MASK == OP_BRANCH => self.do_branch(memory, op),
            _ if op & OP_IMPLIED_MASK == OP_IMPLIED => self.do_implied(memory, op),
            _ => self.do_common(memory, op),
        } 
    }
     * */

    pub fn write (&mut self, address: u16, value: u8){
        if address < 0x3000 {
            self.vram[address as usize] = value;
        }else if address < 0x3F00 {
            self.vram[(address - 0x1000) as usize] = value;
        }else if address < 0x3F20 {
            self.vram[address as usize] = value;
        }else if address < 0x4000 {
            self.vram[(address - 0x100) as usize] = value;
        }else {
            self.vram[(address % 0x4000) as usize] = value;
        }
    }
}
