macro_rules! set_flag {
    ($flags:expr, $val:expr) => ($flags |= $val)
}

macro_rules! unset_flag {
    ($flags:expr, $val:expr) => ($flags &= !$val)
}

macro_rules! copy_flag {
    ($flags:expr, $src:expr, $val:expr) => 
        ($flags = $flags & !$val | $src.0 & $val)
}

macro_rules! is_flag_set {
    ($flags:expr, $val:expr) => ($flags & $val > 0)
}

macro_rules! set_flag_cond {
    ($flags:expr, $val:expr, $cond:expr) => 
        (if $cond {set_flag!($flags, $val)} else {unset_flag!($flags, $val)})
}

macro_rules! set_sign {
    ($flags:expr, $val:expr) => 
        (copy_flag!($flags, $val, FLAG_SIGN))
}

macro_rules! set_zero {
    ($flags:expr, $val:expr) => 
        (set_flag_cond!($flags, FLAG_ZERO, $val == W(0)))
}

macro_rules! W16 {
    ($val:expr) => (W($val.0 as u16))
}

macro_rules! W8 {
    ($val:expr) => (W($val.0 as u8))
}

macro_rules! set_sign_zero {
    ($flags:expr, $val:expr) => (
        set_sign!($flags, $val);
        set_zero!($flags, $val);
    )
}

macro_rules! set_sign_zero_carry_cond {
    ($flags:expr, $val:expr, $cond:expr) => (
        set_sign_zero!($flags, $val);
        set_flag_cond!($flags, FLAG_CARRY, $cond);
    )
}

macro_rules! inst {
    ($addr:ident, $oper:ident, $cycles:expr, $extra:expr, $name:expr) => (
        Instruction {
            addressing  : Regs::$addr,
            operation   : Regs::$oper,
            cycles      : $cycles,
            has_extra   : $extra,
            name        : $name
        }
    )
}

// Has zero cycle penalty
macro_rules! iz {
    ($addr:ident, $oper:ident, $cycles:expr) =>  
        (inst!($addr, $oper, $cycles, false, stringify!($oper)))
}

// Has extra cycle penalty
macro_rules! ix {
    ($addr:ident, $oper:ident, $cycles:expr) => 
        (inst!($addr, $oper, $cycles, true, stringify!($oper)))
}
