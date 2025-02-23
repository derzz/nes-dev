// Opcodes used in testing, unneeded in cpu.rs due to cpu.rs following specific addressing properties
// https://llx.com/Neil/a2/opcodes.html#ins816
#[cfg(test)]
pub mod op {
    // SBI1
    pub const PHP: u8 = 0x08;
    pub const CLC: u8 = 0x18;
    pub const PLP: u8 = 0x28;
    pub const SEC: u8 = 0x38;
    pub const PHA: u8 = 0x48;
    pub const CLI: u8 = 0x58;
    pub const PLA: u8 = 0x68;
    pub const SEI: u8 = 0x78;
    pub const DEY: u8 = 0x88;
    pub const TYA: u8 = 0x98;
    pub const TAY: u8 = 0xA8;
    pub const CLV: u8 = 0xB8;
    pub const INY: u8 = 0xC8;
    pub const CLD: u8 = 0xD8;
    pub const INX: u8 = 0xE8;
    pub const SED: u8 = 0xF8;

    // SBI2
    pub const TXA: u8 = 0x8A;
    pub const TXS: u8 = 0x9A;
    pub const TAX: u8 = 0xAA;
    pub const TSX: u8 = 0xba;
    pub const DEX: u8 = 0xca;
    pub const NOP: u8 = 0xea;
}

// Opcodes used for multi bit group 1 instructions
// Add the addressing mode and the opcode to get full opcode
#[cfg(test)]
pub mod g1_op {
    // LSB used for addressing mode
    pub const INDIRECT: u8 = 0x01; // Used with (zp, X) and (zp), Y
    pub const ZP: u8 = 0x05; // Used with zp, zp, X
    pub const IMMEDIATE_Y: u8 = 0x09; // Used with # and abs, Y
    pub const ABSOLUTE_X: u8 = 0x0D; // Used with abs and abs,X

    // First half will be instructions for (zp, X); zp; #; abs
    // Second half is instructions for (zp), Y; zp,X; abs,Y; abs,X
    pub const FIRST_ORA: u8 = 0x00;
    pub const SECOND_ORA: u8 = 0x10;
    pub const FIRST_AND: u8 = 0x20;
    pub const SECOND_AND: u8 = 0x30;
    pub const FIRST_EOR: u8 = 0x40;
    pub const SECOND_EOR: u8 = 0x50;
    pub const FIRST_ADC: u8 = 0x60;
    pub const SECOND_ADC: u8 = 0x70;
    pub const FIRST_STA: u8 = 0x80;
    pub const SECOND_STA: u8 = 0x90;
    pub const FIRST_LDA: u8 = 0xA0;
    pub const SECOND_LDA: u8 = 0xB0;
    pub const FIRST_CMP: u8 = 0xC0;
    pub const SECOND_CMP: u8 = 0xD0;
    pub const FIRST_SBC: u8 = 0xE0;
    pub const SECOND_SBC: u8 = 0xF0;
}

#[cfg(test)]
pub mod g2_op {
    pub const IMMEDIATE: u8 = 0x02;
    pub const ZP: u8 = 0x06; // Also used with Zp, X
    pub const A: u8 = 0x0A;
    pub const ABS: u8 = 0x0E; // Allso used with abs, X; abs, Y
}

#[cfg(test)]
pub mod g3_op {
    pub const IMMEDIATE: u8 = 0x00;
    pub const ZP: u8 = 0x04; // Used with zp; zp, X
    pub const ABS: u8 = 0x0C; // Used with abs; abs, X

    pub const CPY: u8 = 0xC0;
    pub const CPX: u8 = 0xE0;

    pub const FIRST_LDY: u8 = 0xA0; // LDX is the exact same
    pub const SECOND_LDY: u8 = 0xB0;

    pub const FIRST_STY: u8 = 0x80;
    pub const SECOND_STY: u8 = 0x90; // Only used with zp, X
}
