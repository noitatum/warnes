pub struct Ppu {
    pub ppuctrl     : u8,
    pub ppumask     : u8,
    pub ppustatus   : u8,
    pub oamaddr     : u8,
    pub oamdata     : u8,
    pub ppuscroll   : u8,
    pub ppuaddr     : u8,
    pub ppudata     : u8,
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
        }
    }
}
