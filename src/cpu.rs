use std::fmt;
use mem::{MemState, Memory as Mem};
use std::num::Wrapping as W;

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

#[allow(non_snake_case)]
pub struct CPU {
    A               : W<u8>,    // Accumulator
    X               : W<u8>,    // Indexes
    Y               : W<u8>,    //
    Flags           : u8,       // Status
    SP              : W<u8>,    // Stack pointer
    PC              : W<u16>,   // Program counter

    // Stores the cycles left to execute next_inst
    cycles_left     : u32,      
    // The instruction to execute when cycles_left is 0
    next_inst       : fn(&mut CPU, &mut Mem, W<u16>), 
    // The address for the instruction
    next_addr       : W<u16>,

    dma_address     : W<u16>,
    cycle_parity    : bool,
    dma_cycles      : u16,
    dma_read        : bool,
    dma_length      : u16,
    dma_value       : W<u8>,
} 

impl CPU {
    pub fn new() -> CPU {
        CPU {
            A               : W(0),
            X               : W(0),
            Y               : W(0),
            Flags           : 0x34, 
            SP              : W(0xfd),
            PC              : W(0),

            cycles_left     : 0,
            next_inst       : CPU::nop,
            next_addr       : W(0),
            

            dma_address     : W(0),
            cycle_parity    : true,
            dma_cycles      : 0,
            dma_read        : true,
            dma_length      : 513,
            dma_value       : W(0),
        }
    }

    pub fn execute(&mut self, memory: &mut Mem) {
        if !memory.dma { 
            if self.cycles_left == 0 {
                // Execute the next instruction
                let inst = self.next_inst;
                let addr = self.next_addr;
                inst(self, memory, addr);
                // Load next opcode
                let opcode = memory.load(self.PC).0;
                let instruction = OPCODE_TABLE[opcode as usize]; 
                // Get address and extra cycles from mode
                let (address, extra) = instruction.0(self, memory);
                // Save the address for next instruction
                self.next_addr = address;
                // Add the extra cycle if needed
                self.cycles_left = instruction.2; 
                if instruction.3 {
                    self.cycles_left += extra;
                }
                self.next_inst = instruction.1;
            }
            self.cycles_left -= 1;
        } else {
            self.dma(memory);
        }
        self.cycle_parity = !self.cycle_parity;
    }

