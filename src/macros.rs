macro_rules! set_flag {
    ($flags:expr, $val:expr) => ($flags |= $val);
}

macro_rules! unset_flag {
    ($flags:expr, $val:expr) => ($flags &= !$val);
}

macro_rules! is_flag_set {
    ($flags:expr, $val:expr) => ($flags & $val != 0);
}

macro_rules! set_flag_cond {
    ($flags:expr, $val:expr, $cond:expr) => (
        if $cond {set_flag!($flags, $val)} else {unset_flag!($flags, $val)} );
}

macro_rules! set_sign {
    ($flags:expr, $val:expr) => ( 
        $flags = $flags & !FLAG_SIGN | $val.0 & FLAG_SIGN;
    );
}

macro_rules! set_zero {
    ($flags:expr, $val:expr) => (
        set_flag_cond!($flags, FLAG_ZERO, $val == W(0));
    );
}

macro_rules! set_sign_and_zero {
    ($flags:expr, $val:expr) => (
        set_sign!($flags, $val);
        set_zero!($flags, $val);
    );
}

macro_rules! ror {
    ($val:expr, $flags:expr) => (
        $val = ($val >> 1) | (($val & W($flags & FLAG_CARRY)) << 7)
    );
}

macro_rules! rol {
    ($val:expr, $flags:expr) => (
        $val = ($val << 1) | (($val & W($flags & FLAG_CARRY)) >> 7)
    );
}

macro_rules! W16 {
    ($val:expr) => (W($val.0 as u16));
}

macro_rules! W8 {
    ($val:expr) => (W($val.0 as u8));
}
