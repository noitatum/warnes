use std::fmt;
use mem::Memory as Mem;
use loadstore::LoadStore;
use std::num::Wrapping as W;

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

const OAMDATA           : W<u16> = W(0x2004);
const DMA_CYCLES        : u32 = 512;

#[derive(Default)]
#[derive(Debug)]
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
}

#[derive(Default)]
struct DMA {
    cycles_left : u32,
    address     : W<u16>,
    value       : W<u8>,
}

impl fmt::Debug for DMA {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{{DMA: cycles_left: {}, address: {:#x}, value: {:#x}}}",
               self.cycles_left, self.address.0, self.value.0)
    }
}

impl DMA {
    // Returns true if DMA is active
    pub fn cycle(&mut self, memory: &mut Mem, cycles: u64) -> bool {
        if self.cycles_left > 0 {
            self.execute(memory);
            true
        } else if let Some(page) = memory.get_oamdma() {
            self.start(page, cycles as u32);
            true
        } else {
            false
        }
    }

    fn start(&mut self, page: W<u8>, cycles: u32) {
        self.address = W16!(page) << 8; 
        // Additional cycle if on odd cycle
        self.cycles_left = DMA_CYCLES + cycles & 1;
    }

    fn execute(&mut self, memory: &mut Mem) {
        self.cycles_left -= 1;
        // Simulate idle cycles
        if self.cycles_left < DMA_CYCLES {
            // Read on odd cycles and write on even cycles
            if self.cycles_left & 1 == 1 {
                self.value = memory.load(self.address);
                self.address = self.address + W(1);
            } else {
                memory.store(OAMDATA, self.value);
            }
        }
    }
}

struct Execution {
    cycles_left     : u32,
    address         : W<u16>,
    operation       : FnOperation, 
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

impl fmt::Debug for Execution {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{{Execution: cycles_left: {}, address: {:#x}}}",
               self.cycles_left, self.address.0)
    }
}

impl Execution {

