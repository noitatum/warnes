use std::fmt;
use mem::Memory as Mem;
use loadstore::LoadStore;
use std::num::Wrapping as W;
use dma::DMA;
//use enums::{MemState, IoState};

/* Branch flag types */
const BRANCH_FLAG_CHECK : u8 = 0x20;
const BRANCH_FLAG_TABLE : [u8; 4] = 
    [FLAG_SIGN, FLAG_OVERFLOW, FLAG_CARRY, FLAG_ZERO];

/* Memory */
const STACK_PAGE        : W<u16> = W(0x0100 as u16); 
const PAGE_MASK         : W<u16> = W(0xFF00 as u16);
const ADDRESS_INTERRUPT : W<u16> = W(0xFFFE as u16);
const ADDRESS_RESET     : W<u16> = W(0xFFFC as u16);

/* Flag bits */
const FLAG_CARRY        : u8 = 0x01;
const FLAG_ZERO         : u8 = 0x02;
const FLAG_INTERRUPT    : u8 = 0x04;
const FLAG_DECIMAL      : u8 = 0x08;
const FLAG_BRK          : u8 = 0x10;
const FLAG_PUSHED       : u8 = 0x20;
const FLAG_OVERFLOW     : u8 = 0x40;
const FLAG_SIGN         : u8 = 0x80;

#[allow(non_camel_case_types)]
type fn_operation = fn(&mut Regs, &mut Mem, W<u16>);

#[derive(Default, Debug)]
pub struct Cpu {
    // Cycle count since power up
    cycles      : u64,
    regs        : Regs,
    exec        : Execution,
    dma         : DMA,
} 

struct Execution {
    cycles_left     : u32,
    address         : W<u16>,
    operation       : fn_operation, 
}

struct Instruction {
    addressing  : fn(&mut Regs, &mut Mem) -> (W<u16>, u32),
    operation   : fn_operation, 
    cycles      : u32,
    has_extra   : bool,
    name        : &'static str
}

#[allow(dead_code)] 
#[inline(always)]
impl Instruction {
    pub fn name(&mut self) -> String {
        return self.name.to_string();
    }
}

#[allow(non_snake_case)]
struct Regs {
    A           : W<u8>,    // Accumulator
    X           : W<u8>,    // Indexes
    Y           : W<u8>,    //
    Flags       : u8,       // Status
    SP          : W<u8>,    // Stack pointer
    PC          : W<u16>,   // Program counter
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

    #[allow(dead_code)]
    pub fn next_instr(&mut self, memory: &mut Mem) -> String {
        let index = self.regs.next_opcode(memory) as usize;
        return OPCODE_TABLE[index].name();
    }
}

impl Default for Execution {
    fn default() -> Execution {
        Execution {
            cycles_left     : 0,
            operation       : Regs::nop,
            address         : W(0),
        }
    }
}

impl Execution {

    pub fn cycle(&mut self, memory: &mut Mem, regs: &mut Regs) {
        if self.cycles_left == 0 {
            // Execute the next instruction
            (self.operation)(regs, memory, self.address);
            // Get next instruction
            let index = regs.next_opcode(memory) as usize;
            let instruction = &OPCODE_TABLE[index];
            // Get address and extra cycles from mode
            let (address, extra) = (instruction.addressing)(regs, memory);
            // Save the address for next instruction
            self.address = address;
            self.cycles_left = instruction.cycles; 
            // Add the extra cycles if needed
            if instruction.has_extra {
                self.cycles_left += extra;
            }
            self.operation = instruction.operation;
        }
        self.cycles_left -= 1;
    }
}

impl Default for Regs {
    fn default() -> Regs {
        Regs {
            A               : W(0),
            X               : W(0),
            Y               : W(0),
            Flags           : 0x34, 
            SP              : W(0xFD),
            PC              : W(0),
        }
    }
}

// Util functions

impl Regs {

    pub fn reset(&mut self, memory: &mut Mem) {
        self.PC = memory.load_word(ADDRESS_RESET); 
    }

    pub fn next_opcode(&self, memory: &mut Mem) -> u8 {
        memory.load(self.PC).0
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
        let sum = a + m + W((self.Flags & FLAG_CARRY) as u16);
        set_flag_cond!(self.Flags, FLAG_OVERFLOW, 
                       (a ^ sum) & (m ^ sum) & W(0x80) > W(0));
        set_flag_cond!(self.Flags, FLAG_CARRY, sum > W(0xFF));
        self.A = W8!(sum);
        set_sign_zero!(self.Flags, self.A);
    } 

