use std::fmt;
use mem::Memory as Mem;
use loadstore::LoadStore;
use std::num::Wrapping as W;
use dma::DMA;

/* Branch flag types */
const BRANCH_FLAG_CHECK : u8 = 0x20;
const BRANCH_FLAG_TABLE : [W<u8>; 4] = 
    [FLAG_SIGN, FLAG_OVERFLOW, FLAG_CARRY, FLAG_ZERO];

/* Memory */
const STACK_PAGE        : W<u16> = W(0x0100 as u16); 
const PAGE_MASK         : W<u16> = W(0xFF00 as u16);
const ADDRESS_INTERRUPT : W<u16> = W(0xFFFE as u16);
const ADDRESS_RESET     : W<u16> = W(0xFFFC as u16);

/* Flag bits */
const FLAG_CARRY        : W<u8> = W(0x01);
const FLAG_ZERO         : W<u8> = W(0x02);
const FLAG_INTERRUPT    : W<u8> = W(0x04);
const FLAG_DECIMAL      : W<u8> = W(0x08);
const FLAG_BRK          : W<u8> = W(0x10);
const FLAG_PUSHED       : W<u8> = W(0x20);
const FLAG_OVERFLOW     : W<u8> = W(0x40);
const FLAG_SIGN         : W<u8> = W(0x80);

#[derive(Default, Debug)]
pub struct Cpu {
    // Cycle count since power up
    cycles      : u64,
    regs        : Regs,
    exec        : Execution,
    dma         : DMA,
} 

impl Cpu {

    pub fn reset(&mut self, memory: &mut Mem) {
        *self = Cpu::default();
        self.regs.reset(memory);
    }

    pub fn cycle(&mut self, memory: &mut Mem) {
        // Dma takes priority
        if !self.dma.cycle(memory, self.cycles) {
            self.exec.cycle(memory, &mut self.regs);
        }
        self.cycles += 1;
    }
    
    pub fn registers(&self) -> DebugRegs { 
        self.regs.debug_regs()
    }
}

#[derive(Default)]
struct Execution {
    cycles_left     : u32,
    operation       : Operation, 
}

impl Execution {
    pub fn cycle(&mut self, memory: &mut Mem, regs: &mut Regs) {
        if self.cycles_left == 0 {
            // Get address and extra cycles from mode
            let operand = self.operation.operand;
            let addressing = self.operation.mode;
            let (address, extra) = (addressing.function)(regs, memory, operand);
            // Execute the instruction
            (self.operation.inst.function)(regs, memory, address);
            // Advance the PC
            regs.PC = regs.PC + addressing.size; 
            // Get next operation
            self.operation = Operation::from_address(memory, regs.PC);
            self.cycles_left = self.operation.inst.cycles; 
            // Add the extra cycles if needed
            if self.operation.inst.has_extra {
                self.cycles_left += extra;
            }
        }
        self.cycles_left -= 1;
    }
}

impl fmt::Debug for Execution {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{{Execution: cycles_left: {}, operation: {:?}}}",
               self.cycles_left, self.operation)
    }
}

pub struct Operation { 
    pub inst        : &'static Instruction,
    pub mode        : &'static Addressing,
    pub opcode      : u8,
    pub operand     : W<u16>,
}

impl Operation {
    pub fn from_address(memory: &mut Mem, address: W<u16>) -> Operation {
        let opcode = memory.load(address).0;
        let inst = &OPCODE_TABLE[opcode as usize];
        let mode = &ADDRESSING_TABLE[inst.mode];
        let operand : W<u16> = match mode.size {
            W(1) => W16!(memory.load(address + W(1))),
            W(2) => memory.load_word(address + W(1)),
            _    => W(0),
        };
        Operation {
            inst    : inst,
            mode    : mode,
            opcode  : opcode,
            operand : operand, 
        }
    }
}

impl fmt::Debug for Operation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{{Operation: inst: {:?}, mode: {:?}, opcode: {:02X}, operand: {:04X}}}",
               self.inst.name, self.mode.name, self.opcode, self.operand.0)
    }
}

impl Default for Operation {
    fn default() -> Operation {
        let nop_opcode : u8 = 0xEA;
        let inst = &OPCODE_TABLE[nop_opcode as usize]; 
        Operation {
            inst    : inst,
            mode    : &ADDRESSING_TABLE[inst.mode],
            opcode  : nop_opcode,
            operand : W(0),
        }
    }
}

