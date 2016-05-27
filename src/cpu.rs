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

const OPCODE_NOP        : u8 = 0xEA;

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
    
    pub fn registers(&self) -> Regs { 
        self.regs
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
            let addressing = self.operation.inst.mode;
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
    pub opcode      : u8,
    pub operand     : W<u16>,
}

impl Operation {
    pub fn from_address(memory: &mut Mem, address: W<u16>) -> Operation {
        let opcode = memory.load(address).0;
        let inst = &OPCODE_TABLE[opcode as usize];
        let operand : W<u16> = match inst.mode.size {
            W(1) => W16!(memory.load(address + W(1))),
            W(2) => memory.load_word(address + W(1)),
            _    => W(0),
        };
        Operation {
            inst    : inst,
            opcode  : opcode,
            operand : operand, 
        }
    }
}

impl fmt::Debug for Operation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{{Operation: inst: {:?}, opcode: {:02X}, operand: {:04X}}}",
               self.inst.name, self.opcode, self.operand.0)
    }
}

impl Default for Operation {
    fn default() -> Operation {
        Operation {
            inst    : &OPCODE_TABLE[OPCODE_NOP as usize],
            opcode  : OPCODE_NOP,
            operand : W(0),
        }
    }
}

pub struct Instruction {
    pub function    : fn(&mut Regs, &mut Mem, W<u16>),
    pub mode        : &'static Addressing, 
    pub cycles      : u32,
    pub has_extra   : bool,
    pub name        : &'static str,
}

pub struct Addressing {
    pub function    : fn(&mut Regs, &mut Mem, W<u16>) -> (W<u16>, u32),
    pub size        : W<u16>,
    pub name        : &'static str,
}

#[derive(Clone, Copy)]
pub struct Regs {
    pub A           : W<u8>,    // Accumulator
    pub X           : W<u8>,    // Indexes
    pub Y           : W<u8>,    //
    pub P           : W<u8>,    // Status
    pub SP          : W<u8>,    // Stack pointer
    pub PC          : W<u16>,   // Program counter
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

// Util functions
impl Regs {

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

    fn zpg(&mut self, _: &mut Mem, operand: W<u16>) -> (W<u16>, u32) {
        (operand, 0)
    }

    fn zpx(&mut self, _: &mut Mem, operand: W<u16>) -> (W<u16>, u32) {
        (W16!(W8!(operand) + self.X), 0)
    }

    fn zpy(&mut self, _: &mut Mem, operand: W<u16>) -> (W<u16>, u32) {
        (W16!(W8!(operand) + self.Y), 0)
    }

    fn abs(&mut self, _: &mut Mem, operand: W<u16>) -> (W<u16>, u32) {
        (operand, 0)
    }

    fn abx(&mut self, _: &mut Mem, operand: W<u16>) -> (W<u16>, u32) {
        let address = operand + W16!(self.X);
        (address, (W8!(address) < self.X) as u32)
    }

