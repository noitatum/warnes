use std::fmt;
use mem::Memory as Mem;
use std::num::Wrapping as W;

/* WARNING: Branch instructions are replaced with nops */
/* Addressing, Instruction, Cycles, Has Penalty */

const OPCODE_TABLE : [(fn(&mut CPU, &mut Mem) -> (W<u16>, bool),
                      fn(&mut CPU, &mut Mem, W<u16>), u32, bool); 256] = [
    (CPU::imp, CPU::brk, 7, false), (CPU::idx, CPU::ora, 6, false), 
    (CPU::imp, CPU::nop, 2, false), (CPU::imp, CPU::nop, 2, false), 
    (CPU::imp, CPU::nop, 2, false), (CPU::zpg, CPU::ora, 3, false),
    (CPU::zpg, CPU::asl, 5, false), (CPU::imp, CPU::nop, 2, false),
    (CPU::imp, CPU::php, 3, false), (CPU::imm, CPU::ora, 2, false),
    (CPU::imp, CPU::sla, 2, false), (CPU::imp, CPU::nop, 2, false), 
    (CPU::imp, CPU::nop, 2, false), (CPU::abs, CPU::ora, 4, false),
    (CPU::abs, CPU::asl, 6, false), (CPU::imp, CPU::nop, 2, false), 
    
    (CPU::imp, CPU::nop, 2, false), (CPU::idy, CPU::ora, 5, true), 
    (CPU::imp, CPU::nop, 2, false), (CPU::imp, CPU::nop, 2, false),
    (CPU::imp, CPU::nop, 2, false), (CPU::zpx, CPU::ora, 4, false),
    (CPU::zpx, CPU::asl, 6, false), (CPU::imp, CPU::nop, 2, false),
    (CPU::imp, CPU::clc, 2, false), (CPU::aby, CPU::ora, 4, true),
    (CPU::imp, CPU::nop, 2, false), (CPU::imp, CPU::nop, 2, false),
    (CPU::imp, CPU::nop, 2, false), (CPU::abx, CPU::ora, 4, true), 
    (CPU::abx, CPU::asl, 7, false), (CPU::imp, CPU::nop, 2, false),

    (CPU::abs, CPU::jsr, 6, false), (CPU::idx, CPU::and, 6, false), 
    (CPU::imp, CPU::nop, 2, false), (CPU::imp, CPU::nop, 6, false),
    (CPU::zpg, CPU::bit, 3, false), (CPU::zpg, CPU::and, 3, false),
    (CPU::zpg, CPU::rol, 5, false), (CPU::imp, CPU::nop, 2, false),
    (CPU::imp, CPU::plp, 4, false), (CPU::imm, CPU::and, 2, false),
    (CPU::imp, CPU::rla, 2, false), (CPU::imp, CPU::nop, 2, false),
    (CPU::abs, CPU::bit, 4, false), (CPU::abs, CPU::and, 4, false),
    (CPU::abs, CPU::rol, 6, false), (CPU::imp, CPU::nop, 2, false),

    (CPU::imp, CPU::nop, 2, false), (CPU::idy, CPU::and, 5, true),
    (CPU::imp, CPU::nop, 2, false), (CPU::imp, CPU::nop, 2, false),
    (CPU::imp, CPU::nop, 2, false), (CPU::zpx, CPU::and, 4, false),
    (CPU::zpx, CPU::rol, 6, false), (CPU::imp, CPU::nop, 2, false),
    (CPU::imp, CPU::sec, 2, false), (CPU::aby, CPU::and, 4, true),
    (CPU::imp, CPU::nop, 2, false), (CPU::imp, CPU::nop, 2, true),
    (CPU::imp, CPU::nop, 2, false), (CPU::abx, CPU::and, 4, true),
    (CPU::abx, CPU::rol, 7, false), (CPU::imp, CPU::nop, 2, false),

    (CPU::imp, CPU::rti, 6, false), (CPU::idx, CPU::eor, 6, false),
    (CPU::imp, CPU::nop, 2, false), (CPU::imp, CPU::nop, 2, false),
    (CPU::imp, CPU::nop, 2, false), (CPU::zpg, CPU::eor, 3, false), 
    (CPU::zpg, CPU::lsr, 5, false), (CPU::imp, CPU::nop, 2, false),
    (CPU::imp, CPU::pha, 3, false), (CPU::imm, CPU::eor, 2, false),
    (CPU::imp, CPU::sra, 2, false), (CPU::imp, CPU::nop, 2, false),
    (CPU::abs, CPU::jmp, 3, false), (CPU::abs, CPU::eor, 4, false),
    (CPU::abs, CPU::lsr, 6, false), (CPU::imp, CPU::nop, 2, false),

    (CPU::imp, CPU::nop, 2, false), (CPU::idy, CPU::eor, 5, true), 
    (CPU::imp, CPU::nop, 2, false), (CPU::imp, CPU::nop, 2, false),
    (CPU::imp, CPU::nop, 2, false), (CPU::zpx, CPU::eor, 4, false),
    (CPU::zpx, CPU::lsr, 6, false), (CPU::imp, CPU::nop, 2, false),
    (CPU::imp, CPU::cli, 2, false), (CPU::aby, CPU::eor, 4, true), 
    (CPU::imp, CPU::nop, 2, false), (CPU::imp, CPU::nop, 2, false), 
    (CPU::imp, CPU::nop, 2, false), (CPU::abx, CPU::eor, 4, true),
    (CPU::abx, CPU::lsr, 7, false), (CPU::imp, CPU::nop, 2, false),

    (CPU::imp, CPU::rts, 6, false), (CPU::idx, CPU::adc, 6, false),
    (CPU::imp, CPU::nop, 2, false), (CPU::imp, CPU::nop, 2, false),
    (CPU::imp, CPU::nop, 2, false), (CPU::zpg, CPU::adc, 3, false),
    (CPU::zpg, CPU::ror, 5, false), (CPU::imp, CPU::nop, 2, false),
    (CPU::imp, CPU::pla, 4, false), (CPU::imm, CPU::adc, 2, false),
    (CPU::imp, CPU::rra, 2, false), (CPU::imp, CPU::nop, 2, false),
    (CPU::ind, CPU::jmp, 5, false), (CPU::abs, CPU::adc, 4, false),
    (CPU::abs, CPU::ror, 6, false), (CPU::imp, CPU::nop, 2, false),

    (CPU::imp, CPU::nop, 2, false), (CPU::idy, CPU::adc, 5, true),
    (CPU::imp, CPU::nop, 2, false), (CPU::imp, CPU::nop, 2, false),
    (CPU::imp, CPU::nop, 2, false), (CPU::zpx, CPU::adc, 4, false),
    (CPU::zpx, CPU::ror, 6, false), (CPU::imp, CPU::nop, 2, false),
    (CPU::imp, CPU::sei, 2, false), (CPU::aby, CPU::adc, 4, true),
    (CPU::imp, CPU::nop, 2, false), (CPU::imp, CPU::nop, 2, false), 
    (CPU::imp, CPU::nop, 2, false), (CPU::abx, CPU::adc, 4, true),
    (CPU::abx, CPU::ror, 7, false), (CPU::imp, CPU::nop, 2, false),

    (CPU::imp, CPU::nop, 2, false), (CPU::idx, CPU::sta, 6, false),
    (CPU::imp, CPU::nop, 2, false), (CPU::imp, CPU::nop, 2, false),
    (CPU::zpg, CPU::sty, 3, false), (CPU::zpg, CPU::sta, 3, false),
    (CPU::zpg, CPU::stx, 3, false), (CPU::imp, CPU::nop, 2, false),
    (CPU::imp, CPU::dey, 2, false), (CPU::imp, CPU::nop, 2, false),
    (CPU::imp, CPU::txa, 2, false), (CPU::imp, CPU::nop, 2, false),
    (CPU::abs, CPU::sty, 4, false), (CPU::abs, CPU::sta, 4, false),
    (CPU::abs, CPU::stx, 4, false), (CPU::imp, CPU::nop, 2, false),

    (CPU::imp, CPU::nop, 2, false), (CPU::idy, CPU::sta, 6, false),
    (CPU::imp, CPU::nop, 2, false), (CPU::imp, CPU::nop, 2, false), 
    (CPU::zpx, CPU::sty, 4, false), (CPU::zpx, CPU::sta, 4, false),
    (CPU::zpy, CPU::stx, 4, false), (CPU::imp, CPU::nop, 2, false),
    (CPU::imp, CPU::tya, 2, false), (CPU::aby, CPU::sta, 5, false), 
    (CPU::imp, CPU::txs, 2, false), (CPU::imp, CPU::nop, 2, false), 
    (CPU::imp, CPU::nop, 2, false), (CPU::abx, CPU::sta, 5, false),
    (CPU::imp, CPU::nop, 2, false), (CPU::imp, CPU::nop, 2, false),

    (CPU::imm, CPU::ldy, 2, false), (CPU::idx, CPU::lda, 6, false), 
    (CPU::imm, CPU::ldx, 2, false), (CPU::imp, CPU::nop, 2, false),
    (CPU::zpg, CPU::ldy, 3, false), (CPU::zpg, CPU::lda, 3, false),
    (CPU::zpg, CPU::ldx, 3, false), (CPU::imp, CPU::nop, 2, false),
    (CPU::imp, CPU::tay, 2, false), (CPU::imm, CPU::lda, 2, false),
    (CPU::imp, CPU::tax, 2, false), (CPU::imp, CPU::nop, 2, false),
    (CPU::abs, CPU::ldy, 4, false), (CPU::abs, CPU::lda, 4, false),
    (CPU::abs, CPU::ldx, 4, false), (CPU::imp, CPU::nop, 4, false),

    (CPU::imp, CPU::nop, 2, false), (CPU::idy, CPU::lda, 5, true), 
    (CPU::imp, CPU::nop, 2, false), (CPU::imp, CPU::nop, 2, false),
    (CPU::zpx, CPU::ldy, 4, false), (CPU::zpx, CPU::lda, 4, false),
    (CPU::zpy, CPU::ldx, 4, false), (CPU::imp, CPU::nop, 2, false),
    (CPU::imp, CPU::clv, 2, false), (CPU::aby, CPU::lda, 4, true), 
    (CPU::imp, CPU::tsx, 2, false), (CPU::imp, CPU::nop, 2, false),
    (CPU::abx, CPU::ldy, 4, true),  (CPU::abx, CPU::lda, 4, true),
    (CPU::aby, CPU::ldx, 4, true),  (CPU::imp, CPU::nop, 2, false),

    (CPU::imm, CPU::cpy, 2, false), (CPU::idx, CPU::cmp, 6, false), 
    (CPU::imp, CPU::nop, 2, false), (CPU::imp, CPU::nop, 2, false), 
    (CPU::zpg, CPU::cpy, 3, false), (CPU::zpg, CPU::cmp, 3, false),
    (CPU::zpg, CPU::dec, 5, false), (CPU::imp, CPU::nop, 2, false),
    (CPU::imp, CPU::iny, 2, false), (CPU::imm, CPU::cmp, 2, false),
    (CPU::imp, CPU::dex, 2, false), (CPU::imp, CPU::nop, 2, false),
    (CPU::abs, CPU::cpy, 4, false), (CPU::abs, CPU::cmp, 4, false),
    (CPU::abs, CPU::dec, 6, false), (CPU::imp, CPU::nop, 2, false),

    (CPU::imp, CPU::nop, 2, false), (CPU::idy, CPU::cmp, 5, true),
    (CPU::imp, CPU::nop, 2, false), (CPU::imp, CPU::nop, 2, false),
    (CPU::imp, CPU::nop, 2, false), (CPU::zpx, CPU::cmp, 4, false),
    (CPU::zpx, CPU::dec, 6, false), (CPU::imp, CPU::nop, 2, false),
    (CPU::imp, CPU::cld, 2, false), (CPU::aby, CPU::cmp, 4, true),
    (CPU::imp, CPU::nop, 2, false), (CPU::imp, CPU::nop, 2, false),
    (CPU::imp, CPU::nop, 2, false), (CPU::abx, CPU::cmp, 4, true),
    (CPU::abx, CPU::dec, 7, false), (CPU::imp, CPU::nop, 2, false),

    (CPU::imm, CPU::cpx, 2, false), (CPU::idx, CPU::sbc, 6, false), 
    (CPU::imp, CPU::nop, 2, false), (CPU::imp, CPU::nop, 2, false),
    (CPU::zpg, CPU::cpx, 3, false), (CPU::zpg, CPU::sbc, 3, false),
    (CPU::zpg, CPU::inc, 6, false), (CPU::imp, CPU::nop, 2, false),
    (CPU::imp, CPU::inx, 2, false), (CPU::imm, CPU::sbc, 2, false),
    (CPU::imp, CPU::nop, 2, false), (CPU::imp, CPU::nop, 2, false),
    (CPU::abs, CPU::cpx, 4, false), (CPU::abs, CPU::sbc, 4, false),
    (CPU::abs, CPU::inc, 6, false), (CPU::imp, CPU::nop, 2, false),

    (CPU::imp, CPU::nop, 2, false), (CPU::idy, CPU::sbc, 5, true), 
    (CPU::imp, CPU::nop, 2, false), (CPU::imp, CPU::nop, 2, false),
    (CPU::imp, CPU::nop, 2, false), (CPU::zpx, CPU::sbc, 4, false),
    (CPU::zpx, CPU::inc, 6, false), (CPU::imp, CPU::nop, 2, false),
    (CPU::imp, CPU::sed, 2, false), (CPU::aby, CPU::sbc, 4, true), 
    (CPU::imp, CPU::nop, 2, false), (CPU::imp, CPU::nop, 2, false), 
    (CPU::imp, CPU::nop, 2, false), (CPU::abx, CPU::sbc, 4, true),
    (CPU::abx, CPU::inc, 7, false), (CPU::imp, CPU::nop, 2, false),
    ];