pub struct Instruction {
    pub function    : fn(&mut Regs, &mut Mem, W<u16>), 
    pub mode        : usize, 
    pub cycles      : u32,
    pub has_extra   : bool,
    pub name        : &'static str,
}

pub struct Addressing {
    pub function    : fn(&mut Regs, &mut Mem, W<u16>) -> (W<u16>, u32),
    pub size        : W<u16>,
    pub name        : &'static str,
}

#[allow(non_snake_case)]
pub struct Regs {
    A           : W<u8>,    // Accumulator
    X           : W<u8>,    // Indexes
    Y           : W<u8>,    //
    P           : W<u8>,    // Status
    SP          : W<u8>,    // Stack pointer
    PC          : W<u16>,   // Program counter
}

impl Default for Regs {
    fn default() -> Regs {
        Regs {
            A   : W(0),
            X   : W(0),
            Y   : W(0),
            P   : W(0x34), 
            SP  : W(0xFD),
            PC  : W(0),
        }
    }
}

#[allow(non_snake_case)]
#[derive(Clone, Copy)]
pub struct DebugRegs {
    pub A   : W<u8>,    // Accumulator
    pub X   : W<u8>,    // Indexes
    pub Y   : W<u8>,    //
    pub P   : W<u8>,    // Status
    pub SP  : W<u8>,    // Stack pointer
    pub PC  : W<u16>,   // Program counter
}

// Util functions
impl Regs {

    pub fn debug_regs(&self) -> DebugRegs {
        DebugRegs {
            A   : self.A,
            X   : self.X,
            Y   : self.Y,
            P   : self.P,
            SP  : self.SP,
            PC  : self.PC,
        }
    }

    pub fn reset(&mut self, memory: &mut Mem) {
        self.PC = memory.load_word(ADDRESS_RESET); 
        self.PC = W(0xC000);
    }

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

    fn add_with_carry(&mut self, value: W<u8>) {
        let m = W16!(value);
        let a = W16!(self.A);
        let sum = a + m + W16!(self.P & FLAG_CARRY);
        set_flag_cond!(self.P, FLAG_OVERFLOW, 
                       (a ^ sum) & (m ^ sum) & W(0x80) > W(0));
        set_flag_cond!(self.P, FLAG_CARRY, sum > W(0xFF));
        self.A = W8!(sum);
        set_sign_zero!(self.P, self.A);
    } 

    fn compare(&mut self, reg: W<u8>, value: W<u8>) {
        let comp = W16!(reg) - W16!(value);
        set_sign_zero_carry_cond!(self.P, W8!(comp), comp <= W(0xFF));
    }

    fn rotate_right(&mut self, value: W<u8>) -> W<u8> {
        let carry = value & W(1) > W(0);
        let rot = value >> 1 | (self.P & FLAG_CARRY) << 7;
        set_sign_zero_carry_cond!(self.P, rot, carry);
        rot
    }

    fn rotate_left(&mut self, value: W<u8>) -> W<u8> {
        let carry = value & W(0x80) > W(0);
        let rot = value << 1 | self.P & FLAG_CARRY;
        set_sign_zero_carry_cond!(self.P, rot, carry);
        rot
    }

    fn shift_right(&mut self, value: W<u8>) -> W<u8> {
        let shift = value >> 1;
        set_sign_zero_carry_cond!(self.P, shift, value & W(1) > W(0));
        shift
    }

    fn shift_left(&mut self, value: W<u8>) -> W<u8> {
        let shift = value << 1;
        set_sign_zero_carry_cond!(self.P, shift, value & W(0x80) > W(0));
        shift
    }
}

// Addressing modes

impl Regs {

    fn imp(&mut self, _: &mut Mem, _: W<u16>) -> (W<u16>, u32) {
        (W(0), 0)
    }

    fn imm(&mut self, _: &mut Mem, _: W<u16>) -> (W<u16>, u32) {
        (self.PC + W(1), 0)
    }

    fn ind(&mut self, memory: &mut Mem, operand: W<u16>) -> (W<u16>, u32) {
        (memory.load_word_page_wrap(operand), 0)
    }

