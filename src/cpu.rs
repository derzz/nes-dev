use super::print_title;
use bitflags::bitflags;
use std::fmt;
use std::time::Duration;

mod branch_test;
mod group1_test;
mod group2_test;
mod group3_test;
mod op;
mod other_test;
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
    Indirect, // Used for jmp()
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
            AddressingMode::Indirect => write!(f, "Indirect"),
        }
    }
}

const STACK_RESET: u8 = 0xFD;
const STACK: u16 = 0x0100;
const PROGRAM_START: usize = 0x0600;

pub trait Mem {
    fn mem_read(&self, addr: u16) -> Byte;
    fn mem_write(&mut self, addr: u16, data: u8);
    // Used to read address in little endian
    fn mem_read_u16(&mut self, pos: u16) -> u16 {
        // If interrupt request is enabled, stop program exectuion
        // if pos == 0xFFFE && self.flags.contains(CpuFlags::INTERRUPT_DISABLE) {
        //     // BUG Used for irq handler, mitigating for now
        //     println!("mem_read_u16: Detected break. Reading from IRQ handler...");
        //     return 0xFFFF;
        // }
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
}

impl Mem for CPU {
    fn mem_read(&self, addr: u16) -> u8 {
        let ret = self.memory[addr as usize];
        ret
    }

    fn mem_write(&mut self, addr: u16, data: u8) {
        self.memory[addr as usize] = data;
    }
}

impl CPU {
    pub fn new() -> Self {
        CPU {
            pc: 0,
            a: 0,
            x: 0,
            y: 0,
            sp: STACK_RESET,
            flags: CpuFlags::from_bits_truncate(0b0010_0100),
            memory: [0; 0xFFFF],
            clock_time: Duration::from_millis(1), // Example value
        }
    }

    // Resets RAM from $0000 to $07FF
    // If program_start neds to be changed(eg as in snake, we subtract 1)
    fn ram_reset(&mut self) {
        for i in 0x0..PROGRAM_START as usize {
            self.memory[i] = 0;
        }
    }

