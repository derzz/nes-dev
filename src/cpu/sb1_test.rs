#[cfg(test)]
mod sb1_test {
    use crate::cpu::op::*;
    use crate::cpu::test_fn::*;
    use crate::cpu::CpuFlags;
    use crate::cpu::CPU;
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

    // SB1 Testing
    #[test]
    fn test_php() {
        let mut cpu = CPU::new();
        test_fn::stack_push_test(&mut cpu, vec![0x08], 0b00110100);
    }

    // CLC Test
    #[test]
    fn test_clc() {
        let mut cpu = CPU::new();
        test_fn::flag_removal_test(&mut cpu, vec![op::CLC], CpuFlags::CARRY);
    }

    #[test]
    fn test_plp() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![op::SEC, op::SEI, op::SED, op::PHP, op::PLP]);
        assert!(
            cpu.flags.bits() == 0b00111101,
            "Cpu Flags are {:#b}",
            cpu.flags
        );
    }

    #[test]
    fn test_sec() {
        let mut cpu = CPU::new();
        test_fn::flag_insert_test(&mut cpu, vec![0x38], CpuFlags::CARRY);
    }

    #[test]
    fn test_pha() {
        let mut cpu = CPU::new();
        // Increments Y register twice and transfers from Y to A
        // Then determines if a == 0x02
        test_fn::stack_push_test(&mut cpu, vec![op::INY, op::INY, op::TYA, op::PHA], 0x02);
    }

    #[test]
    fn test_cli() {
        // Interrupt disable checking is interesting as it will reenable due to BRK
        // Testing by pushing the flags with PHP and then checking that.
        let mut cpu = CPU::new();
        cpu.flags = test_fn::FULLFLAGS;
        cpu.load(vec![op::CLI, op::PHP]);
        cpu.pc = cpu.mem_read_u16(0xFFFC);
        cpu.run();
        let check = 0b1111_1011;
        assert!(
            cpu.memory[(0x0100 + cpu.sp.wrapping_add(4) as u16) as usize] == check,
            "pushed value is {:#b}, comparing against {:#b}",
            cpu.memory[(0x0100 + cpu.sp.wrapping_add(4) as u16) as usize],
            check
        )
    }

    #[test]
    fn test_pla() {
        let mut cpu = CPU::new();
        // Increments Y register twice and transfers from Y to A
        // Pushes onto stack
        // Pulls from stack and tests if equals 2
        cpu.load_and_run(vec![op::INY, op::INY, op::TYA, op::PHA, op::TXA, op::PLA]);
        assert!(cpu.a == 0x02);
    }

    #[test]
    fn test_sei() {
        let mut cpu = CPU::new();
        test_fn::flag_insert_test(&mut cpu, vec![0x78], CpuFlags::INTERRUPT_DISABLE);
    }

    #[test]
    fn test_dey() {
        let mut cpu = CPU::new();
        // Adds Y register twice and then subtracts once
        cpu.load_and_run(vec![0xC8, 0xC8, 0x88]);
        assert!(cpu.y == 1);
    }

    #[test]
    fn test_tya() {
        let mut cpu = CPU::new();
        // Increments the Y register and transfers it to a
        cpu.load_and_run(vec![0xC8, 0x98]);
        assert!(cpu.a == 1);
    }

    #[test]
    fn test_tay() {
        let mut cpu = CPU::new();
        // Increments X register, transfers that to a, then transfers a -> y reg
        cpu.load_and_run(vec![op::INX, op::TXA, op::TAY]);
        assert!(cpu.a == cpu.y);
    }

    #[test]
    fn test_clv() {
        let mut cpu = CPU::new();
        test_fn::flag_removal_test(&mut cpu, vec![op::CLV], CpuFlags::OVERFLOW);
    }

    #[test]
    fn test_iny() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xC8]);
        assert!(cpu.y == 1);
    }

    #[test]
    fn test_cld() {
        let mut cpu = CPU::new();
        test_fn::flag_removal_test(&mut cpu, vec![op::CLD], CpuFlags::DECIMAL_MODE);
    }

    #[test]
    fn test_inx() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xE8]);
        assert!(cpu.x == 1)
    }

    #[test]
    fn test_sed() {
        let mut cpu = CPU::new();
        test_fn::flag_insert_test(&mut cpu, vec![0xF8], CpuFlags::DECIMAL_MODE);
    }
}