/* Memory */
const STACK_PAGE        : W<u16> = W(0x0100 as u16); 
const PAGE_MASK         : W<u16> = W(0xFF00 as u16);

/* Flag bits */
const FLAG_CARRY        : u8 = 0x01;
const FLAG_ZERO         : u8 = 0x02;
const FLAG_INTERRUPT    : u8 = 0x04;
const FLAG_DECIMAL      : u8 = 0x08;
const FLAG_BRK          : u8 = 0x10;
const FLAG_PUSHED       : u8 = 0x20;
const FLAG_OVERFLOW     : u8 = 0x40;
const FLAG_SIGN         : u8 = 0x80;

const BRANCH_FLAG_TABLE : [u8; 4] = 
    [FLAG_SIGN, FLAG_OVERFLOW, FLAG_CARRY, FLAG_ZERO];

const CYCLES_BRANCH     : u32 = 2;
const OP_BRANCH         : u8 = 0x10;
const OP_BRANCH_MASK    : u8 = 0x1F;


#[allow(non_snake_case)]
pub struct CPU {
    A       : W<u8>,  // Accumulator
    X       : W<u8>,  // Indexes
    Y       : W<u8>,  
    Flags   : u8,     // Status
    SP      : W<u8>,  // Stack pointer
    PC      : W<u16>, // Program counter
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

