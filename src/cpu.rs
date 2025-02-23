use super::print_title;
use bitflags::bitflags;
use std::fmt;
use std::time::Duration;

mod group1_test;
mod group3_test;
mod op;
mod sb1_test;
mod sb2_test;
mod test_fn;

type Byte = u8;

bitflags! {
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct CpuFlags: u8 {
        const CARRY             = 0b00000001;
        const ZERO              = 0b00000010;
        const INTERRUPT_DISABLE = 0b00000100;
        const DECIMAL_MODE      = 0b00001000;
        const BREAK             = 0b00010000;
        const BREAK2            = 0b00100000;
        const OVERFLOW          = 0b01000000;
        const NEGATIVE          = 0b10000000;
    }
}

pub struct CPU {
    pub pc: u16,
    pub a: Byte,
    pub x: Byte,
    pub y: Byte,
    pub sp: Byte,
    pub flags: CpuFlags,
    // [0x8000... 0xFFFF] is reserved for program ROM
    pub memory: [u8; 0xFFFF],
    pub clock_time: Duration,
}

#[derive(Debug)]
#[allow(non_camel_case_types)]
pub enum AddressingMode {
    Immediate,
    ZeroPage,
    ZeroPage_X,
    ZeroPage_Y,
    Absolute,
    Absolute_X,
    Absolute_Y,
    Indirect_X,
    Indirect_Y,
    NoneAddressing,
    Accumulator, // Retrieves the vlaue of the accumulator
}

impl fmt::Display for AddressingMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AddressingMode::Immediate => write!(f, "Immediate"),
            AddressingMode::ZeroPage => write!(f, "ZeroPage"),
            AddressingMode::ZeroPage_X => write!(f, "ZeroPage_X"),
            AddressingMode::ZeroPage_Y => write!(f, "ZeroPage_Y"),
            AddressingMode::Absolute => write!(f, "Absolute"),
            AddressingMode::Absolute_X => write!(f, "Absolute_X"),
            AddressingMode::Absolute_Y => write!(f, "Absolute_Y"),
            AddressingMode::Indirect_X => write!(f, "Indirect_X"),
            AddressingMode::Indirect_Y => write!(f, "Indirect_Y"),
            AddressingMode::NoneAddressing => write!(f, "NoneAddressing"),
            AddressingMode::Accumulator => write!(f, "Accumulator"),
        }
    }
}

const STACK_RESET: u8 = 0xFD;
const STACK: u16 = 0x0100;

impl CPU {
    pub fn new() -> Self {
        CPU {
            pc: 0,
            a: 0,
            x: 0,
            y: 0,
            sp: STACK_RESET,
            flags: CpuFlags::from_bits_truncate(0b00100100),
            memory: [0; 0xFFFF],
            clock_time: Duration::from_millis(1), // Example value
        }
    }

    // Used to read address in little endian
    fn mem_read_u16(&mut self, pos: u16) -> u16 {
        // If interrupt request is enabled, stop program exectuion
        if pos == 0xFFFE && self.flags.contains(CpuFlags::INTERRUPT_DISABLE) {
            // BUG Used for irq handler, mitigating for now
            println!("mem_read_u16: Detected break. Reading from IRQ handler...");
            return 0xFFFF;
        }
        let lo = self.mem_read(pos) as u16;
        let hi = self.mem_read(pos + 1) as u16;
        (hi << 8) | (lo as u16)
    }

    // Writes to address in terms of little endian
    fn mem_write_u16(&mut self, pos: u16, data: u16) {
        let hi = (data >> 8) as u8;
        let lo = (data & 0xff) as u8;
        self.mem_write(pos, lo);
        self.mem_write(pos + 1, hi);
    }

    // Resets RAM from $0000 to $07FF
    fn ram_reset(&mut self) {
        for i in 0x0..0x07FF {
            self.memory[i] = 0;
        }
    }

    // Restores registers and initalizes PC to the 2 byte value at 0xFFFC
    pub fn reset(&mut self) {
        println!("reset: Initalized");
        self.a = 0;
        self.x = 0;
        self.y = 0;
        self.flags = CpuFlags::from_bits_truncate(0b00100100);
        self.sp = STACK_RESET;
        self.pc = self.mem_read_u16(0xFFFC);
        self.ram_reset();
    }

