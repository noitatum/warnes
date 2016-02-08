use std::fmt;
use mem::Memory as Mem;
use std::num::Wrapping as W;

const FLAG_CARRY   : u8 = 0x01;
const FLAG_ZERO    : u8 = 0x02;
const FLAG_INT     : u8 = 0x04;
const FLAG_DEC     : u8 = 0x08;
const FLAG_BRK     : u8 = 0x10;
const FLAG_UNUSED  : u8 = 0x20;
const FLAG_OVER    : u8 = 0x40;
const FLAG_SIGN    : u8 = 0x80;

macro_rules! set_flag {
    ($flags:expr, $val:expr) => ($flags |= $val);
}

macro_rules! unset_flag {
    ($flags:expr, $val:expr) => ($flags &= !$val);
}

macro_rules! is_flag_set {
    ($flags:expr, $val:expr) => ($flags & $val != 0);
}

macro_rules! set_sign {
    ($flags:expr, $val:expr) => ( 
        $flags = $flags & !FLAG_SIGN | $val & FLAG_SIGN;
    );
}

macro_rules! set_zero {
    ($flags:expr, $val:expr) => (
        set_flag!($flags, (($val == 0) as u8) << 1);
    );
}

const OP_SPECIAL_TABLE : [fn(&mut CPU, &mut Mem) -> u32; 4] = [
    CPU::brk,
    CPU::invalid_s,
    CPU::rti,
    CPU::rts,
];

const OP_BRANCH_TABLE : [fn(P : u8) -> bool; 8] = [
    CPU::bpl,
    CPU::bmi,
    CPU::bvc,
    CPU::bvs,
    CPU::bcc,
    CPU::bcs,
    CPU::bne,
    CPU::beq,
];

const OP_IMPLIED_TABLE : [fn(&mut CPU, &mut Mem); 32] = [
    CPU::php,
    CPU::asl_a,
    CPU::clc,
    CPU::invalid_i,
    CPU::plp, 
    CPU::rol_a,
    CPU::sec,
    CPU::invalid_i,
    CPU::pha,
    CPU::lsr_a,
    CPU::cli,
    CPU::invalid_i,
    CPU::pla,
    CPU::ror_a,
    CPU::sei,
    CPU::invalid_i,
    CPU::dey,
    CPU::txa,
    CPU::tya,
    CPU::txs,
    CPU::tay,
    CPU::tax,
    CPU::clv,
    CPU::tsx,
    CPU::iny,
    CPU::dex,
    CPU::cld,
    CPU::invalid_i,
    CPU::inx,
    CPU::nop,
    CPU::sed,
    CPU::invalid_i,
];

const OP_COMMON_TABLE : [fn(&mut CPU, &mut Mem, u8) -> (); 32] = [
    CPU::invalid_c,
    CPU::ora,
    CPU::asl,
    CPU::invalid_c,
    CPU::bit,
    CPU::and,
    CPU::rol,
    CPU::invalid_c,
    CPU::invalid_c,
    CPU::eor,
    CPU::lsr,
    CPU::invalid_c,
    CPU::invalid_c,
    CPU::adc,
    CPU::ror,
    CPU::invalid_c,
    CPU::sty,
    CPU::sta,
    CPU::stx,
    CPU::invalid_c,
    CPU::ldy,
    CPU::lda,
    CPU::ldx,
    CPU::invalid_c,
    CPU::cpy,
    CPU::cmp,
    CPU::dec,
    CPU::invalid_c,
    CPU::cpx,
    CPU::sbc,
    CPU::inc,
    CPU::invalid_c,
];

const OP_JUMP_MASK      : u8 = 0xDF;
const OP_JUMP           : u8 = 0x4C;
const OP_SPECIAL_MASK   : u8 = 0x9F;
const OP_SPECIAL        : u8 = 0x00;
const OP_BRANCH_MASK    : u8 = 0x1F;
const OP_BRANCH         : u8 = 0x10;
const OP_IMPLIED_MASK   : u8 = 0x1F;
const OP_IMPLIED        : u8 = 0x08;