    fn branch(&mut self, memory: &mut Mem, opcode: u8) -> u32 {
        let index = opcode >> 6;
        let check = ((opcode >> 5) & 1) != 0;
        if is_flag_set!(self.Flags, BRANCH_FLAG_TABLE[index as usize]) != check {
            self.PC = self.PC + W(2);
            return CYCLES_BRANCH;
        }
        let pc = self.PC;
        let mut offset = memory.load(pc + W(1)).0 as i8;
        // From sign-magnitude
        if offset < 0 { 
            offset = -(offset & 0x7F);
        }
        // Calculate branch address and push return address
        let address = pc + W((offset as i16) as u16);
        self.push_word(memory, pc + W(2));
        self.PC = address; 
        // Additional cycle if branch taken and page boundary crossed
        CYCLES_BRANCH + 1 + ((address & PAGE_MASK) != (pc & PAGE_MASK)) as u32
    }

    pub fn execute(&mut self, memory: &mut Mem) -> u32 {
        let opcode = memory.load(self.PC).0;
        if opcode & OP_BRANCH_MASK == OP_BRANCH {
            self.branch(memory, opcode)
        } else {
            let instruction = OPCODE_TABLE[opcode as usize]; 
            // Get address from mode
            let (address, crossed) = instruction.0(self, memory);
            // Execute the instruction
            instruction.1(self, memory, address);
            // Add the extra cycle if needed
            instruction.2 + (instruction.3 && crossed) as u32
        }
    }
}

