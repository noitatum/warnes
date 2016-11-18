#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MemState {
    PpuCtrl,
    PpuMask,
    PpuStatus,
    OamAddr,
    OamData,
    PpuScroll,
    PpuAddr,
    PpuData,
    Io,
    Memory,
    NoState,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum IoState {
    GamePad1,
    GamePad2,
    NoState,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Interrupt {
    NMI,
    IRQ,
}
