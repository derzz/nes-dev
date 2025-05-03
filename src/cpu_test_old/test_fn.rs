#[cfg(test)]
pub mod test_fn {
    use crate::cpu::CpuFlags;
    use crate::cpu::CPU;

    pub const FULLFLAGS: CpuFlags = CpuFlags::from_bits_truncate(0b11111111);
    pub const EMPTYFLAGS: CpuFlags = CpuFlags::from_bits_truncate(0b00000000);

    pub fn stack_push_test(cpu: &mut CPU, instructions: Vec<u8>, check: u8) {
        cpu.load_and_run(instructions);
        // Setting to 4 due to brk doing a stack_push and stack_push_u16
        assert!(
            cpu.memory[(0x0100 + cpu.sp.wrapping_add(4) as u16) as usize] == check,
            "pushed value is {:#b}, comparing against {:#b}",
            cpu.memory[(0x0100 + cpu.sp.wrapping_add(4) as u16) as usize],
            check
        )
    }

    pub fn flag_removal_test(cpu: &mut CPU, instructions: Vec<u8>, check: CpuFlags) {
        cpu.flags = FULLFLAGS;
        // Instruction should only be checking removal of one flag
        cpu.load(instructions);
        cpu.pc = cpu.mem_read_u16(0xFFFC);
        cpu.run();
        assert!(
            !cpu.flags.contains(check),
            "Cpu Flags is {:#b}",
            cpu.flags.bits()
        );
    }

    pub fn flag_insert_test(cpu: &mut CPU, instructions: Vec<u8>, check: CpuFlags) {
        cpu.flags = EMPTYFLAGS;
        cpu.load(instructions);
        cpu.pc = cpu.mem_read_u16(0xFFFC);
        cpu.run();
        assert!(
            cpu.flags.contains(check),
            "Cpu Flags is {:#b}",
            cpu.flags.bits()
        );
    }
}
