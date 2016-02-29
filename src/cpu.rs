use std::fmt;
use mem::{MemState, Memory as Mem};
use loadstore::LoadStore;
use std::num::Wrapping as W;

#[allow(non_camel_case_types)]
type fn_instruction     = fn(&mut Regs, &mut Mem, W<u16>);  
#[allow(non_camel_case_types)]
type fn_addressing      = fn(&mut Regs, &mut Mem) -> (W<u16>, u32); 

type FnInstruction     = fn(&mut Regs, &mut Mem, W<u16>);  
type FnAddressing      = fn(&mut Regs, &mut Mem) -> (W<u16>, u32); 

/* Branch flag types */
const BRANCH_FLAG_TABLE : [u8; 4] = 
    [FLAG_SIGN, FLAG_OVERFLOW, FLAG_CARRY, FLAG_ZERO];

/* Memory */
const STACK_PAGE        : W<u16> = W(0x0100 as u16); 
const PAGE_MASK         : W<u16> = W(0xFF00 as u16);
const ADDRESS_INTERRUPT : W<u16> = W(0xFFFE as u16);

/* Flag bits */
const FLAG_CARRY        : u8 = 0x01;
const FLAG_ZERO         : u8 = 0x02;
const FLAG_INTERRUPT    : u8 = 0x04;
const FLAG_DECIMAL      : u8 = 0x08;
const FLAG_BRK          : u8 = 0x10;
const FLAG_PUSHED       : u8 = 0x20;
const FLAG_OVERFLOW     : u8 = 0x40;
const FLAG_SIGN         : u8 = 0x80;

const PPUCTRL           : W<u16> = W(0x2000);
const PPUMASK           : W<u16> = W(0x2001);
const PPUSTATUS         : W<u16> = W(0x2002);
const OAMADDR           : W<u16> = W(0x2003);
const OAMDATA           : W<u16> = W(0x2004);
const PPUSCROLL         : W<u16> = W(0x2005);
const PPUADDR           : W<u16> = W(0x2006);
const PPUDATA           : W<u16> = W(0x2007);
const OAMDMA            : W<u16> = W(0x4014);

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
            self.start(memory, page, cycles as u32);
            true
        } else {
            false
        }
    }

    fn start(&mut self, memory: &mut Mem, page: W<u8>, cycles: u32) {
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
    instruction     : FnInstruction,
}

impl Default for Execution {
    fn default() -> Execution {
        Execution {
            cycles_left     : 0,
            instruction     : Regs::nop,
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
            let inst = self.instruction;
            inst(regs, memory, self.address);
            // Load next opcode
            let opcode = regs.load_opcode(memory);
            let instruction = OPCODE_TABLE[opcode as usize]; 
            // Get address and extra cycles from mode
            let (address, extra) = instruction.0(regs, memory);
            // Save the address for next instruction
            self.address = address;
            self.cycles_left = instruction.2; 
            // Add the extra cycles if needed
            if instruction.3 {
                self.cycles_left += extra;
            }
            self.instruction = instruction.1;
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
            SP              : W(0xfd),
            PC              : W(0),
        }
    }
}

// Util functions

impl Regs {

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