    pub fn load(&mut self, program: Vec<u8>) {
        println!("load: Initalized");
        self.memory[0x8000..(0x8000 + program.len())].copy_from_slice(&program[..]);
        self.mem_write_u16(0xFFFC, 0x8000); // Save reference to program in 0xFFFC
        println!("load: Finished!");
    }

    // This function is meant for testing, where the test can insert their own values afterwards
    pub fn load_and_reset(&mut self, program: Vec<u8>) {
        self.load(program);
        self.reset();
    }

    pub fn load_and_run(&mut self, program: Vec<u8>) {
        println!("load_and_run: Initalized");
        self.load(program);
        self.reset();
        self.run();
    }

    // used to update the flag based on the given values
    // Z = result == 0
    // N = result bit 7
    pub fn zero_negative_flag(&mut self, value: Byte) {
        if value == 0 {
            self.flags.insert(CpuFlags::ZERO);
        } else {
            self.flags.remove(CpuFlags::ZERO);
        }

        let neg_flag = value >> 7;

        if neg_flag == 1 {
            self.flags.insert(CpuFlags::NEGATIVE);
        } else {
            self.flags.remove(CpuFlags::NEGATIVE);
        }
    }

    fn mem_read(&mut self, addr: u16) -> Byte {
        println!("mem_read: addr is {}", addr);
        let ret = self.memory[addr as usize];
        // self.pc = self.pc.wrapping_add(1);
        ret
    }

    fn mem_write(&mut self, addr: u16, data: u8) {
        let ret = self.memory[addr as usize] = data;
        // self.pc = self.pc.wrapping_add(1);
        ret
    }

    pub fn run(&mut self) {
        println!("run: Initalized");
        loop {
            println!("run: Reading values, starting with pc {}", self.pc);
            if self.pc == 0xFFFF && self.flags.contains(CpuFlags::INTERRUPT_DISABLE) {
                println!("run: IRQ detected, most likely from a brk. Stopping execution...");
                break;
            }
            let op = self.mem_read(self.pc);

            let highnibble = op >> 4;
            let lownibble = op & 0x0F;
            println!(
                "run: Highnibble is {:#x} and lownibble is {:#x}",
                highnibble, lownibble
            );
            let aaa = op >> 5;
            let bbb = (op >> 2) & 0x7;
            let cc = op & 0x3; // Used for identification of group 1, 2, and 3
            println!(
                "run: aaa is {:03b}, bbb is {:03b}, cc is {:02b}",
                aaa, bbb, cc
            );
            // Top is hard coding remaining instructions
            if op == 0x0 {
                self.brk();
            } else if op == 0x20 {
                self.jsr();
            } else if op == 0x40 {
                self.rti();
            } else if op == 0x60 {
                self.rts();
            } else if lownibble == 0x8 {
                self.sb_one(highnibble);
            } else if lownibble == 0xA && highnibble >= 0x8 {
                self.sb_two(highnibble);
            } else if cc == 0b01 {
                self.group_one(aaa, bbb, cc);
            } else if cc == 0b10 {
                self.group_two(aaa, bbb, cc);
            } else if cc == 0b00 {
                // Conditionals are also included in here
                self.group_three(aaa, bbb, cc);
            } else if cc == 0b11 {
                unimplemented!("cc = 11 is not implemented. This is fulfilled by the 65816 cpu.")
            } else {
                unimplemented!("run: Unknown opcode {:#x}", op)
            }
            // Second IRQ check, as self.pc addition occurs after pc is set to 0xFFFF
            if self.pc == 0xFFFF && self.flags.contains(CpuFlags::INTERRUPT_DISABLE) {
                println!("run: IRQ detected, most likely from a brk. Stopping execution...");
                break;
            }
            // NOTE: Before this runs, PC must be at the instruction before the next command
            // BUG: This may affect branching and needs to be adjusted
            self.pc = self.pc.wrapping_add(1);
        }
        // CLeaning program ROM
        for i in 0x8000..=0xFFFE {
            self.memory[i] = 0;
        }
        print_title!("End of current execution");
    }