const OP_JSR            : u8 = 0x20;

/* Implied instructions with more than two cycles */
const OP_IMP_STACK_MASK : u8 = 0x9F;
const OP_IMP_STACK      : u8 = 0x08;
const OP_IMP_PULL_MASK  : u8 = 0xBF;
const OP_IMP_PULL       : u8 = 0x28;

/* Common instructions */

/* ASL, ROL, LSR, ROR, DEC, INC Instructions */
/* WARNING: Non exhaustive, this also selects STX and LDX */
const OP_COMMON_B_MASK  : u8 = 0x03;
const OP_COMMON_B       : u8 = 0x02;

const STACK_PAGE        : u16 = 0x0100;
const PAGE_MASK         : u16 = 0xFF00;

/* Cycle cost */

/* Jumps */
const CYCLES_JUMP       : u32 = 3;
const CYCLES_JSR        : u32 = 6;

/* Branches */
const CYCLES_BRANCH     : u32 = 2;

/* Specials */
const CYCLES_BRK        : u32 = 7;
const CYCLES_RTI        : u32 = 6;
const CYCLES_RTS        : u32 = 6;

/* Implied */
const CYCLES_IMPLIED    : u32 = 2;

/* ASL, ROL, LSR, ROR, DEC, INC Instructions 
   ZPG; ABS; ZPG, X; ABS, X; Addressing modes */
const CYCLES_COMMON_B : [u32; 4] = [5, 6, 6, 7]; 

const CYCLES_COMMON_A : [u32; 8] = [6, 3, 2, 4, 5, 4, 4, 4];
const CYCLES_COMMON_A_EXTRA : [u32; 8] = [0, 0, 0, 0, 1, 0, 1, 1];

#[allow(non_snake_case)]
pub struct CPU {
    A       : W<u8>,  // Accumulator
    X       : W<u8>,  // Indexes
    Y       : W<u8>,  
    Flags   : u8,     // Status
    SP      : W<u8>,  // Stack pointer
    PC      : W<u16>, // Program counter
}

fn load_word(memory: &mut Mem, address: W<u16>) -> u16 {
    let low = memory.load(address.0) as u16;
    (memory.load((address + W(1)).0) as u16) << 8 | low
}

fn write_word(memory: &mut Mem, address: W<u16>, word: u16) {
    memory.write(address.0, (word >> 8) as u8);
    memory.write((address + W(1)).0, word as u8);
}

impl CPU {
    pub fn new() -> CPU {
        CPU {
            A       : W(0),
            X       : W(0),
            Y       : W(0),
            Flags   : 0x24, 
            SP      : W(0xff),
            PC      : W(0),
        }
    }

    fn pop(&mut self, memory: &mut Mem) -> u8 {
        self.SP = self.SP + W(1);
        memory.load(STACK_PAGE | (self.SP.0 as u16))
    }

    fn push(&mut self, memory: &mut Mem, byte: u8) {
        memory.write(STACK_PAGE | (self.SP.0 as u16), byte);
        self.SP = self.SP - W(1);
    }

    fn push_word(&mut self, memory: &mut Mem, word: u16) {
        self.push(memory, (word >> 8) as u8);
        self.push(memory, word as u8);
    }

    fn pop_word(&mut self, memory: &mut Mem) -> u16 {
        let low = self.pop(memory) as u16; 
        (self.pop(memory) as u16) << 8 | low
    }

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

    fn do_jump(&mut self, memory: &mut Mem, opcode: u8) -> u32 {
        let mut cycles = CYCLES_JUMP;
        let mut address = load_word(memory, self.PC + W(1)); 
        if opcode & !OP_JUMP_MASK > 0 {
            address = load_word(memory, W(address));
            // Indirect Jump, additional two cycles
            cycles += 2;
        } 
        self.PC = W(address);
        return cycles;
    }

