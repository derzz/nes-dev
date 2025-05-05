use crate::bus::Bus;

use bitflags::bitflags;
use core::panic;
use log::{debug, info, warn};
use std::{fmt, ops::Add};

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

pub struct CPU<'a> {
    pub pc: u16,
    pub a: Byte,
    pub x: Byte,
    pub y: Byte,
    pub sp: Byte,
    pub flags: CpuFlags,
    pub bus: Bus<'a>,
    pub halted: bool, // Used for successful exits
    pub cycles: u8, // Stores the number of cycles for one instruction, always restarts to 0 at start of run
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
    Relative,    // Relative for branches
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
            AddressingMode::Relative => write!(f, "Relative"),
        }
    }
}

const STACK_RESET: u8 = 0xFD;
const STACK: u16 = 0x0100;

pub trait Mem {
    fn mem_read(&mut self, addr: u16) -> Byte;
    fn mem_write(&mut self, addr: u16, data: u8);
    // Used to read address in little endian
    fn mem_read_u16(&mut self, pos: u16) -> u16 {
        // If interrupt request is enabled, stop program exectuion
        // if pos == 0xFFFE && self.flags.contains(CpuFlags::INTERRUPT_DISABLE) {
        //     // BUG Used for irq handler, mitigating for now
        //     //println!("mem_read_u16: Detected break. Reading from IRQ handler...");
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

impl Mem for CPU<'_> {
    fn mem_read(&mut self, addr: u16) -> u8 {
        self.bus.mem_read(addr)
    }

    fn mem_write(&mut self, addr: u16, data: u8) {
        debug!("Writing value {:2X} to address {:4X}", data, addr);
        self.bus.mem_write(addr, data);
    }

    fn mem_read_u16(&mut self, pos: u16) -> u16 {
        self.bus.mem_read_u16(pos)
    }

    fn mem_write_u16(&mut self, pos: u16, data: u16) {
        self.bus.mem_write_u16(pos, data);
    }
}

impl<'a> CPU<'a> {
    pub fn new<'b>(bus: Bus<'b>) -> CPU<'b> {
        CPU {
            pc: 0,
            a: 0,
            x: 0,
            y: 0,
            halted: false,
            sp: STACK_RESET,
            flags: CpuFlags::from_bits_truncate(0b0010_0100),
            bus: bus,
            cycles: 7, // Starting with 7 clock cycles
        }
    }

    // Restores registers and initalizes PC to the 2 byte value at 0xFFFC
    pub fn reset(&mut self) {
        //println!("reset: Initalized");
        self.a = 0;
        self.x = 0;
        self.y = 0;
        self.flags = CpuFlags::from_bits_truncate(0b00100100);
        self.sp = STACK_RESET;
        // self.pc = 0xC000; // TODO Remove on tests
        self.pc = self.mem_read_u16(0xFFFC);
    }

    // This function adds to cycles. This is to avoid any direct augmentation to the cycles(making it more painful to debug)
    fn reset_cycles(&mut self) {
        self.cycles = 0;
    }

    fn add_cycles(&mut self, val: u8) {
        self.cycles += val;
        debug!(
            "Adding {} cycles, total for this instruction is {}",
            val, self.cycles
        );
    }

    pub fn load(&mut self, program: Vec<u8>) {
        for i in 0..(program.len() as u16) {
            self.mem_write(0x0000 + i, program[i as usize]);
        }
        self.mem_write_u16(0xFFFC, 0x0000);
    }

    // This function is meant for testing, where the test can insert their own values afterwards
    pub fn load_and_reset(&mut self, program: Vec<u8>) {
        self.load(program);
        self.reset();
    }

    pub fn load_and_run(&mut self, program: Vec<u8>) {
        //println!("load_and_run: Initalized");
        self.load(program.clone());
        self.reset();
        // USED FOR TESTING
        //println!("Printing out what's in instructions");
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


    // If nmi interrupt is encountered
    
    fn interrupt_nmi(&mut self){
        self.stack_push_u16(self.pc); // Push PC and Status flag on stack
        let mut flag = self.flags.clone();
        flag.set(CpuFlags::BREAK, false);
        flag.set(CpuFlags::BREAK2, true);

        self.stack_push(flag.bits());
        self.flags.insert(CpuFlags::INTERRUPT_DISABLE);

        self.bus.tick(2);
        self.pc = self.mem_read_u16(0xFFFA);
    }

    pub fn run(&mut self) {
        self.run_with_callback(|_| {});
    }

    

    pub fn run_with_callback<F>(&mut self, mut callback: F)
    where
        F: FnMut(&mut CPU),
    {

        loop {
            if let Some(_nmi) = self.bus.poll_nmi_status(){
                self.interrupt_nmi();
            }

            if self.halted{
                debug!("Got EOF signal! Exiting program...");
                break;
            }
            debug!(
                "start of run the flags are {:#X}, pc is currently at {:#X}",
                self.flags.bits(),
                self.pc
            );
            callback(self);
            debug!("run: Reading values, starting with pc {:4X}", self.pc);
            debug!("run: Flags [NV-BDIZC]: {:08b}", self.flags.bits());
            warn!("The value of 7F is {:4X}", self.mem_read(0x7F));
            self.reset_cycles();
            if self.pc == 0xFFFF && self.flags.contains(CpuFlags::INTERRUPT_DISABLE) {
                debug!("run: IRQ detected, most likely from a brk. Stopping execution...");
                break;
            }
            let op = self.mem_read(self.pc);
            debug!("op is {:#4X}", op);

            let highnibble = op >> 4;
            let lownibble = op & 0x0F;
            //println!(
            //     "run: Highnibble is {:#x} and lownibble is {:#x}",
            //     highnibble, lownibble
            // );
            let aaa = op >> 5;
            let bbb = (op >> 2) & 0x7;
            let cc = op & 0x3; // Used for identification of group 1, 2, and 3
                               //println!(
                               //     "run: aaa is {:03b}, bbb is {:03b}, cc is {:02b}",
                               //     aaa, bbb, cc
                               // );
                               // Top is hard coding remaining instructions
            match op {
                // Special and illegal opcodes
                0x0 => {
                    self.brk();
                    return;
                }
                0x20 => self.jsr(),
                0x40 => self.rti(),
                0x60 => self.rts(),
                // NOP
                0x1A | 0x3A | 0x5A | 0x7A | 0xDA | 0xEA | 0xFA => {
                    self.add_cycles(2);
                }
                // SKB
                0x80 | 0x82 | 0x89 | 0xC2 | 0xE2 => {
                    self.add_cycles(2);
                    // Adds 1 to the pc to "read" an immediate byte
                    self.pc = self.pc.wrapping_add(1);
                }

                // IGN a
                0x0c => {
                    self.add_cycles(4);
                    self.pc = self.pc.wrapping_add(2);
                }

                // IGN a, X
                // Absolute X addressing, basically follow G1 Cycles calculations
                0x1C | 0x3C | 0x5C | 0x7C | 0xDC | 0xFC => {
                    let init_addr = self.mem_read_u16(self.pc.wrapping_add(1));
                    // Adds the needed cycles
                    self.g1_cycles(
                        &AddressingMode::Absolute_X,
                        init_addr.wrapping_add(self.x as u16),
                        false,
                    );
                    self.pc = self.pc.wrapping_add(2)
                }

                // IGN d
                0x04 | 0x44 | 0x64 => {
                    self.add_cycles(3);
                    self.pc = self.pc.wrapping_add(1);
                }

                // IGN d,X
                // Since it's still a NOP, we won't be reading the value and just increment the PC
                0x14 | 0x34 | 0x54 | 0x74 | 0xD4 | 0xF4 => {
                    self.add_cycles(4);
                    self.pc = self.pc.wrapping_add(1)
                }
                // ===== END OF UNOFFICAL NOP =====
                // ===== COMBINED OPERATIONS =====
                // ALR #i
                0x4b => {
                    let addr = self.get_operand_address(&AddressingMode::Immediate);
                    self.and(addr);
                    self.lsr(addr, false);
                }

                // ANC #i
                0x0b => {
                    let addr = self.get_operand_address(&AddressingMode::Immediate);
                    self.and(addr);
                    if self.flags.contains(CpuFlags::NEGATIVE) {
                        self.flags.insert(CpuFlags::CARRY);
                    } else {
                        self.flags.remove(CpuFlags::CARRY);
                    }
                }

                // ARR #i
                0x6b => {
                    let addr = self.get_operand_address(&AddressingMode::Immediate);
                    self.ror(addr, false);
                    let new_val = self.mem_read(addr);
                    // Override the carry and overflow bit
                    let sixth = (new_val >> 5) & 1;
                    let fifth = (new_val >> 4) & 1;
                    if sixth == 1 {
                        self.flags.insert(CpuFlags::CARRY);
                    } else {
                        self.flags.remove(CpuFlags::CARRY);
                    }

                    if (sixth ^ fifth) == 1 {
                        self.flags.insert(CpuFlags::OVERFLOW);
                    } else {
                        self.flags.remove(CpuFlags::OVERFLOW);
                    }
                }

                // AXS #i
                0xCB => {
                    let addr = self.get_operand_address(&AddressingMode::Immediate);
                    let result = (self.x & self.a) as u16;
                    let sub = result.wrapping_sub(addr);

                    self.x = sub as u8;
                    if result >= addr {
                        self.flags.insert(CpuFlags::CARRY);
                    } else {
                        self.flags.remove(CpuFlags::CARRY);
                    }
                    self.zero_negative_flag(self.x);
                }

                // LAX(d, X)
                0xA3 => {
                    self.add_cycles(6);
                    self.lax(&AddressingMode::Indirect_X);
                }

                // LAX d
                0xA7 => {
                    self.add_cycles(3);
                    self.lax(&AddressingMode::ZeroPage);
                }

                // LAX a
                0xAF => {
                    self.add_cycles(4);
                    self.lax(&AddressingMode::Absolute);
                }

                // LAX (d), Y
                0xB3 => {
                    // BUG need to consider page skip
                    self.add_cycles(5);
                    let new_address =
                        self.get_relative_address(&AddressingMode::Indirect_Y, self.pc);
                    let old_address = new_address.wrapping_sub(self.y as u16);
                    self.check_page_cross(old_address, new_address, false);
                    self.lax(&AddressingMode::Indirect_Y);
                }

                // LAX d, Y
                0xb7 => {
                    self.add_cycles(4);
                    self.lax(&AddressingMode::ZeroPage_Y);
                }

                // LAX a,Y
                0xbf => {
                    // BUG need to consider page skip
                    self.add_cycles(4);
                    self.lax(&AddressingMode::Absolute_Y);
                }

                // SAX (d, X)
                0x83 => {
                    self.add_cycles(6);
                    self.sax(&AddressingMode::Indirect_X);
                }

                // SAX d
                0x87 => {
                    self.add_cycles(3);
                    self.sax(&AddressingMode::ZeroPage);
                }

                // SAX a
                0x8F => {
                    self.add_cycles(4);
                    self.sax(&AddressingMode::Absolute);
                }

                // SAX d, Y
                0x97 => {
                    self.add_cycles(4);
                    self.sax(&AddressingMode::ZeroPage_Y);
                }

                // ===== END OF COMBINED OPERATIONS =====
                // RMW INSTRUCTIONS follow a pattern with high and low nibbles, will be covered in the if else statements

                // ===== DUPLICATED INSTRUCTIONS =====
                // ADC #i
                0xEB => {
                    let addr = self.get_operand_address(&AddressingMode::Immediate);
                    self.add_cycles(2);
                    self.sbc(addr);
                }

                _ => {
                    // Single byte and group territory
                    if lownibble == 0x8 {
                        self.sb_one(highnibble);
                    } else if lownibble == 0xA && highnibble >= 0x8 {
                        self.sb_two(highnibble);
                        // Combined operations have been covered above
                    } else if lownibble == 0x3
                        || lownibble == 0x7
                        || lownibble == 0xF
                        || lownibble == 0xB
                    {
                        self.rmw(highnibble, lownibble);
                    } else if cc == 0b01 {
                        self.group_one(aaa, bbb, cc);
                    } else if cc == 0b10 {
                        self.group_two(aaa, bbb, cc, op);
                    } else if cc == 0b00 && op != 0x04 {
                        // Conditionals are also included in here
                        debug!("Found group 3 cc = 0b00!");
                        self.group_three(aaa, bbb, cc);
                    } else {
                        panic!("Cpu: Unknown opcode {:2X}", op);
                    }
                }
            }
            // Second IRQ check, as self.pc addition occurs after pc is set to 0xFFFF
            // NOTE: Before this runs, PC must be at the instruction before the next command
            if self.pc == 0xFFFF && self.flags.contains(CpuFlags::INTERRUPT_DISABLE) {
                //println!("run: IRQ detected, most likely from a brk. Stopping execution...");
                break;
            }

            debug!("Calling tick, total number of cycles is {}", self.cycles);
            // Call tick to allow the PPU to catch up
            self.bus.tick(self.cycles); // self.cycles only contain the number of cycles after the current instruction

            self.pc = self.pc.wrapping_add(1);

            debug!("end of run the flags are {:#X}", self.flags.bits());
        }
        // print_title!("End of current execution");
    }

    fn rmw(&mut self, high: u8, low: u8) {
        let part: bool = (high % 2) == 0;
        let mode = match (part, low) {
            (true, 0x3) => {
                self.add_cycles(8);
                AddressingMode::Indirect_X
            }
            (true, 0x7) => {
                self.add_cycles(5);
                AddressingMode::ZeroPage
            }
            (true, 0xF) => {
                self.add_cycles(6);
                AddressingMode::Absolute
            }
            (false, 0x3) => {
                self.add_cycles(8);
                AddressingMode::Indirect_Y
            }
            (false, 0x7) => {
                self.add_cycles(6);
                AddressingMode::ZeroPage_X
            }
            (false, 0xB) => {
                self.add_cycles(7);
                AddressingMode::Absolute_Y
            }
            (false, 0xF) => {
                self.add_cycles(7);
                AddressingMode::Absolute_X
            }
            _ => panic!("Invalid mode combination: part={part}, low={low}"),
        };
        let addr = self.get_operand_address(&mode);

        // Actual logic for different functions
        match high {
            0xC | 0xD => self.dcp(addr),
            0xE | 0xF => self.isc(addr),
            0x2 | 0x3 => self.rla(addr),
            0x6 | 0x7 => self.rra(addr),
            0x0 | 0x1 => self.slo(addr),
            0x4 | 0x5 => self.sre(addr),
            _ => {
                panic!("rmw: Unknown high nibble {:X}", high);
            }
        }
    }

    fn dcp(&mut self, addr: u16) {
        self.dec(addr);
        self.cmp(addr);
    }

    fn isc(&mut self, addr: u16) {
        self.inc(addr);
        self.sbc(addr);
    }

    fn rla(&mut self, addr: u16) {
        self.rol(addr, false);
        self.and(addr);
    }

    fn rra(&mut self, addr: u16) {
        self.ror(addr, false);
        self.adc(addr);
    }

    fn slo(&mut self, addr: u16) {
        self.asl(addr, false);
        self.ora(addr);
    }

    fn sre(&mut self, addr: u16) {
        self.lsr(addr, false);
        self.eor(addr);
    }

    fn lax(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        self.lda(addr);
        self.ldx(addr);
    }

    fn sax(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let comb = self.a & self.x;
        self.mem_write(addr, comb);
    }

    fn get_operand_address(&mut self, mode: &AddressingMode) -> u16 {
        // PC is currently on the instruction
        let ret = self.get_relative_address(mode, self.pc);
        // NOTE PC Manipulation here is done here to allow for relative to be done generally
        self.pc = self.pc.wrapping_add(1);
        match mode {
            AddressingMode::Absolute
            | AddressingMode::Absolute_X
            | AddressingMode::Absolute_Y
            | AddressingMode::Indirect => self.pc = self.pc.wrapping_add(1),
            _ => {}
        }
        // PC will end at the last byte of the instruction
        // If branching is chosen, the new pc value will always be ret nonetheless
        ret
    }

    // this fn will take the address of where the instruction is
    // if passed the program counter, this will not change it
    // The address passed must be the location where addressing starts(or the next isntruction if immediate is chosen)
    pub fn get_relative_address(&mut self, mode: &AddressingMode, instr_addr: u16) -> u16 {
        //println!("get_operand_address: Initalized");
        let address = instr_addr.wrapping_add(1);
        match mode {
            AddressingMode::Immediate => address, // No need to add

            AddressingMode::Accumulator => unimplemented!(
                "get_operand_address: Accumulator addressing are not supported from this function"
            ),

            AddressingMode::ZeroPage => self.mem_read(address) as u16,

            AddressingMode::Absolute => {
                //println!("get_operand_address: in absolute mode");
                let ret = self.mem_read_u16(address);
                // self.pc = self.pc.wrapping_add(1);
                ret
            }

            AddressingMode::ZeroPage_X => {
                let pos = self.mem_read(address);
                let addr = pos.wrapping_add(self.x) as u16;
                addr
            }
            // (Indirect), Y in NESDev wiki
            AddressingMode::ZeroPage_Y => {
                let pos = self.mem_read(address);
                let addr = pos.wrapping_add(self.y) as u16;
                addr
            }

            AddressingMode::Absolute_X => {
                //println!("get_operand_address: In Absolute_X");
                let base = self.mem_read_u16(address);
                // self.pc = self.pc.wrapping_add(1);
                let addr = base.wrapping_add(self.x as u16);
                addr
            }
            AddressingMode::Absolute_Y => {
                let base = self.mem_read_u16(address);
                // self.pc = self.pc.wrapping_add(1);
                let addr = base.wrapping_add(self.y as u16);
                addr
            }
            // Used for JMP
            AddressingMode::Indirect => {
                debug!("get_operand_address: In Indirect");
                let base = self.mem_read_u16(address);
                // self.pc = self.pc.wrapping_add(1);
                //println!("get_operand_address: Indirect:: base is {:#x}", base);
                let lo = self.mem_read(base as u16);
                let read = if base & 0xFF == 0xFF {
                    base & 0xFF00
                } else {
                    (base as u16).wrapping_add(1) as u16
                };
                let hi = self.mem_read(read);
                let deref_base = (hi as u16) << 8 | (lo as u16);

                deref_base // Returns indirect address
            }

            // (c0, X)
            // Looks at the address at LSB = c0 + X and MSB = c0 + X + 1 => Address LSB + MSB
            AddressingMode::Indirect_X => {
                debug!("get_operand_address: In Indirect_X");
                let base = self.mem_read(address);

                let ptr: u8 = (base as u8).wrapping_add(self.x);
                let lo = self.mem_read(ptr as u16);
                let hi = self.mem_read(ptr.wrapping_add(1) as u16);
                (hi as u16) << 8 | (lo as u16)
            }
            //($c0), Y
            // Look at address at LSB = c0 and MSB = C0 + 1 => Address LSB + MSB + Y
            AddressingMode::Indirect_Y => {
                let base = self.mem_read(address);
                let lo = self.mem_read(base as u16);
                let hi = self.mem_read((base as u8).wrapping_add(1) as u16); // BUG if base is FF we need to wrap to 00
                let deref_base = (hi as u16) << 8 | (lo as u16);
                let deref = deref_base.wrapping_add(self.y as u16);
                deref
            }

            AddressingMode::NoneAddressing => {
                panic!("mode {:?} is not supported", mode);
            }
            AddressingMode::Relative => {
                panic!("Relative mode not supported in getting relative address");
            }
        }
    }

    fn check_page_cross(&mut self, base_addr: u16, final_addr: u16, is_sta: bool) {
        if (base_addr >> 8) != (final_addr >> 8) || is_sta {
            debug!(
                "Page cross detected: base_addr is {:4X} and final_addr is {:4X}",
                base_addr, final_addr
            );
            self.add_cycles(1);
        }
    }
    // Note cycles have been initiated to 2, these calculations are filling up the remaining cycles
    // STX LDX STY LDY works in here
    fn g1_cycles(&mut self, mode: &AddressingMode, new_address: u16, is_sta: bool) {
        match mode {
            AddressingMode::Immediate => self.add_cycles(2),
            AddressingMode::ZeroPage => self.add_cycles(3),
            AddressingMode::ZeroPage_X => self.add_cycles(4),
            AddressingMode::ZeroPage_Y => self.add_cycles(4), // This is only used for STX
            AddressingMode::Absolute => self.add_cycles(4),
            AddressingMode::Absolute_X => {
                self.add_cycles(4);
                let old_address = new_address.wrapping_sub(self.x as u16);
                self.check_page_cross(old_address, new_address, is_sta);
            }
            AddressingMode::Absolute_Y => {
                self.add_cycles(4);
                let old_address = new_address.wrapping_sub(self.y as u16);
                self.check_page_cross(old_address, new_address, is_sta);
            }
            AddressingMode::Indirect_X => {
                self.add_cycles(6);
            }
            AddressingMode::Indirect_Y => {
                self.add_cycles(5);
                // Change the old_address to the Y subtracted value
                let old_address = new_address.wrapping_sub(self.y as u16);
                self.check_page_cross(old_address, new_address, is_sta);
            }
            _ => panic!("{} not supported in group 1, no cycles added.", mode),
        }
    }

    // Cycles already have been initiated to 2
    fn g2_default_cycles(&mut self, mode: &AddressingMode) {
        match mode {
            AddressingMode::Accumulator => self.add_cycles(2),
            AddressingMode::ZeroPage => self.add_cycles(5),
            AddressingMode::ZeroPage_X => self.add_cycles(6),
            AddressingMode::Absolute => self.add_cycles(6),
            AddressingMode::Absolute_X => self.add_cycles(7),
            _ => panic!("{} not supported in group 2, no cycles added. ", mode),
        }
    }

    fn stack_push(&mut self, data: u8) {
        self.mem_write((STACK as u16) + self.sp as u16, data);
        debug!("stack_push: pushed {:02X}", data);
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
        if self.sp > STACK_RESET {
            debug!("EOF!");
            self.halted = true;
            return 0;
        }
        let ret = self.mem_read((STACK as u16) + self.sp as u16);
        debug!("stack_pop: popped {:2X}", ret);
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
        //println!("php: Initialized- PC is {}", self.pc);
        let mut flags = self.flags.clone();
        flags.insert(CpuFlags::BREAK);
        flags.insert(CpuFlags::BREAK2);
        self.stack_push(flags.bits());
        self.add_cycles(1); // Added 2 already due to SB
                            //println!("php: Finished execution- PC is {}", self.pc);
    }

    fn plp(&mut self) {
        let brk = self.flags.contains(CpuFlags::BREAK);
        self.flags = CpuFlags::from_bits_truncate(self.stack_pop());
        self.flags.insert(CpuFlags::BREAK2); // Always need to push as 1
                                             // Set to ignore break flag
        if brk {
            self.flags.insert(CpuFlags::BREAK);
        } else {
            self.flags.remove(CpuFlags::BREAK);
        }
        self.add_cycles(2); // Added 2 already due to SB
    }

    fn pha(&mut self) {
        //println!("pha: Initalized");
        self.stack_push(self.a);
        // Due to SB, already added 2
        self.add_cycles(1);
    }

    fn pla(&mut self) {
        self.a = self.stack_pop();
        self.zero_negative_flag(self.a);
        //println!("pla: pulled {}", self.a);
        self.add_cycles(2);
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
        //println!("inx: Initalized(Incrementing x)");
        self.x = self.x.wrapping_add(1);
        self.zero_negative_flag(self.x);
    }

    // Used for grouping addressing modes
    fn sb_one(&mut self, highnibble: u8) {
        //println!("sb_one: Initalized");
        // Single Byte instructions, don't need to read Bytes past the value
        // Eg. PHP, CLC, INX
        // lower nibble of opcode is 0x_8(eg. 0x08...0xF8)
        // Pattern represents (_ _ _ _ 1000)
        self.add_cycles(2);

        match highnibble {
            0 => {
                self.php();
            }
            // CLC clears Carry flag
            1 => {
                //println!("clc: Initalized");
                self.flags.remove(CpuFlags::CARRY);
                //println!("clc: Flags are now {:#b}", self.flags);
            }
            2 => {
                self.plp();
            }
            // SEC(set carry) sets carry flag to 1
            3 => {
                self.flags.insert(CpuFlags::CARRY);
            }
            // PHA(Push A) stores the value of A to the current stack position
            4 => {
                self.pha();
            }
            // CLI(Clear Interrupt Disable) clears the interrupt disable flag
            5 => {
                self.flags.remove(CpuFlags::INTERRUPT_DISABLE);
            }
            // PLA(Pull A) increments the stack pointer and loads the value at that stack position into A
            6 => {
                self.pla();
            }
            //SEI(Set Interrupt Disable) sets the interrupt disable flag
            7 => {
                self.flags.insert(CpuFlags::INTERRUPT_DISABLE);
            }
            // DEY subtracts 1 from the Y register
            8 => {
                self.dey();
            }
            // TYA transfers the Y register to the accumulator
            9 => {
                self.tya();
            }
            // TAY transfer accumulator to Y register
            10 => {
                self.tay();
            }
            // CLV clears the overflow tag
            11 => {
                self.flags.remove(CpuFlags::OVERFLOW);
            }
            // INY increases the Y register
            12 => {
                self.iny();
            }
            // CLD clears the decimal flag
            13 => {
                self.flags.remove(CpuFlags::DECIMAL_MODE);
            }
            // INX increases the X register
            14 => {
                self.inx();
            }
            // SED sets the decimal flag
            15 => {
                debug!("accessed SED!");
                self.flags.insert(CpuFlags::DECIMAL_MODE);
            }
            _ => unimplemented!("Unknown high nibble {} for SB1)", highnibble),
        };
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
        //println!("tsx: Initalized. Stack pointer is {}", self.sp);
        self.x = self.sp;
        self.zero_negative_flag(self.x);
    }

    fn dex(&mut self) {
        self.x = self.x.wrapping_sub(1);
        self.zero_negative_flag(self.x);
    }

    pub fn sb_two(&mut self, highnibble: u8) {
        // Group 2 single byte instructions, lownibble A and high nibble >= 8
        //println!("sb_two: Initalized");
        self.add_cycles(2);
        match highnibble {
            // TXA
            8 => {
                self.txa();
            }
            // TXS
            9 => {
                self.sp = self.x;
            }
            // TAX
            10 => {
                self.tax();
            }
            11 => {
                self.tsx();
            }
            12 => {
                self.dex();
            }
            // 0xDA, unofficial NOP
            13 | 14 | 15 => return,
            _ => unimplemented!("Unknown highnibble {} with low nibble 0xA(SB2)", highnibble),
        };
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
        //println!("ora: a: {:#b} and cmp: {:3b}", self.a, cmp);
        self.a |= cmp;
        self.zero_negative_flag(self.a);
        //println!("ora: Finished!")
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
        //println!("add_to_a: a is {:#b}, val is {:#b}", self.a, val);
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
                                //println!("result: {:#b}, a: {:#b}, val: {:#b}", result, self.a, val);

        if ((result ^ self.a) & (result ^ val) & 0x80) != 0 {
            // Signed overflow(or underflow) occured
            //println!("add_to_a: overflow assigned!");
            self.flags.insert(CpuFlags::OVERFLOW);
        } else {
            //println!("add_to_a: overflow removed!");
            self.flags.remove(CpuFlags::OVERFLOW);
        }

        self.a = result;
        self.zero_negative_flag(self.a);
        //println!("add to a: final result is {}", self.a);
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
        //println!("sbc: Old value is {:#b}, reverted value is {:#b}", val, mem);
        self.add_to_a(mem);
    }

    fn sta(&mut self, addr: u16) {
        self.mem_write(addr, self.a);
        let checked_value = self.mem_read(addr);
        debug!(
            "sta wrote {:4X} on address {:4X} with checked value {:4X}",
            self.a, addr, checked_value
        );
    }

    fn lda(&mut self, addr: u16) {
        debug!("lda: Initalized, reading address {:4X}", addr);
        self.a = self.mem_read(addr);
        debug!("lda: a register is {:4X}", self.a);
        self.zero_negative_flag(self.a);
    }

    // Used for CPY, CMP, CPX
    fn compare(&mut self, addr: u16, val: u8) {
        // BUG need to figure out val and mem[addr]
        let addr_val = self.mem_read(addr);
        debug!("compare: val is {:#x}, addr_val is {:#x}", val, addr_val);

        let res = val.wrapping_sub(addr_val);
        debug!("res value is {:#X}", res);
        if val >= addr_val {
            debug!("Added carry!");
            self.flags.insert(CpuFlags::CARRY);
        } else {
            debug!("removed carry!");
            self.flags.remove(CpuFlags::CARRY);
        }
        if val == addr_val {
            debug!("added zero!");
            self.flags.insert(CpuFlags::ZERO);
        } else {
            self.flags.remove(CpuFlags::ZERO);
        }
        let neg_test = val.wrapping_sub(addr_val) >> 7;

        if neg_test == 1 {
            debug!("added negative");
            self.flags.insert(CpuFlags::NEGATIVE);
        } else {
            debug!("removed negative");
            self.flags.remove(CpuFlags::NEGATIVE);
        }

        debug!("the flags are {:#X}", self.flags.bits());
    }

    fn cmp(&mut self, addr: u16) {
        debug!("a value is {:#x}", self.a);
        self.compare(addr, self.a);
    }

    pub fn group_one(&mut self, aaa: u8, bbb: u8, _cc: u8) {
        // Group 1
        //println!("group_one: Initalized");
        let mode = self.group_one_bbb(bbb);
        //println!("group_one: Selected mode {}, bbb is {:3b}", mode, bbb);
        let old_addr = self.mem_read_u16(self.pc);
        let addr = self.get_operand_address(&mode); // Memory location of the value to extract
        self.g1_cycles(&mode, addr, aaa == 4); // Adds cycles based on addressing mode, if aaa is 4, we're dealing with STA
        let instr = match aaa {
            0 => {
                self.ora(addr);
                "ORA"
            }
            1 => {
                self.and(addr);
                "AND"
            }
            2 => {
                self.eor(addr);
                "EOR"
            }
            3 => {
                self.adc(addr);
                "ADC"
            }
            4 => {
                self.sta(addr);
                "STA"
            }
            5 => {
                self.lda(addr);
                "LDA"
            }
            6 => {
                self.cmp(addr);
                "CMP"
            }
            7 => {
                self.sbc(addr);
                "SBC"
            }
            _ => unimplemented!("aaa"),
        };
        debug!("g1 the flags are {:#X}", self.flags.bits());
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
        //println!("asl: val is {:#b}", val);
        let carry_bit = val >> 7;
        //println!("asl: Carry bit is {:#b}", carry_bit);
        if carry_bit == 1 {
            self.flags.insert(CpuFlags::CARRY);
        } else {
            self.flags.remove(CpuFlags::CARRY);
        }
        let new_val: u8;
        if !accum {
            //println!("asl: Shifting {:#b}!", val);
            new_val = val << 1;
            self.mem_write(addr, new_val);
        } else {
            new_val = self.a << 1;
            //println!("asl: Modifying accumulator, old value is {:#b}", self.a);
            self.a = new_val;
            //println!("asl: accumulator new value is {:#b}", self.a);
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
        self.mem_write(addr, self.x);
        let test = self.mem_read(addr);
        debug!(
            "wrote value {:4X} into {:4X}, checked value is {:4X}",
            self.x, addr, test
        );
    }

    fn ldx(&mut self, addr: u16) {
        debug!("LDX: Address is {:4X}", addr);
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

    fn group_two(&mut self, aaa: u8, bbb: u8, _cc: u8, op: u8) {
        let mode = {
            match op {
                // LDX is special
                0xB6 | 0x96 => AddressingMode::ZeroPage_Y,
                0xBE => AddressingMode::Absolute_Y,
                _ => self.group_two_three_bbb(bbb),
            }
        };
        let accum = matches!(mode, AddressingMode::Accumulator);
        let old_addr = self.mem_read_u16(self.pc);
        let addr = if !accum {
            self.get_operand_address(&mode)
        } else {
            0
        };
        // Adding cycles
        match aaa {
            4 | 5 => self.g1_cycles(&mode, addr, false), // stx and ldx works separately
            _ => self.g2_default_cycles(&mode),          // Rest of g2 cycles can go here instead
        }

        match aaa {
            0 => {
                self.asl(addr, accum);
            }
            1 => {
                self.rol(addr, accum);
            }
            2 => {
                self.lsr(addr, accum);
            }
            3 => {
                self.ror(addr, accum);
            }
            4 => {
                self.stx(addr);
            }
            5 => {
                self.ldx(addr);
            }
            6 => {
                self.dec(addr);
            }
            7 => {
                self.inc(addr);
            }
            _ => unimplemented!("Unknown aaa code {}", aaa),
        };
    }

    fn bit(&mut self, addr: u16) {
        let val = self.mem_read(addr);
        if (self.a & val) == 0 {
            self.flags.insert(CpuFlags::ZERO);
        } else {
            self.flags.remove(CpuFlags::ZERO);
        }
        //println!("bit: val is {:#b}", val);
        let overflow = (val >> 6) & 0b01;
        //println!("bit: overflow {:#b}", overflow);
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
        debug!("jmp: Initalized with address {:#x}", addr);
        let val = addr;
        //println!("jmp: val is {:#x}", val);
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
        //println!("cpx: Initalized");
        //println!("x is {}", self.x);
        self.compare(addr, self.x);
    }

    // This code will read the next item in the pc and set the pc to jump there with + 1 to go to the next instruction
    fn branch(&mut self) {
        //println!(
        //     "branch: Initalized, starting to branch from pc {:#x}!",
        //     self.pc
        // );
        self.add_cycles(1); // Branch taken
        let old_page = self.pc.wrapping_add(1) >> 8; // Relative to the start of the NEXT instruction!
        let jump = self.mem_read(self.pc) as i8;
        //println!("branch: jump is {:x}", jump);
        // NOTE We do not need to add 2 to the pc as at then end of every run cycle will add 1, the other 1 is added since the pc is on the address instead of the instruction

        self.pc = self.pc.wrapping_add(jump as u16);
        // NOTE For cycles, we add an additional 1 to emulate for last pc at the end of run(this does not edit the current pc value)
        let new_page = self.pc.wrapping_add(1) >> 8;
        debug!("branch: old_page is {:2X}, new_page is {:2X}", old_page, new_page);
        if old_page != new_page {
            self.add_cycles(1);
        }


        //println!("Finished branch, pc is now on {:#x}", self.pc);
    }

    fn if_contain_flag_branch(&mut self, flag: CpuFlags) {
        //println!("if_contain_flag_branch: Checking flag {:#b}", flag);
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
        //println!("brk: Initalized");
        //println!("brk: pc is {}", self.pc);
        self.stack_push_u16(self.pc.wrapping_add(2));
        self.stack_push(self.flags.bits());
        self.flags.insert(CpuFlags::INTERRUPT_DISABLE);
        self.pc = self.mem_read_u16(0xFFFE);
        //println!("brk: Set pc to {}", self.pc);
        self.add_cycles(7);
    }

    fn jsr(&mut self) {
        // Pushes the 16 bit value after self.pc
        // Note that self.pc is already on the memory value so we just need to push this part + 1
        // Eg. JSR 0xAA 0xBB, we would be pushing the memory address of 0xBB
        // When rts is called, pc will add 1 automatically so it returns from the next function
        //println!(
        //     "jsr: Initalized! The instruction's address is {:#x}",
        //     self.pc
        // );
        self.stack_push_u16(self.pc.wrapping_add(2));
        // Need to subtract one at the end as run() will add one automatically
        let new_pc = self.mem_read_u16(self.pc.wrapping_add(1)).wrapping_sub(1);
        //println!("jsr: Going to new address: {:#x}", new_pc + 1);
        self.pc = new_pc;
        self.add_cycles(6); // 6 cycles no matter what
    }

    fn rti(&mut self) {
        // Most likely coming from a BRK(software IRQ)- BRK is treated as a 2 byte instruction with an unused immediate
        let temp_flag = self.stack_pop();
        debug!("temp_flag is {:02X}", temp_flag);
        self.flags = CpuFlags::from_bits_truncate(temp_flag);
        self.flags.remove(CpuFlags::BREAK);
        self.flags.insert(CpuFlags::BREAK2);
        self.pc = self.stack_pop_u16();
        // Need to subtract one pc to balance out with the end of run(), which adds one to pc
        self.pc = self.pc.wrapping_sub(1);
        self.add_cycles(6);
    }

    fn rts(&mut self) {
        self.pc = self.stack_pop_u16();
        //println!(
        //     "rts: Finished. The pc before finishing run is {:#x}",
        //     self.pc
        // );
        // self.pc does not need to be added as at the end of run, the pc will be added by 1 automatically
        self.add_cycles(6);
    }

    fn group_three(&mut self, aaa: u8, bbb: u8, cc: u8) {
        debug!("group_three: Initalized");
        if bbb == 0b010 {
            unimplemented!(
                "group_three: Group Three bbb does not support accumulator! {}",
                bbb
            )
        } else if bbb == 0b100 {
            // Group 3 cycles: 2 will be added no matter what
            // 1 additional if branch is taken(self.branch is called)
            // 1 addition if page crossed(checked in self.branch)
            self.add_cycles(2);
            self.pc = self.pc.wrapping_add(1);
            // Checking for branches
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
            debug!("group_three: Actually in group 3!");
            let mut mode = self.group_two_three_bbb(bbb);
            // Hardcoding jmp rel
            if aaa == 0b011 && bbb == 0b011 {
                // This is jump relative, replace the mode
                //println!("group_three: This is jmp indirect!");
                mode = AddressingMode::Indirect;
            }
            let addr = self.get_operand_address(&mode);
            //println!(
            //     "group_three: Deciding what instruction with aaa: {:#b} and address {:#x}",
            //     aaa, addr
            // );
            // Everything but jmp follows the cycles from group 1
            if aaa != 0b010 && aaa != 0b011 {
                self.g1_cycles(&mode, addr, false);
            } else {
                // jmp cycles
                match mode {
                    AddressingMode::Absolute => self.add_cycles(3),
                    AddressingMode::Indirect => self.add_cycles(5),
                    _ => panic!("{} not implemented for JMP", mode),
                }
            }
            match aaa {
                1 => {
                    self.bit(addr);
                }
                0b010 | 0b011 => {
                    debug!("jumping!");
                    self.jmp(addr);
                }
                4 => {
                    self.sty(addr);
                }
                5 => {
                    self.ldy(addr);
                }
                6 => {
                    self.cpy(addr);
                }
                7 => {
                    self.cpx(addr);
                }
                _ => unimplemented!(
                    "Unknown  code for group three {:#3b}{:#3b}{:#2b}",
                    aaa,
                    bbb,
                    cc
                ),
            }
        }
    }
}