    fn get_operand_address(&mut self, mode: &AddressingMode) -> u16 {
        println!("get_operand_address: Initalized");
        self.pc = self.pc.wrapping_add(1);
        match mode {
            AddressingMode::Immediate => self.pc,

            AddressingMode::Accumulator => self.a as u16,

            AddressingMode::ZeroPage => self.mem_read(self.pc) as u16,

            AddressingMode::Absolute => {
                let ret = self.mem_read_u16(self.pc);
                self.pc = self.pc.wrapping_add(1);
                ret
            }

            AddressingMode::ZeroPage_X => {
                let pos = self.mem_read(self.pc);
                let addr = pos.wrapping_add(self.x) as u16;
                addr
            }
            AddressingMode::ZeroPage_Y => {
                let pos = self.mem_read(self.pc);
                let addr = pos.wrapping_add(self.y) as u16;
                addr
            }

            AddressingMode::Absolute_X => {
                println!("get_operand_address: In Absolute_X");
                let base = self.mem_read_u16(self.pc);
                self.pc = self.pc.wrapping_add(1);
                let addr = base.wrapping_add(self.x as u16);
                addr
            }
            AddressingMode::Absolute_Y => {
                let base = self.mem_read_u16(self.pc);
                self.pc = self.pc.wrapping_add(1);
                let addr = base.wrapping_add(self.y as u16);
                addr
            }

            // (c0, X)
            // Looks at the address at LSB = c0 + X and MSB = c0 + X + 1 => Address LSB + MSB
            AddressingMode::Indirect_X => {
                println!("get_operand_address: In Indirect_X");
                let base = self.mem_read(self.pc);

                let ptr: u8 = (base as u8).wrapping_add(self.x);
                let lo = self.mem_read(ptr as u16);
                let hi = self.mem_read(ptr.wrapping_add(1) as u16);
                (hi as u16) << 8 | (lo as u16)
            }
            //($c0), Y
            // Look at address at LSB = c0 and MSB = C0 + 1 => Address LSB + MSB + Y
            AddressingMode::Indirect_Y => {
                let base = self.mem_read(self.pc);

                let lo = self.mem_read(base as u16);
                let hi = self.mem_read((base as u8).wrapping_add(1) as u16);
                let deref_base = (hi as u16) << 8 | (lo as u16);
                let deref = deref_base.wrapping_add(self.y as u16);
                deref
            }

            AddressingMode::NoneAddressing => {
                panic!("mode {:?} is not supported", mode);
            }
        }
    }

    fn stack_push(&mut self, data: u8) {
        self.mem_write((STACK as u16) + self.sp as u16, data);
        self.sp = self.sp.wrapping_sub(1);
    }

    fn stack_push_u16(&mut self, data: u16) {
        let hi = (data >> 8) as u8;
        let lo = (data & 0xFF) as u8;
        self.stack_push(hi);
        self.stack_push(lo);
    }

    fn stack_pop(&mut self) -> u8 {
        self.sp = self.sp.wrapping_add(1);
        let ret = self.mem_read((STACK as u16) + self.sp as u16);
        println!("stack_pop: popped {}", ret);
        ret
    }

    fn stack_pop_u16(&mut self) -> u16 {
        let lo = self.stack_pop() as u16;
        let hi = self.stack_pop() as u16;
        hi << 8 | lo
    }

    // PHP(push processor status) stores a Byte to the stack containing the flags NV11DDIZC and decrements stack pointer
    // Note B Flag is marked as 1 for PHP
    fn php(&mut self) {
        println!("php: Initialized- PC is {}", self.pc);
        let mut flags = self.flags.clone();
        flags.insert(CpuFlags::BREAK);
        flags.insert(CpuFlags::BREAK2);
        self.stack_push(flags.bits());
        println!("php: Finished execution- PC is {}", self.pc);
    }

    fn plp(&mut self) {
        self.flags = CpuFlags::from_bits_truncate(self.stack_pop());
    }

    fn pha(&mut self) {
        println!("pha: Initalized");
        self.stack_push(self.a);
        println!(
            "pha: Pushed {}",
            self.memory[(0x0100 + self.sp.wrapping_add(1) as u16) as usize]
        );
    }

    fn pla(&mut self) {
        self.a = self.stack_pop();
        self.zero_negative_flag(self.a);
        println!("pla: pulled {}", self.a);
    }

