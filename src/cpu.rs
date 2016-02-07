use std::fmt;
use mem;

const OP_SPECIAL_TABLE : [fn(&CPU, &mem::Memory) -> (); 4] = [
    CPU::brk,
    CPU::invalid,
    CPU::rti,
    CPU::rts,
];

const OP_BRANCH_TABLE : [fn(&CPU, &mem::Memory, i8) -> (); 8] = [
    CPU::bpl,
    CPU::bmi,
    CPU::bvc,
    CPU::bvs,
    CPU::bcc,
    CPU::bcs,
    CPU::bne,
    CPU::beq,
];

const OP_IMPLIED_TABLE : [fn(&CPU, &mem::Memory) -> (); 32] = [
    CPU::php,
    CPU::asl_a,
    CPU::clc,
    CPU::invalid,
    CPU::plp, 
    CPU::rol_a,
    CPU::sec,
    CPU::invalid,
    CPU::pha,
    CPU::lsr_a,
    CPU::cli,
    CPU::invalid,
    CPU::pla,
    CPU::ror_a,
    CPU::sei,
    CPU::invalid,
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
    CPU::invalid,
    CPU::inx,
    CPU::nop,
    CPU::sed,
    CPU::invalid,
];

const OP_COMMON_TABLE : [fn(&CPU, &mem::Memory, u8) -> (); 32] = [
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

const OP_JUMP_MASK     : u8 = 0xDF;
const OP_JUMP          : u8 = 0x4C;
const OP_SPECIAL_MASK  : u8 = 0x9F;
const OP_SPECIAL       : u8 = 0x00;
const OP_BRANCH_MASK   : u8 = 0x1F;
const OP_BRANCH        : u8 = 0x10;
const OP_IMPLIED_MASK  : u8 = 0x1F;
const OP_IMPLIED       : u8 = 0x08;
const OP_JSR           : u8 = 0x20;

#[allow(non_snake_case)]
pub struct CPU {
    A : u8,  // Accumulator
    X : u8,  // Indexes
    Y : u8,  
    P : u8,  // Status
    SP: u8,  // Stack pointer
    PC: u16, // Program counter
}

fn load_word(memory: &mem::Memory, address: u16) -> u16 () {
    let mut ret : u16 = memory.load(address) as u16;
    ret <<= 8;
    ret |= memory.load(address + 1) as u16;
    return ret;
}

impl CPU {
    pub fn new() -> CPU {
        CPU {
            A : 0,
            X : 0,
            Y : 0,
            P : 0x24, 
            SP : 0xfd,
            PC : 0,
        }
    }

    pub fn execute(&self, memory: &mem::Memory) -> () {
        let mut pc = self.PC;
        let opcode = memory.load(pc);
        pc += 1;
        if opcode & OP_JUMP_MASK == OP_JUMP {
            /* JMP */
            let mut address = load_word(memory, pc); 
            if opcode & !OP_JUMP_MASK > 0 {
                // Indirect Jump, +2 Cycles
                address = load_word(memory, address);
            } 
            self.jmp(memory, address);
        } else if opcode & OP_SPECIAL_MASK == OP_SPECIAL {
            /* Special */
            if opcode == OP_JSR {
                // TODO: Check PC
                let address = load_word(memory, pc);
                self.jsr(memory, address);
            } else {
                let index = (opcode >> 5) & 0x3;
                OP_SPECIAL_TABLE[index as usize](self, memory);
            }
        } else if opcode & OP_BRANCH_MASK == OP_BRANCH {
            /* Branch */
            let mut offset = memory.load(pc) as i8;
            // To sign-magnitude
            if offset < 0 { 
                offset = -(offset & 0x7F);
            }
            let index = opcode >> 5;
            OP_BRANCH_TABLE[index as usize](self, memory, offset);
        } else if opcode & OP_IMPLIED_MASK == OP_IMPLIED {
            /* Implied */
            let index = ((opcode >> 4) & 0xE) + ((opcode >> 1) & 1);
            OP_IMPLIED_TABLE[index as usize](self, memory);
        } else { 
            /* Common Operations */
            let addressing = (opcode >> 2) & 0x3;
            let index = ((opcode >> 3) & 0x1C) + (opcode & 0x3);
            OP_COMMON_TABLE[index as usize](self, memory, addressing);
        } 
    }
}

// Instructions

impl CPU {

    // Special

    fn invalid(&self, memory: &mem::Memory) -> () {

    }

    fn brk(&self, memory: &mem::Memory) -> () {
        
    }

    fn rti(&self, memory: &mem::Memory) -> () {
        
    }

    fn rts(&self, memory: &mem::Memory) -> () {
        
    }

    // Jumps

    fn jmp(&self, memory: &mem::Memory, address: u16) {

    }

    fn jsr(&self, memory: &mem::Memory, address: u16) {

    }

    // Branches

    fn bpl (&self, memory: &mem::Memory, offset: i8) {

    }

    fn bmi (&self, memory: &mem::Memory, offset: i8) {

    }

    fn bvc (&self, memory: &mem::Memory, offset: i8) {

    }

    fn bvs (&self, memory: &mem::Memory, offset: i8) {

    }

    fn bcc (&self, memory: &mem::Memory, offset: i8) {

    }

    fn bcs (&self, memory: &mem::Memory, offset: i8) {

    }

    fn bne (&self, memory: &mem::Memory, offset: i8) {

    }

    fn beq (&self, memory: &mem::Memory, offset: i8) {

    }
    
    // Implied

    fn php (&self, memory: &mem::Memory) {

    }

    fn asl_a (&self, memory: &mem::Memory) {

    }

    fn clc (&self, memory: &mem::Memory) {

    }

    fn plp (&self, memory: &mem::Memory) {

    }

    fn rol_a (&self, memory: &mem::Memory) {

    }

    fn sec (&self, memory: &mem::Memory) {

    }

    fn pha (&self, memory: &mem::Memory) {

    }

    fn lsr_a (&self, memory: &mem::Memory) {

    }

    fn cli (&self, memory: &mem::Memory) {

    }

    fn pla (&self, memory: &mem::Memory) {

    }

    fn ror_a (&self, memory: &mem::Memory) {

    }

    fn sei (&self, memory: &mem::Memory) {

    }

    fn dey (&self, memory: &mem::Memory) {

    }

    fn txa (&self, memory: &mem::Memory) {

    }

    fn tya (&self, memory: &mem::Memory) {

    }

    fn txs (&self, memory: &mem::Memory) {

    }

    fn tay (&self, memory: &mem::Memory) {

    }

    fn tax (&self, memory: &mem::Memory) {

    }

    fn clv (&self, memory: &mem::Memory) {

    }

    fn tsx (&self, memory: &mem::Memory) {

    }

    fn iny (&self, memory: &mem::Memory) {

    }

    fn dex (&self, memory: &mem::Memory) {

    }

    fn cld (&self, memory: &mem::Memory) {

    }

    fn inx (&self, memory: &mem::Memory) {

    }

    fn nop (&self, memory: &mem::Memory) {

    }

    fn sed (&self, memory: &mem::Memory) {

    }

    // Common

    fn invalid_c(&self, memory: &mem::Memory, addressing: u8) -> () {

    }

    fn ora (&self, memory: &mem::Memory, addressing: u8) {

    }

    fn asl (&self, memory: &mem::Memory, addressing: u8) {

    }

    fn bit (&self, memory: &mem::Memory, addressing: u8) {

    }

    fn and (&self, memory: &mem::Memory, addressing: u8) {

    }

    fn rol (&self, memory: &mem::Memory, addressing: u8) {

    }

    fn eor (&self, memory: &mem::Memory, addressing: u8) {

    }

    fn lsr (&self, memory: &mem::Memory, addressing: u8) {

    }

    fn adc (&self, memory: &mem::Memory, addressing: u8) {

    }

    fn ror (&self, memory: &mem::Memory, addressing: u8) {

    }

    fn sty (&self, memory: &mem::Memory, addressing: u8) {

    }

    fn sta (&self, memory: &mem::Memory, addressing: u8) {

    }

    fn stx (&self, memory: &mem::Memory, addressing: u8) {

    }

    fn ldy (&self, memory: &mem::Memory, addressing: u8) {

    }

    fn lda (&self, memory: &mem::Memory, addressing: u8) {

    }

    fn ldx (&self, memory: &mem::Memory, addressing: u8) {

    }

    fn cpy (&self, memory: &mem::Memory, addressing: u8) {

    }

    fn cmp (&self, memory: &mem::Memory, addressing: u8) {

    }

    fn dec (&self, memory: &mem::Memory, addressing: u8) {

    }

    fn cpx (&self, memory: &mem::Memory, addressing: u8) {

    }

    fn sbc (&self, memory: &mem::Memory, addressing: u8) {

    }
   
    fn inc (&self, memory: &mem::Memory, addressing: u8) {

    }
}

impl fmt::Display for CPU {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{{ A: {}, X: {}, Y: {}, P: {}, SP: {}, PC: {} }}",
               self.A, self.X, self.Y, self.P, self.SP, self.PC)
    }
}
