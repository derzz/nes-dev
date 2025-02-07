use std::{thread, time::{self, Duration}};

type byte = u8;
pub struct mos{
    pc: u16,
    a: byte,
    x: byte,
    y: byte,
    stackPointer: byte,
    flags: byte,
    // address bus
    address: u16,
    memory : [u8; 0x10000],
    CLOCK_TIME: Duration, // TODO change
    // 256 x 224 pixels(NTSC)
}

impl mos {
    pub fn new() -> Self {
        let mut emu = mos {
            pc: 0,
            a: 0,
            x: 0,
            y: 0,
            stackPointer: 0xFD,
            // sets to (00110100)
            flags: 0x34 , 
            address: 0,
            memory: [0; 0x10000],
            CLOCK_TIME: Duration::from_millis(1), // Example value
        };
        emu.reset();
        emu
    }

    pub fn reset(&mut self) {
        self.pc = (self.memory[0xFFFD] as u16) << 8 | self.memory[0xFFFC] as u16;
        self.stackPointer = 0xFD;
        self.flags = 0x34;
    }

    fn read_byte(&self, address:usize) -> byte{
        thread::sleep(self.CLOCK_TIME);
        self.memory[address]
    }

    fn write_byte(&mut self, address: usize, value: byte){
        thread::sleep(self.CLOCK_TIME);
        self.memory[address] = value;
    }

    fn read_address(&self, offset: usize) -> u16 {
        let mut val = self.read_byte(offset + 1) as u16;
        val <<= 8;
        val |= self.read_byte(offset) as u16;
        val
    }

    //TODO may need push and pop for stack

    // instructions have bit patterns aaabbbcc
    // aaa and cc determine opcode, bbb determines address
    fn execute(&mut self, op: byte){
        let highnibble = op >> 4;
        let lownibble = copcode & 0x0F;

        if(lownibble == 0){
            // Single byte instructions, don't need to read bytes past the value
            // Eg. PHP, CLC, INX
            // lower nibble of opcode is 0x_8(eg. 0x08...0xF8)
            // Pattern represents (_ _ _ _ 1000)
            match highnibble {
                0 =>{
                    // PHP(push processor status) stores a byte to the stack containing the flags NV11DDIZC and decrements stack pointer
                    // Note B Flag is marked as 1 for PHP
                    memory[0x0100 + self.stackPointer] = self.flags | 0x30; // 0x30 = 00110000
                    self.stackPointer = self.stackPointer.wrapping_sub(1) // Subtracts 1 but if goes negative, will wrap to max val of u8(255)
                },

                _ => unimplemented!("Unknown high nibble for group 1(single byte instructions): {} \n (how did you even get here, high nibble is all covered?)", highnibble),
            }
        } 


    }

    // cc = 01 codes

}
