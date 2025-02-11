use std::{thread, time::Duration};

type Byte = u8;

pub struct Mos{
    pub pc: u16,
    pub a: Byte,
    pub x: Byte,
    pub y: Byte,
    pub stack_pointer: Byte,
    pub flags: Byte,
    // address bus
    address: u16,
    // Array of 8 bit integers(FF) with length of 0x10000(0x0000 to 0xFFFF)
    pub memory : [u8; 0x10000],
    pub clock_time:  Duration, // TODO change
    // 256 x 224 pixels(NTSC)
}

impl Mos{
    pub fn initalize() -> Self{
        let mut emu = Mos {
            pc: 0,
            a: 0,
            x: 0,
            y: 0,
            stack_pointer: 0xFD,
            // sets to (00110100)
            flags: 0x34 , 
            address: 0,
            memory: [0; 0x10000],
            clock_time: Duration::from_millis(1), // Example value
        };
        emu.reset();
        emu
    }

    pub fn reset(&mut self) {
        self.pc = (self.memory[0xFFFD] as u16) << 8 | self.memory[0xFFFC] as u16;
        self.stack_pointer = 0xFD;
        self.flags = 0x34;
    }

    fn read_byte(&self, address: &Byte) -> Byte{
        thread::sleep(self.clock_time);
        *address as Byte
   }

   pub fn read_pc(&mut self) -> Byte{
        let ret = self.read_byte(&self.memory[self.pc as usize]);
        self.pc += 1;
        ret
   }

    // This function writes directly to memory given an address
    // Used to bypass immutable passes that write_Byte occurs when trying to reference self.memory
    fn write_byte_memory(&mut self, address: usize, value: Byte){
        thread::sleep(self.clock_time);
        self.memory[address] = value;
    }

    fn read_address(&self, offset: usize) -> u16 {
        let mut val = self.read_byte(&self.memory[offset + 1]) as u16;
        val <<= 8;
        val |= self.read_byte(&self.memory[offset]) as u16;
        val
    }

    pub fn push_stack(&mut self, value: Byte){
        let address = 0x0100 + self.stack_pointer as usize;
        self.write_byte_memory(address, value);
        self.stack_pointer = self.stack_pointer.wrapping_sub(1);
    }

    // Pop from value from the top of the stack
    pub fn pull_stack(&mut self) -> u8{
        self.stack_pointer = self.stack_pointer.wrapping_add(1);
        let value = self.read_byte(&self.memory[0x100 + self.stack_pointer as usize]);

        self.flag_update(value);

        value
    }
    
    pub fn delay(&self){
        thread::sleep(self.clock_time);
    }


    // used to update the flag based on the given values
    // Z = result == 0
    // N = result bit 7
    pub fn flag_update(&mut self, value: Byte){
        self.flags = (self.flags & 0x7D) | (value & 0x80) | ((if value == 0 {1} else {0}) << 1);
        thread::sleep(self.clock_time);
    }

}