// Util functions

impl CPU {

    fn pop(&mut self, memory: &mut Mem) -> W<u8> {
        self.SP = self.SP + W(1);
        memory.load(STACK_PAGE | W16!(self.SP))
    }

    fn push(&mut self, memory: &mut Mem, byte: W<u8>) {
        memory.store(STACK_PAGE | W16!(self.SP), byte);
        self.SP = self.SP - W(1);
    }

    fn push_word(&mut self, memory: &mut Mem, word: W<u16>) {
        self.push(memory, W8!(word >> 8));
        self.push(memory, W8!(word));
    }

    fn pop_word(&mut self, memory: &mut Mem) -> W<u16> {
        let low = W16!(self.pop(memory)); 
        (W16!(self.pop(memory)) << 8) | low
    }
}

// Addressing modes

impl CPU {

    fn imp(&mut self, _: &mut Mem) -> (W<u16>, bool) {
        self.PC = self.PC + W(1);
        (W(0), false)
    }

    fn imm(&mut self, _: &mut Mem) -> (W<u16>, bool) {
        self.PC = self.PC + W(2);
        (self.PC - W(1), false)
    }

    fn zpg(&mut self, memory: &mut Mem) -> (W<u16>, bool) {
        let address = W16!(memory.load(self.PC + W(1)));
        self.PC = self.PC + W(2);
        (address, false)
    }