    fn dey(&mut self) {
        self.y = self.y.wrapping_sub(1);
        self.zero_negative_flag(self.y);
    }

    fn tya(&mut self) {
        self.a = self.y;
        self.zero_negative_flag(self.a);
    }

    fn tay(&mut self) {
        self.y = self.a;
        self.zero_negative_flag(self.y);
    }

    fn iny(&mut self) {
        self.y = self.y.wrapping_add(1);
        self.zero_negative_flag(self.y)
    }

    fn inx(&mut self) {
        println!("inx: Initalized(Incrementing x)");
        self.x = self.x.wrapping_add(1);
        self.zero_negative_flag(self.x);
    }

    // Used for grouping addressing modes
    fn sb_one(&mut self, highnibble: u8) {
        println!("sb_one: Initalized");
        // Single Byte instructions, don't need to read Bytes past the value
        // Eg. PHP, CLC, INX
        // lower nibble of opcode is 0x_8(eg. 0x08...0xF8)
        // Pattern represents (_ _ _ _ 1000)
        match highnibble {
            0 => self.php(),
            // CLC clears Carry flag
            1 => {
                println!("clc: Initalized");
                self.flags.remove(CpuFlags::CARRY);
                println!("clc: Flags are now {:#b}", self.flags);
            }
            2 => self.plp(),
            // SEC(set carry) sets carry flag to 1
            3 => self.flags.insert(CpuFlags::CARRY),
            // PHA(Push A) stores the value of A to the current stack position
            4 => self.pha(),
            // CLI(Clear Interrupt Disable) clears the interrupt disable flag
            5 => self.flags.remove(CpuFlags::INTERRUPT_DISABLE),
            // PLA(Pull A) increments the stack pointer and loads the value at that stack position into A
            6 => self.pla(),
            //SEI(Set Interrupt Disable) sets the interrupt disable flag
            7 => self.flags.insert(CpuFlags::INTERRUPT_DISABLE),
            // DEY subtracts 1 from the Y register
            8 => self.dey(),
            // TYA transfers the Y register to the accumulator
            9 => self.tya(),
            // TAY transfer accumulator to Y register
            10 => self.tay(),
            // CLV clears the overflow tag
            11 => self.flags.remove(CpuFlags::OVERFLOW),
            // INY increases the Y register
            12 => self.iny(),
            // CLD clears the decimal flag
            13 => self.flags.remove(CpuFlags::DECIMAL_MODE),
            // INX increases the X register
            14 => self.inx(),
            // SED sets the decimal flag
            15 => self.flags.insert(CpuFlags::DECIMAL_MODE),
            _ => unimplemented!("Unknown high nibble {} for SB1)", highnibble),
        }
    }

    fn txa(&mut self) {
        self.a = self.x;
        self.zero_negative_flag(self.a);
    }

    fn tax(&mut self) {
        self.x = self.a;
        self.zero_negative_flag(self.x);
    }

    fn tsx(&mut self) {
        println!("tsx: Initalized. Stack pointer is {}", self.sp);
        self.x = self.sp;
        self.zero_negative_flag(self.x);
    }

    fn dex(&mut self) {
        self.x = self.x.wrapping_sub(1);
        self.zero_negative_flag(self.x);
    }

    pub fn sb_two(&mut self, highnibble: u8) {
        // Group 2 single byte instructions, lownibble A and high nibble >= 8
        println!("sb_two: Initalized");
        match highnibble {
            // TXA
            8 => self.txa(),
            // TXS
            9 => self.sp = self.x,
            // TAX
            10 => self.tax(),
            11 => self.tsx(),
            12 => self.dex(),
            13 => unimplemented!("Phx not implemented"),
            // NOP
            14 => return,
            15 => unimplemented!("Plx not implemented"),
            _ => unimplemented!("Unknown highnibble {} with low nibble 0xA(SB2)", highnibble),
        }
    }

    // Used to determine addressing mode based on bbb bits
    // Can call get_operand_address to then determine how to recieve fields
    fn group_one_bbb(&mut self, bbb: u8) -> AddressingMode {
        match bbb {
            0 => AddressingMode::Indirect_X,
            1 => AddressingMode::ZeroPage,
            2 => AddressingMode::Immediate,
            3 => AddressingMode::Absolute,
            4 => AddressingMode::Indirect_Y,
            5 => AddressingMode::ZeroPage_X,
            6 => AddressingMode::Absolute_Y,
            7 => AddressingMode::Absolute_X,
            _ => {
                unimplemented!("Unknown addressing mode for group 1 bbb {}", bbb);
            }
        }
    }