    fn idx(&mut self, memory: &mut Mem, operand: W<u16>) -> (W<u16>, u32) {
        (memory.load_word_page_wrap(operand + W16!(self.X)), 0)
    }

    fn idy(&mut self, memory: &mut Mem, operand: W<u16>) -> (W<u16>, u32) {
        let dest = memory.load_word_page_wrap(operand) + W16!(self.Y);
        (dest, (W8!(dest) < self.Y) as u32)
    }

    fn zpg(&mut self, memory: &mut Mem, operand: W<u16>) -> (W<u16>, u32) {
        (operand, 0)
    }

    fn zpx(&mut self, memory: &mut Mem, operand: W<u16>) -> (W<u16>, u32) {
        (W16!(W8!(operand) + self.X), 0)
    }

    fn zpy(&mut self, memory: &mut Mem, operand: W<u16>) -> (W<u16>, u32) {
        (W16!(W8!(operand) + self.Y), 0)
    }

    fn abs(&mut self, memory: &mut Mem, operand: W<u16>) -> (W<u16>, u32) {
        (operand, 0)
    }

    fn abx(&mut self, memory: &mut Mem, operand: W<u16>) -> (W<u16>, u32) {
        let address = operand + W16!(self.X);
        (address, (W8!(address) < self.X) as u32)
    }

    fn aby(&mut self, memory: &mut Mem, operand: W<u16>) -> (W<u16>, u32) {
        let address = operand + W16!(self.Y);
        (address, (W8!(address) < self.Y) as u32)
    }

    fn rel(&mut self, memory: &mut Mem, operand: W<u16>) -> (W<u16>, u32) {
        let opcode = memory.load(self.PC).0;
        let index = (opcode >> 6) as usize;
        let check = is_flag_set!(opcode, BRANCH_FLAG_CHECK);
        let next_opcode = self.PC + W(2);
        if is_bit_set!(self.P, BRANCH_FLAG_TABLE[index]) != check {
            (next_opcode, 0)
        } else {
            // Branch taken
            let offset = W(operand.0 as i8 as u16);  
            let branch = next_opcode + offset;
            let crossed = (branch & PAGE_MASK) != (next_opcode & PAGE_MASK); 
            // Additional cycle if branch taken and page boundary crossed
            (branch, 1 + crossed as u32) 
        }
    }
}

// Instructions

impl Regs {   
    
    // Jump

    fn jsr(&mut self, memory: &mut Mem, address: W<u16>) {
        // Load destination address and push return address
        let ret = self.PC - W(1);
        self.push_word(memory, ret);
        self.PC = address;
    }

    fn jmp(&mut self, _: &mut Mem, address: W<u16>) {
        self.PC = address;
    }

    // Implied special

    fn brk(&mut self, memory: &mut Mem, _: W<u16>) {
       // Two bits are set on memory when pushing flags 
       let flags = self.P | FLAG_PUSHED | FLAG_BRK;
       let pc = self.PC + W(1);
       self.push_word(memory, pc);
       self.push(memory, flags);
       set_flag!(self.P, FLAG_INTERRUPT);
       self.PC = memory.load_word(ADDRESS_INTERRUPT);
    }

    fn rti(&mut self, memory: &mut Mem, _: W<u16>) {
        // Ignore the two bits not present
        self.P = self.pop(memory) & !(FLAG_PUSHED | FLAG_BRK);
        self.PC = self.pop_word(memory);
    }

    fn rts(&mut self, memory: &mut Mem, _: W<u16>) {
        self.PC = self.pop_word(memory) + W(1);
    }

    // Implied

    fn php(&mut self, memory: &mut Mem, _: W<u16>) {
        // Two bits are set on memory when pushing flags 
        let flags = self.P | FLAG_PUSHED | FLAG_BRK;
        self.push(memory, flags);
    }

    fn sal(&mut self, _: &mut Mem, _: W<u16>) {
        let a = self.A;
        self.A = self.shift_left(a);
    }

    fn clc(&mut self, _: &mut Mem, _: W<u16>) {
        unset_flag!(self.P, FLAG_CARRY);
    }

    fn plp(&mut self, memory: &mut Mem, _: W<u16>) {
        // Ignore the two bits not present
        self.P = self.pop(memory) & !(FLAG_PUSHED | FLAG_BRK);
    }

    fn ral(&mut self, _: &mut Mem, _: W<u16>) {
        let a = self.A;
        self.A = self.rotate_left(a);
    }