    fn abs(&mut self, memory: &mut Mem) -> (W<u16>, bool) {
        let address = W16!(memory.load_word(self.PC + W(1)));
        self.PC = self.PC + W(3);
        (address, false)
    }

    fn ind(&mut self, memory: &mut Mem) -> (W<u16>, bool) {
        let address = memory.load_word(self.PC + W(1));
        self.PC = self.PC + W(3);
        (memory.load_word_page_wrap(address), false)
    }

    fn idx(&mut self, memory: &mut Mem) -> (W<u16>, bool) {
        let address = W16!(memory.load(self.PC + W(1)) + self.X);
        self.PC = self.PC + W(2);
        (memory.load_word_page_wrap(address), false)
    }

    fn idy(&mut self, memory: &mut Mem) -> (W<u16>, bool) {
        let addr = W16!(memory.load(self.PC + W(1))); 
        let dest = W16!(memory.load_word_page_wrap(addr) + W16!(self.Y));
        self.PC = self.PC + W(2);
        (dest, W8!(dest) < self.Y)
    }

    fn zpx(&mut self, memory: &mut Mem) -> (W<u16>, bool) {
        let address = W16!(memory.load(self.PC + W(1)) + self.X);
        self.PC = self.PC + W(2);
        (address, false)
    }

    fn zpy(&mut self, memory: &mut Mem) -> (W<u16>, bool) {
        let address = W16!(memory.load(self.PC + W(1)) + self.Y);
        self.PC = self.PC + W(2);
        (address, false)
    }

    fn abx(&mut self, memory: &mut Mem) -> (W<u16>, bool) {
        let address = memory.load_word(self.PC + W(1)) + W16!(self.X);
        self.PC = self.PC + W(3);
        (address, W8!(address) < self.X)
    }

