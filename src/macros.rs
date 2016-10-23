macro_rules! set_flag {
    ($flags:expr, $val:expr) => ($flags = $flags | $val)
}

macro_rules! unset_flag {
    ($flags:expr, $val:expr) => ($flags = $flags & !$val)
}

macro_rules! copy_bits {
    ($dest:expr, $src:expr, $mask:expr) =>
        ($dest = $dest & !$mask | $src & $mask)
}

macro_rules! is_bit_set {
    ($flags:expr, $val:expr) => ($flags & $val > W(0))
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
        (copy_bits!($flags, $val, FLAG_SIGN))
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

macro_rules! get_bit {
        ($flags:expr, $flag_bit:expr) => ($flags & $flag_bit;);
}

macro_rules! set_low_byte {
    ($val:expr, $byte:expr) => ($val & W(0xFF00) | W16!($byte))
}

macro_rules! set_high_byte {
    ($val:expr, $byte:expr) => ($val & W(0xFF) | W16!($byte) << 8)
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
