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

macro_rules! inst {
    ($addr:ident, $oper:ident, $cycles:expr, $extra:expr, $name:expr, $size:expr) => 
    (
        Instruction {
            addressing  : Regs::$addr,
            operation   : Regs::$oper,
            cycles      : $cycles,
            has_extra   : $extra,
            name        : $name,
            size        : $size,
            op_type     : OpType::$addr,
        }
    )
}

// Has zero cycle penalty
macro_rules! iz {
    ($addr:ident, $oper:ident, $cycles:expr, $size:expr) =>
        (inst!($addr, $oper, $cycles, false, stringify!($oper), $size))
}

// Has extra cycle penalty
macro_rules! ix {
    ($addr:ident, $oper:ident, $cycles:expr, $size:expr) =>
        (inst!($addr, $oper, $cycles, true, stringify!($oper), $size))
}

macro_rules! in_render_range {
    ($scanline:expr) => ($scanline < 257 && $scanline > 1)
}

macro_rules! render_on {
    ($selfie:expr) => ($selfie.show_sprites() || $selfie.show_background())
}

/*
macro_rules! sprite_pattern_base {
    ($selfie:expr) =>  (if $selfie.mask & CTRL_SPRITE_PATTERN == 0 {
                            0x0000
                        } else {
                            0x1000
                        })
}
*/

macro_rules! scanline_end {
    ($selfie:expr) =>
        (($selfie.scanline_width == 340 && $selfie.scanline == 261))
}

macro_rules! attr_bit {
    ($attr:expr, $bit:expr) => (($attr & (ATTR_BIT - $bit)) >> 7)
}

macro_rules! tile_bit {
    ($tile:expr, $bit:expr) => (($tile & (TILE_BIT - (($bit as u16) << 7)) >> 15))
}

macro_rules! join_bits {
    ($b1:expr, $b2:expr, $b3:expr, $b4:expr) =>
        (((($b1 as u16) << 3) | (($b2 as u16) << 2) | (($b3 as u16) << 1) | ($b4 as u16)) & 0x00FF)  
}

macro_rules! shift_bits {
    ($selfie:expr) => ($selfie.ltile_sreg = $selfie.ltile_sreg << 1;
                       $selfie.htile_sreg = $selfie.htile_sreg << 1;
                       $selfie.attr1_sreg = $selfie.attr1_sreg << 1;
                       $selfie.attr2_sreg = $selfie.attr2_sreg << 1;
                      )
}

macro_rules! to_RGB {
    ($r:expr, $g:expr, $b:expr) => { 
        Color::RGB($r, $g, $b) 
    }
}