    fn aby(&mut self, memory: &mut Mem) -> (W<u16>, bool) {
        let address = memory.load_word(self.PC + W(1)) + W16!(self.Y);
        self.PC = self.PC + W(3);
        (address, W8!(address) < self.Y)
    }
}

// Instructions

impl CPU {   
    
    // Jump

    fn jsr(&mut self, memory: &mut Mem, address: W<u16>) {
        // Load destination address and push return address
        let ret = self.PC;
        self.push_word(memory, ret);
        self.PC = address;
    }

    fn jmp(&mut self, _: &mut Mem, address: W<u16>) {
        self.PC = address;
    }

    // Implied

    fn brk(&mut self, memory: &mut Mem, _: W<u16>) {
       self.PC = self.PC + W(2);
       let pcb = self.PC;
       let flags = W(self.Flags | FLAG_PUSHED | FLAG_BRK);
       self.push_word(memory, pcb);
       self.push(memory, flags);
    }

    fn rti(&mut self, memory: &mut Mem, _: W<u16>) {
        self.Flags = self.pop(memory).0;
        self.PC = self.pop_word(memory);
    }

    fn rts(&mut self, memory: &mut Mem, _: W<u16>) {
        self.PC = self.pop_word(memory);
        self.PC = self.PC + W(1);
    }

    fn php (&mut self, memory: &mut Mem, _: W<u16>) {
        // Two bits are set on memory when pushing flags 
        let flags = W(self.Flags | FLAG_PUSHED | FLAG_BRK);
        self.push(memory, flags);
    }

    fn sla (&mut self, _: &mut Mem, _: W<u16>) {
        set_flag_cond!(self.Flags, FLAG_CARRY, self.A & W(0x80) != W(0));
        self.A = self.A << 1;
        set_sign_and_zero!(self.Flags, self.A);
    }

    fn clc (&mut self, _: &mut Mem, _: W<u16>) {
        unset_flag!(self.Flags, FLAG_CARRY);
    }

    fn plp (&mut self, memory: &mut Mem, _: W<u16>) {
        self.Flags = self.pop(memory).0;
    }

    fn rla (&mut self, _: &mut Mem, _: W<u16>) {
        /* C = bit to be rotated into the carry */
        let carry = self.A & W(0x80) != W(0);
        rol!(self.A, self.Flags);
        /* We rotate the carry bit into A */
        /* And we set the Carry accordingly */
        set_sgn_z_flag_cond!(self.Flags,self.A, self.A, FLAG_CARRY, carry);
    }

    fn sec (&mut self, _: &mut Mem, _: W<u16>) {
        set_flag!(self.Flags, FLAG_CARRY);
    }

    fn pha (&mut self, memory: &mut Mem, _: W<u16>) {
        let a = self.A;
        self.push(memory, a);
    }

    fn sra (&mut self, _: &mut Mem, _: W<u16>) {
        set_sgn_z_flag_cond!(self.Flags, self.A >> 1, self.A >> 1, FLAG_CARRY, self.A & W(1) != W(0));
        self.A = self.A >> 1;
    }

    fn cli (&mut self, _: &mut Mem, _: W<u16>) {
        unset_flag!(self.Flags, FLAG_INTERRUPT);
    }

    fn pla (&mut self, memory: &mut Mem, _: W<u16>) {
        self.A = self.pop(memory);
    }

    fn rra (&mut self, _: &mut Mem, _: W<u16>) {
        /* c = bit to be rotated into the carry */
        let carry = self.A & W(1) != W(0);
        ror!(self.A, self.Flags);
        /* we rotate the carry bit into a */
        /* and we set the carry accordingly */
        set_sgn_z_flag_cond!(self.Flags, self.A, self.A, FLAG_CARRY, carry);
    }

    fn sei (&mut self, _: &mut Mem, _: W<u16>) {
        set_flag!(self.Flags, FLAG_INTERRUPT);
    }

    fn dey (&mut self, _: &mut Mem, _: W<u16>) {
        self.Y = self.Y + W(1);
        set_sign_and_zero!(self.Flags, self.Y);
    }