    fn fn_reset(&mut self) {
        for i in PROGRAM_START as usize..0xFFFF {
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
        self.memory[PROGRAM_START as usize..(PROGRAM_START as usize + program.len())]
            .copy_from_slice(&program[..]);
        self.mem_write_u16(0xFFFC, PROGRAM_START as u16); // Save reference to program in 0xFFFC
        println!("load: Finished!");
    }

    // This function is meant for testing, where the test can insert their own values afterwards
    pub fn load_and_reset(&mut self, program: Vec<u8>) {
        self.fn_reset();
        self.load(program);
        self.reset();
    }

    pub fn instruction_print(&self, program: Vec<u8>) {
        let program_len = program.len();
        println!(
            "Memory dump ({} bytes from 0x{:04X}):",
            program_len, PROGRAM_START
        );
        println!("Addr    | Hex                                      | ASCII");
        println!("--------+------------------------------------------+------------------");

        for i in 0..program_len {
            let addr = PROGRAM_START + i;

            // Print address at start of each line
            if i % 16 == 0 {
                if i > 0 {
                    print!(" | ");
                    // Print ASCII representation for previous line
                    for j in i - 16..i {
                        let byte = self.memory[PROGRAM_START + j];
                        if byte >= 32 && byte <= 126 {
                            print!("{}", byte as char);
                        } else {
                            print!(".");
                        }
                    }
                    println!();
                }
                print!("{:04X}    | ", addr);
            }

            // Print byte value
            print!("{:02X} ", self.memory[addr]);

            // Add extra space after 8 bytes
            if i % 16 == 7 {
                print!(" ");
            }
        }

        // Print ASCII for the last line
        let remaining = program_len % 16;
        if remaining > 0 {
            // Pad for alignment
            for i in remaining..16 {
                // Use 'i' instead of '_'
                print!("   ");
                if remaining <= 8 && i == 7 {
                    print!(" ");
                }
            }
        }

        print!(" | ");
        let start_idx = program_len - (if remaining > 0 { remaining } else { 16 });
        for j in start_idx..program_len {
            let byte = self.memory[PROGRAM_START + j];
            if byte >= 32 && byte <= 126 {
                print!("{}", byte as char);
            } else {
                print!(".");
            }
        }
        println!("\n");
    }

    pub fn load_and_run(&mut self, program: Vec<u8>) {
        println!("load_and_run: Initalized");
        self.load(program.clone());
        self.reset();
        // USED FOR TESTING
        println!("Printing out what's in instructions");
        self.instruction_print(program);
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

    pub fn run(&mut self) {
        self.run_with_callback(|_| {});
    }

    pub fn run_with_callback<F>(&mut self, mut callback: F)
    where
        F: FnMut(&mut CPU),
    {
        println!("run: Initalized");
        loop {
            print_title!("Starting run!");
            println!("run: Reading values, starting with pc {:#x}", self.pc);
            println!("run: Flags [NV-BDIZC]: {:08b}", self.flags.bits());
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
                return; // NOTE: Break will return without PC needing to jump anywhere
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
            self.pc = self.pc.wrapping_add(1);
            callback(self);
        }
        print_title!("End of current execution");
    }

    fn get_operand_address(&mut self, mode: &AddressingMode) -> u16 {
        println!("get_operand_address: Initalized");
        self.pc = self.pc.wrapping_add(1);
        match mode {
            AddressingMode::Immediate => self.pc,

            AddressingMode::Accumulator => unimplemented!(
                "get_operand_address: Accumulator addressing are not supported from this function"
            ),

            AddressingMode::ZeroPage => self.mem_read(self.pc) as u16,

            AddressingMode::Absolute => {
                println!("get_operand_address: in absolute mode");
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

            AddressingMode::Indirect => {
                println!("get_operand_address: In Indirect");
                let base = self.mem_read_u16(self.pc);
                self.pc = self.pc.wrapping_add(1);
                println!("get_operand_address: Indirect:: base is {:#x}", base);
                let lo = self.mem_read(base as u16);
                let read = if base & 0xFF == 0xFF {
                    base & 0xFF00
                } else {
                    (base as u16).wrapping_add(1) as u16
                };
                let hi = self.mem_read(read);
                let deref_base = (hi as u16) << 8 | (lo as u16);

                deref_base
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
                let hi = self.mem_read((base as u16).wrapping_add(1) as u16);
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
        println!("add to a: final result is {}", self.a);
    }

    fn adc(&mut self, addr: u16) {
        let val = self.mem_read(addr);
        self.add_to_a(val);
    }

    fn sbc(&mut self, addr: u16) {
        let val = self.mem_read(addr);
        // wrapping_neg calculates two's complement negation
        // 2s complements adds 1 at the end, we subtract 1 to just get the not version of memory
        // Clear now doesn't need to be negated as this counters the 1
        let mem = ((val as i8).wrapping_neg().wrapping_sub(1)) as u8;
        println!("sbc: Old value is {:#b}, reverted value is {:#b}", val, mem);
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
        println!("compare: val is {:#x}, addr_val is {:#x}", val, addr_val);
        let res = val.wrapping_sub(addr_val) as i8;

        if res >= 0 {
            self.flags.insert(CpuFlags::CARRY);
        } else {
            self.flags.remove(CpuFlags::CARRY);
        }
        if res == 0 {
            self.flags.insert(CpuFlags::ZERO);
        } else {
            self.flags.remove(CpuFlags::ZERO);
        }
        if res < 0 {
            // Subtraction is negative
            self.flags.insert(CpuFlags::NEGATIVE);
        } else {
            self.flags.remove(CpuFlags::NEGATIVE);
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

    fn asl(&mut self, addr: u16, accum: bool) {
        // Set carry to be bit 7
        let val = if !accum { self.mem_read(addr) } else { self.a };
        println!("asl: val is {:#b}", val);
        let carry_bit = val >> 7;
        println!("asl: Carry bit is {:#b}", carry_bit);
        if carry_bit == 1 {
            self.flags.insert(CpuFlags::CARRY);
        } else {
            self.flags.remove(CpuFlags::CARRY);
        }
        let new_val: u8;
        if !accum {
            println!("asl: Shifting {:#b}!", val);
            new_val = val << 1;
            self.mem_write(addr, new_val);
        } else {
            new_val = self.a << 1;
            println!("asl: Modifying accumulator, old value is {:#b}", self.a);
            self.a = new_val;
            println!("asl: accumulator new value is {:#b}", self.a);
        }
        self.zero_negative_flag(new_val);
    }

    fn rol(&mut self, addr: u16, accum: bool) {
        let val = if !accum { self.mem_read(addr) } else { self.a };
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
        let new_val: u8;
        if !accum {
            new_val = (val << 1) | carry_in;
            self.mem_write(addr, new_val);
        } else {
            new_val = (self.a << 1) | carry_in;
            self.a = new_val;
        }
        self.zero_negative_flag(new_val);
    }

    fn lsr(&mut self, addr: u16, accum: bool) {
        // Set carry to be bit 0
        let val = if !accum { self.mem_read(addr) } else { self.a };
        let carry_bit = val & 0b0000_0001;
        if carry_bit == 1 {
            self.flags.insert(CpuFlags::CARRY);
        } else {
            self.flags.remove(CpuFlags::CARRY);
        }
        let new_val: u8;
        if !accum {
            new_val = val >> 1;
            self.mem_write(addr, new_val);
        } else {
            new_val = val >> 1;
            self.a = new_val;
        }
        self.zero_negative_flag(new_val);
    }

    fn ror(&mut self, addr: u16, accum: bool) {
        let val = if !accum { self.mem_read(addr) } else { self.a };
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
        let new_val: u8;
        if !accum {
            new_val = (val >> 1) | (carry_in << 7);
            self.mem_write(addr, new_val);
        } else {
            new_val = (self.a >> 1) | (carry_in << 7);
            self.a = new_val;
        }
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
        let accum = matches!(mode, AddressingMode::Accumulator);
        let addr = if !accum {
            self.get_operand_address(&mode)
        } else {
            0
        };
        match aaa {
            0 => self.asl(addr, accum),
            1 => self.rol(addr, accum),
            2 => self.lsr(addr, accum),
            3 => self.ror(addr, accum),
            4 => self.stx(addr),
            5 => self.ldx(addr),
            6 => self.dec(addr),
            7 => self.inc(addr),
            _ => unimplemented!("Unknown aaa code {}", aaa),
        }
    }

    fn bit(&mut self, addr: u16) {
        let val = self.mem_read(addr);
        if (self.a & val) == 0 {
            self.flags.insert(CpuFlags::ZERO);
        } else {
            self.flags.remove(CpuFlags::ZERO);
        }
        println!("bit: val is {:#b}", val);
        let overflow = (val >> 6) & 0b01;
        println!("bit: overflow {:#b}", overflow);
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
        // address already has address to jump to
        println!("jmp: Initalized with address {:#x}", addr);
        let val = addr;
        println!("jmp: val is {:#x}", val);
        // Need to subtract pc by one as it will be added at the end of run
        self.pc = val.wrapping_sub(1);
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
        println!(
            "branch: Initalized, starting to branch from pc {:#x}!",
            self.pc
        );
        let jump = self.mem_read(self.pc) as i8;
        println!("branch: jump is {:x}", jump);
        // NOTE We do not need to add 2 as at then end of every run cycle will add 1, the other 1 is added since the pc is on the address instead of the instruction
        self.pc = self.pc.wrapping_add(jump as u16);
        println!("Finished branch, pc is now on {:#x}", self.pc);
    }

    fn if_contain_flag_branch(&mut self, flag: CpuFlags) {
        println!("if_contain_flag_branch: Checking flag {:#b}", flag);
        if self.flags.contains(flag) {
            self.branch();
        }
    }

    fn if_clear_flag_branch(&mut self, flag: CpuFlags) {
        if !self.flags.contains(flag) {
            self.branch();
        }
    }

    fn brk(&mut self) {
        println!("brk: Initalized");
        println!("brk: pc is {}", self.pc);
        self.stack_push_u16(self.pc.wrapping_add(2));
        self.stack_push(self.flags.bits());
        self.flags.insert(CpuFlags::INTERRUPT_DISABLE);
        self.pc = self.mem_read_u16(0xFFFE);
        println!("brk: Set pc to {}", self.pc);
    }

    fn jsr(&mut self) {
        // Pushes the 16 bit value after self.pc
        // Note that self.pc is already on the memory value so we just need to push this part + 1
        // Eg. JSR 0xAA 0xBB, we would be pushing the memory address of 0xBB
        // When rts is called, pc will add 1 automatically so it returns from the next function
        println!(
            "jsr: Initalized! The instruction's address is {:#x}",
            self.pc
        );
        self.stack_push_u16(self.pc.wrapping_add(2));
        // Need to subtract one at the end as run() will add one automatically
        let new_pc = self.mem_read_u16(self.pc.wrapping_add(1)).wrapping_sub(1);
        println!("jsr: Going to new address: {:#x}", new_pc + 1);
        self.pc = new_pc;
    }

    fn rti(&mut self) {
        // Most likely coming from a BRK(software IRQ)- BRK is treated as a 2 byte instruction with an unused immediate
        self.flags = CpuFlags::from_bits_truncate(self.stack_pop());
        self.pc = self.stack_pop_u16();
        // Need to subtract one pc to balance out with the end of run(), which adds one to pc
        self.pc = self.pc.wrapping_sub(1);
    }

    fn rts(&mut self) {
        self.pc = self.stack_pop_u16();
        println!(
            "rts: Finished. The pc before finishing run is {:#x}",
            self.pc
        );
        // self.pc does not need to be added as at the end of run, the pc will be added by 1 automatically
    }

    fn group_three(&mut self, aaa: u8, bbb: u8, _cc: u8) {
        println!("group_three: Initalized");
        if bbb == 0b010 {
            unimplemented!(
                "group_three: Group Three bbb does not support accumulator! {}",
                bbb
            )
        } else if bbb == 0b100 {
            // Add pc by 1 to go to reading address
            self.pc = self.pc.wrapping_add(1);
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
            println!("group_three: Actually in group 3!");
            let mut mode = self.group_two_three_bbb(bbb);
            // Hardcoding jmp rel
            if aaa == 0b011 && bbb == 0b011 {
                // This is jump relative, replace the mode
                println!("group_three: This is jmp indirect!");
                mode = AddressingMode::Indirect;
            }
            let addr = self.get_operand_address(&mode);
            println!(
                "group_three: Deciding what instruction with aaa: {:#b} and address {:#x}",
                aaa, addr
            );
            match aaa {
                1 => self.bit(addr),
                0b010 | 0b011 => self.jmp(addr),
                4 => self.sty(addr),
                5 => self.ldy(addr),
                6 => self.cpy(addr),
                7 => self.cpx(addr),
                _ => unimplemented!("Unknown aaa code for group three {}", aaa),
            }
        }
    }
}
