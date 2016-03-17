use std::fmt;
use mem::Memory as Mem;
use loadstore::LoadStore;
use std::num::Wrapping as W;
use dma::DMA;
//use enums::{MemState, IoState};
use enums::OpType;

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
    
    /// Debug
    pub fn next_instr(&mut self, memory: &mut Mem) -> (String, u32, Vec<u8>, bool, OpType) {
        let index = self.regs.next_opcode(memory) as usize;
        let op_name = OPCODE_TABLE[index].name();
        let mut arr = vec!(0, 0, 0);
        let mut two_bytes : bool = false;
        match OPCODE_TABLE[index].num_bytes() { 
            1 => { arr[0] = 1; },
            2 => { arr[1] = memory.load_no_side_effect(self.regs.pc().0 + W(1)).0; },
            3 => { arr[2] = memory.load_no_side_effect(self.regs.pc().0 + W(2)).0;
                   arr[2] = memory.load_no_side_effect(self.regs.pc().0 + W(2)).0; 
                   two_bytes = true; },
            _ => { panic!("no operation has this size of bytes: {}", 
                         OPCODE_TABLE[index].num_bytes()); }
        }
        // two_bytes is true if the operation takes two more bytes
        // generally for adressing
        return (op_name, OPCODE_TABLE[index].cycles(),
                 arr, two_bytes, OPCODE_TABLE[index].op_type);
    }
}

struct Execution {
    cycles_left     : u32,
    operation       : fn(&mut Regs, &mut Mem, W<u16>),  
    address         : W<u16>,
}

