use crate::cpu::AddressingMode;
use crate::cpu::Mem;
use crate::cpu::CPU;
use std::collections::HashMap;

// This will return the current state of the cpu based on its parameters in trace
pub fn trace(cpu: &CPU) -> String {
    let code = cpu.instr;
    let pc = cpu.pc - cpu.cycles as u16; // Artifically pulling to the start of the command(PC is on the )
    let mode = cpu.mode;

    let mut hex_dump = vec![];

     let (mem_addr, stored_value) = match mode {
        AddressingMode::Immediate | AddressingMode::NoneAddressing => (0, 0),
        _ => {
            let addr = cpu.get_absolute_address(&ops.mode, begin + 1);
            (addr, cpu.mem_read(addr))
        }
    };

}