    fn compare(&mut self, reg: W<u8>, value: W<u8>) {
        let comp = W16!(reg) - W16!(value);
        set_sign_zero_carry_cond!(self.Flags, W8!(comp), comp <= W(0xFF));
    }

    fn rotate_right(&mut self, value: W<u8>) -> W<u8> {
        let carry = value & W(1) > W(0);
        let rot = (value >> 1) | (W(self.Flags & FLAG_CARRY) << 7);
        set_sign_zero_carry_cond!(self.Flags, rot, carry);
        rot
    }

    fn rotate_left(&mut self, value: W<u8>) -> W<u8> {
        let carry = value & W(0x80) > W(0);
        let rot = (value << 1) | W(self.Flags & FLAG_CARRY);
        set_sign_zero_carry_cond!(self.Flags, rot, carry);
        rot
    }

    fn shift_right(&mut self, value: W<u8>) -> W<u8> {
        let shift = value >> 1;
        set_sign_zero_carry_cond!(self.Flags, shift, value & W(1) > W(0));
        shift
    }

    fn shift_left(&mut self, value: W<u8>) -> W<u8> {
        let shift = value << 1;
        set_sign_zero_carry_cond!(self.Flags, shift, value & W(0x80) > W(0));
        shift
    }
}

// Addressing modes

impl Regs {

    fn imp(&mut self, _: &mut Mem) -> (W<u16>, u32) {
        self.PC = self.PC + W(1);
        (W(0), 0)
    }

    fn imm(&mut self, _: &mut Mem) -> (W<u16>, u32) {
        self.PC = self.PC + W(2);
        (self.PC - W(1), 0)
    }

    fn ind(&mut self, memory: &mut Mem) -> (W<u16>, u32) {
        let address = memory.load_word(self.PC + W(1));
        self.PC = self.PC + W(3);
        (memory.load_word_page_wrap(address), 0)
    }

    fn idx(&mut self, memory: &mut Mem) -> (W<u16>, u32) {
        let address = W16!(memory.load(self.PC + W(1)) + self.X);
        self.PC = self.PC + W(2);
        (memory.load_word_page_wrap(address), 0)
    }

    fn idy(&mut self, memory: &mut Mem) -> (W<u16>, u32) {
        let addr = W16!(memory.load(self.PC + W(1))); 
        let dest = W16!(memory.load_word_page_wrap(addr) + W16!(self.Y));
        self.PC = self.PC + W(2);
        (dest, (W8!(dest) < self.Y) as u32)
    }

    fn zpg(&mut self, memory: &mut Mem) -> (W<u16>, u32) {
        let address = W16!(memory.load(self.PC + W(1)));
        self.PC = self.PC + W(2);
        (address, 0)
    }

    fn zpx(&mut self, memory: &mut Mem) -> (W<u16>, u32) {
        let address = W16!(memory.load(self.PC + W(1)) + self.X);
        self.PC = self.PC + W(2);
        (address, 0)
    }

    fn zpy(&mut self, memory: &mut Mem) -> (W<u16>, u32) {
        let address = W16!(memory.load(self.PC + W(1)) + self.Y);
        self.PC = self.PC + W(2);
        (address, 0)
    }

    fn abs(&mut self, memory: &mut Mem) -> (W<u16>, u32) {
        let address = memory.load_word(self.PC + W(1));
        self.PC = self.PC + W(3);
        (address, 0)
    }

    fn abx(&mut self, memory: &mut Mem) -> (W<u16>, u32) {
        let address = memory.load_word(self.PC + W(1)) + W16!(self.X);
        self.PC = self.PC + W(3);
        (address, (W8!(address) < self.X) as u32)
    }

    fn aby(&mut self, memory: &mut Mem) -> (W<u16>, u32) {
        let address = memory.load_word(self.PC + W(1)) + W16!(self.Y);
        self.PC = self.PC + W(3);
        (address, (W8!(address) < self.Y) as u32)
    }

