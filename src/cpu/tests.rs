use crate::cpu::CPU;

#[cfg(test)]
mod cpu_test {
    use super::*;

    // Tests LDA, a = 5
    #[test]
    fn test_0xa9_lda_immediate_load_data() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x05, 0x00]);
        assert_eq!(cpu.a, 0x05);
        assert!(cpu.flags.bits() & 0b0000_0010 == 0b00);
        assert!(cpu.flags.bits() & 0b1000_0000 == 0);
    }

    // Tests for zero flag being activated when loading 0
    #[test]
    fn test_0xa9_lda_zero_flag() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x00, 0x00]);
        assert!(cpu.flags.bits() & 0b0000_0010 == 0b10);
    }

    // LDA accumulator
    #[test]
    fn test_lda_from_memory() {
        let mut cpu = CPU::new();
        cpu.mem_write(0x10, 0x55);

        cpu.load_and_run(vec![0xa5, 0x10, 0x00]);

        assert_eq!(cpu.a, 0x55);
    }

    // PHP Test for flags
    #[test]
    fn test_php() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0x08, 0x00]);
        assert!(
            cpu.memory[(0x0100 + cpu.sp.wrapping_add(1) as u16) as usize] == 0b00110100,
            "pushed value is {:#b}",
            cpu.memory[(cpu.sp + 1) as usize]
        )
    }
}