    // Takes in the address location
    fn ora(&mut self, addr: u16) {
        let cmp = self.mem_read(addr);
        println!("ora: a: {:#b} and cmp: {:3b}", self.a, cmp);
        self.a |= cmp;
        self.zero_negative_flag(self.a);
        println!("ora: Finished!")
    }

    fn and(&mut self, addr: u16) {
        self.a &= self.mem_read(addr);
        self.zero_negative_flag(self.a);
    }

    fn eor(&mut self, addr: u16) {
        self.a ^= self.mem_read(addr);
        self.zero_negative_flag(self.a);
    }

    fn add_to_a(&mut self, val: u8) {
        println!("add_to_a: a is {:#b}, val is {:#b}", self.a, val);
        let sum = self.a as u16
            + val as u16
            + if self.flags.contains(CpuFlags::CARRY) {
                1
            } else {
                0
            };

        if sum > 0xFF {
            self.flags.insert(CpuFlags::CARRY);
        } else {
            self.flags.remove(CpuFlags::CARRY);
        }

        let result = sum as u8; // Truncates as now carry flag is on
        println!("result: {:#b}, a: {:#b}, val: {:#b}", result, self.a, val);

        if ((result ^ self.a) & (result ^ val) & 0x80) != 0 {
            // Signed overflow(or underflow) occured
            println!("add_to_a: overflow assigned!");
            self.flags.insert(CpuFlags::OVERFLOW);
        } else {
            println!("add_to_a: overflow removed!");
            self.flags.remove(CpuFlags::OVERFLOW);
        }

        self.a = result;
        self.zero_negative_flag(self.a);
    }

    fn adc(&mut self, addr: u16) {
        let val = self.mem_read(addr);
        self.add_to_a(val);
    }

    fn sbc(&mut self, addr: u16) {
        let val = self.mem_read(addr);
        // wrapping_neg calculates two's complement negation
        let mem = (val as i8).wrapping_neg() as u8;
        println!("sbc: Old value is {:#b}, new value is {:#b}", val, mem);
        self.add_to_a(mem);
    }

    fn sta(&mut self, addr: u16) {
        self.mem_write(addr, self.a);
    }

    fn lda(&mut self, addr: u16) {
        println!("lda: Initalized, reading address {}", addr);
        self.a = self.mem_read(addr);
        self.zero_negative_flag(self.a);
    }

    // Used for CPY, CMP, CPX
    fn compare(&mut self, addr: u16, val: u8) {
        // BUG need to figure out val and mem[addr]
        let addr_val = self.mem_read(addr);
        println!("compare: val is {}, addr_val is {}", val, addr_val);
        let res = val.wrapping_sub(addr_val) as i8;

        if res >= 0 {
            self.flags.insert(CpuFlags::CARRY);
        }
        if res == 0 {
            self.flags.insert(CpuFlags::ZERO);
        }
        if res < 0 {
            // Subtraction is negative
            self.flags.insert(CpuFlags::NEGATIVE);
        }
    }

    fn cmp(&mut self, addr: u16) {
        self.compare(addr, self.a);
    }

    pub fn group_one(&mut self, aaa: u8, bbb: u8, _cc: u8) {
        // Group 1
        println!("group_one: Initalized");
        let mode = self.group_one_bbb(bbb);
        println!("group_one: Selected mode {}, bbb is {:3b}", mode, bbb);
        let addr = self.get_operand_address(&mode); // Memory location of the value to extract
        match aaa {
            0 => self.ora(addr),
            1 => self.and(addr),
            2 => self.eor(addr),
            3 => self.adc(addr),
            4 => self.sta(addr),
            5 => self.lda(addr),
            6 => self.cmp(addr),
            7 => self.sbc(addr),
            _ => unimplemented!("aaa"),
        }
    }