    fn do_special(&mut self, memory: &mut Mem, opcode: u8) -> u32 { 
        if opcode == OP_JSR {
            // Load destination address and push return address
            let pc = self.PC;
            let address = load_word(memory, pc + W(1));
            self.push_word(memory, (pc + W(3)).0);
            self.PC = W(address);
            CYCLES_JSR
        } else {
            let index = (opcode >> 5) & 0x3;
            OP_SPECIAL_TABLE[index as usize](self, memory)
        }
    }

    fn do_branch(&mut self, memory: &mut Mem, opcode: u8) -> u32 {
        let index = opcode >> 5;
        let mut cycles = CYCLES_BRANCH;
        if OP_BRANCH_TABLE[index as usize](self.Flags) {
            let pc = self.PC;
            let mut offset = memory.load((pc + W(1)).0) as i8;
            // From sign-magnitude
            if offset < 0 { 
                offset = -(offset & 0x7F);
            }
            // Calculate branch address and push return address
            let address = pc + W((offset as i16) as u16);
            self.push_word(memory, (pc + W(3)).0);
            self.PC = address; 
            // Additional cycle if branch taken and page boundary crossed
            cycles += 1 + ((address & W(PAGE_MASK)) != (pc & W(PAGE_MASK))) as u32;
        }
        return cycles;
    }

    fn do_implied(&mut self, memory: &mut Mem, opcode: u8) -> u32 {
        let index = ((opcode >> 4) & 0xE) + ((opcode >> 1) & 1);
        OP_IMPLIED_TABLE[index as usize](self, memory);
        // Stack instructions get one additional cycle
        // An additional one if it is a pull
        CYCLES_IMPLIED + 
            (opcode & OP_IMP_STACK_MASK == OP_IMP_STACK) as u32 +
            (opcode & OP_IMP_PULL_MASK == OP_IMP_PULL) as u32
    }

    fn do_common(&mut self, memory: &mut Mem, opcode: u8) -> u32 {
        /* Common Operations */
        let addressing = (opcode >> 2) & 0x3;

        let index = ((opcode >> 3) & 0x1C) + (opcode & 0x3);
        OP_COMMON_TABLE[index as usize](self, memory, addressing);
        return 0;
    }
}

// Branch conditions

impl CPU {
    fn bpl (flags: u8) -> bool {
        !is_flag_set!(flags, FLAG_SIGN)
    }

    fn bmi (flags: u8) -> bool {
        is_flag_set!(flags, FLAG_SIGN)
    }

    fn bvc (flags: u8) -> bool {
        !is_flag_set!(flags, FLAG_OVER)
    }

    fn bvs (flags: u8) -> bool {
        is_flag_set!(flags, FLAG_OVER)
    }

    fn bcc (flags: u8) -> bool {
        !is_flag_set!(flags, FLAG_CARRY)
    }

    fn bcs (flags: u8) -> bool {
        is_flag_set!(flags, FLAG_CARRY)
    }

    fn bne (flags: u8) -> bool {
        !is_flag_set!(flags, FLAG_ZERO)
    }

    fn beq (flags: u8) -> bool {
        is_flag_set!(flags, FLAG_ZERO)
    }
}

// Instructions

impl CPU {

    // Special

    fn invalid_s(&mut self, memory: &mut Mem) -> u32 {
        assert!(false);
        return 0;
    }

    fn brk(&mut self, memory: &mut Mem) -> u32 {
       
       return CYCLES_BRK; 
    }

    fn rti(&mut self, memory: &mut Mem) -> u32 {
        
       return CYCLES_RTI;
    }

    fn rts(&mut self, memory: &mut Mem) -> u32 {
        
       return CYCLES_RTS;
    }

    // Implied

    fn invalid_i(&mut self, memory: &mut Mem) { 
        assert!(false);
    }

    fn php (&mut self, memory: &mut Mem) {

    }

    fn asl_a (&mut self, memory: &mut Mem) {

    }

    fn clc (&mut self, memory: &mut Mem) {

    }