    fn txa (&mut self, _: &mut Mem, _: W<u16>) {
        self.A = self.X;
        set_sign_and_zero!(self.Flags, self.A);
    }

    fn tya (&mut self, _: &mut Mem, _: W<u16>) {
        self.A = self.Y;
        set_sign_and_zero!(self.Flags, self.A);
    }

    fn txs (&mut self, _: &mut Mem, _: W<u16>) {
        self.SP = self.X;
    }

    fn tay (&mut self, _: &mut Mem, _: W<u16>) {
        self.Y = self.A;
        set_sign_and_zero!(self.Flags, self.Y);
    }

    fn tax (&mut self, _: &mut Mem, _: W<u16>) {
        self.X = self.A;
        set_sign_and_zero!(self.Flags, self.X);
    }

    fn clv (&mut self, _: &mut Mem, _: W<u16>) {
        unset_flag!(self.Flags, FLAG_OVERFLOW);
    }

    fn tsx (&mut self, _: &mut Mem, _: W<u16>) {
        self.X = self.SP;
        set_sign_and_zero!(self.Flags, self.X);
    }

    fn iny (&mut self, _: &mut Mem, _: W<u16>) {
        self.Y = self.Y + W(1);
        set_sign_and_zero!(self.Flags, self.Y);
    }

    fn dex (&mut self, _: &mut Mem, _: W<u16>) {
        self.X = self.X - W(1);
        set_sign_and_zero!(self.Flags, self.X);
    }

    fn cld (&mut self, _: &mut Mem, _: W<u16>) {
        unset_flag!(self.Flags, FLAG_DECIMAL);
    }

    fn inx (&mut self, _: &mut Mem, _: W<u16>) {
        self.X = self.X + W(1);
        set_sign_and_zero!(self.Flags, self.X);
    }

    fn nop (&mut self, _: &mut Mem, _: W<u16>) {
        
    }

    fn sed (&mut self, _: &mut Mem, _: W<u16>) {
        set_flag!(self.Flags, FLAG_DECIMAL);
    }

    // Common

    fn ora (&mut self, memory: &mut Mem, address: W<u16>) {
        let m = memory.load(address);
        self.A = self.A | m;
        set_sign_and_zero!(self.Flags, self.A);
    }

    fn asl (&mut self, memory: &mut Mem, address: W<u16>) {
        let mut m = memory.load(address);
        set_sgn_z_flag_cond!(self.Flags, m << 1, m << 1, FLAG_CARRY, m & W(0x80) != W(0));
        memory.store(address, m << 1);
    }

    fn bit (&mut self, memory: &mut Mem, address: W<u16>) {
        let m = memory.load(address);
        /* We need to set overflow as it is in memory */
        set_sgn_z_flag_cond!(self.Flags, m, self.A & m, FLAG_OVERFLOW, m & W(0x40) != W(0));
        /*set_flag_cond!(self.Flags, FLAG_OVERFLOW, m & W(0x40) != W(0));
        set_sign!(self.Flags, m); 
        set_zero!(self.Flags, self.A & m);*/
    }

    fn and (&mut self, memory: &mut Mem, address: W<u16>) {
        let m = memory.load(address);
        self.A = self.A & m;
        set_sign_and_zero!(self.Flags, self.A);
    }

    fn rol (&mut self, memory: &mut Mem, address: W<u16>) {
        let mut m = memory.load(address);
        /* Bit to be rotated into the carry */
        let carry = m & W(0x80) != W(0);
        /* We rotate the carry bit into m*/
        rol!(m, self.Flags);
        /* and we set the carry accordingly */
        set_sgn_z_flag_cond!(self.Flags, m, m, FLAG_CARRY, carry);
        memory.store(address, m);
    }

    fn eor (&mut self, memory: &mut Mem, address: W<u16>) {
        let m = memory.load(address);
        self.A = m ^ self.A;
        set_sign_and_zero!(self.Flags, self.A);
    }