    fn dma(&mut self, memory: &mut Mem){
        if let MemState::Oamdma = memory.write_status {
            self.dma_address = W((memory.oamdma as u16) << 8); 
            memory.write_status = MemState::NoState;
        } // We copy the page adress we wrote to oamdma. 

        if self.cycle_parity && self.dma_cycles == 1 || self.dma_cycles > 2 {
            if self.dma_read {
                self.dma_value = memory.load(self.dma_address);
                self.dma_read = !self.dma_read;
                self.dma_address = self.dma_address + W(1);
            } else {
                memory.store(OAMDATA, self.dma_value);
                self.dma_read = !self.dma_read;
            }
        } else if self.dma_cycles == 0 {
        
        } else {
            // one cycle more for parity
            self.dma_length +=1;        // To see if we have to do 514 or 513 cycles.
        }

        self.dma_cycles += 1;

        if self.dma_cycles == self.dma_length {
            memory.dma = false;
            self.dma_cycles = 0;
            self.dma_length = 513;
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

impl CPU {   
    
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

impl fmt::Display for CPU {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{{ A: {:#x}, X: {:#x}, Y: {:#x}, P: {:#x}, SP: {:#x}, PC: {:#x} }}",
               self.A.0 , self.X.0 , self.Y.0 , self.Flags , self.SP.0 , self.PC.0)
    }
}

impl fmt::Debug for CPU {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut output = "CPU: ".to_string();
        output.push_str(&format!("{{ A: {:#x}, X: {:#x}, Y: {:#x}, P: {:#x}, SP: {:#x}, PC: {:#x} }}",
               self.A.0 , self.X.0 , self.Y.0 , self.Flags , self.SP.0 , self.PC.0));
        write!(f, "{}", output)
    }
}


/* WARNING: Branch instructions are replaced with jumps */
/* Addressing, Instruction, Cycles, Has Penalty */
const OPCODE_TABLE : [(fn(&mut CPU, &mut Mem) -> (W<u16>, u32),
                       fn(&mut CPU, &mut Mem, W<u16>), u32, bool); 256] = [
    (CPU::imp, CPU::brk, 7, false), (CPU::idx, CPU::ora, 6, false), 
    (CPU::imp, CPU::nop, 2, false), (CPU::imp, CPU::nop, 2, false), 
    (CPU::imp, CPU::nop, 2, false), (CPU::zpg, CPU::ora, 3, false),
    (CPU::zpg, CPU::asl, 5, false), (CPU::imp, CPU::nop, 2, false),
    (CPU::imp, CPU::php, 3, false), (CPU::imm, CPU::ora, 2, false),
    (CPU::imp, CPU::sal, 2, false), (CPU::imp, CPU::nop, 2, false), 
    (CPU::imp, CPU::nop, 2, false), (CPU::abs, CPU::ora, 4, false),
    (CPU::abs, CPU::asl, 6, false), (CPU::imp, CPU::nop, 2, false), 
    
    (CPU::rel, CPU::jmp, 2, true), (CPU::idy, CPU::ora, 5, true), 
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
    (CPU::imp, CPU::ral, 2, false), (CPU::imp, CPU::nop, 2, false),
    (CPU::abs, CPU::bit, 4, false), (CPU::abs, CPU::and, 4, false),
    (CPU::abs, CPU::rol, 6, false), (CPU::imp, CPU::nop, 2, false),

    (CPU::rel, CPU::jmp, 2, true), (CPU::idy, CPU::and, 5, true),
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
    (CPU::imp, CPU::sar, 2, false), (CPU::imp, CPU::nop, 2, false),
    (CPU::abs, CPU::jmp, 3, false), (CPU::abs, CPU::eor, 4, false),
    (CPU::abs, CPU::lsr, 6, false), (CPU::imp, CPU::nop, 2, false),

    (CPU::rel, CPU::jmp, 2, true), (CPU::idy, CPU::eor, 5, true), 
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
    (CPU::imp, CPU::rar, 2, false), (CPU::imp, CPU::nop, 2, false),
    (CPU::ind, CPU::jmp, 5, false), (CPU::abs, CPU::adc, 4, false),
    (CPU::abs, CPU::ror, 6, false), (CPU::imp, CPU::nop, 2, false),

    (CPU::rel, CPU::jmp, 2, true), (CPU::idy, CPU::adc, 5, true),
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

    (CPU::rel, CPU::jmp, 2, true), (CPU::idy, CPU::sta, 6, false),
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

    (CPU::rel, CPU::jmp, 2, true), (CPU::idy, CPU::lda, 5, true), 
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

    (CPU::rel, CPU::jmp, 2, true), (CPU::idy, CPU::cmp, 5, true),
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

    (CPU::rel, CPU::jmp, 2, true), (CPU::idy, CPU::sbc, 5, true), 
    (CPU::imp, CPU::nop, 2, false), (CPU::imp, CPU::nop, 2, false),
    (CPU::imp, CPU::nop, 2, false), (CPU::zpx, CPU::sbc, 4, false),
    (CPU::zpx, CPU::inc, 6, false), (CPU::imp, CPU::nop, 2, false),
    (CPU::imp, CPU::sed, 2, false), (CPU::aby, CPU::sbc, 4, true), 
    (CPU::imp, CPU::nop, 2, false), (CPU::imp, CPU::nop, 2, false), 
    (CPU::imp, CPU::nop, 2, false), (CPU::abx, CPU::sbc, 4, true),
    (CPU::abx, CPU::inc, 7, false), (CPU::imp, CPU::nop, 2, false),
    ];