    // Group Two Functions
    fn group_two_three_bbb(&self, bbb: u8) -> AddressingMode {
        match bbb {
            0 => AddressingMode::Immediate,
            1 => AddressingMode::ZeroPage,
            0b10 => AddressingMode::Accumulator,
            0b11 => AddressingMode::Absolute,
            0b101 => AddressingMode::ZeroPage_X,
            0b111 => AddressingMode::Absolute_X,
            _ => {
                unimplemented!("Unknown addressing mode for group 1 {}", bbb);
            }
        }
    }

    fn asl(&mut self, addr: u16) {
        // Set carry to be bit 7
        let val = self.mem_read(addr);
        let carry_bit = val >> 7;
        if carry_bit == 1 {
            self.flags.insert(CpuFlags::CARRY);
        } else {
            self.flags.remove(CpuFlags::CARRY);
        }

        let new_val = val << 1;

        self.mem_write(addr, new_val);
        self.zero_negative_flag(new_val);
    }

    fn rol(&mut self, addr: u16) {
        let val = self.mem_read(addr);
        let carry_in = if self.flags.contains(CpuFlags::CARRY) {
            1
        } else {
            0
        };
        let carry_out = val >> 7;

        if carry_out == 1 {
            self.flags.insert(CpuFlags::CARRY);
        } else {
            self.flags.remove(CpuFlags::CARRY);
        }

        let new_val = (val << 1) | carry_in;
        self.mem_write(addr, new_val);
        self.zero_negative_flag(new_val);
    }

    fn lsr(&mut self, addr: u16) {
        // Set carry to be bit 0
        let val = self.mem_read(addr);
        let carry_bit = val & 0b0000001;
        if carry_bit == 1 {
            self.flags.insert(CpuFlags::CARRY);
        } else {
            self.flags.remove(CpuFlags::CARRY);
        }

        self.mem_write(addr, val >> 1);
        self.zero_negative_flag(val);
    }

    fn ror(&mut self, addr: u16) {
        let val = self.mem_read(addr);
        let carry_in = if self.flags.contains(CpuFlags::CARRY) {
            1
        } else {
            0
        };
        let carry_out = val & 0b00000001;

        if carry_out == 1 {
            self.flags.insert(CpuFlags::CARRY);
        } else {
            self.flags.remove(CpuFlags::CARRY);
        }

        let new_val = (val >> 1) | (carry_in << 7);
        self.mem_write(addr, new_val);
        self.zero_negative_flag(new_val);
    }

    fn stx(&mut self, addr: u16) {
        self.mem_write(addr, self.x)
    }

    fn ldx(&mut self, addr: u16) {
        self.x = self.mem_read(addr);
        self.zero_negative_flag(self.x);
    }

    fn dec(&mut self, addr: u16) {
        let old_val = self.mem_read(addr);
        let new_val = old_val.wrapping_sub(1);
        self.mem_write(addr, new_val);
        self.zero_negative_flag(new_val);
    }

    fn inc(&mut self, addr: u16) {
        let old_val = self.mem_read(addr);
        let new_val = old_val.wrapping_add(1);
        self.mem_write(addr, new_val);
        self.zero_negative_flag(new_val);
    }

    fn group_two(&mut self, aaa: u8, bbb: u8, _cc: u8) {
        let mode = self.group_two_three_bbb(bbb);
        let addr = self.get_operand_address(&mode);
        match aaa {
            0 => self.asl(addr),
            1 => self.rol(addr),
            2 => self.lsr(addr),
            3 => self.ror(addr),
            4 => self.stx(addr),
            5 => self.ldx(addr),
            6 => self.dec(addr),
            7 => self.inc(addr),
            _ => unimplemented!("Unknown aaa code {}", aaa),
        }
    }

    fn bit(&mut self, addr: u16) {
        let val = self.mem_read(addr);
        if self.a & val == 0 {
            self.flags.insert(CpuFlags::ZERO);
        } else {
            self.flags.remove(CpuFlags::ZERO);
        }
        let overflow = (val >> 6) & 0b01;
        let negative = val >> 7;
        if overflow == 1 {
            self.flags.insert(CpuFlags::OVERFLOW);
        } else {
            self.flags.remove(CpuFlags::OVERFLOW);
        }

        if negative == 1 {
            self.flags.insert(CpuFlags::NEGATIVE);
        } else {
            self.flags.remove(CpuFlags::NEGATIVE);
        }
    }