    fn load_opcode(&mut self, memory: &mut Mem) -> u8 {
        memory.load(self.PC).0
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
        let opcode = self.load_opcode(memory);
        let index = opcode >> 6;
        let check = ((opcode >> 5) & 1) != 0;
        let next_opcode = self.PC + W(2);
        if is_flag_set!(self.Flags, BRANCH_FLAG_TABLE[index as usize]) != check {
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

    // Implied

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

    fn php (&mut self, memory: &mut Mem, _: W<u16>) {
        // Two bits are set on memory when pushing flags 
        let flags = W(self.Flags | FLAG_PUSHED | FLAG_BRK);
        self.push(memory, flags);
    }

    fn sal (&mut self, _: &mut Mem, _: W<u16>) {
        set_sign_zero_carry_cond!(self.Flags, self.A << 1, self.A & W(0x80) != W(0));
        self.A = self.A << 1;
    }

    fn clc (&mut self, _: &mut Mem, _: W<u16>) {
        unset_flag!(self.Flags, FLAG_CARRY);
    }

    fn plp (&mut self, memory: &mut Mem, _: W<u16>) {
        // Ignore the two bits not present
        self.Flags = self.pop(memory).0 & !(FLAG_PUSHED | FLAG_BRK);
    }

    fn ral (&mut self, _: &mut Mem, _: W<u16>) {
        /* Bit to be rotated into the carry */
        let carry = self.A & W(0x80) != W(0);
        /* We rotate the carry bit into A */
        rol!(self.A, self.Flags);
        /* And we set the Carry accordingly */
        set_sign_zero_carry_cond!(self.Flags, self.A, carry);
    }

    fn sec (&mut self, _: &mut Mem, _: W<u16>) {
        set_flag!(self.Flags, FLAG_CARRY);
    }

    fn pha (&mut self, memory: &mut Mem, _: W<u16>) {
        let a = self.A;
        self.push(memory, a);
    }

    fn sar (&mut self, _: &mut Mem, _: W<u16>) {
        set_sign_zero_carry_cond!(self.Flags, self.A >> 1, self.A & W(1) != W(0));
        self.A = self.A >> 1;
    }

    fn cli (&mut self, _: &mut Mem, _: W<u16>) {
        unset_flag!(self.Flags, FLAG_INTERRUPT);
    }

    fn pla (&mut self, memory: &mut Mem, _: W<u16>) {
        self.A = self.pop(memory);
    }

    fn rar (&mut self, _: &mut Mem, _: W<u16>) {
        /* Bit to be rotated into the carry */
        let carry = self.A & W(1) != W(0);
        /* We rotate the carry bit into a */
        ror!(self.A, self.Flags);
        /* And we set the carry accordingly */
        set_sign_zero_carry_cond!(self.Flags, self.A, carry);
    }

    fn sei (&mut self, _: &mut Mem, _: W<u16>) {
        set_flag!(self.Flags, FLAG_INTERRUPT);
    }

    fn dey (&mut self, _: &mut Mem, _: W<u16>) {
        self.Y = self.Y + W(1);
        set_sign_zero!(self.Flags, self.Y);
    }

    fn txa (&mut self, _: &mut Mem, _: W<u16>) {
        self.A = self.X;
        set_sign_zero!(self.Flags, self.A);
    }

    fn tya (&mut self, _: &mut Mem, _: W<u16>) {
        self.A = self.Y;
        set_sign_zero!(self.Flags, self.A);
    }

    fn txs (&mut self, _: &mut Mem, _: W<u16>) {
        self.SP = self.X;
    }

    fn tay (&mut self, _: &mut Mem, _: W<u16>) {
        self.Y = self.A;
        set_sign_zero!(self.Flags, self.Y);
    }

    fn tax (&mut self, _: &mut Mem, _: W<u16>) {
        self.X = self.A;
        set_sign_zero!(self.Flags, self.X);
    }

    fn clv (&mut self, _: &mut Mem, _: W<u16>) {
        unset_flag!(self.Flags, FLAG_OVERFLOW);
    }

    fn tsx (&mut self, _: &mut Mem, _: W<u16>) {
        self.X = self.SP;
        set_sign_zero!(self.Flags, self.X);
    }

    fn iny (&mut self, _: &mut Mem, _: W<u16>) {
        self.Y = self.Y + W(1);
        set_sign_zero!(self.Flags, self.Y);
    }

    fn dex (&mut self, _: &mut Mem, _: W<u16>) {
        self.X = self.X - W(1);
        set_sign_zero!(self.Flags, self.X);
    }

    fn cld (&mut self, _: &mut Mem, _: W<u16>) {
        unset_flag!(self.Flags, FLAG_DECIMAL);
    }

    fn inx (&mut self, _: &mut Mem, _: W<u16>) {
        self.X = self.X + W(1);
        set_sign_zero!(self.Flags, self.X);
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
        set_sign_zero!(self.Flags, self.A);
    }

    fn asl (&mut self, memory: &mut Mem, address: W<u16>) {
        let m = memory.load(address);
        set_sign_zero_carry_cond!(self.Flags, m << 1, m & W(0x80) != W(0));
        memory.store(address, m << 1);
    }

    fn bit (&mut self, memory: &mut Mem, address: W<u16>) {
        let m = memory.load(address);
        /* We need to set overflow as it is in memory */
        set_flag_cond!(self.Flags, FLAG_OVERFLOW, m & W(0x40) != W(0));
        set_sign!(self.Flags, m); 
        set_zero!(self.Flags, self.A & m);
    }

    fn and (&mut self, memory: &mut Mem, address: W<u16>) {
        let m = memory.load(address);
        self.A = self.A & m;
        set_sign_zero!(self.Flags, self.A);
    }

    fn rol (&mut self, memory: &mut Mem, address: W<u16>) {
        let mut m = memory.load(address);
        /* Bit to be rotated into the carry */
        let carry = m & W(0x80) != W(0);
        /* We rotate the carry bit into m*/
        rol!(m, self.Flags);
        /* and we set the carry accordingly */
        set_sign_zero_carry_cond!(self.Flags, m, carry);
        memory.store(address, m);
    }

    fn eor (&mut self, memory: &mut Mem, address: W<u16>) {
        let m = memory.load(address);
        self.A = m ^ self.A;
        set_sign_zero!(self.Flags, self.A);
    }

    fn lsr (&mut self, memory: &mut Mem, address: W<u16>) {
        let m = memory.load(address);
        set_sign_zero_carry_cond!(self.Flags, m >> 1, m & W(1) != W(0));
        memory.store(address, m >> 1);
    }

    fn adc (&mut self, memory: &mut Mem, address: W<u16>) {
        let m = memory.load(address);
        let v = W16!(self.A) + W16!(m) + W((self.Flags & FLAG_CARRY) as u16);
        self.A = W8!(v);
        set_sign_zero!(self.Flags, self.A);
        set_flag_cond!(self.Flags, FLAG_OVERFLOW | FLAG_CARRY, v > W(0xFF));
    }

    fn ror (&mut self, memory: &mut Mem, address: W<u16>) {
        let mut m = memory.load(address);
        let carry = m & W(1) != W(0);
        ror!(m, self.Flags);
        /* we rotate the carry bit into a */
        /* and we set the carry accordingly */
        set_sign_zero_carry_cond!(self.Flags, m, carry);
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
        set_sign_zero!(self.Flags, self.Y);
    }

    fn ldx (&mut self, memory: &mut Mem, address: W<u16>) {
        self.X = memory.load(address);
        set_sign_zero!(self.Flags, self.X);
    }

    fn lda (&mut self, memory: &mut Mem, address: W<u16>) {
        self.A = memory.load(address);
        set_sign_zero!(self.Flags, self.A);
    }

    fn cpy (&mut self, memory: &mut Mem, address: W<u16>) {
        let m = memory.load(address);
        let comp = W16!(self.Y) - W16!(m);
        set_sign_zero_carry_cond!(self.Flags, W8!(comp), comp <= W(0xFF));
    }

    fn cpx (&mut self, memory: &mut Mem, address: W<u16>) {
        let m = memory.load(address);
        let comp = W16!(self.X) - W16!(m);
        set_sign_zero_carry_cond!(self.Flags, W8!(comp), comp <= W(0xFF));
    }

    fn cmp (&mut self, memory: &mut Mem, address: W<u16>) {
        let m = memory.load(address);
        let comp = W16!(self.A) - W16!(m);
        set_sign_zero_carry_cond!(self.Flags, W8!(comp), comp <= W(0xFF));
    }

    fn dec (&mut self, memory: &mut Mem, address: W<u16>) {
        let mut m = memory.load(address);
        m = m - W(1);
        set_sign_zero!(self.Flags, m);
        memory.store(address, m);
    }


    fn sbc (&mut self, memory: &mut Mem, address: W<u16>) {
        let m = memory.load(address);
        let v = W16!(self.A) - W16!(m) - W((self.Flags & FLAG_CARRY) as u16);
        self.A = W8!(v);
        set_sign_zero!(self.Flags, self.A);
        set_flag_cond!(self.Flags, FLAG_OVERFLOW | FLAG_CARRY, v <= W(0xFF));
    }
   
    fn inc (&mut self, memory: &mut Mem, address: W<u16>) {
        let mut m = memory.load(address);
        m = m + W(1);
        set_sign_zero!(self.Flags, m);
        memory.store(address, m);
    }
}

impl fmt::Debug for Regs {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{{Regs: A: {:#x}, X: {:#x}, Y: {:#x}, P: {:#x}, SP: {:#x}, PC: {:#x} }}",
               self.A.0 , self.X.0 , self.Y.0 , self.Flags , self.SP.0 , self.PC.0)
    }
}


/* WARNING: Branch instructions are replaced with jumps */
/* Addressing, Instruction, Cycles, Has Penalty */
const OPCODE_TABLE : [(FnAddressing, FnInstruction, u32, bool); 256] = [
    (Regs::imp, Regs::brk, 7, false), (Regs::idx, Regs::ora, 6, false), 
    (Regs::imp, Regs::nop, 2, false), (Regs::imp, Regs::nop, 2, false), 
    (Regs::imp, Regs::nop, 2, false), (Regs::zpg, Regs::ora, 3, false),
    (Regs::zpg, Regs::asl, 5, false), (Regs::imp, Regs::nop, 2, false),
    (Regs::imp, Regs::php, 3, false), (Regs::imm, Regs::ora, 2, false),
    (Regs::imp, Regs::sal, 2, false), (Regs::imp, Regs::nop, 2, false), 
    (Regs::imp, Regs::nop, 2, false), (Regs::abs, Regs::ora, 4, false),
    (Regs::abs, Regs::asl, 6, false), (Regs::imp, Regs::nop, 2, false), 
    
    (Regs::rel, Regs::jmp, 2, true),  (Regs::idy, Regs::ora, 5, true), 
    (Regs::imp, Regs::nop, 2, false), (Regs::imp, Regs::nop, 2, false),
    (Regs::imp, Regs::nop, 2, false), (Regs::zpx, Regs::ora, 4, false),
    (Regs::zpx, Regs::asl, 6, false), (Regs::imp, Regs::nop, 2, false),
    (Regs::imp, Regs::clc, 2, false), (Regs::aby, Regs::ora, 4, true),
    (Regs::imp, Regs::nop, 2, false), (Regs::imp, Regs::nop, 2, false),
    (Regs::imp, Regs::nop, 2, false), (Regs::abx, Regs::ora, 4, true), 
    (Regs::abx, Regs::asl, 7, false), (Regs::imp, Regs::nop, 2, false),

    (Regs::abs, Regs::jsr, 6, false), (Regs::idx, Regs::and, 6, false), 
    (Regs::imp, Regs::nop, 2, false), (Regs::imp, Regs::nop, 6, false),
    (Regs::zpg, Regs::bit, 3, false), (Regs::zpg, Regs::and, 3, false),
    (Regs::zpg, Regs::rol, 5, false), (Regs::imp, Regs::nop, 2, false),
    (Regs::imp, Regs::plp, 4, false), (Regs::imm, Regs::and, 2, false),
    (Regs::imp, Regs::ral, 2, false), (Regs::imp, Regs::nop, 2, false),
    (Regs::abs, Regs::bit, 4, false), (Regs::abs, Regs::and, 4, false),
    (Regs::abs, Regs::rol, 6, false), (Regs::imp, Regs::nop, 2, false),

    (Regs::rel, Regs::jmp, 2, true),  (Regs::idy, Regs::and, 5, true),
    (Regs::imp, Regs::nop, 2, false), (Regs::imp, Regs::nop, 2, false),
    (Regs::imp, Regs::nop, 2, false), (Regs::zpx, Regs::and, 4, false),
    (Regs::zpx, Regs::rol, 6, false), (Regs::imp, Regs::nop, 2, false),
    (Regs::imp, Regs::sec, 2, false), (Regs::aby, Regs::and, 4, true),
    (Regs::imp, Regs::nop, 2, false), (Regs::imp, Regs::nop, 2, true),
    (Regs::imp, Regs::nop, 2, false), (Regs::abx, Regs::and, 4, true),
    (Regs::abx, Regs::rol, 7, false), (Regs::imp, Regs::nop, 2, false),

    (Regs::imp, Regs::rti, 6, false), (Regs::idx, Regs::eor, 6, false),
    (Regs::imp, Regs::nop, 2, false), (Regs::imp, Regs::nop, 2, false),
    (Regs::imp, Regs::nop, 2, false), (Regs::zpg, Regs::eor, 3, false), 
    (Regs::zpg, Regs::lsr, 5, false), (Regs::imp, Regs::nop, 2, false),
    (Regs::imp, Regs::pha, 3, false), (Regs::imm, Regs::eor, 2, false),
    (Regs::imp, Regs::sar, 2, false), (Regs::imp, Regs::nop, 2, false),
    (Regs::abs, Regs::jmp, 3, false), (Regs::abs, Regs::eor, 4, false),
    (Regs::abs, Regs::lsr, 6, false), (Regs::imp, Regs::nop, 2, false),

    (Regs::rel, Regs::jmp, 2, true),  (Regs::idy, Regs::eor, 5, true), 
    (Regs::imp, Regs::nop, 2, false), (Regs::imp, Regs::nop, 2, false),
    (Regs::imp, Regs::nop, 2, false), (Regs::zpx, Regs::eor, 4, false),
    (Regs::zpx, Regs::lsr, 6, false), (Regs::imp, Regs::nop, 2, false),
    (Regs::imp, Regs::cli, 2, false), (Regs::aby, Regs::eor, 4, true), 
    (Regs::imp, Regs::nop, 2, false), (Regs::imp, Regs::nop, 2, false), 
    (Regs::imp, Regs::nop, 2, false), (Regs::abx, Regs::eor, 4, true),
    (Regs::abx, Regs::lsr, 7, false), (Regs::imp, Regs::nop, 2, false),

    (Regs::imp, Regs::rts, 6, false), (Regs::idx, Regs::adc, 6, false),
    (Regs::imp, Regs::nop, 2, false), (Regs::imp, Regs::nop, 2, false),
    (Regs::imp, Regs::nop, 2, false), (Regs::zpg, Regs::adc, 3, false),
    (Regs::zpg, Regs::ror, 5, false), (Regs::imp, Regs::nop, 2, false),
    (Regs::imp, Regs::pla, 4, false), (Regs::imm, Regs::adc, 2, false),
    (Regs::imp, Regs::rar, 2, false), (Regs::imp, Regs::nop, 2, false),
    (Regs::ind, Regs::jmp, 5, false), (Regs::abs, Regs::adc, 4, false),
    (Regs::abs, Regs::ror, 6, false), (Regs::imp, Regs::nop, 2, false),

    (Regs::rel, Regs::jmp, 2, true),  (Regs::idy, Regs::adc, 5, true),
    (Regs::imp, Regs::nop, 2, false), (Regs::imp, Regs::nop, 2, false),
    (Regs::imp, Regs::nop, 2, false), (Regs::zpx, Regs::adc, 4, false),
    (Regs::zpx, Regs::ror, 6, false), (Regs::imp, Regs::nop, 2, false),
    (Regs::imp, Regs::sei, 2, false), (Regs::aby, Regs::adc, 4, true),
    (Regs::imp, Regs::nop, 2, false), (Regs::imp, Regs::nop, 2, false), 
    (Regs::imp, Regs::nop, 2, false), (Regs::abx, Regs::adc, 4, true),
    (Regs::abx, Regs::ror, 7, false), (Regs::imp, Regs::nop, 2, false),

    (Regs::imp, Regs::nop, 2, false), (Regs::idx, Regs::sta, 6, false),
    (Regs::imp, Regs::nop, 2, false), (Regs::imp, Regs::nop, 2, false),
    (Regs::zpg, Regs::sty, 3, false), (Regs::zpg, Regs::sta, 3, false),
    (Regs::zpg, Regs::stx, 3, false), (Regs::imp, Regs::nop, 2, false),
    (Regs::imp, Regs::dey, 2, false), (Regs::imp, Regs::nop, 2, false),
    (Regs::imp, Regs::txa, 2, false), (Regs::imp, Regs::nop, 2, false),
    (Regs::abs, Regs::sty, 4, false), (Regs::abs, Regs::sta, 4, false),
    (Regs::abs, Regs::stx, 4, false), (Regs::imp, Regs::nop, 2, false),

    (Regs::rel, Regs::jmp, 2, true),  (Regs::idy, Regs::sta, 6, false),
    (Regs::imp, Regs::nop, 2, false), (Regs::imp, Regs::nop, 2, false), 
    (Regs::zpx, Regs::sty, 4, false), (Regs::zpx, Regs::sta, 4, false),
    (Regs::zpy, Regs::stx, 4, false), (Regs::imp, Regs::nop, 2, false),
    (Regs::imp, Regs::tya, 2, false), (Regs::aby, Regs::sta, 5, false), 
    (Regs::imp, Regs::txs, 2, false), (Regs::imp, Regs::nop, 2, false), 
    (Regs::imp, Regs::nop, 2, false), (Regs::abx, Regs::sta, 5, false),
    (Regs::imp, Regs::nop, 2, false), (Regs::imp, Regs::nop, 2, false),

    (Regs::imm, Regs::ldy, 2, false), (Regs::idx, Regs::lda, 6, false), 
    (Regs::imm, Regs::ldx, 2, false), (Regs::imp, Regs::nop, 2, false),
    (Regs::zpg, Regs::ldy, 3, false), (Regs::zpg, Regs::lda, 3, false),
    (Regs::zpg, Regs::ldx, 3, false), (Regs::imp, Regs::nop, 2, false),
    (Regs::imp, Regs::tay, 2, false), (Regs::imm, Regs::lda, 2, false),
    (Regs::imp, Regs::tax, 2, false), (Regs::imp, Regs::nop, 2, false),
    (Regs::abs, Regs::ldy, 4, false), (Regs::abs, Regs::lda, 4, false),
    (Regs::abs, Regs::ldx, 4, false), (Regs::imp, Regs::nop, 4, false),

    (Regs::rel, Regs::jmp, 2, true),  (Regs::idy, Regs::lda, 5, true), 
    (Regs::imp, Regs::nop, 2, false), (Regs::imp, Regs::nop, 2, false),
    (Regs::zpx, Regs::ldy, 4, false), (Regs::zpx, Regs::lda, 4, false),
    (Regs::zpy, Regs::ldx, 4, false), (Regs::imp, Regs::nop, 2, false),
    (Regs::imp, Regs::clv, 2, false), (Regs::aby, Regs::lda, 4, true), 
    (Regs::imp, Regs::tsx, 2, false), (Regs::imp, Regs::nop, 2, false),
    (Regs::abx, Regs::ldy, 4, true),  (Regs::abx, Regs::lda, 4, true),
    (Regs::aby, Regs::ldx, 4, true),  (Regs::imp, Regs::nop, 2, false),

    (Regs::imm, Regs::cpy, 2, false), (Regs::idx, Regs::cmp, 6, false), 
    (Regs::imp, Regs::nop, 2, false), (Regs::imp, Regs::nop, 2, false), 
    (Regs::zpg, Regs::cpy, 3, false), (Regs::zpg, Regs::cmp, 3, false),
    (Regs::zpg, Regs::dec, 5, false), (Regs::imp, Regs::nop, 2, false),
    (Regs::imp, Regs::iny, 2, false), (Regs::imm, Regs::cmp, 2, false),
    (Regs::imp, Regs::dex, 2, false), (Regs::imp, Regs::nop, 2, false),
    (Regs::abs, Regs::cpy, 4, false), (Regs::abs, Regs::cmp, 4, false),
    (Regs::abs, Regs::dec, 6, false), (Regs::imp, Regs::nop, 2, false),

    (Regs::rel, Regs::jmp, 2, true),  (Regs::idy, Regs::cmp, 5, true),
    (Regs::imp, Regs::nop, 2, false), (Regs::imp, Regs::nop, 2, false),
    (Regs::imp, Regs::nop, 2, false), (Regs::zpx, Regs::cmp, 4, false),
    (Regs::zpx, Regs::dec, 6, false), (Regs::imp, Regs::nop, 2, false),
    (Regs::imp, Regs::cld, 2, false), (Regs::aby, Regs::cmp, 4, true),
    (Regs::imp, Regs::nop, 2, false), (Regs::imp, Regs::nop, 2, false),
    (Regs::imp, Regs::nop, 2, false), (Regs::abx, Regs::cmp, 4, true),
    (Regs::abx, Regs::dec, 7, false), (Regs::imp, Regs::nop, 2, false),

    (Regs::imm, Regs::cpx, 2, false), (Regs::idx, Regs::sbc, 6, false), 
    (Regs::imp, Regs::nop, 2, false), (Regs::imp, Regs::nop, 2, false),
    (Regs::zpg, Regs::cpx, 3, false), (Regs::zpg, Regs::sbc, 3, false),
    (Regs::zpg, Regs::inc, 6, false), (Regs::imp, Regs::nop, 2, false),
    (Regs::imp, Regs::inx, 2, false), (Regs::imm, Regs::sbc, 2, false),
    (Regs::imp, Regs::nop, 2, false), (Regs::imp, Regs::nop, 2, false),
    (Regs::abs, Regs::cpx, 4, false), (Regs::abs, Regs::sbc, 4, false),
    (Regs::abs, Regs::inc, 6, false), (Regs::imp, Regs::nop, 2, false),

    (Regs::rel, Regs::jmp, 2, true),  (Regs::idy, Regs::sbc, 5, true), 
    (Regs::imp, Regs::nop, 2, false), (Regs::imp, Regs::nop, 2, false),
    (Regs::imp, Regs::nop, 2, false), (Regs::zpx, Regs::sbc, 4, false),
    (Regs::zpx, Regs::inc, 6, false), (Regs::imp, Regs::nop, 2, false),
    (Regs::imp, Regs::sed, 2, false), (Regs::aby, Regs::sbc, 4, true), 
    (Regs::imp, Regs::nop, 2, false), (Regs::imp, Regs::nop, 2, false), 
    (Regs::imp, Regs::nop, 2, false), (Regs::abx, Regs::sbc, 4, true),
    (Regs::abx, Regs::inc, 7, false), (Regs::imp, Regs::nop, 2, false),
    ];

