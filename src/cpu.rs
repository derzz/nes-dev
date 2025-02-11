use std::{ops::Add, thread, time::Duration};

type Byte = u8;
mod tests;
pub struct CPU {
    pub pc: u16,
    pub a: Byte,
    pub x: Byte,
    pub y: Byte,
    pub sp: Byte,
    pub flags: Byte,
    // address bus
    address: u16,
    // [0x8000... 0xFFFF] is reserved for program ROM
    pub memory: [u8; 0xFFFF],
    pub clock_time: Duration, // TODO change
                              // 256 x 224 pixels(NTSC)
}


#[derive(Debug)]
#[allow(non_camel_case_types)]
pub enum AddressingMode{
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
}
#[derive(Debug, Clone, Copy)]
pub enum CpuFlags {
    Carry = 0b00000001,
    Zero = 0b00000010,
    InterruptDisable = 0b00000100,
    DecimalMode = 0b00001000,
    Break = 0b00010000,
    Break2 = 0b00100000,
    Overflow = 0b01000000,
    Negative = 0b10000000,
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
            flags: 0b00100100,
            address: 0,
            memory: [0; 0xFFFF],
            clock_time: Duration::from_millis(1), // Example value
        }
    }

    // Used to read address in little endian
    fn mem_read_u16(&mut self, pos: u16) -> u16 {
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

   // Restores registers and initalizes PC to the 2 byte value at 0xFFFC
   pub fn reset(&mut self){
        self.a = 0;
        self.x = 0;
        self.y = 0;
        self.flags = 0b00100100;
        self.sp = STACK_RESET;
        self.pc = self.mem_read_u16(0xFFFC);
   }

   pub fn load(&mut self, program: Vec<u8>){
    self.memory[0x8000..(0x8000 + program.len())].copy_from_slice(&program[..]);
    self.mem_write_u16(0xFFFC, 0x8000); // Save reference to program in 0xFFFC
   }

   pub fn load_and_run(&mut self, program: Vec<u8>){
    println!("in load and run!");
    self.load(program);
    self.reset();
    self.run();
   }

    // used to update the flag based on the given values
    // Z = result == 0
    // N = result bit 7
    pub fn zero_negative_flag(&mut self, value: Byte) {
        self.flags = (self.flags & 0x7D) | (value & 0x80) | ((if value == 0 { 1 } else { 0 }) << 1);
    }

    fn mem_read(&mut self, addr: u16) -> Byte {
        let ret = self.memory[addr as usize];
        self.pc += 1;
        ret
    }

    fn mem_write(&mut self, addr: u16, data: u8) {
        let ret = self.memory[addr as usize] = data;
        self.pc += 1;
        ret
    }


    pub fn run(&mut self) {
        loop {
            println!("reading");
            let op = self.mem_read(self.pc);

            let highnibble = op >> 4;
            let lownibble = op & 0x0F;
            println!("Highnibble {} and lownibble {}", highnibble, lownibble);
            let aaa = op >> 5;
            let bbb = (op >> 2) & 0x7;
            let cc = op & 0x3; // Used for identification of group 1, 2, and 3

            if lownibble == 0x8 {
                self.sb_one(highnibble);
            } else if lownibble == 0xA && highnibble >= 0x8 {
                self.sb_two(highnibble);
            } else if cc == 0x01 {
                self.group_one(aaa, bbb, cc);
            } else if op == 0x00 {
                return;
            } else {
                unimplemented!("Unknown opcode {}", op)
            }
        }
    }

    fn get_operand_address(&mut self, mode: &AddressingMode) -> u16 {

       match mode {
           AddressingMode::Immediate => self.pc,

           AddressingMode::ZeroPage  => self.mem_read(self.pc) as u16,
          
           AddressingMode::Absolute => self.mem_read_u16(self.pc),
        
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
               let base = self.mem_read_u16(self.pc);
               let addr = base.wrapping_add(self.x as u16);
               addr
           }
           AddressingMode::Absolute_Y => {
               let base = self.mem_read_u16(self.pc);
               let addr = base.wrapping_add(self.y as u16);
               addr
           }

           // (c0, X)
           // Looks at the address at LSB = c0 + X and MSB = c0 + X + 1 => Address LSB + MSB
           AddressingMode::Indirect_X => {
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

   fn stack_push(&mut self, data: u8){
    self.mem_write((STACK as u16) + self.sp as u16, data);
    self.sp = self.sp.wrapping_sub(1);
   }

   fn stack_pull(&mut self) -> u8{
    let ret = self.mem_read((STACK as u16) + self.sp as u16);
    self.sp = self.sp.wrapping_add(1);
    ret
   }

    fn php(&mut self){
        let mut flags = self.flags.clone();
        flags |= 0b00110000;
        self.stack_push(flags);
    }
    
    // Used for grouping addressing modes
    fn sb_one(&mut self, highnibble: u8) {
        println!("In single Byte!");
        // Single Byte instructions, don't need to read Bytes past the value
        // Eg. PHP, CLC, INX
        // lower nibble of opcode is 0x_8(eg. 0x08...0xF8)
        // Pattern represents (_ _ _ _ 1000)
        match highnibble {
            0 => {
                // PHP(push processor status) stores a Byte to the stack containing the flags NV11DDIZC and decrements stack pointer
                // Note B Flag is marked as 1 for PHP
                self.php();
                
            }
            1 => {
                // CLC(Clear carry flag) clears the carry flag
                self.flags &= 0xFE;
            }
            2 => {
                // PLP(Pull processor status) increments the stack pointer and loads the value into the stack position into the 6 status flags
                // NVxxDIZC
                todo!("PLP")
            }
            3 => {
                // SEC(set carry) sets carry flag to 1
                self.flags |= 0x01
            }
            4 => {
                // PHA(Push A) stores the value of A to the current stack position
                todo!("PHA")
            }
            5 => {
                // CLI(Clear Interrupt Disable) clears the interrupt disable flag
                self.flags &= 0xFB;
            }
            6 => {
                // PLA(Pull A) increments the stack pointer and loads the value at that stack position into A
                todo!("PLA")
            }
            7 => {
                //SEI(Set Interrupt Disable) sets the interrupt disable flag
                // BUG The effect is delayed on instruction(not implemented yet)
                // IRQ allows this and next instruction to be serviced
                self.flags |= 0x04
            }
            8 => {
                // DEY subtracts 1 from the Y register
                self.y -= 1;
                self.zero_negative_flag(self.y);
            }
            9 => {
                // TYA transfers the Y register to the accumulator
                self.a = self.y;
                self.zero_negative_flag(self.a);
            }
            10 => {
                // TAY transfer accumulator to register
                self.y = self.a;
                self.zero_negative_flag(self.y);
            }
            11 => {
                // CLV clears the overflow tag
                self.flags &= 0xBF;
            }
            12 => {
                // INY increases the Y register
                self.y += 1;
                self.zero_negative_flag(self.y);
            }
            13 => {
                // CLD clears the decimal flag
                self.flags &= 0xF7;
            }
            14 => {
                // INX increases the X register
                self.x += 1;
                self.zero_negative_flag(self.x);
            }
            15 => {
                // SED sets the decimal flag
                self.flags |= 0x8
            }

            _ => unimplemented!("Unknown high nibble {} for SB1)", highnibble),
        }
    }

    pub fn sb_two(&mut self, highnibble: u8) {
        // Group 2 single byte instructions
        match highnibble {
            8 => {
                // TXA
                self.a = self.x;
                self.zero_negative_flag(self.a);
            }
            9 => {
                // TXS transfers x to stack pointer
                self.sp = self.x;
                // No need to change flags
            }
            10 => {
                // TAX
                self.x = self.a;
                self.zero_negative_flag(self.x);
            }
            11 => {
                // TSX
                self.x = self.sp;
            }
            12 => {
                // DEX
                self.x -= 1;
                self.zero_negative_flag(self.x);
            }
            13 => {
                // Phx
                unimplemented!("Phx not implemented")
            }
            14 => {
                // NOP
                // BUG may delay time
                todo!("NOP")
            }
            15 => {
                unimplemented!("Plx not implemented")
            }
            _ => {
                unimplemented!("Unknown highnibble {} with low nibble 0xA(SB2)", highnibble)
            }
        }
    }

    // Takes in the address location
    fn lda(&mut self, addr: u16) {
        println!("In lda, reading address {}", addr);
        self.a = self.mem_read(addr);
        self.zero_negative_flag(self.a);
    }

    fn group_one_bbb(&mut self, bbb: u8) -> AddressingMode{
        println!("in bbb");
        match bbb{
            0 => AddressingMode::Indirect_X,
            1 => AddressingMode::ZeroPage,
            2 => AddressingMode::Immediate,
            3 => AddressingMode::Absolute,
            4 => AddressingMode::ZeroPage_Y,
            5 => AddressingMode::ZeroPage_X,
            6 => AddressingMode::Absolute_Y,
            7 => AddressingMode::Absolute_Y,
            _ => {
                unimplemented!("Unknown addressing mode for group 1 {}", bbb);
            }
        }
    }

    pub fn group_one(&mut self, aaa: u8, bbb: u8, cc: u8) {
        // Group 1
        println!("In group one");
        let cmp = aaa == 7;
        let mode = self.group_one_bbb(bbb);
        let addr = self.get_operand_address(&mode); // Memory location of the value to extract
        match aaa {
            5 => {
                // LDA
                self.lda(addr);
                // Bytes read will increment by themselves
            }
            _ => {
                unimplemented!("aaa")
            }
        }
    }
}
