use std::{thread, time::Duration};

type Byte = u8;
pub struct Mos{
    pc: u16,
    a: Byte,
    x: Byte,
    y: Byte,
    stack_pointer: Byte,
    flags: Byte,
    // address bus
    address: u16,
    memory : [u8; 0x10000],
    clock_time:  Duration, // TODO change
    // 256 x 224 pixels(NTSC)
}

    fn main(){
        let mut emulator = Mos::new();
        emulator.execute(0xF8); // TODO Revamp instruction, used to accept instructions
    }
impl Mos {
    pub fn new() -> Self {
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

    // This function writes directly to memory given an address
    // Used to bypass immutable passes that write_Byte occurs when trying to reference self.memory
    fn write_byte_memory(&mut self, address: usize, value: Byte){
        thread::sleep(self.clock_time);
        self.memory[address] = value;
    }

    fn read_address(&self, offset: usize) -> u16 {
        let mut val = self.read_byte(&self.memory[offset + 1]) as u16;
        val <<= 8;
        val |= self.read_byte(&self.memory[offset + 1]) as u16;
        val
    }

    // Pushes a value to the stack
    fn push_stack(&mut self, value: Byte){
        let address = 0x0100 + self.stack_pointer as usize;
        self.write_byte_memory(address, value);
        self.stack_pointer = self.stack_pointer.wrapping_sub(1);
    }

    // Pop from value from the top of the stack
    fn pull_stack(&mut self) -> u8{
        self.stack_pointer = self.stack_pointer.wrapping_add(1);
        let value = self.read_byte(&self.memory[0x100 + self.stack_pointer as usize]);

        // Update flags Z if result == 0; N based on result bit 7
        // Reset flags at start
        // 0x80 is 1000 000(twos notation)
        self.flags = (self.flags & 0x7D) | (value & 0x80) | ((if value == 0 {1} else {0}) << 1);

        thread::sleep(self.clock_time);

        value
    }

    // used to update the flag based on the given values
    // Z = result == 0
    // N = result bit 7
    fn flag_update(&mut self, value: Byte){
        self.flags = (self.flags & 0x7D) | (value & 0x80) | ((if value == 0 {1} else {0}) << 1);
        thread::sleep(self.clock_time);

    }





    // instructions have bit patterns aaabbbcc
    // aaa and cc determine opcode, bbb determines address
    fn execute(&mut self, op: Byte){
        let highnibble = op >> 4;
        let lownibble = op & 0x0F;
        println!("in execute!");
        println!("lownibble {}", lownibble);

        if lownibble == 0x8{
            println!("In single Byte!");
            // Single Byte instructions, don't need to read Bytes past the value
            // Eg. PHP, CLC, INX
            // lower nibble of opcode is 0x_8(eg. 0x08...0xF8)
            // Pattern represents (_ _ _ _ 1000)
            match highnibble {
                0 =>{
                    // PHP(push processor status) stores a Byte to the stack containing the flags NV11DDIZC and decrements stack pointer
                    // Note B Flag is marked as 1 for PHP
                    self.push_stack(self.flags | 0x30);
                },
                1 =>{
                    // CLC(Clear carry flag) clears the carry flag
                    self.flags &= 0xFE;
                }
                2 =>{
                    // PLP(Pull processor status) increments the stack pointer and loads the value into the stack position into the 6 status flags
                    // NVxxDIZC
                    // BUG may not workout for flags
                    self.flags  = self.pull_stack() & 0xEF | 0x20;
                    // BUG Changing I needs to be delayed on instruction
                }
                3 => {
                   // SEC(set carry) sets carry flag to 1
                    self.flags |= 0x01
                }
                4 =>{
                    // PHA(Push A) stores the value of A to the current stack position and decrements it
                    self.push_stack(self.a)
                }
                5 => {
                    // CLI(Clear Interrupt Disable) clears the interrupt disable flag
                    self.flags &= 0xFB;
                }
                6 =>{
                    // PLA(Pull A) increments the stack pointer and loads the value at that stack position into A
                    self.a = self.pull_stack();
                }
                7 =>{
                    //SEI(Set Interrupt Disable) sets the interrupt disable flag
                    // BUG The effect is delayed on instruction(not implemented yet)
                    // IRQ allows this and next instruction to be serviced
                    self.flags |= 0x04
                }
                8 =>{
                    // DEY subtracts 1 from the Y register
                    self.y -= 1;
                    self.flag_update(self.y);
                }

                _ => unimplemented!("Unknown high nibble for group 1(single Byte instructions): {} \n (how did you even get here, high nibble is all covered?)", highnibble),
            }
        } 


    }


    // cc = 01 codes

}