impl Execution {
    pub fn cycle(&mut self, memory: &mut Mem, regs: &mut Regs) {
        if self.cycles_left == 0 {
            // Execute the next instruction
            // regs = &mut self.
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

    #[allow(dead_code)]
    pub fn address(&mut self) -> W<u16> {
        return self.address;
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

struct Instruction {
    addressing  : fn(&mut Regs, &mut Mem) -> (W<u16>, u32),
    operation   : fn(&mut Regs, &mut Mem, W<u16>), 
    cycles      : u32,
    has_extra   : bool,
    name        : &'static str,
    size        : u8, // instr size in bytes
    op_type     : OpType,
}

impl Instruction {
    pub fn name(&mut self) -> String {
        return self.name.to_string();
    }

    #[inline(always)]
    pub fn num_bytes(&mut self) -> u8 {
        return self.size;
    }

    pub fn cycles(&mut self) -> u32 {
        return self.cycles;
    }
}

#[allow(non_snake_case)]
struct Regs {
    A           : W<u8>,    // Accumulator
    X           : W<u8>,    // Indexes
    Y           : W<u8>,    //
    P           : W<u8>,    // Status
    SP          : W<u8>,    // Stack pointer
    PC          : W<u16>,   // Program counter
    PC_DEBUG    : W<u16>,   // PC to list instructions when using the debugger.
}

impl Default for Regs {
    fn default() -> Regs {
        Regs {
            A               : W(0),
            X               : W(0),
            Y               : W(0),
            P               : W(0x34), 
            SP              : W(0xfd),
            PC              : W(0),
            PC_DEBUG        : W(0),
        }
    }
}

// Util functions
impl Regs {
    pub fn pc(&mut self) -> (W<u16>, W<u16>) {
        return (self.PC, self.PC_DEBUG);
    } 

    pub fn reset(&mut self, memory: &mut Mem) {
        self.PC = memory.load_word(ADDRESS_RESET); 
        self.PC = W(0xC000);
        self.PC_DEBUG = self.PC;
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
        if is_bit_set!(self.P, BRANCH_FLAG_TABLE[index]) != check {
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
        self.Y = self.Y - W(1);
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
        self.Y = self.Y + W(1);
        set_sign_zero!(self.P, self.Y);
    }

    fn dex(&mut self, _: &mut Mem, _: W<u16>) {
        self.X = self.X - W(1);
        set_sign_zero!(self.P, self.X);
    }

    fn cld(&mut self, _: &mut Mem, _: W<u16>) {
        unset_flag!(self.P, FLAG_DECIMAL);
    }

    fn inx(&mut self, _: &mut Mem, _: W<u16>) {
        self.X = self.X + W(1);
        set_sign_zero!(self.P, self.X);
    }

    fn nop(&mut self, _: &mut Mem, _: W<u16>) {
        
    }

    fn sed(&mut self, _: &mut Mem, _: W<u16>) {
        set_flag!(self.P, FLAG_DECIMAL);
    }

    // Common

    fn ora(&mut self, memory: &mut Mem, address: W<u16>) {
        let m = memory.load(address);
        self.A = self.A | m;
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
        let m = memory.load(address);
        self.A = self.A & m;
        set_sign_zero!(self.P, self.A);
    }

    fn rol(&mut self, memory: &mut Mem, address: W<u16>) {
        let m = self.rotate_left(memory.load(address));
        memory.store(address, m);
    }

    fn eor(&mut self, memory: &mut Mem, address: W<u16>) {
        let m = memory.load(address);
        self.A = m ^ self.A;
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
        self.A = self.A | shift;
        set_sign_zero!(self.P, self.A);
    }

    fn rla(&mut self, memory: &mut Mem, address: W<u16>) {
        let rot = self.rotate_left(memory.load(address)); 
        memory.store(address, rot);
        self.A = self.A & rot;
        set_sign_zero!(self.P, self.A);
    }

    fn sre(&mut self, memory: &mut Mem, address: W<u16>) {
        let shift = self.shift_right(memory.load(address));
        memory.store(address, shift);
        self.A = self.A ^ shift;
        set_sign_zero!(self.P, self.A);
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
               self.A.0 , self.X.0 , self.Y.0 , self.P.0 , self.SP.0 , self.PC.0)
    }
}

/* WARNING: Branch instructions are replaced with jumps */
const OPCODE_TABLE : [Instruction; 256] = [    
    // 0x00
    iz!(imp, brk, 7, 1), iz!(idx, ora, 6, 2), iz!(imp, nop, 2, 1), iz!(idx, slo, 8, 2), 
    iz!(zpg, nop, 3, 2), iz!(zpg, ora, 3, 2), iz!(zpg, asl, 5, 2), iz!(zpg, slo, 5, 2), 
    iz!(imp, php, 3, 1), iz!(imm, ora, 2, 2), iz!(imp, sal, 2, 1), iz!(imp, nop, 2, 1), 
    iz!(abs, nop, 4, 3), iz!(abs, ora, 4, 3), iz!(abs, asl, 6, 3), iz!(abs, slo, 6, 3),
    // 0x10 
    ix!(rel, jmp, 2, 1), ix!(idy, ora, 5, 2), iz!(imp, nop, 2, 1), iz!(idy, slo, 8, 2),
    iz!(zpx, nop, 4, 2), iz!(zpx, ora, 4, 2), iz!(zpx, asl, 6, 2), iz!(zpx, slo, 6, 2),
    iz!(imp, clc, 2, 1), ix!(aby, ora, 4, 3), iz!(imp, nop, 2, 1), iz!(aby, slo, 7, 3),
    ix!(abx, nop, 4, 3), ix!(abx, ora, 4, 3), iz!(abx, asl, 7, 3), iz!(abx, slo, 7, 3),
    // 0x20
    iz!(abs, jsr, 6, 3), iz!(idx, and, 6, 2), iz!(imp, nop, 2, 1), iz!(idx, rla, 8, 2),
    iz!(zpg, bit, 3, 2), iz!(zpg, and, 3, 2), iz!(zpg, rol, 5, 2), iz!(zpg, rla, 5, 2),
    iz!(imp, plp, 4, 1), iz!(imm, and, 2, 2), iz!(imp, ral, 2, 1), iz!(imp, nop, 2, 1),
    iz!(abs, bit, 4, 3), iz!(abs, and, 4, 3), iz!(abs, rol, 6, 3), iz!(abs, rla, 6, 3),
    // 0x30
    ix!(rel, jmp, 2, 1), ix!(idy, and, 5, 2), iz!(imp, nop, 2, 1), iz!(idy, rla, 8, 2),
    iz!(zpx, nop, 4, 2), iz!(zpx, and, 4, 2), iz!(zpx, rol, 6, 2), iz!(zpx, rla, 6, 2),
    iz!(imp, sec, 2, 1), ix!(aby, and, 4, 3), iz!(imp, nop, 2, 1), iz!(aby, rla, 7, 3),
    ix!(abx, nop, 4, 3), ix!(abx, and, 4, 3), iz!(abx, rol, 7, 3), iz!(abx, rla, 7, 3),
    // 0x40
    iz!(imp, rti, 6, 1), iz!(idx, eor, 6, 2), iz!(imp, nop, 2, 1), iz!(idx, sre, 8, 2),
    iz!(zpg, nop, 3, 2), iz!(zpg, eor, 3, 2), iz!(zpg, lsr, 5, 2), iz!(zpg, sre, 5, 2),
    iz!(imp, pha, 3, 1), iz!(imm, eor, 2, 2), iz!(imp, sar, 2, 1), iz!(imp, nop, 2, 1),
    iz!(abs, jmp, 3, 3), iz!(abs, eor, 4, 3), iz!(abs, lsr, 6, 3), iz!(abs, sre, 6, 3),
    // 0x50
    ix!(rel, jmp, 2, 1), ix!(idy, eor, 5, 2), iz!(imp, nop, 2, 1), iz!(idy, sre, 8, 2),
    iz!(zpx, nop, 4, 2), iz!(zpx, eor, 4, 2), iz!(zpx, lsr, 6, 2), iz!(zpx, sre, 6, 2),
    iz!(imp, cli, 2, 1), ix!(aby, eor, 4, 3), iz!(imp, nop, 2, 1), iz!(aby, sre, 7, 3), 
    ix!(abx, nop, 4, 3), ix!(abx, eor, 4, 3), iz!(abx, lsr, 7, 3), iz!(abx, sre, 7, 3),
    // 0x60
    iz!(imp, rts, 6, 1), iz!(idx, adc, 6, 2), iz!(imp, nop, 2, 1), iz!(idx, rra, 8, 2),
    iz!(zpg, nop, 3, 2), iz!(zpg, adc, 3, 2), iz!(zpg, ror, 5, 2), iz!(zpg, rra, 5, 2),
    iz!(imp, pla, 4, 1), iz!(imm, adc, 2, 2), iz!(imp, rar, 2, 1), iz!(imp, nop, 2, 1),
    iz!(ind, jmp, 5, 3), iz!(abs, adc, 4, 3), iz!(abs, ror, 6, 3), iz!(abs, rra, 6, 3),
    // 0x70
    ix!(rel, jmp, 2, 1), ix!(idy, adc, 5, 2), iz!(imp, nop, 2, 1), iz!(idy, rra, 8, 2),
    iz!(zpx, nop, 4, 2), iz!(zpx, adc, 4, 2), iz!(zpx, ror, 6, 2), iz!(zpx, rra, 6, 2),
    iz!(imp, sei, 2, 1), ix!(aby, adc, 4, 3), iz!(imp, nop, 2, 1), iz!(aby, rra, 7, 3), 
    ix!(abx, nop, 4, 3), ix!(abx, adc, 4, 3), iz!(abx, ror, 7, 3), iz!(abx, rra, 7, 3),
    // 0x80
    iz!(imm, nop, 2, 2), iz!(idx, sta, 6, 2), iz!(imm, nop, 2, 2), iz!(idx, sax, 6, 2),
    iz!(zpg, sty, 3, 2), iz!(zpg, sta, 3, 2), iz!(zpg, stx, 3, 2), iz!(zpg, sax, 3, 2),
    iz!(imp, dey, 2, 1), iz!(imm, nop, 2, 2), iz!(imp, txa, 2, 1), iz!(imp, nop, 2, 1),
    iz!(abs, sty, 4, 3), iz!(abs, sta, 4, 3), iz!(abs, stx, 4, 3), iz!(abs, sax, 4, 3),
    // 0x90
    ix!(rel, jmp, 2, 1), iz!(idy, sta, 6, 2), iz!(imp, nop, 2, 1), iz!(imp, nop, 2, 1), 
    iz!(zpx, sty, 4, 2), iz!(zpx, sta, 4, 2), iz!(zpy, stx, 4, 2), iz!(zpy, sax, 4, 2),
    iz!(imp, tya, 2, 1), iz!(aby, sta, 5, 3), iz!(imp, txs, 2, 1), iz!(imp, nop, 2, 1), 
    iz!(imp, nop, 2, 1), iz!(abx, sta, 5, 3), iz!(imp, nop, 2, 1), iz!(imp, nop, 2, 1),
    // 0xA0
    iz!(imm, ldy, 2, 2), iz!(idx, lda, 6, 2), iz!(imm, ldx, 2, 2), iz!(idx, lax, 6, 2),
    iz!(zpg, ldy, 3, 2), iz!(zpg, lda, 3, 2), iz!(zpg, ldx, 3, 2), iz!(zpg, lax, 3, 2),
    iz!(imp, tay, 2, 1), iz!(imm, lda, 2, 2), iz!(imp, tax, 2, 1), iz!(imm, lax, 2, 2),
    iz!(abs, ldy, 4, 3), iz!(abs, lda, 4, 3), iz!(abs, ldx, 4, 3), iz!(abs, lax, 4, 3),
    // 0xB0
    ix!(rel, jmp, 2, 1), ix!(idy, lda, 5, 2), iz!(imp, nop, 2, 1), ix!(idy, lax, 5, 2),
    iz!(zpx, ldy, 4, 2), iz!(zpx, lda, 4, 2), iz!(zpy, ldx, 4, 2), iz!(zpy, lax, 4, 2),
    iz!(imp, clv, 2, 1), ix!(aby, lda, 4, 3), iz!(imp, tsx, 2, 1), iz!(imp, nop, 2, 1),
    ix!(abx, ldy, 4, 3), ix!(abx, lda, 4, 3), ix!(aby, ldx, 4, 3), ix!(aby, lax, 4, 3),
    // 0xC0
    iz!(imm, cpy, 2, 2), iz!(idx, cmp, 6, 2), iz!(imm, nop, 2, 2), iz!(idx, dcp, 8, 2), 
    iz!(zpg, cpy, 3, 2), iz!(zpg, cmp, 3, 2), iz!(zpg, dec, 5, 2), iz!(zpg, dcp, 5, 2),
    iz!(imp, iny, 2, 1), iz!(imm, cmp, 2, 2), iz!(imp, dex, 2, 1), iz!(imp, nop, 2, 1),
    iz!(abs, cpy, 4, 3), iz!(abs, cmp, 4, 3), iz!(abs, dec, 6, 3), iz!(abs, dcp, 6, 3),
    // 0xD0
    ix!(rel, jmp, 2, 1), ix!(idy, cmp, 5, 2), iz!(imp, nop, 2, 1), iz!(idy, dcp, 8, 2),
    iz!(zpx, nop, 4, 2), iz!(zpx, cmp, 4, 2), iz!(zpx, dec, 6, 2), iz!(zpx, dcp, 6, 2),
    iz!(imp, cld, 2, 1), ix!(aby, cmp, 4, 3), iz!(imp, nop, 2, 1), iz!(aby, dcp, 7, 3),
    ix!(abx, nop, 4, 3), ix!(abx, cmp, 4, 3), iz!(abx, dec, 7, 3), iz!(abx, dcp, 7, 3),
    // 0xE0
    iz!(imm, cpx, 2, 2), iz!(idx, sbc, 6, 2), iz!(imm, nop, 2, 2), iz!(idx, isc, 8, 2),
    iz!(zpg, cpx, 3, 2), iz!(zpg, sbc, 3, 2), iz!(zpg, inc, 5, 2), iz!(zpg, isc, 5, 2),
    iz!(imp, inx, 2, 1), iz!(imm, sbc, 2, 2), iz!(imp, nop, 2, 1), iz!(imm, sbc, 2, 2),
    iz!(abs, cpx, 4, 3), iz!(abs, sbc, 4, 3), iz!(abs, inc, 6, 3), iz!(abs, isc, 6, 3),
    // 0xF0
    ix!(rel, jmp, 2, 1), ix!(idy, sbc, 5, 2), iz!(imp, nop, 2, 1), iz!(idy, isc, 8, 2),
    iz!(zpx, nop, 4, 2), iz!(zpx, sbc, 4, 2), iz!(zpx, inc, 6, 2), iz!(zpx, isc, 6, 2),
    iz!(imp, sed, 2, 1), ix!(aby, sbc, 4, 3), iz!(imp, nop, 2, 1), iz!(aby, isc, 7, 3), 
    ix!(abx, nop, 4, 3), ix!(abx, sbc, 4, 3), iz!(abx, inc, 7, 3), iz!(abx, isc, 7, 3),
];