    fn rel(&mut self, memory: &mut Mem) -> (W<u16>, u32) {
        let opcode = memory.load(self.PC).0;
        let index = (opcode >> 6) as usize;
        let check = is_flag_set!(opcode, BRANCH_FLAG_CHECK);
        let next_opcode = self.PC + W(2);
        if is_flag_set!(self.Flags, BRANCH_FLAG_TABLE[index]) != check {
            (next_opcode, 0)
        } else {
            // Branch taken
            let offset = W(memory.load(self.PC + W(1)).0 as i8 as u16);  
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
       let flags = W(self.Flags | FLAG_PUSHED | FLAG_BRK);
       let pc = self.PC + W(1);
       self.push_word(memory, pc);
       self.push(memory, flags);
       set_flag!(self.Flags, FLAG_INTERRUPT);
       self.PC = memory.load_word(ADDRESS_INTERRUPT);
    }

    fn rti(&mut self, memory: &mut Mem, _: W<u16>) {
        // Ignore the two bits not present
        self.Flags = self.pop(memory).0 & !(FLAG_PUSHED | FLAG_BRK);
        self.PC = self.pop_word(memory);
    }

    fn rts(&mut self, memory: &mut Mem, _: W<u16>) {
        self.PC = self.pop_word(memory) + W(1);
    }

    // Implied

    fn php(&mut self, memory: &mut Mem, _: W<u16>) {
        // Two bits are set on memory when pushing flags 
        let flags = W(self.Flags | FLAG_PUSHED | FLAG_BRK);
        self.push(memory, flags);
    }

    fn sal(&mut self, _: &mut Mem, _: W<u16>) {
        let a = self.A;
        self.A = self.shift_left(a);
    }

    fn clc(&mut self, _: &mut Mem, _: W<u16>) {
        unset_flag!(self.Flags, FLAG_CARRY);
    }

    fn plp(&mut self, memory: &mut Mem, _: W<u16>) {
        // Ignore the two bits not present
        self.Flags = self.pop(memory).0 & !(FLAG_PUSHED | FLAG_BRK);
    }

    fn ral(&mut self, _: &mut Mem, _: W<u16>) {
        let a = self.A;
        self.A = self.rotate_left(a);
    }

    fn sec(&mut self, _: &mut Mem, _: W<u16>) {
        set_flag!(self.Flags, FLAG_CARRY);
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
        unset_flag!(self.Flags, FLAG_INTERRUPT);
    }

    fn pla(&mut self, memory: &mut Mem, _: W<u16>) {
        self.A = self.pop(memory);
        set_sign_zero!(self.Flags, self.A);
    }

    fn rar(&mut self, _: &mut Mem, _: W<u16>) {
        let a = self.A;
        self.A = self.rotate_right(a);
    }

    fn sei(&mut self, _: &mut Mem, _: W<u16>) {
        set_flag!(self.Flags, FLAG_INTERRUPT);
    }

    fn dey(&mut self, _: &mut Mem, _: W<u16>) {
        self.Y = self.Y - W(1);
        set_sign_zero!(self.Flags, self.Y);
    }

    fn txa(&mut self, _: &mut Mem, _: W<u16>) {
        self.A = self.X;
        set_sign_zero!(self.Flags, self.A);
    }

    fn tya(&mut self, _: &mut Mem, _: W<u16>) {
        self.A = self.Y;
        set_sign_zero!(self.Flags, self.A);
    }

    fn txs(&mut self, _: &mut Mem, _: W<u16>) {
        self.SP = self.X;
    }

    fn tay(&mut self, _: &mut Mem, _: W<u16>) {
        self.Y = self.A;
        set_sign_zero!(self.Flags, self.Y);
    }

    fn tax(&mut self, _: &mut Mem, _: W<u16>) {
        self.X = self.A;
        set_sign_zero!(self.Flags, self.X);
    }

    fn clv(&mut self, _: &mut Mem, _: W<u16>) {
        unset_flag!(self.Flags, FLAG_OVERFLOW);
    }

    fn tsx(&mut self, _: &mut Mem, _: W<u16>) {
        self.X = self.SP;
        set_sign_zero!(self.Flags, self.X);
    }

    fn iny(&mut self, _: &mut Mem, _: W<u16>) {
        self.Y = self.Y + W(1);
        set_sign_zero!(self.Flags, self.Y);
    }

    fn dex(&mut self, _: &mut Mem, _: W<u16>) {
        self.X = self.X - W(1);
        set_sign_zero!(self.Flags, self.X);
    }

    fn cld(&mut self, _: &mut Mem, _: W<u16>) {
        unset_flag!(self.Flags, FLAG_DECIMAL);
    }

    fn inx(&mut self, _: &mut Mem, _: W<u16>) {
        self.X = self.X + W(1);
        set_sign_zero!(self.Flags, self.X);
    }

    fn nop(&mut self, _: &mut Mem, _: W<u16>) {
        
    }

    fn sed(&mut self, _: &mut Mem, _: W<u16>) {
        set_flag!(self.Flags, FLAG_DECIMAL);
    }

    // Common

    fn ora(&mut self, memory: &mut Mem, address: W<u16>) {
        let m = memory.load(address);
        self.A = self.A | m;
        set_sign_zero!(self.Flags, self.A);
    }

    fn asl(&mut self, memory: &mut Mem, address: W<u16>) {
        let m = self.shift_left(memory.load(address));
        memory.store(address, m);
    }

    fn bit(&mut self, memory: &mut Mem, address: W<u16>) {
        let m = memory.load(address);
        copy_flag!(self.Flags, m, FLAG_OVERFLOW);
        set_sign!(self.Flags, m); 
        set_zero!(self.Flags, self.A & m);
    }

    fn and(&mut self, memory: &mut Mem, address: W<u16>) {
        let m = memory.load(address);
        self.A = self.A & m;
        set_sign_zero!(self.Flags, self.A);
    }

    fn rol(&mut self, memory: &mut Mem, address: W<u16>) {
        let m = self.rotate_left(memory.load(address));
        memory.store(address, m);
    }

    fn eor(&mut self, memory: &mut Mem, address: W<u16>) {
        let m = memory.load(address);
        self.A = m ^ self.A;
        set_sign_zero!(self.Flags, self.A);
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
        set_sign_zero!(self.Flags, self.Y);
    }

    fn ldx(&mut self, memory: &mut Mem, address: W<u16>) {
        self.X = memory.load(address);
        set_sign_zero!(self.Flags, self.X);
    }

    fn lda(&mut self, memory: &mut Mem, address: W<u16>) {
        self.A = memory.load(address);
        set_sign_zero!(self.Flags, self.A);
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
        set_sign_zero!(self.Flags, m);
        memory.store(address, m);
    }

    fn sbc(&mut self, memory: &mut Mem, address: W<u16>) {
        self.add_with_carry(!memory.load(address));
    }
   
    fn inc(&mut self, memory: &mut Mem, address: W<u16>) {
        let m = memory.load(address) + W(1);
        set_sign_zero!(self.Flags, m);
        memory.store(address, m);
    }
}

// Unofficial Instructions

impl Regs {