    fn sec(&mut self, _: &mut Mem, _: W<u16>) {
        set_flag!(self.P, FLAG_CARRY);
    }

    fn pha(&mut self, memory: &mut Mem, _: W<u16>) {
        let a = self.A;
        self.push(memory, a);
    }

    fn sar(&mut self, _: &mut Mem, _: W<u16>) {
        let a = self.A;
        self.A = self.shift_right(a);
    }

    fn cli(&mut self, _: &mut Mem, _: W<u16>) {
        unset_flag!(self.P, FLAG_INTERRUPT);
    }

    fn pla(&mut self, memory: &mut Mem, _: W<u16>) {
        self.A = self.pop(memory);
        set_sign_zero!(self.P, self.A);
    }

    fn rar(&mut self, _: &mut Mem, _: W<u16>) {
        let a = self.A;
        self.A = self.rotate_right(a);
    }

    fn sei(&mut self, _: &mut Mem, _: W<u16>) {
        set_flag!(self.P, FLAG_INTERRUPT);
    }

    fn dey(&mut self, _: &mut Mem, _: W<u16>) {
        self.Y -= W(1);
        set_sign_zero!(self.P, self.Y);
    }

    fn txa(&mut self, _: &mut Mem, _: W<u16>) {
        self.A = self.X;
        set_sign_zero!(self.P, self.A);
    }

    fn tya(&mut self, _: &mut Mem, _: W<u16>) {
        self.A = self.Y;
        set_sign_zero!(self.P, self.A);
    }

    fn txs(&mut self, _: &mut Mem, _: W<u16>) {
        self.SP = self.X;
    }

    fn tay(&mut self, _: &mut Mem, _: W<u16>) {
        self.Y = self.A;
        set_sign_zero!(self.P, self.Y);
    }

    fn tax(&mut self, _: &mut Mem, _: W<u16>) {
        self.X = self.A;
        set_sign_zero!(self.P, self.X);
    }

    fn clv(&mut self, _: &mut Mem, _: W<u16>) {
        unset_flag!(self.P, FLAG_OVERFLOW);
    }

    fn tsx(&mut self, _: &mut Mem, _: W<u16>) {
        self.X = self.SP;
        set_sign_zero!(self.P, self.X);
    }

    fn iny(&mut self, _: &mut Mem, _: W<u16>) {
        self.Y += W(1);
        set_sign_zero!(self.P, self.Y);
    }

    fn dex(&mut self, _: &mut Mem, _: W<u16>) {
        self.X -= W(1);
        set_sign_zero!(self.P, self.X);
    }

    fn cld(&mut self, _: &mut Mem, _: W<u16>) {
        unset_flag!(self.P, FLAG_DECIMAL);
    }

    fn inx(&mut self, _: &mut Mem, _: W<u16>) {
        self.X += W(1);
        set_sign_zero!(self.P, self.X);
    }

    fn nop(&mut self, _: &mut Mem, _: W<u16>) {}

    fn sed(&mut self, _: &mut Mem, _: W<u16>) {
        set_flag!(self.P, FLAG_DECIMAL);
    }

    // Common

    fn ora(&mut self, memory: &mut Mem, address: W<u16>) {
        self.A |= memory.load(address);
        set_sign_zero!(self.P, self.A);
    }

    fn asl(&mut self, memory: &mut Mem, address: W<u16>) {
        let m = self.shift_left(memory.load(address));
        memory.store(address, m);
    }

    fn bit(&mut self, memory: &mut Mem, address: W<u16>) {
        let m = memory.load(address);
        copy_bits!(self.P, m, FLAG_OVERFLOW);
        set_sign!(self.P, m); 
        set_zero!(self.P, self.A & m);
    }

    fn and(&mut self, memory: &mut Mem, address: W<u16>) {
        self.A &= memory.load(address);
        set_sign_zero!(self.P, self.A);
    }

    fn rol(&mut self, memory: &mut Mem, address: W<u16>) {
        let m = self.rotate_left(memory.load(address));
        memory.store(address, m);
    }

    fn eor(&mut self, memory: &mut Mem, address: W<u16>) {
        self.A ^= memory.load(address);
        set_sign_zero!(self.P, self.A);
    }

