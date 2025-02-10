use crate::mos::Mos;

pub mod instruction{
    use super::Mos;

    pub fn execute(mos: &mut Mos, op: u8){
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
                    mos.push_stack(mos.flags | 0x30);
                },
                1 =>{
                    // CLC(Clear carry flag) clears the carry flag
                    mos.flags &= 0xFE;
                }
                2 =>{
                    // PLP(Pull processor status) increments the stack pointer and loads the value into the stack position into the 6 status flags
                    // NVxxDIZC
                    // BUG may not workout for flags
                    mos.flags  = mos.pull_stack() & 0xEF | 0x20;
                    // BUG Changing I needs to be delayed on instruction
                }
                3 => {
                   // SEC(set carry) sets carry flag to 1
                    mos.flags |= 0x01
                }
                4 =>{
                    // PHA(Push A) stores the value of A to the current stack position
                    mos.push_stack(mos.a)
                }
                5 => {
                    // CLI(Clear Interrupt Disable) clears the interrupt disable flag
                    mos.flags &= 0xFB;
                }
                6 =>{
                    // PLA(Pull A) increments the stack pointer and loads the value at that stack position into A
                    mos.a = mos.pull_stack();
                }
                7 =>{
                    //SEI(Set Interrupt Disable) sets the interrupt disable flag
                    // BUG The effect is delayed on instruction(not implemented yet)
                    // IRQ allows this and next instruction to be serviced
                    mos.flags |= 0x04
                }
                8 =>{
                    // DEY subtracts 1 from the Y register
                    mos.y -= 1;
                    mos.flag_update(mos.y);
                }9 =>{
                    // TYA transfers the Y register to the accumulator
                    mos.a = mos.y;
                    mos.flag_update(mos.a);
                }
                10 =>{
                    // TAY transfer accumulator to register
                    mos.y = mos.a;
                    mos.flag_update(mos.y);
                }
                11 =>{
                    // CLV clears the overflow tag
                    mos.flags &= 0xBF;
                }
                12 =>{
                    // INY increases the Y register
                    mos.y += 1;
                    mos.flag_update(mos.y);
                }
                13 =>{
                    // CLD clears the decimal flag
                    mos.flags &= 0xF7;
                }
                14 =>{
                    // INX increases the X register
                    mos.x += 1;
                    mos.flag_update(mos.x);
                }
                15 =>{
                  // SED sets the decimal flag
                  mos.flags |=  0x8
                },

                _ => unimplemented!("Unknown high nibble {} for SB1)", highnibble),
            }
        } 
        else if lownibble == 0xA && highnibble >= 0x8{
            // Group 2 single byte instructions
            match highnibble{
                8 =>{
                    // TXA
                    mos.a = mos.x;
                    mos.flag_update(mos.a);
                }
                9 => {
                    // TXS transfers x to stack pointer
                    mos.stack_pointer= mos.x;
                    // No need to change flags
                }
                10 =>{
                    // TAX
                    mos.x = mos.a;
                    mos.flag_update(mos.x);
                }
                11 =>{
                    // TSX
                    mos.x = mos.stack_pointer;
                }
                12 =>{
                    // DEX
                    mos.x -= 1;
                    mos.flag_update(mos.x);
                }
                13 =>{
                    // Phx
                    unimplemented!("Phx not implemented")
                }
                14 =>{
                    // NOP
                    // BUG may delay time
                    mos.delay();
                }
                15 =>{
                    unimplemented!("Plx not implemented")
                }
                _ => {
                    unimplemented!("Unknown highnibble {} with low nibble 0xA(SB2)", highnibble)
                }
            }
        }
        else{
            unimplemented!("Unknown opcode {}", op)
        }


    }
}
