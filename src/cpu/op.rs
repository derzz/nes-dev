// Opcodes used in testing, unneeded in cpu.rs due to cpu.rs following specific addressing properties
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