    fn lsr(&mut self, memory: &mut Mem, address: W<u16>) {
        let m = self.shift_right(memory.load(address));
        memory.store(address, m);
    }

    fn adc(&mut self, memory: &mut Mem, address: W<u16>) {
        self.add_with_carry(memory.load(address));
    }

    fn ror(&mut self, memory: &mut Mem, address: W<u16>) {
        let m = self.rotate_right(memory.load(address));
        memory.store(address, m);
    }

    fn sty(&mut self, memory: &mut Mem, address: W<u16>) {
        memory.store(address, self.Y);
    }

    fn stx(&mut self, memory: &mut Mem, address: W<u16>) {
        memory.store(address, self.X);
    }

    fn sta(&mut self, memory: &mut Mem, address: W<u16>) {
        memory.store(address, self.A);
    }

    fn ldy(&mut self, memory: &mut Mem, address: W<u16>) {
        self.Y = memory.load(address);
        set_sign_zero!(self.P, self.Y);
    }

    fn ldx(&mut self, memory: &mut Mem, address: W<u16>) {
        self.X = memory.load(address);
        set_sign_zero!(self.P, self.X);
    }

    fn lda(&mut self, memory: &mut Mem, address: W<u16>) {
        self.A = memory.load(address);
        set_sign_zero!(self.P, self.A);
    }

    fn cpy(&mut self, memory: &mut Mem, address: W<u16>) {
        let y = self.Y;
        self.compare(y, memory.load(address));
    }

    fn cpx(&mut self, memory: &mut Mem, address: W<u16>) {
        let x = self.X;
        self.compare(x, memory.load(address));
    }

    fn cmp(&mut self, memory: &mut Mem, address: W<u16>) {
        let a = self.A;
        self.compare(a, memory.load(address));
    }

    fn dec(&mut self, memory: &mut Mem, address: W<u16>) {
        let m = memory.load(address) - W(1);
        set_sign_zero!(self.P, m);
        memory.store(address, m);
    }

    fn sbc(&mut self, memory: &mut Mem, address: W<u16>) {
        self.add_with_carry(!memory.load(address));
    }
   
    fn inc(&mut self, memory: &mut Mem, address: W<u16>) {
        let m = memory.load(address) + W(1);
        set_sign_zero!(self.P, m);
        memory.store(address, m);
    }
}

// Unofficial Instructions

impl Regs {

    fn lax(&mut self, memory: &mut Mem, address: W<u16>) {
        let m = memory.load(address);
        self.A = m;
        self.X = m;
        set_sign_zero!(self.P, m);
    }

    fn sax(&mut self, memory: &mut Mem, address: W<u16>) {
        memory.store(address, self.A & self.X);
    }

    fn dcp(&mut self, memory: &mut Mem, address: W<u16>) {
        let m = memory.load(address) - W(1);
        let a = self.A;
        memory.store(address, m);
        self.compare(a, m);
    }

    fn isc(&mut self, memory: &mut Mem, address: W<u16>) {
        let m = memory.load(address) + W(1);
        memory.store(address, m);
        self.add_with_carry(!m);
    }

    fn slo(&mut self, memory: &mut Mem, address: W<u16>) {
        let shift = self.shift_left(memory.load(address));
        memory.store(address, shift);
        self.A |= shift;
        set_sign_zero!(self.P, self.A);
    }

    fn rla(&mut self, memory: &mut Mem, address: W<u16>) {
        let rot = self.rotate_left(memory.load(address)); 
        memory.store(address, rot);
        self.A &= rot;
        set_sign_zero!(self.P, self.A);
    }

    fn sre(&mut self, memory: &mut Mem, address: W<u16>) {
        let shift = self.shift_right(memory.load(address));
        memory.store(address, shift);
        self.A ^= shift;
        set_sign_zero!(self.P, self.A);
    }

    fn rra(&mut self, memory: &mut Mem, address: W<u16>) {
        let rot = self.rotate_right(memory.load(address)); 
        memory.store(address, rot);
        self.add_with_carry(rot);
    }
}

impl fmt::Debug for Regs {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{{Regs: A: {:02X}, X: {:02X}, Y: {:02X}, P: {:02X}, SP: {:02X}, PC: {:04X} }}",
               self.A.0 , self.X.0 , self.Y.0 , self.P.0 , self.SP.0 , self.PC.0)
    }
}