    fn lax(&mut self, memory: &mut Mem, address: W<u16>) {
        let m = memory.load(address);
        self.A = m;
        self.X = m;
        set_sign_zero!(self.Flags, m);
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
        self.A = self.A | shift;
        set_sign_zero!(self.Flags, self.A);
    }

    fn rla(&mut self, memory: &mut Mem, address: W<u16>) {
        let rot = self.rotate_left(memory.load(address)); 
        memory.store(address, rot);
        self.A = self.A & rot;
        set_sign_zero!(self.Flags, self.A);
    }

    fn sre(&mut self, memory: &mut Mem, address: W<u16>) {
        let shift = self.shift_right(memory.load(address));
        memory.store(address, shift);
        self.A = self.A ^ shift;
        set_sign_zero!(self.Flags, self.A);
    }

    fn rra(&mut self, memory: &mut Mem, address: W<u16>) {
        let rot = self.rotate_right(memory.load(address)); 
        memory.store(address, rot);
        self.add_with_carry(rot);
    }
}

impl fmt::Debug for Execution {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{{Execution: cycles_left: {}, address: {:#x}}}",
               self.cycles_left, self.address.0)
    }
}

impl fmt::Debug for Regs {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{{Regs: A: {:02X}, X: {:02X}, Y: {:02X}, P: {:02X}, SP: {:02X}, PC: {:04X} }}",
               self.A.0 , self.X.0 , self.Y.0 , self.Flags , self.SP.0 , self.PC.0)
    }
}
  

