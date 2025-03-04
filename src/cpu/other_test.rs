#[cfg(test)]
mod branch_test {
    use crate::cpu::op::*;
    use crate::cpu::CPU;

    // BRK is not tested as every other test hinges on BRK working...

    // JSR and RTS are tested together to see if INX is called
    // 0x8000 is for jsr, 8001 + 8002 is to store address
    // One test for jumping to a forward subroutine, another test for jumping to a previous subroutine
    #[test]
    fn test_jsr_rts() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![
            other_op::JSR,
            0x05,
            0x80,
            op::INY,
            0x00,
            op::INX,
            other_op::RTS,
        ]);
        assert!(
            cpu.x == 1 && cpu.y == 1,
            "JSR and RTS test failed! cpu.x is {}, cpu.y is {}",
            cpu.x,
            cpu.y
        );
    }

    // JSR, PHP are done for RTI testing
    // Check if custom flags are the same from the stack AND if INX is called
    // in fffe link to address, run programs in ram from 0x0000 to 0x8000,
}
