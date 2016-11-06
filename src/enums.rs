use std::fmt;

#[derive(Clone, Copy, PartialEq)]
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

#[derive(Clone, Copy, PartialEq)]
pub enum IoState {
    GamePad1,
    GamePad2,
    NoState,
}

impl fmt::Display for MemState {
    fn fmt (&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}",
            match *self {
                MemState::PpuCtrl       => "PpuCtrl",
                MemState::PpuMask       => "PpuMask",
                MemState::PpuStatus     => "PpuStatus",
                MemState::OamAddr       => "OamAddr",
                MemState::OamData       => "OamData",
                MemState::PpuScroll     => "PpuScroll",
                MemState::PpuAddr       => "PpuAddr",
                MemState::PpuData       => "PpuData",
                MemState::Memory        => "Memory",
                MemState::Io            => "Io",
                MemState::NoState       => "NoState",
            }
        )
    }
}

impl fmt::Display for IoState {
    fn fmt (&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}",
            match *self {
                IoState::GamePad1      => "GamePad1",
                IoState::GamePad2      => "GamePad2",
                IoState::NoState       => "NoState",
            }
        )
    }
}
