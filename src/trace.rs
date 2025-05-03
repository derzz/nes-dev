use crate::cpu::AddressingMode;
use crate::cpu::Mem;
use crate::cpu::CPU;
use crate::op::OpCode;
use crate::op::CPU_OPCODES;
use crate::op::OPCODES_MAP;
use crate::ppu::PPU;
use std::collections::HashMap;
use std::ops::Add;

use log::{debug, info, warn};

// This will return the current state of the cpu based on its parameters in trace
// THIS WILL NOT TEST THE LAST COLUMN(PPU AND CPU CLOCK CYCLES)
pub fn trace(cpu: &mut CPU) -> String {
    // Extract the PC
    let op_map = &OPCODES_MAP;
    let pc = cpu.pc;
    let instr = cpu.mem_read(pc);
    let op = op_map
        .get(&instr)
        .unwrap_or_else(|| panic!("Trace: Unknown opcode: {:#04X}", instr));

    // Current value of the PC
    let ret_pc = format!("{:04X}", pc);

    // Producing the raw hex dump for the instruction and addressing
    let times = op.len;
    let mut raw_ar = Vec::new();
    for n in 0..times {
        raw_ar.push(format!("{:02X}", cpu.mem_read(pc + n as u16)));
    }
    let ret_raw = raw_ar.join(" ");

    // Providing the instruction
    let mut instr_dump: Vec<String> = Vec::new();
    // Pushing string value for instructinos
    instr_dump.push(op.lit.to_string());
    // Logic needed for determining what items to push
    let addr: u16 = if !(matches!(op.mode, AddressingMode::NoneAddressing)
        || matches!(op.mode, AddressingMode::Relative)
        || matches!(op.mode, AddressingMode::Accumulator))
    {
        let ret = cpu.get_relative_address(&op.mode, pc);
        ret
    } else {
        warn!("no trace address needed!");
        0
    };
    // Format the address based on what mode it is
    let addr_format: String = match op.mode {
        AddressingMode::Accumulator => "A".to_string(),
        AddressingMode::Immediate => format!("#${:02X}", cpu.mem_read(addr)),
        AddressingMode::ZeroPage => format!("${:02X} = {:02X}", addr, cpu.mem_read(addr)),
        // TODO Add the hard coded values for STX(what is the content of the previous value)
        AddressingMode::Absolute => {
            match op.code {
                // JMP Absolute
                0x4C | 0x20 => format!("${:04X}", addr),
                _ => format!("${:04X} = {:02X}", addr, cpu.mem_read(addr)),
            }
        }
        // First number is the address we are looking at
        // Second number is the value fetched
        // Final number is the content of the value fetched
        AddressingMode::ZeroPage_X => format!(
            "${:02X},X @ {:02X} = {:02X}",
            cpu.mem_read(pc + 1),
            addr,
            cpu.mem_read(addr)
        ),
        AddressingMode::ZeroPage_Y => format!(
            "${:02X},Y @ {:02X} = {:02X}",
            cpu.mem_read(pc + 1),
            addr,
            cpu.mem_read(addr)
        ),
        AddressingMode::Absolute_X => format!(
            "${:04X},X @ {:04X} = {:02X}",
            cpu.mem_read_u16(pc + 1),
            addr,
            cpu.mem_read(addr)
        ),
        // BUG Should be mem_read_u16 not mem_read
        AddressingMode::Absolute_Y => format!(
            "${:04X},Y @ {:04X} = {:02X}",
            cpu.mem_read_u16(pc + 1),
            addr,
            cpu.mem_read(addr)
        ),
        AddressingMode::Indirect => {
            match op.code {
                // JMP Indirect
                0x6C => format!("(${:04X}) = {:04X}", cpu.mem_read_u16(pc + 1), addr),
                _ => format!("({:04X} = {:04X})", cpu.mem_read_u16(pc), addr),
            }
        }
        AddressingMode::Indirect_X => format!(
            "(${:02X},X) @ {:02X} = {:04X} = {:02X}",
            cpu.mem_read(pc + 1),
            cpu.mem_read(pc + 1).wrapping_add(cpu.x),
            addr,
            cpu.mem_read(addr)
        ),
        // NOTE: Second value is initial dereferenced value
        AddressingMode::Indirect_Y => {
            format!(
                "(${:02X}),Y = {:04X} @ {:04X} = {:02X}",
                cpu.mem_read(pc + 1),
                addr.wrapping_sub(cpu.y as u16),
                addr,
                cpu.mem_read(addr)
            )
        }
        AddressingMode::Relative => {
            format!("${:4X}", {
                let branch_offset = (cpu.mem_read(pc.wrapping_add(1)) as i8).wrapping_add(2);
                pc.wrapping_add(branch_offset as u16)
            })
        }
        // Accumulator(do nothing) and None Addressing
        _ => "".to_string(),
    };
    instr_dump.push(addr_format);
    let ret_instr = instr_dump.join(" ");

    // Cpu registers
    warn!("the flags are {:#X}", cpu.flags.bits());
    let ret_reg = format!(
        "A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X}",
        cpu.a,
        cpu.x,
        cpu.y,
        cpu.flags.bits(),
        cpu.sp
    );

    let (ret_raw_pad, ret_instr_pad) = {
        if op.lit.len() == 3 {
            debug!("chose this one!");
            (format!("{:<10}", ret_raw), format!("{:<32}", ret_instr))
        } else {
            (format!("{:<9}", ret_raw), format!("{:<33}", ret_instr))
        }
    };

    // PPU Scanlines and Cycles
    let ppu_scan = cpu.bus.ppu.scanline;
    let ppu_cycles = cpu.bus.ppu.cycles;
    let cpu_cycles = cpu.bus.cycles;
    
    let ppu_str= format!("PPU:{:>3},{:>3} CYC:{}", ppu_scan, ppu_cycles, cpu_cycles);


    // CPU Cycles

    // Format everything with proper spacing
    // Use format width specifiers to align columns
    let trace_line = format!(
        "{:<6}{}{}{} {}",
        ret_pc,        // PC address, left-aligned, 6 chars wide
        ret_raw_pad,   // Raw bytes, left-aligned, 10 chars wide
        ret_instr_pad, // Instruction with operand, left-aligned, 30 chars wide
        ret_reg,        // Register values
        ppu_str
    );

    trace_line
}
