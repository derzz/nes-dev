#[cfg(test)]
mod sb2_test {
    use crate::cpu::op::*;
    use crate::cpu::test_fn::*;
    use crate::cpu::CpuFlags;
    use crate::cpu::CPU;
    use crate::cpu::STACK;
    use crate::cpu::STACK_RESET;

    #[test]
    fn test_txa() {
        let mut cpu = CPU::new();
        // Increments X and transfers to A
        cpu.load_and_run(vec![op::INX, op::TXA]);
        assert!(cpu.a == cpu.x);
        assert!(!cpu.flags.contains(CpuFlags::ZERO) && !cpu.flags.contains(CpuFlags::NEGATIVE));

        cpu.load_and_run(vec![op::TXA]);
        assert!(cpu.flags.contains(CpuFlags::ZERO) && !cpu.flags.contains(CpuFlags::NEGATIVE));

        // Decrements X and tranfers to A
        cpu.load_and_run(vec![op::DEX, op::TXA]);
        assert!(cpu.flags.contains(CpuFlags::NEGATIVE))
    }
    #[test]
    fn test_txs() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![op::TXS]);
        // Need to add 3 due to brk modifying stack pointer with a 16 bit memory operation + 8 bit memory operation
        assert!(cpu.sp.wrapping_add(3) == cpu.x);
    }
    #[test]
    fn test_tax() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![op::INX, op::TAX]);
        assert!(cpu.x == cpu.a);
        assert!(cpu.flags.contains(CpuFlags::ZERO) && !cpu.flags.contains(CpuFlags::NEGATIVE));

        cpu.load_and_run(vec![op::INY, op::TYA, op::TAX]);
        assert!(cpu.x == cpu.a);
        assert!(!cpu.flags.contains(CpuFlags::ZERO) && !cpu.flags.contains(CpuFlags::NEGATIVE));

        cpu.load_and_run(vec![op::DEY, op::TYA, op::TAX]);
        assert!(cpu.x == cpu.a);
        assert!(!cpu.flags.contains(CpuFlags::ZERO) && cpu.flags.contains(CpuFlags::NEGATIVE));
    }
    #[test]
    fn test_tsx() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![op::TSX]);
        assert!(cpu.x == STACK_RESET);
        assert!(!cpu.flags.contains(CpuFlags::ZERO) && cpu.flags.contains(CpuFlags::NEGATIVE));

        cpu.load_and_run(vec![op::PHP, op::TSX]);
        assert!(
            cpu.x == STACK_RESET.wrapping_sub(1),
            "x is {:#b}, stack_reset-1 is {:#b}",
            cpu.x,
            STACK_RESET
        );
        assert!(!cpu.flags.contains(CpuFlags::ZERO) && cpu.flags.contains(CpuFlags::NEGATIVE));
    }
    #[test]
    fn test_dex() {}
    #[test]
    fn test_nop() {}
}