    pub fn cycle(&mut self, memory: &mut Mem, regs: &mut Regs) {
        if self.cycles_left == 0 {
            // Execute the next instruction
            (self.operation)(regs, memory, self.address);
            // Get next instruction
            let index = regs.next_opcode(memory) as usize;
            let ref instruction = OPCODE_TABLE[index];
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

#[allow(non_snake_case)]
struct Regs {
    A           : W<u8>,    // Accumulator
    X           : W<u8>,    // Indexes
    Y           : W<u8>,    //
    Flags       : u8,       // Status
    SP          : W<u8>,    // Stack pointer
    PC          : W<u16>,   // Program counter
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

impl fmt::Debug for Regs {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{{Regs: A: {:02X}, X: {:02X}, Y: {:02X}, P: {:02X}, SP: {:02X}, PC: {:04X} }}",
               self.A.0 , self.X.0 , self.Y.0 , self.Flags , self.SP.0 , self.PC.0)
    }
}

type FnOperation = fn(&mut Regs, &mut Mem, W<u16>);

struct Instruction {
    addressing  : fn(&mut Regs, &mut Mem) -> (W<u16>, u32),
    operation   : FnOperation, 
    cycles      : u32,
    has_extra   : bool,
}

macro_rules! inst {
    ($addr:ident, $oper:ident, $cycles:expr, $extra:expr) => (
        Instruction {
            addressing  : Regs::$addr,
            operation   : Regs::$oper,
            cycles      : $cycles,
            has_extra   : $extra,
        }
    )
}

/* WARNING: Branch instructions are replaced with jumps */
const OPCODE_TABLE : [Instruction; 256] = [    
    // 0x00
    inst!(imp, brk, 7, false), inst!(idx, ora, 6, false), 
    inst!(imp, nop, 2, false), inst!(idx, slo, 8, false), 
    inst!(zpg, nop, 3, false), inst!(zpg, ora, 3, false),
    inst!(zpg, asl, 5, false), inst!(zpg, slo, 5, false),
    inst!(imp, php, 3, false), inst!(imm, ora, 2, false),
    inst!(imp, sal, 2, false), inst!(imp, nop, 2, false), 
    inst!(abs, nop, 4, false), inst!(abs, ora, 4, false),
    inst!(abs, asl, 6, false), inst!(abs, slo, 6, false), 
    // 0x10 
    inst!(rel, jmp, 2, true),  inst!(idy, ora, 5, true), 
    inst!(imp, nop, 2, false), inst!(idy, slo, 4, false),
    inst!(zpx, nop, 4, false), inst!(zpx, ora, 4, false),
    inst!(zpx, asl, 6, false), inst!(zpx, slo, 6, false),
    inst!(imp, clc, 2, false), inst!(aby, ora, 4, true),
    inst!(imp, nop, 2, false), inst!(aby, slo, 7, false),
    inst!(abx, nop, 4, true),  inst!(abx, ora, 4, true), 
    inst!(abx, asl, 7, false), inst!(abx, slo, 7, false),
    // 0x20
    inst!(abs, jsr, 6, false), inst!(idx, and, 6, false), 
    inst!(imp, nop, 2, false), inst!(idx, rla, 8, false),
    inst!(zpg, bit, 3, false), inst!(zpg, and, 3, false),
    inst!(zpg, rol, 5, false), inst!(zpg, rla, 5, false),
    inst!(imp, plp, 4, false), inst!(imm, and, 2, false),
    inst!(imp, ral, 2, false), inst!(imp, nop, 2, false),
    inst!(abs, bit, 4, false), inst!(abs, and, 4, false),
    inst!(abs, rol, 6, false), inst!(abs, rla, 6, false),
    // 0x30
    inst!(rel, jmp, 2, true),  inst!(idy, and, 5, true),
    inst!(imp, nop, 2, false), inst!(idy, rla, 8, false),
    inst!(zpx, nop, 4, false), inst!(zpx, and, 4, false),
    inst!(zpx, rol, 6, false), inst!(zpx, rla, 6, false),
    inst!(imp, sec, 2, false), inst!(aby, and, 4, true),
    inst!(imp, nop, 2, false), inst!(aby, rla, 7, true),
    inst!(abx, nop, 4, true),  inst!(abx, and, 4, true),
    inst!(abx, rol, 7, false), inst!(abx, rla, 7, false),
    // 0x40
    inst!(imp, rti, 6, false), inst!(idx, eor, 6, false),
    inst!(imp, nop, 2, false), inst!(idx, sre, 8, false),
    inst!(zpg, nop, 3, false), inst!(zpg, eor, 3, false), 
    inst!(zpg, lsr, 5, false), inst!(zpg, sre, 5, false),
    inst!(imp, pha, 3, false), inst!(imm, eor, 2, false),
    inst!(imp, sar, 2, false), inst!(imp, nop, 2, false),
    inst!(abs, jmp, 3, false), inst!(abs, eor, 4, false),
    inst!(abs, lsr, 6, false), inst!(abs, sre, 6, false),
    // 0x50
    inst!(rel, jmp, 2, true),  inst!(idy, eor, 5, true), 
    inst!(imp, nop, 2, false), inst!(idy, sre, 8, false),
    inst!(zpx, nop, 4, false), inst!(zpx, eor, 4, false),
    inst!(zpx, lsr, 6, false), inst!(zpx, sre, 6, false),
    inst!(imp, cli, 2, false), inst!(aby, eor, 4, true), 
    inst!(imp, nop, 2, false), inst!(aby, sre, 7, false), 
    inst!(abx, nop, 4, true),  inst!(abx, eor, 4, true),
    inst!(abx, lsr, 7, false), inst!(abx, sre, 7, false),
    // 0x60
    inst!(imp, rts, 6, false), inst!(idx, adc, 6, false),
    inst!(imp, nop, 2, false), inst!(idx, rra, 8, false),
    inst!(zpg, nop, 3, false), inst!(zpg, adc, 3, false),
    inst!(zpg, ror, 5, false), inst!(zpg, rra, 5, false),
    inst!(imp, pla, 4, false), inst!(imm, adc, 2, false),
    inst!(imp, rar, 2, false), inst!(imp, nop, 2, false),
    inst!(ind, jmp, 5, false), inst!(abs, adc, 4, false),
    inst!(abs, ror, 6, false), inst!(abs, rra, 6, false),
    // 0x70
    inst!(rel, jmp, 2, true),  inst!(idy, adc, 5, true),
    inst!(imp, nop, 2, false), inst!(idy, rra, 8, false),
    inst!(zpx, nop, 4, false), inst!(zpx, adc, 4, false),
    inst!(zpx, ror, 6, false), inst!(zpx, rra, 6, false),
    inst!(imp, sei, 2, false), inst!(aby, adc, 4, true),
    inst!(imp, nop, 2, false), inst!(aby, rra, 7, false), 
    inst!(abx, nop, 4, true),  inst!(abx, adc, 4, true),
    inst!(abx, ror, 7, false), inst!(abx, rra, 7, false),
    // 0x80
    inst!(imm, nop, 2, false), inst!(idx, sta, 6, false),
    inst!(imm, nop, 2, false), inst!(idx, sax, 6, false),
    inst!(zpg, sty, 3, false), inst!(zpg, sta, 3, false),
    inst!(zpg, stx, 3, false), inst!(zpg, sax, 3, false),
    inst!(imp, dey, 2, false), inst!(imm, nop, 2, false),
    inst!(imp, txa, 2, false), inst!(imp, nop, 2, false),
    inst!(abs, sty, 4, false), inst!(abs, sta, 4, false),
    inst!(abs, stx, 4, false), inst!(abs, sax, 4, false),
    // 0x90
    inst!(rel, jmp, 2, true),  inst!(idy, sta, 6, false),
    inst!(imp, nop, 2, false), inst!(imp, nop, 2, false), 
    inst!(zpx, sty, 4, false), inst!(zpx, sta, 4, false),
    inst!(zpy, stx, 4, false), inst!(zpy, sax, 4, false),
    inst!(imp, tya, 2, false), inst!(aby, sta, 5, false), 
    inst!(imp, txs, 2, false), inst!(imp, nop, 2, false), 
    inst!(imp, nop, 2, false), inst!(abx, sta, 5, false),
    inst!(imp, nop, 2, false), inst!(imp, nop, 2, false),
    // 0xA0
    inst!(imm, ldy, 2, false), inst!(idx, lda, 6, false), 
    inst!(imm, ldx, 2, false), inst!(idx, lax, 6, false),
    inst!(zpg, ldy, 3, false), inst!(zpg, lda, 3, false),
    inst!(zpg, ldx, 3, false), inst!(zpg, lax, 3, false),
    inst!(imp, tay, 2, false), inst!(imm, lda, 2, false),
    inst!(imp, tax, 2, false), inst!(imm, lax, 2, false),
    inst!(abs, ldy, 4, false), inst!(abs, lda, 4, false),
    inst!(abs, ldx, 4, false), inst!(abs, lax, 4, false),
    // 0xB0
    inst!(rel, jmp, 2, true),  inst!(idy, lda, 5, true), 
    inst!(imp, nop, 2, false), inst!(idy, lax, 5, true),
    inst!(zpx, ldy, 4, false), inst!(zpx, lda, 4, false),
    inst!(zpy, ldx, 4, false), inst!(zpy, lax, 4, false),
    inst!(imp, clv, 2, false), inst!(aby, lda, 4, true), 
    inst!(imp, tsx, 2, false), inst!(imp, nop, 2, false),
    inst!(abx, ldy, 4, true),  inst!(abx, lda, 4, true),
    inst!(aby, ldx, 4, true),  inst!(aby, lax, 4, true),
    // 0xC0
    inst!(imm, cpy, 2, false), inst!(idx, cmp, 6, false), 
    inst!(imm, nop, 2, false), inst!(idx, dcp, 8, false), 
    inst!(zpg, cpy, 3, false), inst!(zpg, cmp, 3, false),
    inst!(zpg, dec, 5, false), inst!(zpg, dcp, 5, false),
    inst!(imp, iny, 2, false), inst!(imm, cmp, 2, false),
    inst!(imp, dex, 2, false), inst!(imp, nop, 2, false),
    inst!(abs, cpy, 4, false), inst!(abs, cmp, 4, false),
    inst!(abs, dec, 6, false), inst!(abs, dcp, 6, false),
    // 0xD0
    inst!(rel, jmp, 2, true),  inst!(idy, cmp, 5, true),
    inst!(imp, nop, 2, false), inst!(idy, dcp, 8, false),
    inst!(zpx, nop, 4, false), inst!(zpx, cmp, 4, false),
    inst!(zpx, dec, 6, false), inst!(zpx, dcp, 6, false),
    inst!(imp, cld, 2, false), inst!(aby, cmp, 4, true),
    inst!(imp, nop, 2, false), inst!(aby, dcp, 7, false),
    inst!(abx, nop, 4, true),  inst!(abx, cmp, 4, true),
    inst!(abx, dec, 7, false), inst!(abx, dcp, 7, false),
    // 0xE0
    inst!(imm, cpx, 2, false), inst!(idx, sbc, 6, false), 
    inst!(imm, nop, 2, false), inst!(idx, isc, 8, false),
    inst!(zpg, cpx, 3, false), inst!(zpg, sbc, 3, false),
    inst!(zpg, inc, 6, false), inst!(zpg, isc, 5, false),
    inst!(imp, inx, 2, false), inst!(imm, sbc, 2, false),
    inst!(imp, nop, 2, false), inst!(imm, sbc, 2, false),
    inst!(abs, cpx, 4, false), inst!(abs, sbc, 4, false),
    inst!(abs, inc, 6, false), inst!(abs, isc, 6, false),
    // 0xF0
    inst!(rel, jmp, 2, true),  inst!(idy, sbc, 5, true), 
    inst!(imp, nop, 2, false), inst!(idy, isc, 8, false),
    inst!(zpx, nop, 4, false), inst!(zpx, sbc, 4, false),
    inst!(zpx, inc, 6, false), inst!(zpx, isc, 6, false),
    inst!(imp, sed, 2, false), inst!(aby, sbc, 4, true), 
    inst!(imp, nop, 2, false), inst!(aby, isc, 7, false), 
    inst!(abx, nop, 4, true),  inst!(abx, sbc, 4, true),
    inst!(abx, inc, 7, false), inst!(abx, isc, 7, false),
    ];