    fn plp (&mut self, memory: &mut Mem) {

    }

    fn rol_a (&mut self, memory: &mut Mem) {

    }

    fn sec (&mut self, memory: &mut Mem) {

    }

    fn pha (&mut self, memory: &mut Mem) {

    }

    fn lsr_a (&mut self, memory: &mut Mem) {

    }

    fn cli (&mut self, memory: &mut Mem) {

    }

    fn pla (&mut self, memory: &mut Mem) {

    }

    fn ror_a (&mut self, memory: &mut Mem) {

    }

    fn sei (&mut self, memory: &mut Mem) {

    }

    fn dey (&mut self, memory: &mut Mem) {

    }

    fn txa (&mut self, memory: &mut Mem) {

    }

    fn tya (&mut self, memory: &mut Mem) {

    }

    fn txs (&mut self, memory: &mut Mem) {

    }

    fn tay (&mut self, memory: &mut Mem) {

    }

    fn tax (&mut self, memory: &mut Mem) {

    }

    fn clv (&mut self, memory: &mut Mem) {

    }

    fn tsx (&mut self, memory: &mut Mem) {

    }

    fn iny (&mut self, memory: &mut Mem) {
        self.Y = self.Y + W(1);
        set_zero!(self.Flags, self.Y.0);
        set_sign!(self.Flags, self.Y.0);
    }

    fn dex (&mut self, memory: &mut Mem) {

    }

    fn cld (&mut self, memory: &mut Mem) {

    }

    fn inx (&mut self, memory: &mut Mem) {
        self.X = self.X + W(1);
        set_zero!(self.Flags, self.X.0);
        set_sign!(self.Flags, self.X.0);
    }

    fn nop (&mut self, memory: &mut Mem) {

    }

    fn sed (&mut self, memory: &mut Mem) {

    }

    // Common

    fn invalid_c(&mut self, memory: &mut Mem, addressing: u8) {
        assert!(false);
    }

    fn ora (&mut self, memory: &mut Mem, addressing: u8) {

    }

    fn asl (&mut self, memory: &mut Mem, addressing: u8) {

    }

    fn bit (&mut self, memory: &mut Mem, addressing: u8) {

    }

    fn and (&mut self, memory: &mut Mem, addressing: u8) {

    }

    fn rol (&mut self, memory: &mut Mem, addressing: u8) {

    }

    fn eor (&mut self, memory: &mut Mem, addressing: u8) {

    }

    fn lsr (&mut self, memory: &mut Mem, addressing: u8) {

    }

    fn adc (&mut self, memory: &mut Mem, addressing: u8) {

    }

    fn ror (&mut self, memory: &mut Mem, addressing: u8) {

    }

    fn sty (&mut self, memory: &mut Mem, addressing: u8) {

    }

    fn sta (&mut self, memory: &mut Mem, addressing: u8) {

    }

    fn stx (&mut self, memory: &mut Mem, addressing: u8) {

    }

    fn ldy (&mut self, memory: &mut Mem, addressing: u8) {

    }

    fn lda (&mut self, memory: &mut Mem, addressing: u8) {

    }

    fn ldx (&mut self, memory: &mut Mem, addressing: u8) {

    }

    fn cpy (&mut self, memory: &mut Mem, addressing: u8) {

    }

    fn cmp (&mut self, memory: &mut Mem, addressing: u8) {

    }

    fn dec (&mut self, memory: &mut Mem, addressing: u8) {

    }

    fn cpx (&mut self, memory: &mut Mem, addressing: u8) {

    }

    fn sbc (&mut self, memory: &mut Mem, addressing: u8) {

    }
   
    fn inc (&mut self, memory: &mut Mem, addressing: u8) {

    }
}

impl fmt::Display for CPU {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{{ A: {}, X: {}, Y: {}, P: {}, SP: {}, PC: {} }}",
               self.A.0 , self.X.0 , self.Y.0 , self.Flags , self.SP.0 , self.PC.0)
    }
}