    fn jmp(&mut self, addr: u16) {
        let val: u16;
        // Implementing Cpu Bug
        if addr & 0x0011 == 0xFF {
            let lo = self.mem_read(addr) as u16;
            let hi = self.mem_read(addr & 0x1100) as u16; // Allows addressing ending in $FF to not cross the page
            val = (hi << 8) | (lo as u16)
        } else {
            // No bug
            val = self.mem_read_u16(addr);
        }
        self.pc = val;
    }

    fn jmp_abs(&mut self) {
        self.pc = self.mem_read_u16(self.pc);
    }

    fn sty(&mut self, addr: u16) {
        self.mem_write(addr, self.y);
    }

    fn ldy(&mut self, addr: u16) {
        self.y = self.mem_read(addr);
        self.zero_negative_flag(self.y);
    }

    fn cpy(&mut self, addr: u16) {
        self.compare(addr, self.y);
    }

    fn cpx(&mut self, addr: u16) {
        println!("cpx: Initalized");
        println!("x is {}", self.x);
        self.compare(addr, self.x);
    }

    // This code will read the next item in the pc and set the pc to jump there with + 1 to go to the next instruction
    fn branch(&mut self) {
        let jump = self.mem_read(self.pc) as i8;
        self.pc = self.pc.wrapping_add(1).wrapping_add(jump as u16);
    }

    fn if_contain_flag_branch(&mut self, flag: CpuFlags) {
        if self.flags.contains(flag) {
            self.branch();
        }
    }

    fn if_clear_flag_branch(&mut self, flag: CpuFlags) {
        if self.flags.contains(flag) {
            self.branch();
        }
    }

    fn brk(&mut self) {
        println!("brk: Initalized");
        println!("brk: pc is {}", self.pc);
        self.stack_push_u16(self.pc + 2 - 1);
        self.stack_push(self.flags.bits());
        self.flags.insert(CpuFlags::INTERRUPT_DISABLE);
        self.pc = self.mem_read_u16(0xFFFE);
        println!("brk: Set pc to {}", self.pc);
    }

    fn jsr(&mut self) {
        // Pushes the 16 bit value after self.pc
        self.stack_push_u16(self.pc + 2 - 1);
        self.pc = self.mem_read_u16(self.pc);
    }

    fn rti(&mut self) {
        self.flags = CpuFlags::from_bits_truncate(self.stack_pop());
        self.pc = self.stack_pop_u16();
    }

    fn rts(&mut self) {
        self.pc = self.stack_pop_u16();
        self.pc += 1;
    }

    fn group_three(&mut self, aaa: u8, bbb: u8, _cc: u8) {
        println!("group_three: Initalized");
        if bbb == 0b010 {
            unimplemented!(
                "group_three: Group Three bbb does not support accumulator! {}",
                bbb
            )
        } else if bbb == 0b100 {
            // Checking for branches
            match aaa {
                // BPL
                0b000 => self.if_clear_flag_branch(CpuFlags::NEGATIVE),
                // BMI
                0b001 => self.if_contain_flag_branch(CpuFlags::NEGATIVE),
                // BVC
                0b010 => self.if_clear_flag_branch(CpuFlags::OVERFLOW),
                // BVS
                0b011 => self.if_contain_flag_branch(CpuFlags::OVERFLOW),
                // BCC
                0b100 => self.if_clear_flag_branch(CpuFlags::CARRY),
                // BCS
                0b101 => self.if_contain_flag_branch(CpuFlags::CARRY),
                // BNE
                0b110 => self.if_clear_flag_branch(CpuFlags::ZERO),
                // BEQ
                0b111 => self.if_contain_flag_branch(CpuFlags::ZERO),
                _ => unimplemented!("Unknown branch aaa code {}", aaa),
            }
        } else {
            // Group Three Instructions
            let mode = self.group_two_three_bbb(bbb);
            let addr = self.get_operand_address(&mode);
            match aaa {
                1 => self.bit(addr),
                2 => self.jmp(addr),
                3 => self.jmp_abs(),
                4 => self.sty(addr),
                5 => self.ldy(addr),
                6 => self.cpy(addr),
                7 => self.cpx(addr),
                _ => unimplemented!("Unknown aaa code for group three {}", aaa),
            }
        }
    }
}
