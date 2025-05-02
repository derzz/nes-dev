// Opcodes used for trace, not needed in the main cpu implementation atm
// Cycles are not implemented due to trace reading cpu cycles
// https://llx.com/Neil/a2/opcodes.html#ins816

use crate::cpu::AddressingMode;
use lazy_static::lazy_static;
use std::collections::HashMap;

pub struct OpCode {
    pub code: u8,
    pub lit: &'static str,
    pub len: u8,
    pub mode: AddressingMode,
}

impl OpCode {
    fn new(code: u8, lit: &'static str, len: u8, mode: AddressingMode) -> Self {
        OpCode {
            code: code,
            lit: lit,
            len: len,
            mode: mode,
        }
    }
}

lazy_static! {
    // Opcodes are in format:
    // Opcode, InstructionName, Expected Length, AddressingMode
    pub static ref CPU_OPCODES: Vec<OpCode> = vec![
        // SBI1 instructions (Single Byte, Group 1)
        OpCode::new(0x08, "PHP", 1, AddressingMode::NoneAddressing),
        OpCode::new(0x18, "CLC", 1, AddressingMode::NoneAddressing),
        OpCode::new(0x28, "PLP", 1, AddressingMode::NoneAddressing),
        OpCode::new(0x38, "SEC", 1, AddressingMode::NoneAddressing),
        OpCode::new(0x48, "PHA", 1, AddressingMode::NoneAddressing),
        OpCode::new(0x58, "CLI", 1, AddressingMode::NoneAddressing),
        OpCode::new(0x68, "PLA", 1, AddressingMode::NoneAddressing),
        OpCode::new(0x78, "SEI", 1, AddressingMode::NoneAddressing),
        OpCode::new(0x88, "DEY", 1, AddressingMode::NoneAddressing),
        OpCode::new(0x98, "TYA", 1, AddressingMode::NoneAddressing),
        OpCode::new(0xA8, "TAY", 1, AddressingMode::NoneAddressing),
        OpCode::new(0xB8, "CLV", 1, AddressingMode::NoneAddressing),
        OpCode::new(0xC8, "INY", 1, AddressingMode::NoneAddressing),
        OpCode::new(0xD8, "CLD", 1, AddressingMode::NoneAddressing),
        OpCode::new(0xE8, "INX", 1, AddressingMode::NoneAddressing),
        OpCode::new(0xF8, "SED", 1, AddressingMode::NoneAddressing),

        // SBI2 instructions (Single Byte, Group 2)
        OpCode::new(0x8A, "TXA", 1, AddressingMode::NoneAddressing),
        OpCode::new(0x9A, "TXS", 1, AddressingMode::NoneAddressing),
        OpCode::new(0xAA, "TAX", 1, AddressingMode::NoneAddressing),
        OpCode::new(0xBA, "TSX", 1, AddressingMode::NoneAddressing),
        OpCode::new(0xCA, "DEX", 1, AddressingMode::NoneAddressing),
        OpCode::new(0xEA, "NOP", 1, AddressingMode::NoneAddressing),

        // Group 1 Instructions
        // ORA
        OpCode::new(0x01, "ORA", 2, AddressingMode::Indirect_X), // (Indirect,X)
        OpCode::new(0x05, "ORA", 2, AddressingMode::ZeroPage), // Zero Page
        OpCode::new(0x09, "ORA", 2, AddressingMode::Immediate), // Immediate
        OpCode::new(0x0D, "ORA", 3, AddressingMode::Absolute), // Absolute
        OpCode::new(0x11, "ORA", 2, AddressingMode::Indirect_Y), // (Indirect),Y
        OpCode::new(0x15, "ORA", 2, AddressingMode::ZeroPage_X), // Zero Page,X
        OpCode::new(0x19, "ORA", 3, AddressingMode::Absolute_Y), // Absolute,Y
        OpCode::new(0x1D, "ORA", 3, AddressingMode::Absolute_X), // Absolute,X

        // AND
        OpCode::new(0x21, "AND", 2, AddressingMode::Indirect_X), // (Indirect,X)
        OpCode::new(0x25, "AND", 2, AddressingMode::ZeroPage), // Zero Page
        OpCode::new(0x29, "AND", 2, AddressingMode::Immediate), // Immediate
        OpCode::new(0x2D, "AND", 3, AddressingMode::Absolute), // Absolute
        OpCode::new(0x31, "AND", 2, AddressingMode::Indirect_Y), // (Indirect),Y
        OpCode::new(0x35, "AND", 2, AddressingMode::ZeroPage_X), // Zero Page,X
        OpCode::new(0x39, "AND", 3, AddressingMode::Absolute_Y), // Absolute,Y
        OpCode::new(0x3D, "AND", 3, AddressingMode::Absolute_X), // Absolute,X

        // EOR
        OpCode::new(0x41, "EOR", 2, AddressingMode::Indirect_X), // (Indirect,X)
        OpCode::new(0x45, "EOR", 2, AddressingMode::ZeroPage), // Zero Page
        OpCode::new(0x49, "EOR", 2, AddressingMode::Immediate), // Immediate
        OpCode::new(0x4D, "EOR", 3, AddressingMode::Absolute), // Absolute
        OpCode::new(0x51, "EOR", 2, AddressingMode::Indirect_Y), // (Indirect),Y
        OpCode::new(0x55, "EOR", 2, AddressingMode::ZeroPage_X), // Zero Page,X
        OpCode::new(0x59, "EOR", 3, AddressingMode::Absolute_Y), // Absolute,Y
        OpCode::new(0x5D, "EOR", 3, AddressingMode::Absolute_X), // Absolute,X

        // ADC
        OpCode::new(0x61, "ADC", 2, AddressingMode::Indirect_X), // (Indirect,X)
        OpCode::new(0x65, "ADC", 2, AddressingMode::ZeroPage), // Zero Page
        OpCode::new(0x69, "ADC", 2, AddressingMode::Immediate), // Immediate
        OpCode::new(0x6D, "ADC", 3, AddressingMode::Absolute), // Absolute
        OpCode::new(0x71, "ADC", 2, AddressingMode::Indirect_Y), // (Indirect),Y
        OpCode::new(0x75, "ADC", 2, AddressingMode::ZeroPage_X), // Zero Page,X
        OpCode::new(0x79, "ADC", 3, AddressingMode::Absolute_Y), // Absolute,Y
        OpCode::new(0x7D, "ADC", 3, AddressingMode::Absolute_X), // Absolute,X

        // STA
        OpCode::new(0x81, "STA", 2, AddressingMode::Indirect_X), // (Indirect,X)
        OpCode::new(0x85, "STA", 2, AddressingMode::ZeroPage), // Zero Page
        OpCode::new(0x8D, "STA", 3, AddressingMode::Absolute), // Absolute
        OpCode::new(0x91, "STA", 2, AddressingMode::Indirect_Y), // (Indirect),Y
        OpCode::new(0x95, "STA", 2, AddressingMode::ZeroPage_X), // Zero Page,X
        OpCode::new(0x99, "STA", 3, AddressingMode::Absolute_Y), // Absolute,Y
        OpCode::new(0x9D, "STA", 3, AddressingMode::Absolute_X), // Absolute,X

        // LDA
        OpCode::new(0xA1, "LDA", 2, AddressingMode::Indirect_X), // (Indirect,X)
        OpCode::new(0xA5, "LDA", 2, AddressingMode::ZeroPage), // Zero Page
        OpCode::new(0xA9, "LDA", 2, AddressingMode::Immediate), // Immediate
        OpCode::new(0xAD, "LDA", 3, AddressingMode::Absolute), // Absolute
        OpCode::new(0xB1, "LDA", 2, AddressingMode::Indirect_Y), // (Indirect),Y
        OpCode::new(0xB5, "LDA", 2, AddressingMode::ZeroPage_X), // Zero Page,X
        OpCode::new(0xB9, "LDA", 3, AddressingMode::Absolute_Y), // Absolute,Y
        OpCode::new(0xBD, "LDA", 3, AddressingMode::Absolute_X), // Absolute,X

        // CMP
        OpCode::new(0xC1, "CMP", 2, AddressingMode::Indirect_X), // (Indirect,X)
        OpCode::new(0xC5, "CMP", 2, AddressingMode::ZeroPage), // Zero Page
        OpCode::new(0xC9, "CMP", 2, AddressingMode::Immediate), // Immediate
        OpCode::new(0xCD, "CMP", 3, AddressingMode::Absolute), // Absolute
        OpCode::new(0xD1, "CMP", 2, AddressingMode::Indirect_Y), // (Indirect),Y
        OpCode::new(0xD5, "CMP", 2, AddressingMode::ZeroPage_X), // Zero Page,X
        OpCode::new(0xD9, "CMP", 3, AddressingMode::Absolute_Y), // Absolute,Y
        OpCode::new(0xDD, "CMP", 3, AddressingMode::Absolute_X), // Absolute,X

        // SBC
        OpCode::new(0xE1, "SBC", 2, AddressingMode::Indirect_X), // (Indirect,X)
        OpCode::new(0xE5, "SBC", 2, AddressingMode::ZeroPage), // Zero Page
        OpCode::new(0xE9, "SBC", 2, AddressingMode::Immediate), // Immediate
        OpCode::new(0xED, "SBC", 3, AddressingMode::Absolute), // Absolute
        OpCode::new(0xF1, "SBC", 2, AddressingMode::Indirect_Y), // (Indirect),Y
        OpCode::new(0xF5, "SBC", 2, AddressingMode::ZeroPage_X), // Zero Page,X
        OpCode::new(0xF9, "SBC", 3, AddressingMode::Absolute_Y), // Absolute,Y
        OpCode::new(0xFD, "SBC", 3, AddressingMode::Absolute_X), // Absolute,X

        // Group 2 Instructions
        // ASL
        OpCode::new(0x06, "ASL", 2, AddressingMode::ZeroPage), // Zero Page
        OpCode::new(0x0A, "ASL", 1, AddressingMode::Accumulator), // Accumulator
        OpCode::new(0x0E, "ASL", 3, AddressingMode::Absolute), // Absolute
        OpCode::new(0x16, "ASL", 2, AddressingMode::ZeroPage_X), // Zero Page,X
        OpCode::new(0x1E, "ASL", 3, AddressingMode::Absolute_X), // Absolute,X

        // ROL
        OpCode::new(0x26, "ROL", 2, AddressingMode::ZeroPage), // Zero Page
        OpCode::new(0x2A, "ROL", 1, AddressingMode::Accumulator), // Accumulator
        OpCode::new(0x2E, "ROL", 3, AddressingMode::Absolute), // Absolute
        OpCode::new(0x36, "ROL", 2, AddressingMode::ZeroPage_X), // Zero Page,X
        OpCode::new(0x3E, "ROL", 3, AddressingMode::Absolute_X), // Absolute,X

        // LSR
        OpCode::new(0x46, "LSR", 2, AddressingMode::ZeroPage), // Zero Page
        OpCode::new(0x4A, "LSR", 1, AddressingMode::Accumulator), // Accumulator
        OpCode::new(0x4E, "LSR", 3, AddressingMode::Absolute), // Absolute
        OpCode::new(0x56, "LSR", 2, AddressingMode::ZeroPage_X), // Zero Page,X
        OpCode::new(0x5E, "LSR", 3, AddressingMode::Absolute_X), // Absolute,X

        // ROR
        OpCode::new(0x66, "ROR", 2, AddressingMode::ZeroPage), // Zero Page
        OpCode::new(0x6A, "ROR", 1, AddressingMode::Accumulator), // Accumulator
        OpCode::new(0x6E, "ROR", 3, AddressingMode::Absolute), // Absolute
        OpCode::new(0x76, "ROR", 2, AddressingMode::ZeroPage_X), // Zero Page,X
        OpCode::new(0x7E, "ROR", 3, AddressingMode::Absolute_X), // Absolute,X

        // DEC
        OpCode::new(0xC6, "DEC", 2, AddressingMode::ZeroPage), // Zero Page
        OpCode::new(0xCE, "DEC", 3, AddressingMode::Absolute), // Absolute
        OpCode::new(0xD6, "DEC", 2, AddressingMode::ZeroPage_X), // Zero Page,X
        OpCode::new(0xDE, "DEC", 3, AddressingMode::Absolute_X), // Absolute,X

        // INC
        OpCode::new(0xE6, "INC", 2, AddressingMode::ZeroPage), // Zero Page
        OpCode::new(0xEE, "INC", 3, AddressingMode::Absolute), // Absolute
        OpCode::new(0xF6, "INC", 2, AddressingMode::ZeroPage_X), // Zero Page,X
        OpCode::new(0xFE, "INC", 3, AddressingMode::Absolute_X), // Absolute,X

        // Group 3 Instructions
        // CPY
        OpCode::new(0xC0, "CPY", 2, AddressingMode::Immediate), // Immediate
        OpCode::new(0xC4, "CPY", 2, AddressingMode::ZeroPage), // Zero Page
        OpCode::new(0xCC, "CPY", 3, AddressingMode::Absolute), // Absolute

        // CPX
        OpCode::new(0xE0, "CPX", 2, AddressingMode::Immediate), // Immediate
        OpCode::new(0xE4, "CPX", 2, AddressingMode::ZeroPage), // Zero Page
        OpCode::new(0xEC, "CPX", 3, AddressingMode::Absolute), // Absolute

        // LDY
        OpCode::new(0xA0, "LDY", 2, AddressingMode::Immediate), // Immediate
        OpCode::new(0xA4, "LDY", 2, AddressingMode::ZeroPage), // Zero Page
        OpCode::new(0xAC, "LDY", 3, AddressingMode::Absolute), // Absolute
        OpCode::new(0xB4, "LDY", 2, AddressingMode::ZeroPage_X), // Zero Page,X
        OpCode::new(0xBC, "LDY", 3, AddressingMode::Absolute_X), // Absolute,X

        // LDX
        OpCode::new(0xA2, "LDX", 2, AddressingMode::Immediate), // Immediate
        OpCode::new(0xA6, "LDX", 2, AddressingMode::ZeroPage), // Zero Page
        OpCode::new(0xAE, "LDX", 3, AddressingMode::Absolute), // Absolute
        OpCode::new(0xB6, "LDX", 2, AddressingMode::ZeroPage_Y), // Zero Page,Y
        OpCode::new(0xBE, "LDX", 3, AddressingMode::Absolute_Y), // Absolute,Y

        // STY
        OpCode::new(0x84, "STY", 2, AddressingMode::ZeroPage), // Zero Page
        OpCode::new(0x8C, "STY", 3, AddressingMode::Absolute), // Absolute
        OpCode::new(0x94, "STY", 2, AddressingMode::ZeroPage_X), // Zero Page,X

        // STX
        OpCode::new(0x86, "STX", 2, AddressingMode::ZeroPage), // Zero Page
        OpCode::new(0x8E, "STX", 3, AddressingMode::Absolute), // Absolute
        OpCode::new(0x96, "STX", 2, AddressingMode::ZeroPage_Y), // Zero Page,Y

        // JMP
        OpCode::new(0x4C, "JMP", 3, AddressingMode::Absolute), // Absolute
        OpCode::new(0x6C, "JMP", 3, AddressingMode::Indirect), // Indirect

        // BIT
        OpCode::new(0x24, "BIT", 2, AddressingMode::ZeroPage), // Zero Page
        OpCode::new(0x2C, "BIT", 3, AddressingMode::Absolute), // Absolute

        // Branch Instructions
        OpCode::new(0x10, "BPL", 2, AddressingMode::Relative), // Branch on Plus
        OpCode::new(0x30, "BMI", 2, AddressingMode::Relative), // Branch on Minus
        OpCode::new(0x50, "BVC", 2, AddressingMode::Relative), // Branch on Overflow Clear
        OpCode::new(0x70, "BVS", 2, AddressingMode::Relative), // Branch on Overflow Set
        OpCode::new(0x90, "BCC", 2, AddressingMode::Relative), // Branch on Carry Clear
        OpCode::new(0xB0, "BCS", 2, AddressingMode::Relative), // Branch on Carry Set
        OpCode::new(0xD0, "BNE", 2, AddressingMode::Relative), // Branch on Not Equal
        OpCode::new(0xF0, "BEQ", 2, AddressingMode::Relative), // Branch on Equal

        // Other Instructions
        OpCode::new(0x00, "BRK", 1, AddressingMode::NoneAddressing), // Break
        OpCode::new(0x20, "JSR", 3, AddressingMode::Absolute), // Jump to Subroutine
        OpCode::new(0x40, "RTI", 1, AddressingMode::NoneAddressing), // Return from Interrupt
        OpCode::new(0x60, "RTS", 1, AddressingMode::NoneAddressing),  // Return from Subroutine

// Unofficial NOP instructions
OpCode::new(0x1A, "*NOP", 1, AddressingMode::NoneAddressing),
OpCode::new(0x3A, "*NOP", 1, AddressingMode::NoneAddressing),
OpCode::new(0x5A, "*NOP", 1, AddressingMode::NoneAddressing),
OpCode::new(0x7A, "*NOP", 1, AddressingMode::NoneAddressing),
OpCode::new(0xDA, "*NOP", 1, AddressingMode::NoneAddressing),
OpCode::new(0xFA, "*NOP", 1, AddressingMode::NoneAddressing),


// SKB instructions (Skip Byte - read immediate and ignore)
OpCode::new(0x80, "*NOP", 2, AddressingMode::Immediate),
OpCode::new(0x82, "*NOP", 2, AddressingMode::Immediate),
OpCode::new(0x89, "*NOP", 2, AddressingMode::Immediate),
OpCode::new(0xC2, "*NOP", 2, AddressingMode::Immediate),
OpCode::new(0xE2, "*NOP", 2, AddressingMode::Immediate),

// IGN instructions (Ignore - read and ignore)
// Absolute
OpCode::new(0x0C, "*NOP", 3, AddressingMode::Absolute),

// Absolute,X
OpCode::new(0x1C, "*NOP", 3, AddressingMode::Absolute_X),
OpCode::new(0x3C, "*NOP", 3, AddressingMode::Absolute_X),
OpCode::new(0x5C, "*NOP", 3, AddressingMode::Absolute_X),
OpCode::new(0x7C, "*NOP", 3, AddressingMode::Absolute_X),
OpCode::new(0xDC, "*NOP", 3, AddressingMode::Absolute_X),
OpCode::new(0xFC, "*NOP", 3, AddressingMode::Absolute_X),

// Zero Page
OpCode::new(0x04, "*NOP", 2, AddressingMode::ZeroPage),
OpCode::new(0x44, "*NOP", 2, AddressingMode::ZeroPage),
OpCode::new(0x64, "*NOP", 2, AddressingMode::ZeroPage),

// Zero Page,X
OpCode::new(0x14, "*NOP", 2, AddressingMode::ZeroPage_X),
OpCode::new(0x34, "*NOP", 2, AddressingMode::ZeroPage_X),
OpCode::new(0x54, "*NOP", 2, AddressingMode::ZeroPage_X),
OpCode::new(0x74, "*NOP", 2, AddressingMode::ZeroPage_X),
OpCode::new(0xD4, "*NOP", 2, AddressingMode::ZeroPage_X),
OpCode::new(0xF4, "*NOP", 2, AddressingMode::ZeroPage_X),

// SHX and SHY
OpCode::new(0x9E, "SHX", 3, AddressingMode::Absolute_Y), // SHX a, Y
OpCode::new(0x9C, "SHY", 3, AddressingMode::Absolute_X), // SHY a, X

// ALR/ASR
OpCode::new(0x4B, "ALR", 2, AddressingMode::Immediate), // ALR #i

// ANC
OpCode::new(0x0B, "ANC", 2, AddressingMode::Immediate), // ANC #i
OpCode::new(0x2B, "ANC", 2, AddressingMode::Immediate), // ANC #i

// ARR
OpCode::new(0x6B, "ARR", 2, AddressingMode::Immediate), // ARR #i

// AXS/SBX
OpCode::new(0xCB, "AXS", 2, AddressingMode::Immediate), // AXS #i

// LAX
OpCode::new(0xA3, "*LAX", 2, AddressingMode::Indirect_X), // LAX (d,X)
OpCode::new(0xA7, "*LAX", 2, AddressingMode::ZeroPage),   // *LAX d
OpCode::new(0xAF, "*LAX", 3, AddressingMode::Absolute),   // *LAX a
OpCode::new(0xB3, "*LAX", 2, AddressingMode::Indirect_Y), // *LAX (d),Y
OpCode::new(0xB7, "*LAX", 2, AddressingMode::ZeroPage_Y), // *LAX d,Y
OpCode::new(0xBF, "*LAX", 3, AddressingMode::Absolute_Y), // *LAX a,Y

// SAX
OpCode::new(0x83, "*SAX", 2, AddressingMode::Indirect_X), // *SAX (d,X)
OpCode::new(0x87, "*SAX", 2, AddressingMode::ZeroPage),   // *SAX d
OpCode::new(0x8F, "*SAX", 3, AddressingMode::Absolute),   // *SAX a
OpCode::new(0x97, "*SAX", 2, AddressingMode::ZeroPage_Y), // *SAX d,Y

// SHA
OpCode::new(0x93, "*SHA", 2, AddressingMode::Indirect_Y), // *SHA (d),Y
OpCode::new(0x9F, "*SHA", 3, AddressingMode::Absolute_Y), // *SHA a,Y

OpCode::new(0xEB, "*SBC", 2, AddressingMode::Immediate), // *SBC #i (Unofficial SBC immediate)

// DCP
OpCode::new(0xC3, "*DCP", 2, AddressingMode::Indirect_X), // *DCP (d,X)
OpCode::new(0xC7, "*DCP", 2, AddressingMode::ZeroPage),   // *DCP d
OpCode::new(0xCF, "*DCP", 3, AddressingMode::Absolute),   // *DCP a
OpCode::new(0xD3, "*DCP", 2, AddressingMode::Indirect_Y), // *DCP (d),Y
OpCode::new(0xD7, "*DCP", 2, AddressingMode::ZeroPage_X), // *DCP d,X
OpCode::new(0xDB, "*DCP", 3, AddressingMode::Absolute_Y), // *DCP a,Y
OpCode::new(0xDF, "*DCP", 3, AddressingMode::Absolute_X), // *DCP a,X

// ISC
OpCode::new(0xE3, "*ISB", 2, AddressingMode::Indirect_X), // *ISC (d,X)
OpCode::new(0xE7, "*ISB", 2, AddressingMode::ZeroPage),   // *ISC d
OpCode::new(0xEF, "*ISB", 3, AddressingMode::Absolute),   // *ISC a
OpCode::new(0xF3, "*ISB", 2, AddressingMode::Indirect_Y), // *ISC (d),Y
OpCode::new(0xF7, "*ISB", 2, AddressingMode::ZeroPage_X), // *ISC d,X
OpCode::new(0xFB, "*ISB", 3, AddressingMode::Absolute_Y), // *ISC a,Y
OpCode::new(0xFF, "*ISB", 3, AddressingMode::Absolute_X), // *ISC a,X

// RLA
OpCode::new(0x23, "*RLA", 2, AddressingMode::Indirect_X), // *RLA (d,X)
OpCode::new(0x27, "*RLA", 2, AddressingMode::ZeroPage),   // *RLA d
OpCode::new(0x2F, "*RLA", 3, AddressingMode::Absolute),   // *RLA a
OpCode::new(0x33, "*RLA", 2, AddressingMode::Indirect_Y), // *RLA (d),Y
OpCode::new(0x37, "*RLA", 2, AddressingMode::ZeroPage_X), // *RLA d,X
OpCode::new(0x3B, "*RLA", 3, AddressingMode::Absolute_Y), // *RLA a,Y
OpCode::new(0x3F, "*RLA", 3, AddressingMode::Absolute_X), // *RLA a,X

// RRA
OpCode::new(0x63, "*RRA", 2, AddressingMode::Indirect_X), // *RRA (d,X)
OpCode::new(0x67, "*RRA", 2, AddressingMode::ZeroPage),   // *RRA d
OpCode::new(0x6F, "*RRA", 3, AddressingMode::Absolute),   // *RRA a
OpCode::new(0x73, "*RRA", 2, AddressingMode::Indirect_Y), // *RRA (d),Y
OpCode::new(0x77, "*RRA", 2, AddressingMode::ZeroPage_X), // *RRA d,X
OpCode::new(0x7B, "*RRA", 3, AddressingMode::Absolute_Y), // *RRA a,Y
OpCode::new(0x7F, "*RRA", 3, AddressingMode::Absolute_X), // *RRA a,X

// SLO
OpCode::new(0x03, "*SLO", 2, AddressingMode::Indirect_X), // *SLO (d,X)
OpCode::new(0x07, "*SLO", 2, AddressingMode::ZeroPage),   // *SLO d
OpCode::new(0x0F, "*SLO", 3, AddressingMode::Absolute),   // *SLO a
OpCode::new(0x13, "*SLO", 2, AddressingMode::Indirect_Y), // *SLO (d),Y
OpCode::new(0x17, "*SLO", 2, AddressingMode::ZeroPage_X), // *SLO d,X
OpCode::new(0x1B, "*SLO", 3, AddressingMode::Absolute_Y), // *SLO a,Y
OpCode::new(0x1F, "*SLO", 3, AddressingMode::Absolute_X), // *SLO a,X

// SRE
OpCode::new(0x43, "*SRE", 2, AddressingMode::Indirect_X), // *SRE (d,X)
OpCode::new(0x47, "*SRE", 2, AddressingMode::ZeroPage),   // *SRE d
OpCode::new(0x4F, "*SRE", 3, AddressingMode::Absolute),   // *SRE a
OpCode::new(0x53, "*SRE", 2, AddressingMode::Indirect_Y), // *SRE (d),Y
OpCode::new(0x57, "*SRE", 2, AddressingMode::ZeroPage_X), // *SRE d,X
OpCode::new(0x5B, "*SRE", 3, AddressingMode::Absolute_Y), // *SRE a,Y
OpCode::new(0x5F, "*SRE", 3, AddressingMode::Absolute_X), // *SRE a,X
    ];

    pub static ref OPCODES_MAP: HashMap<u8, &'static OpCode> = {
        let mut map = HashMap::new();
        for cpuop in &*CPU_OPCODES {
            map.insert(cpuop.code, cpuop);
        }
        map
    };
}