const ADDRESSING_TABLE : &'static [Addressing; 12] = &[
    addressing!(imp, 1), addressing!(imm, 2), addressing!(rel, 2), 
    addressing!(zpx, 2), addressing!(zpy, 2), addressing!(zpg, 2),
    addressing!(idx, 2), addressing!(idy, 2), addressing!(ind, 3),
    addressing!(abx, 3), addressing!(aby, 3), addressing!(abs, 3),
];

const IMP : usize = 0;
const IMM : usize = 1;
const REL : usize = 2;
const ZPX : usize = 3;
const ZPY : usize = 4;
const ZPG : usize = 5;
const IDX : usize = 6;
const IDY : usize = 7;
const IND : usize = 8;
const ABX : usize = 9;
const ABY : usize = 10;
const ABS : usize = 11;

/* WARNING: Branch instructions are replaced with jumps */
const OPCODE_TABLE : &'static [Instruction; 256] = &[    
    // 0x00
    iz!(IMP, brk, 7, noi), iz!(IDX, ora, 6, noi), iz!(IMP, nop, 2, noi), iz!(IDX, slo, 8, noi),
    iz!(ZPG, nop, 3, noi), iz!(ZPG, ora, 3, noi), iz!(ZPG, asl, 5, noi), iz!(ZPG, slo, 5, noi), 
    iz!(IMP, php, 3, noi), iz!(IMM, ora, 2, noi), iz!(IMP, sal, 2, noi), iz!(IMP, nop, 2, noi), 
    iz!(ABS, nop, 4, noi), iz!(ABS, ora, 4, noi), iz!(ABS, asl, 6, noi), iz!(ABS, slo, 6, noi),
    // 0x10                                                                
    jj!(REL, jmp, 2, bpl), ix!(IDY, ora, 5, noi), iz!(IMP, nop, 2, noi), iz!(IDY, slo, 8, noi),
    iz!(ZPX, nop, 4, noi), iz!(ZPX, ora, 4, noi), iz!(ZPX, asl, 6, noi), iz!(ZPX, slo, 6, noi),
    iz!(IMP, clc, 2, noi), ix!(ABY, ora, 4, noi), iz!(IMP, nop, 2, noi), iz!(ABY, slo, 7, noi),
    ix!(ABX, nop, 4, noi), ix!(ABX, ora, 4, noi), iz!(ABX, asl, 7, noi), iz!(ABX, slo, 7, noi),
    // 0x20                                                                
    iz!(ABS, jsr, 6, noi), iz!(IDX, and, 6, noi), iz!(IMP, nop, 2, noi), iz!(IDX, rla, 8, noi),
    iz!(ZPG, bit, 3, noi), iz!(ZPG, and, 3, noi), iz!(ZPG, rol, 5, noi), iz!(ZPG, rla, 5, noi),
    iz!(IMP, plp, 4, noi), iz!(IMM, and, 2, noi), iz!(IMP, ral, 2, noi), iz!(IMP, nop, 2, noi),
    iz!(ABS, bit, 4, noi), iz!(ABS, and, 4, noi), iz!(ABS, rol, 6, noi), iz!(ABS, rla, 6, noi),
    // 0x30                                                                
    jj!(REL, jmp, 2, bmi), ix!(IDY, and, 5, noi), iz!(IMP, nop, 2, noi), iz!(IDY, rla, 8, noi),
    iz!(ZPX, nop, 4, noi), iz!(ZPX, and, 4, noi), iz!(ZPX, rol, 6, noi), iz!(ZPX, rla, 6, noi),
    iz!(IMP, sec, 2, noi), ix!(ABY, and, 4, noi), iz!(IMP, nop, 2, noi), iz!(ABY, rla, 7, noi),
    ix!(ABX, nop, 4, noi), ix!(ABX, and, 4, noi), iz!(ABX, rol, 7, noi), iz!(ABX, rla, 7, noi),
    // 0x40                                                                
    iz!(IMP, rti, 6, noi), iz!(IDX, eor, 6, noi), iz!(IMP, nop, 2, noi), iz!(IDX, sre, 8, noi),
    iz!(ZPG, nop, 3, noi), iz!(ZPG, eor, 3, noi), iz!(ZPG, lsr, 5, noi), iz!(ZPG, sre, 5, noi),
    iz!(IMP, pha, 3, noi), iz!(IMM, eor, 2, noi), iz!(IMP, sar, 2, noi), iz!(IMP, nop, 2, noi),
    iz!(ABS, jmp, 3, noi), iz!(ABS, eor, 4, noi), iz!(ABS, lsr, 6, noi), iz!(ABS, sre, 6, noi),
    // 0x50                                                                
    jj!(REL, jmp, 2, bvc), ix!(IDY, eor, 5, noi), iz!(IMP, nop, 2, noi), iz!(IDY, sre, 8, noi),
    iz!(ZPX, nop, 4, noi), iz!(ZPX, eor, 4, noi), iz!(ZPX, lsr, 6, noi), iz!(ZPX, sre, 6, noi),
    iz!(IMP, cli, 2, noi), ix!(ABY, eor, 4, noi), iz!(IMP, nop, 2, noi), iz!(ABY, sre, 7, noi), 
    ix!(ABX, nop, 4, noi), ix!(ABX, eor, 4, noi), iz!(ABX, lsr, 7, noi), iz!(ABX, sre, 7, noi),
    // 0x60                                                                
    iz!(IMP, rts, 6, noi), iz!(IDX, adc, 6, noi), iz!(IMP, nop, 2, noi), iz!(IDX, rra, 8, noi),
    iz!(ZPG, nop, 3, noi), iz!(ZPG, adc, 3, noi), iz!(ZPG, ror, 5, noi), iz!(ZPG, rra, 5, noi),
    iz!(IMP, pla, 4, noi), iz!(IMM, adc, 2, noi), iz!(IMP, rar, 2, noi), iz!(IMP, nop, 2, noi),
    iz!(IND, jmp, 5, noi), iz!(ABS, adc, 4, noi), iz!(ABS, ror, 6, noi), iz!(ABS, rra, 6, noi),
    // 0x70                                                                
    jj!(REL, jmp, 2, bvs), ix!(IDY, adc, 5, noi), iz!(IMP, nop, 2, noi), iz!(IDY, rra, 8, noi),
    iz!(ZPX, nop, 4, noi), iz!(ZPX, adc, 4, noi), iz!(ZPX, ror, 6, noi), iz!(ZPX, rra, 6, noi),
    iz!(IMP, sei, 2, noi), ix!(ABY, adc, 4, noi), iz!(IMP, nop, 2, noi), iz!(ABY, rra, 7, noi), 
    ix!(ABX, nop, 4, noi), ix!(ABX, adc, 4, noi), iz!(ABX, ror, 7, noi), iz!(ABX, rra, 7, noi),
    // 0x80                                                                
    iz!(IMM, nop, 2, noi), iz!(IDX, sta, 6, noi), iz!(IMM, nop, 2, noi), iz!(IDX, sax, 6, noi),
    iz!(ZPG, sty, 3, noi), iz!(ZPG, sta, 3, noi), iz!(ZPG, stx, 3, noi), iz!(ZPG, sax, 3, noi),
    iz!(IMP, dey, 2, noi), iz!(IMM, nop, 2, noi), iz!(IMP, txa, 2, noi), iz!(IMP, nop, 2, noi),
    iz!(ABS, sty, 4, noi), iz!(ABS, sta, 4, noi), iz!(ABS, stx, 4, noi), iz!(ABS, sax, 4, noi),
    // 0x90                                                                
    jj!(REL, jmp, 2, bcc), iz!(IDY, sta, 6, noi), iz!(IMP, nop, 2, noi), iz!(IMP, nop, 2, noi), 
    iz!(ZPX, sty, 4, noi), iz!(ZPX, sta, 4, noi), iz!(ZPY, stx, 4, noi), iz!(ZPY, sax, 4, noi),
    iz!(IMP, tya, 2, noi), iz!(ABY, sta, 5, noi), iz!(IMP, txs, 2, noi), iz!(IMP, nop, 2, noi), 
    iz!(IMP, nop, 2, noi), iz!(ABX, sta, 5, noi), iz!(IMP, nop, 2, noi), iz!(IMP, nop, 2, noi),
    // 0xA0                                                                
    iz!(IMM, ldy, 2, noi), iz!(IDX, lda, 6, noi), iz!(IMM, ldx, 2, noi), iz!(IDX, lax, 6, noi),
    iz!(ZPG, ldy, 3, noi), iz!(ZPG, lda, 3, noi), iz!(ZPG, ldx, 3, noi), iz!(ZPG, lax, 3, noi),
    iz!(IMP, tay, 2, noi), iz!(IMM, lda, 2, noi), iz!(IMP, tax, 2, noi), iz!(IMM, lax, 2, noi),
    iz!(ABS, ldy, 4, noi), iz!(ABS, lda, 4, noi), iz!(ABS, ldx, 4, noi), iz!(ABS, lax, 4, noi),
    // 0xB0                                                                
    jj!(REL, jmp, 2, bcs), ix!(IDY, lda, 5, noi), iz!(IMP, nop, 2, noi), ix!(IDY, lax, 5, noi),
    iz!(ZPX, ldy, 4, noi), iz!(ZPX, lda, 4, noi), iz!(ZPY, ldx, 4, noi), iz!(ZPY, lax, 4, noi),
    iz!(IMP, clv, 2, noi), ix!(ABY, lda, 4, noi), iz!(IMP, tsx, 2, noi), iz!(IMP, nop, 2, noi),
    ix!(ABX, ldy, 4, noi), ix!(ABX, lda, 4, noi), ix!(ABY, ldx, 4, noi), ix!(ABY, lax, 4, noi),
    // 0xC0                                                                
    iz!(IMM, cpy, 2, noi), iz!(IDX, cmp, 6, noi), iz!(IMM, nop, 2, noi), iz!(IDX, dcp, 8, noi), 
    iz!(ZPG, cpy, 3, noi), iz!(ZPG, cmp, 3, noi), iz!(ZPG, dec, 5, noi), iz!(ZPG, dcp, 5, noi),
    iz!(IMP, iny, 2, noi), iz!(IMM, cmp, 2, noi), iz!(IMP, dex, 2, noi), iz!(IMP, nop, 2, noi),
    iz!(ABS, cpy, 4, noi), iz!(ABS, cmp, 4, noi), iz!(ABS, dec, 6, noi), iz!(ABS, dcp, 6, noi),
    // 0xD0                                                                
    jj!(REL, jmp, 2, bne), ix!(IDY, cmp, 5, noi), iz!(IMP, nop, 2, noi), iz!(IDY, dcp, 8, noi),
    iz!(ZPX, nop, 4, noi), iz!(ZPX, cmp, 4, noi), iz!(ZPX, dec, 6, noi), iz!(ZPX, dcp, 6, noi),
    iz!(IMP, cld, 2, noi), ix!(ABY, cmp, 4, noi), iz!(IMP, nop, 2, noi), iz!(ABY, dcp, 7, noi),
    ix!(ABX, nop, 4, noi), ix!(ABX, cmp, 4, noi), iz!(ABX, dec, 7, noi), iz!(ABX, dcp, 7, noi),
    // 0xE0                                                                
    iz!(IMM, cpx, 2, noi), iz!(IDX, sbc, 6, noi), iz!(IMM, nop, 2, noi), iz!(IDX, isc, 8, noi),
    iz!(ZPG, cpx, 3, noi), iz!(ZPG, sbc, 3, noi), iz!(ZPG, inc, 5, noi), iz!(ZPG, isc, 5, noi),
    iz!(IMP, inx, 2, noi), iz!(IMM, sbc, 2, noi), iz!(IMP, nop, 2, noi), iz!(IMM, sbc, 2, noi),
    iz!(ABS, cpx, 4, noi), iz!(ABS, sbc, 4, noi), iz!(ABS, inc, 6, noi), iz!(ABS, isc, 6, noi),
    // 0xF0                                                                
    jj!(REL, jmp, 2, beq), ix!(IDY, sbc, 5, noi), iz!(IMP, nop, 2, noi), iz!(IDY, isc, 8, noi),
    iz!(ZPX, nop, 4, noi), iz!(ZPX, sbc, 4, noi), iz!(ZPX, inc, 6, noi), iz!(ZPX, isc, 6, noi),
    iz!(IMP, sed, 2, noi), ix!(ABY, sbc, 4, noi), iz!(IMP, nop, 2, noi), iz!(ABY, isc, 7, noi), 
    ix!(ABX, nop, 4, noi), ix!(ABX, sbc, 4, noi), iz!(ABX, inc, 7, noi), iz!(ABX, isc, 7, noi),
];
