use std::fmt;
use mem::Memory as Mem;
use std::num::Wrapping as W;

macro_rules! set_overflow {
    ($flags:expr) => ($flags = $flags | (1 << 6));
}

macro_rules! unset_overflow {
    ($flags:expr) => ($flags = $flags & !(1 << 6));
}

macro_rules! uset_negative {
    ($flags:expr, $val:expr) => ( 
        if ($val & W(1 << 7)) == W(0) {
            $flags = $flags & !(1 << 7)
        }else{
            $flags = $flags | (1 << 7)
        });
}

macro_rules! set_break {
    ($flags:expr) => ($flags = $flags | (1 << 4));
}

macro_rules! unset_break {
    ($flags:expr) => ($flags = $flags & !(1 << 4));
}

macro_rules! set_decimal {
    ($flags:expr) => ($flags = $flags | (1 << 3));
}

macro_rules! unset_decimal {
    ($flags:expr) => ($flags = $flags & !(1 << 3));
}

macro_rules! set_interrupt {
    ($flags:expr) => ($flags = $flags | (1 << 2));
}

macro_rules! unset_interrupt {
    ($flags:expr) => ($flags = $flags & !(1 << 2));
}

macro_rules! set_zero {
    ($flags:expr) => ($flags = $flags | (1 << 1));
}

macro_rules! unset_zero {
    ($flags:expr) => ($flags = $flags & !(1 << 1));
}

macro_rules! set_carry {
    ($flags:expr) => ($flags = $flags | (1));
}

macro_rules! unset_carry {
    ($flags:expr) => ($flags = $flags & !(1));
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

const STACK_PAGE        : u16 = 0x0100;
const PAGE_MASK         : u16 = 0xFF00;

const FLAG_CARRY_MASK   : u8 = 0x01;
const FLAG_ZERO_MASK    : u8 = 0x02;
const FLAG_INT_MASK     : u8 = 0x04;
const FLAG_DEC_MASK     : u8 = 0x08;
const FLAG_BRK_MASK     : u8 = 0x10;
const FLAG_UNUSED_MASK  : u8 = 0x20;
const FLAG_OF_MASK      : u8 = 0x40;
const FLAG_SIGN_MASK    : u8 = 0x80;

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

#[allow(non_snake_case)]
pub struct CPU {
    A       : W<u8>,  // Accumulator
    X       : W<u8>,  // Indexes
    Y       : W<u8>,  
    Flags   : u8,  // Status
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
        let mut cycles : u32 = 0;
        let opcode = memory.load(self.PC.0);

        if opcode & OP_JUMP_MASK == OP_JUMP {
            /* JMP */
            cycles = CYCLES_JUMP;
            let mut address = load_word(memory, self.PC + W(1)); 
            if opcode & !OP_JUMP_MASK > 0 {
                // Indirect Jump, additional two cycles
                cycles += 2;
                address = load_word(memory, W(address));
            } 
            self.PC = W(address);
        } else if opcode & OP_SPECIAL_MASK == OP_SPECIAL {
            /* Special */
            cycles = if opcode == OP_JSR {
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
        } else if opcode & OP_BRANCH_MASK == OP_BRANCH {
            /* Branch */
            cycles = CYCLES_BRANCH;
            let index = opcode >> 5;
            if OP_BRANCH_TABLE[index as usize](self.Flags) {
                // Additional cycle if branch taken
                cycles += 1;
                let pc = self.PC;
                let mut offset = memory.load((pc + W(1)).0) as i8;
                // To sign-magnitude
                if offset < 0 { 
                    offset = -(offset & 0x7F);
                }
                // Calculate branch address and push return address
                let address = self.PC + W((offset as i16) as u16);
                if (address & W(PAGE_MASK)) != (self.PC & W(PAGE_MASK)) {
                    // Additional cycle if page boundary crossed
                    cycles += 1;
                }
                self.push_word(memory, (pc + W(3)).0);
                self.PC = address; 
            }
        } else if opcode & OP_IMPLIED_MASK == OP_IMPLIED {
            /* Implied */
            cycles = CYCLES_IMPLIED;
            // Stack instructions get one additional cycle
            cycles += (opcode & OP_IMP_STACK_MASK == OP_IMP_STACK) as u32; 
            // An additional if it is a pull
            cycles += (opcode & OP_IMP_PULL_MASK == OP_IMP_PULL) as u32;
            let index = ((opcode >> 4) & 0xE) + ((opcode >> 1) & 1);
            OP_IMPLIED_TABLE[index as usize](self, memory);
        } else { 
            /* Common Operations */
            let addressing = (opcode >> 2) & 0x3;
            let index = ((opcode >> 3) & 0x1C) + (opcode & 0x3);
            OP_COMMON_TABLE[index as usize](self, memory, addressing);
        } 

        return cycles;
    }
}

// Branch conditions

impl CPU {

    fn bpl (P: u8) -> bool {
        P & FLAG_SIGN_MASK == 0
    }

    fn bmi (P: u8) -> bool {
        P & FLAG_SIGN_MASK != 0
    }

    fn bvc (P: u8) -> bool {
        P & FLAG_OF_MASK == 0
    }

    fn bvs (P: u8) -> bool {
        P & FLAG_OF_MASK != 0
    }

    fn bcc (P: u8) -> bool {
        P & FLAG_CARRY_MASK == 0
    }

    fn bcs (P: u8) -> bool {
        P & FLAG_CARRY_MASK != 0
    }

    fn bne (P: u8) -> bool {
        P & FLAG_ZERO_MASK == 0
    }

    fn beq (P: u8) -> bool {
        P & FLAG_ZERO_MASK != 0
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
        if self.Y == W(0){
            set_zero!(self.Flags);
        }else{
            unset_zero!(self.Flags);
        }
        uset_negative!(self.Flags, self.Y)
    }

    fn dex (&mut self, memory: &mut Mem) {

    }

    fn cld (&mut self, memory: &mut Mem) {

    }

    fn inx (&mut self, memory: &mut Mem) {
        self.X = self.X + W(1);
        if self.X == W(0){
            set_zero!(self.Flags);
        }else{
            unset_zero!(self.Flags);
        }
        uset_negative!(self.Flags, self.X)
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