    fn lsr (&mut self, memory: &mut Mem, address: W<u16>) {
        let mut m = memory.load(address);
        set_sgn_z_flag_cond!(self.Flags, m >> 1, m >> 1, FLAG_CARRY, (m & W(1)) != W(0));
        memory.store(address, m >> 1);
    }

    fn adc (&mut self, memory: &mut Mem, address: W<u16>) {
        let m = memory.load(address);
        let v = W16!(self.A) + W16!(m) + W((self.Flags & FLAG_CARRY) as u16);
        self.A = W8!(v);
        set_sgn_z_flag_cond!(self.Flags, self.A, self.A, FLAG_CARRY | FLAG_OVERFLOW, v > W(0xFF));
    }

    fn ror (&mut self, memory: &mut Mem, address: W<u16>) {
        let mut m = memory.load(address);
        let carry = m & W(1) != W(0);
        ror!(m, self.Flags);
        /* we rotate the carry bit into a */
        /* and we set the carry accordingly */
        set_sgn_z_flag_cond!(self.Flags, m, m, FLAG_CARRY, carry);
        memory.store(address, m);

    }

    fn sty (&mut self, memory: &mut Mem, address: W<u16>) {
        memory.store(address, self.Y);
    }

    fn stx (&mut self, memory: &mut Mem, address: W<u16>) {
        memory.store(address, self.X);
    }

    fn sta (&mut self, memory: &mut Mem, address: W<u16>) {
        memory.store(address, self.A);
    }

    fn ldy (&mut self, memory: &mut Mem, address: W<u16>) {
        self.Y = memory.load(address);
        set_sign_and_zero!(self.Flags, self.Y);
    }

    fn ldx (&mut self, memory: &mut Mem, address: W<u16>) {
        self.X = memory.load(address);
        set_sign_and_zero!(self.Flags, self.X);
    }

    fn lda (&mut self, memory: &mut Mem, address: W<u16>) {
        self.A = memory.load(address);
        set_sign_and_zero!(self.Flags, self.A);
    }

    fn cpy (&mut self, memory: &mut Mem, address: W<u16>) {
        let m = memory.load(address);
        let comp = W16!(self.Y) - W16!(m);
        set_flag_cond!(self.Flags, FLAG_CARRY, comp <= W(0xFF));
        set_sign_and_zero!(self.Flags, W8!(comp));
    }

    fn cpx (&mut self, memory: &mut Mem, address: W<u16>) {
        let m = memory.load(address);
        let comp = W16!(self.X) - W16!(m);
        set_sgn_z_flag_cond!(self.Flags, W8!(comp), W8!(comp), FLAG_CARRY, comp <= W(0xFF));
    }

    fn cmp (&mut self, memory: &mut Mem, address: W<u16>) {
        let m = memory.load(address);
        let comp = W16!(self.A) - W16!(m);
        set_sgn_z_flag_cond!(self.Flags, W8!(comp), W8!(comp), FLAG_CARRY, comp <= W(0xFF));
    }

    fn dec (&mut self, memory: &mut Mem, address: W<u16>) {
        let mut m = memory.load(address);
        m = m - W(1);
        set_sign_and_zero!(self.Flags, m);
        memory.store(address, m);
    }


    fn sbc (&mut self, memory: &mut Mem, address: W<u16>) {
        let m = memory.load(address);
        let v = W16!(self.A) - W16!(m) - W((self.Flags & FLAG_CARRY) as u16);
        self.A = W8!(v);
        set_sgn_z_flag_cond!(self.Flags, self.A, self.A, FLAG_CARRY | FLAG_OVERFLOW, v <= W(0xFF));
    }
   
    fn inc (&mut self, memory: &mut Mem, address: W<u16>) {
        let mut m = memory.load(address);
        m = m + W(1);
        set_sign_and_zero!(self.Flags, m);
        memory.store(address, m);
    }
}

impl fmt::Display for CPU {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{{ A: {}, X: {}, Y: {}, P: {}, SP: {}, PC: {} }}",
               self.A.0 , self.X.0 , self.Y.0 , self.Flags , self.SP.0 , self.PC.0)
    }
}