    fn aby(&mut self, _: &mut Mem, operand: W<u16>) -> (W<u16>, u32) {
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

    // Branch

    fn bpl(&mut self, _: &mut Mem, address: W<u16>) {
        self.PC = address;
    }

    fn bmi(&mut self, _: &mut Mem, address: W<u16>) {
        self.PC = address;
    }

    fn bvc(&mut self, _: &mut Mem, address: W<u16>) {
        self.PC = address;
    }

    fn bvs(&mut self, _: &mut Mem, address: W<u16>) {
        self.PC = address;
    }

    fn bcc(&mut self, _: &mut Mem, address: W<u16>) {
        self.PC = address;
    }

    fn bcs(&mut self, _: &mut Mem, address: W<u16>) {
        self.PC = address;
    }

    fn bne(&mut self, _: &mut Mem, address: W<u16>) {
        self.PC = address;
    }

    fn beq(&mut self, _: &mut Mem, address: W<u16>) {
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

macro_rules! addressing {
    ($addr:ident, $size:expr) => {
        Addressing {
            function    : Regs::$addr,
            size        : W($size),
            name        : stringify!($addr),
        }
    }
}

macro_rules! inst {
    ($addr:expr, $oper:ident, $cycles:expr, $extra:expr) => (
        Instruction {
            function    : Regs::$oper,
            mode        : $addr,
            cycles      : $cycles,
            has_extra   : $extra,
            name        : stringify!($oper),
        }           
    )
}

// Has zero cycle penalty
macro_rules! iz {
    ($addr:ident, $oper:ident, $cycles:expr) =>
        (inst!($addr, $oper, $cycles, false))
}

// Has extra cycle penalty
macro_rules! ix {
    ($addr:ident, $oper:ident, $cycles:expr) =>
        (inst!($addr, $oper, $cycles, true))
}

const IMP : &'static Addressing = &addressing!(imp, 1);
const IMM : &'static Addressing = &addressing!(imm, 2);
const REL : &'static Addressing = &addressing!(rel, 2);
const ZPX : &'static Addressing = &addressing!(zpx, 2);
const ZPY : &'static Addressing = &addressing!(zpy, 2);
const ZPG : &'static Addressing = &addressing!(zpg, 2);
const IDX : &'static Addressing = &addressing!(idx, 2);
const IDY : &'static Addressing = &addressing!(idy, 2);
const IND : &'static Addressing = &addressing!(ind, 2);
const ABX : &'static Addressing = &addressing!(abx, 2);
const ABY : &'static Addressing = &addressing!(aby, 2);
const ABS : &'static Addressing = &addressing!(abs, 2);

/* WARNING: Branch instructions are replaced with jumps */
const OPCODE_TABLE : &'static [Instruction; 256] = &[    
    // 0x00
    iz!(IMP, brk, 7), iz!(IDX, ora, 6), iz!(IMP, nop, 2), iz!(IDX, slo, 8),
    iz!(ZPG, nop, 3), iz!(ZPG, ora, 3), iz!(ZPG, asl, 5), iz!(ZPG, slo, 5), 
    iz!(IMP, php, 3), iz!(IMM, ora, 2), iz!(IMP, sal, 2), iz!(IMP, nop, 2), 
    iz!(ABS, nop, 4), iz!(ABS, ora, 4), iz!(ABS, asl, 6), iz!(ABS, slo, 6),
    // 0x10                                                                
    ix!(REL, bpl, 2), ix!(IDY, ora, 5), iz!(IMP, nop, 2), iz!(IDY, slo, 8),
    iz!(ZPX, nop, 4), iz!(ZPX, ora, 4), iz!(ZPX, asl, 6), iz!(ZPX, slo, 6),
    iz!(IMP, clc, 2), ix!(ABY, ora, 4), iz!(IMP, nop, 2), iz!(ABY, slo, 7),
    ix!(ABX, nop, 4), ix!(ABX, ora, 4), iz!(ABX, asl, 7), iz!(ABX, slo, 7),
    // 0x20                                                                
    iz!(ABS, jsr, 6), iz!(IDX, and, 6), iz!(IMP, nop, 2), iz!(IDX, rla, 8),
    iz!(ZPG, bit, 3), iz!(ZPG, and, 3), iz!(ZPG, rol, 5), iz!(ZPG, rla, 5),
    iz!(IMP, plp, 4), iz!(IMM, and, 2), iz!(IMP, ral, 2), iz!(IMP, nop, 2),
    iz!(ABS, bit, 4), iz!(ABS, and, 4), iz!(ABS, rol, 6), iz!(ABS, rla, 6),
    // 0x30                                                                
    ix!(REL, bmi, 2), ix!(IDY, and, 5), iz!(IMP, nop, 2), iz!(IDY, rla, 8),
    iz!(ZPX, nop, 4), iz!(ZPX, and, 4), iz!(ZPX, rol, 6), iz!(ZPX, rla, 6),
    iz!(IMP, sec, 2), ix!(ABY, and, 4), iz!(IMP, nop, 2), iz!(ABY, rla, 7),
    ix!(ABX, nop, 4), ix!(ABX, and, 4), iz!(ABX, rol, 7), iz!(ABX, rla, 7),
    // 0x40                                                                
    iz!(IMP, rti, 6), iz!(IDX, eor, 6), iz!(IMP, nop, 2), iz!(IDX, sre, 8),
    iz!(ZPG, nop, 3), iz!(ZPG, eor, 3), iz!(ZPG, lsr, 5), iz!(ZPG, sre, 5),
    iz!(IMP, pha, 3), iz!(IMM, eor, 2), iz!(IMP, sar, 2), iz!(IMP, nop, 2),
    iz!(ABS, jmp, 3), iz!(ABS, eor, 4), iz!(ABS, lsr, 6), iz!(ABS, sre, 6),
    // 0x50                                                                
    ix!(REL, bvc, 2), ix!(IDY, eor, 5), iz!(IMP, nop, 2), iz!(IDY, sre, 8),
    iz!(ZPX, nop, 4), iz!(ZPX, eor, 4), iz!(ZPX, lsr, 6), iz!(ZPX, sre, 6),
    iz!(IMP, cli, 2), ix!(ABY, eor, 4), iz!(IMP, nop, 2), iz!(ABY, sre, 7), 
    ix!(ABX, nop, 4), ix!(ABX, eor, 4), iz!(ABX, lsr, 7), iz!(ABX, sre, 7),
    // 0x60                                                                
    iz!(IMP, rts, 6), iz!(IDX, adc, 6), iz!(IMP, nop, 2), iz!(IDX, rra, 8),
    iz!(ZPG, nop, 3), iz!(ZPG, adc, 3), iz!(ZPG, ror, 5), iz!(ZPG, rra, 5),
    iz!(IMP, pla, 4), iz!(IMM, adc, 2), iz!(IMP, rar, 2), iz!(IMP, nop, 2),
    iz!(IND, jmp, 5), iz!(ABS, adc, 4), iz!(ABS, ror, 6), iz!(ABS, rra, 6),
    // 0x70                                                                
    ix!(REL, bvs, 2), ix!(IDY, adc, 5), iz!(IMP, nop, 2), iz!(IDY, rra, 8),
    iz!(ZPX, nop, 4), iz!(ZPX, adc, 4), iz!(ZPX, ror, 6), iz!(ZPX, rra, 6),
    iz!(IMP, sei, 2), ix!(ABY, adc, 4), iz!(IMP, nop, 2), iz!(ABY, rra, 7), 
    ix!(ABX, nop, 4), ix!(ABX, adc, 4), iz!(ABX, ror, 7), iz!(ABX, rra, 7),
    // 0x80                                                                
    iz!(IMM, nop, 2), iz!(IDX, sta, 6), iz!(IMM, nop, 2), iz!(IDX, sax, 6),
    iz!(ZPG, sty, 3), iz!(ZPG, sta, 3), iz!(ZPG, stx, 3), iz!(ZPG, sax, 3),
    iz!(IMP, dey, 2), iz!(IMM, nop, 2), iz!(IMP, txa, 2), iz!(IMP, nop, 2),
    iz!(ABS, sty, 4), iz!(ABS, sta, 4), iz!(ABS, stx, 4), iz!(ABS, sax, 4),
    // 0x90                                                                
    ix!(REL, bcc, 2), iz!(IDY, sta, 6), iz!(IMP, nop, 2), iz!(IMP, nop, 2), 
    iz!(ZPX, sty, 4), iz!(ZPX, sta, 4), iz!(ZPY, stx, 4), iz!(ZPY, sax, 4),
    iz!(IMP, tya, 2), iz!(ABY, sta, 5), iz!(IMP, txs, 2), iz!(IMP, nop, 2), 
    iz!(IMP, nop, 2), iz!(ABX, sta, 5), iz!(IMP, nop, 2), iz!(IMP, nop, 2),
    // 0xA0                                                                
    iz!(IMM, ldy, 2), iz!(IDX, lda, 6), iz!(IMM, ldx, 2), iz!(IDX, lax, 6),
    iz!(ZPG, ldy, 3), iz!(ZPG, lda, 3), iz!(ZPG, ldx, 3), iz!(ZPG, lax, 3),
    iz!(IMP, tay, 2), iz!(IMM, lda, 2), iz!(IMP, tax, 2), iz!(IMM, lax, 2),
    iz!(ABS, ldy, 4), iz!(ABS, lda, 4), iz!(ABS, ldx, 4), iz!(ABS, lax, 4),
    // 0xB0                                                                
    ix!(REL, bcs, 2), ix!(IDY, lda, 5), iz!(IMP, nop, 2), ix!(IDY, lax, 5),
    iz!(ZPX, ldy, 4), iz!(ZPX, lda, 4), iz!(ZPY, ldx, 4), iz!(ZPY, lax, 4),
    iz!(IMP, clv, 2), ix!(ABY, lda, 4), iz!(IMP, tsx, 2), iz!(IMP, nop, 2),
    ix!(ABX, ldy, 4), ix!(ABX, lda, 4), ix!(ABY, ldx, 4), ix!(ABY, lax, 4),
    // 0xC0                                                                
    iz!(IMM, cpy, 2), iz!(IDX, cmp, 6), iz!(IMM, nop, 2), iz!(IDX, dcp, 8), 
    iz!(ZPG, cpy, 3), iz!(ZPG, cmp, 3), iz!(ZPG, dec, 5), iz!(ZPG, dcp, 5),
    iz!(IMP, iny, 2), iz!(IMM, cmp, 2), iz!(IMP, dex, 2), iz!(IMP, nop, 2),
    iz!(ABS, cpy, 4), iz!(ABS, cmp, 4), iz!(ABS, dec, 6), iz!(ABS, dcp, 6),
    // 0xD0                                                                
    ix!(REL, bne, 2), ix!(IDY, cmp, 5), iz!(IMP, nop, 2), iz!(IDY, dcp, 8),
    iz!(ZPX, nop, 4), iz!(ZPX, cmp, 4), iz!(ZPX, dec, 6), iz!(ZPX, dcp, 6),
    iz!(IMP, cld, 2), ix!(ABY, cmp, 4), iz!(IMP, nop, 2), iz!(ABY, dcp, 7),
    ix!(ABX, nop, 4), ix!(ABX, cmp, 4), iz!(ABX, dec, 7), iz!(ABX, dcp, 7),
    // 0xE0                                                                
    iz!(IMM, cpx, 2), iz!(IDX, sbc, 6), iz!(IMM, nop, 2), iz!(IDX, isc, 8),
    iz!(ZPG, cpx, 3), iz!(ZPG, sbc, 3), iz!(ZPG, inc, 5), iz!(ZPG, isc, 5),
    iz!(IMP, inx, 2), iz!(IMM, sbc, 2), iz!(IMP, nop, 2), iz!(IMM, sbc, 2),
    iz!(ABS, cpx, 4), iz!(ABS, sbc, 4), iz!(ABS, inc, 6), iz!(ABS, isc, 6),
    // 0xF0                                                                
    ix!(REL, beq, 2), ix!(IDY, sbc, 5), iz!(IMP, nop, 2), iz!(IDY, isc, 8),
    iz!(ZPX, nop, 4), iz!(ZPX, sbc, 4), iz!(ZPX, inc, 6), iz!(ZPX, isc, 6),
    iz!(IMP, sed, 2), ix!(ABY, sbc, 4), iz!(IMP, nop, 2), iz!(ABY, isc, 7), 
    ix!(ABX, nop, 4), ix!(ABX, sbc, 4), iz!(ABX, inc, 7), iz!(ABX, isc, 7),
];