/* WARNING: Branch instructions are replaced with jumps */
const OPCODE_TABLE : [Instruction; 256] = [    
    // 0x00
    iz!(imp, brk, 7, "brk"), iz!(idx, ora, 6, "ora"), iz!(imp, nop, 2, "nop"), iz!(idx, slo, 8, "slo"), 
    iz!(zpg, nop, 3, "nop"), iz!(zpg, ora, 3, "ora"), iz!(zpg, asl, 5, "asl"), iz!(zpg, slo, 5, "slo"), 
    iz!(imp, php, 3, "php"), iz!(imm, ora, 2, "ora"), iz!(imp, sal, 2, "sal"), iz!(imp, nop, 2, "nop"), 
    iz!(abs, nop, 4, "nop"), iz!(abs, ora, 4, "ora"), iz!(abs, asl, 6, "asl"), iz!(abs, slo, 6, "slo"),
    // 0x10 
    ix!(rel, jmp, 2, "jmp"), ix!(idy, ora, 5, "ora"), iz!(imp, nop, 2, "nop"), iz!(idy, slo, 4, "slo"),
    iz!(zpx, nop, 4, "nop"), iz!(zpx, ora, 4, "ora"), iz!(zpx, asl, 6, "asl"), iz!(zpx, slo, 6, "slo"),
    iz!(imp, clc, 2, "clc"), ix!(aby, ora, 4, "ora"), iz!(imp, nop, 2, "nop"), iz!(aby, slo, 7, "slo"),
    ix!(abx, nop, 4, "nop"), ix!(abx, ora, 4, "ora"), iz!(abx, asl, 7, "asl"), iz!(abx, slo, 7, "slo"),
    // 0x20
    iz!(abs, jsr, 6, "jsr"), iz!(idx, and, 6, "and"), iz!(imp, nop, 2, "nop"), iz!(idx, rla, 8, "rla"),
    iz!(zpg, bit, 3, "bit"), iz!(zpg, and, 3, "and"), iz!(zpg, rol, 5, "rol"), iz!(zpg, rla, 5, "rla"),
    iz!(imp, plp, 4, "plp"), iz!(imm, and, 2, "and"), iz!(imp, ral, 2, "ral"), iz!(imp, nop, 2, "nop"),
    iz!(abs, bit, 4, "bit"), iz!(abs, and, 4, "and"), iz!(abs, rol, 6, "rol"), iz!(abs, rla, 6, "rla"),
    // 0x30
    ix!(rel, jmp, 2, "jmp"), ix!(idy, and, 5, "and"), iz!(imp, nop, 2, "nop"), iz!(idy, rla, 8, "rla"),
    iz!(zpx, nop, 4, "nop"), iz!(zpx, and, 4, "and"), iz!(zpx, rol, 6, "rol"), iz!(zpx, rla, 6, "rla"),
    iz!(imp, sec, 2, "sec"), ix!(aby, and, 4, "and"), iz!(imp, nop, 2, "nop"), iz!(aby, rla, 7, "rla"),
    ix!(abx, nop, 4, "nop"), ix!(abx, and, 4, "and"), iz!(abx, rol, 7, "rol"), iz!(abx, rla, 7, "rla"),
    // 0x40
    iz!(imp, rti, 6, "rti"), iz!(idx, eor, 6, "eor"), iz!(imp, nop, 2, "nop"), iz!(idx, sre, 8, "sre"),
    iz!(zpg, nop, 3, "nop"), iz!(zpg, eor, 3, "eor"), iz!(zpg, lsr, 5, "lsr"), iz!(zpg, sre, 5, "sre"),
    iz!(imp, pha, 3, "pha"), iz!(imm, eor, 2, "eor"), iz!(imp, sar, 2, "sar"), iz!(imp, nop, 2, "nop"),
    iz!(abs, jmp, 3, "jmp"), iz!(abs, eor, 4, "eor"), iz!(abs, lsr, 6, "lsr"), iz!(abs, sre, 6, "sre"),
    // 0x50
    ix!(rel, jmp, 2, "jmp"), ix!(idy, eor, 5, "eor"), iz!(imp, nop, 2, "nop"), iz!(idy, sre, 8, "sre"),
    iz!(zpx, nop, 4, "nop"), iz!(zpx, eor, 4, "eor"), iz!(zpx, lsr, 6, "lsr"), iz!(zpx, sre, 6, "sre"),
    iz!(imp, cli, 2, "cli"), ix!(aby, eor, 4, "eor"), iz!(imp, nop, 2, "nop"), iz!(aby, sre, 7, "sre"), 
    ix!(abx, nop, 4, "nop"), ix!(abx, eor, 4, "eor"), iz!(abx, lsr, 7, "lsr"), iz!(abx, sre, 7, "sre"),
    // 0x60
    iz!(imp, rts, 6, "rts"), iz!(idx, adc, 6, "adc"), iz!(imp, nop, 2, "nop"), iz!(idx, rra, 8, "rra"),
    iz!(zpg, nop, 3, "nop"), iz!(zpg, adc, 3, "adc"), iz!(zpg, ror, 5, "ror"), iz!(zpg, rra, 5, "rra"),
    iz!(imp, pla, 4, "pla"), iz!(imm, adc, 2, "adc"), iz!(imp, rar, 2, "rar"), iz!(imp, nop, 2, "nop"),
    iz!(ind, jmp, 5, "jmp"), iz!(abs, adc, 4, "adc"), iz!(abs, ror, 6, "ror"), iz!(abs, rra, 6, "rra"),
    // 0x70
    ix!(rel, jmp, 2, "jmp"), ix!(idy, adc, 5, "adc"), iz!(imp, nop, 2, "nop"), iz!(idy, rra, 8, "rra"),
    iz!(zpx, nop, 4, "nop"), iz!(zpx, adc, 4, "adc"), iz!(zpx, ror, 6, "ror"), iz!(zpx, rra, 6, "rra"),
    iz!(imp, sei, 2, "sei"), ix!(aby, adc, 4, "adc"), iz!(imp, nop, 2, "nop"), iz!(aby, rra, 7, "rra"), 
    ix!(abx, nop, 4, "nop"), ix!(abx, adc, 4, "adc"), iz!(abx, ror, 7, "ror"), iz!(abx, rra, 7, "rra"),
    // 0x80
    iz!(imm, nop, 2, "nop"), iz!(idx, sta, 6, "sta"), iz!(imm, nop, 2, "nop"), iz!(idx, sax, 6, "sax"),
    iz!(zpg, sty, 3, "sty"), iz!(zpg, sta, 3, "sta"), iz!(zpg, stx, 3, "stx"), iz!(zpg, sax, 3, "sax"),
    iz!(imp, dey, 2, "dey"), iz!(imm, nop, 2, "nop"), iz!(imp, txa, 2, "txa"), iz!(imp, nop, 2, "nop"),
    iz!(abs, sty, 4, "sty"), iz!(abs, sta, 4, "sta"), iz!(abs, stx, 4, "stx"), iz!(abs, sax, 4, "sax"),
    // 0x90
    ix!(rel, jmp, 2, "jmp"), iz!(idy, sta, 6, "sta"), iz!(imp, nop, 2, "nop"), iz!(imp, nop, 2, "nop"), 
    iz!(zpx, sty, 4, "sty"), iz!(zpx, sta, 4, "sta"), iz!(zpy, stx, 4, "stx"), iz!(zpy, sax, 4, "sax"),
    iz!(imp, tya, 2, "tya"), iz!(aby, sta, 5, "sta"), iz!(imp, txs, 2, "txs"), iz!(imp, nop, 2, "nop"), 
    iz!(imp, nop, 2, "nop"), iz!(abx, sta, 5, "sta"), iz!(imp, nop, 2, "nop"), iz!(imp, nop, 2, "nop"),
    // 0xA0
    iz!(imm, ldy, 2, "ldy"), iz!(idx, lda, 6, "lda"), iz!(imm, ldx, 2, "ldx"), iz!(idx, lax, 6, "lax"),
    iz!(zpg, ldy, 3, "ldy"), iz!(zpg, lda, 3, "lda"), iz!(zpg, ldx, 3, "ldx"), iz!(zpg, lax, 3, "lax"),
    iz!(imp, tay, 2, "tay"), iz!(imm, lda, 2, "lda"), iz!(imp, tax, 2, "tax"), iz!(imm, lax, 2, "lax"),
    iz!(abs, ldy, 4, "ldy"), iz!(abs, lda, 4, "lda"), iz!(abs, ldx, 4, "ldx"), iz!(abs, lax, 4, "lax"),
    // 0xB0
    ix!(rel, jmp, 2, "jmp"), ix!(idy, lda, 5, "lda"), iz!(imp, nop, 2, "nop"), ix!(idy, lax, 5, "lax"),
    iz!(zpx, ldy, 4, "ldy"), iz!(zpx, lda, 4, "lda"), iz!(zpy, ldx, 4, "ldx"), iz!(zpy, lax, 4, "lax"),
    iz!(imp, clv, 2, "clv"), ix!(aby, lda, 4, "lda"), iz!(imp, tsx, 2, "tsx"), iz!(imp, nop, 2, "nop"),
    ix!(abx, ldy, 4, "ldy"), ix!(abx, lda, 4, "lda"), ix!(aby, ldx, 4, "ldx"), ix!(aby, lax, 4, "lax"),
    // 0xC0
    iz!(imm, cpy, 2, "cpy"), iz!(idx, cmp, 6, "cmp"), iz!(imm, nop, 2, "nop"), iz!(idx, dcp, 8, "dcp"), 
    iz!(zpg, cpy, 3, "cpy"), iz!(zpg, cmp, 3, "cmp"), iz!(zpg, dec, 5, "dec"), iz!(zpg, dcp, 5, "dcp"),
    iz!(imp, iny, 2, "iny"), iz!(imm, cmp, 2, "cmp"), iz!(imp, dex, 2, "dex"), iz!(imp, nop, 2, "nop"),
    iz!(abs, cpy, 4, "cpy"), iz!(abs, cmp, 4, "cmp"), iz!(abs, dec, 6, "dec"), iz!(abs, dcp, 6, "dcp"),
    // 0xD0
    ix!(rel, jmp, 2, "jmp"), ix!(idy, cmp, 5, "cmp"), iz!(imp, nop, 2, "nop"), iz!(idy, dcp, 8, "dcp"),
    iz!(zpx, nop, 4, "nop"), iz!(zpx, cmp, 4, "cmp"), iz!(zpx, dec, 6, "dec"), iz!(zpx, dcp, 6, "dcp"),
    iz!(imp, cld, 2, "cld"), ix!(aby, cmp, 4, "cmp"), iz!(imp, nop, 2, "nop"), iz!(aby, dcp, 7, "dcp"),
    ix!(abx, nop, 4, "nop"), ix!(abx, cmp, 4, "cmp"), iz!(abx, dec, 7, "dec"), iz!(abx, dcp, 7, "dcp"),
    // 0xE0
    iz!(imm, cpx, 2, "cpx"), iz!(idx, sbc, 6, "sbc"), iz!(imm, nop, 2, "nop"), iz!(idx, isc, 8, "isc"),
    iz!(zpg, cpx, 3, "cpx"), iz!(zpg, sbc, 3, "sbc"), iz!(zpg, inc, 6, "inc"), iz!(zpg, isc, 5, "isc"),
    iz!(imp, inx, 2, "inx"), iz!(imm, sbc, 2, "sbc"), iz!(imp, nop, 2, "nop"), iz!(imm, sbc, 2, "sbc"),
    iz!(abs, cpx, 4, "cpx"), iz!(abs, sbc, 4, "sbc"), iz!(abs, inc, 6, "inc"), iz!(abs, isc, 6, "isc"),
    // 0xF0
    ix!(rel, jmp, 2, "jmp"), ix!(idy, sbc, 5, "sbc"), iz!(imp, nop, 2, "nop"), iz!(idy, isc, 8, "isc"),
    iz!(zpx, nop, 4, "nop"), iz!(zpx, sbc, 4, "sbc"), iz!(zpx, inc, 6, "inc"), iz!(zpx, isc, 6, "isc"),
    iz!(imp, sed, 2, "sed"), ix!(aby, sbc, 4, "sbc"), iz!(imp, nop, 2, "nop"), iz!(aby, isc, 7, "isc"), 
    ix!(abx, nop, 4, "nop"), ix!(abx, sbc, 4, "sbc"), iz!(abx, inc, 7, "inc"), iz!(abx, isc, 7, "isc"),
];
