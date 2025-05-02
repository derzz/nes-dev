// NOTE: STX and LDX are in group3_tests

// ASL, ROL, LSR, and LSR can be done in conjunction
// Note a separate test for accumulator must be done

// DEC and INC need to check memory vlaues

#[cfg(test)]
mod group2_test {
    use crate::cpu::op::*;
    use crate::cpu::CpuFlags;
    use crate::cpu::CPU;

    fn g2_flag_test(cpu: &CPU, carry: bool, zero: bool, negative: bool) {
        if carry {
            assert!(
                cpu.flags.contains(CpuFlags::CARRY),
                "Expected carry flag to be set"
            );
        } else {
            assert!(
                !cpu.flags.contains(CpuFlags::CARRY),
                "Expected carry flag to be clear"
            );
        }
        if zero {
            assert!(
                cpu.flags.contains(CpuFlags::ZERO),
                "Expected zero flag to be set"
            );
        } else {
            assert!(
                !cpu.flags.contains(CpuFlags::ZERO),
                "Expected zero flag to be clear"
            );
        }

        if negative {
            assert!(
                cpu.flags.contains(CpuFlags::NEGATIVE),
                "Expected negative flag to be set"
            );
        } else {
            assert!(
                !cpu.flags.contains(CpuFlags::NEGATIVE),
                "Expected negative flag to be clear"
            );
        }
    }
    // This function will set a value into a memory location
    // Zero Page will be on address 0xAB
    // Absolute is on address 0x0200
    // Flags will be tested on each step
    // If accum is set to true, it will test accumulators(ONLY FOR ROTATE FUNCTIONS)
    fn memory_modifier(
        cpu: &mut CPU,
        accum: bool,
        first_half: u8,
        second_half: u8,
        load_mem: u8,
        expected_val: u8,
        carry: bool,
        zero: bool,
        negative: bool,
    ) {
        // Absolute
        cpu.load_and_reset(vec![first_half + g2_op::ABS, 0x00, 0x02]);
        cpu.memory[0x200] = load_mem;
        cpu.run();
        assert!(cpu.memory[0x200] == expected_val, "Failed on absolute test, provided value is {:#x}, the contents of 0x200 is {:#b}, expected value is{:#b}",load_mem, cpu.memory[0x200], expected_val);
        g2_flag_test(cpu, carry, zero, negative);

        // Absolute + x
        cpu.load_and_reset(vec![second_half + g2_op::ABS, 0x00, 0x02]);
        cpu.x = 1;
        cpu.memory[0x201] = load_mem;
        cpu.run();
        assert!(
            cpu.memory[0x201] == expected_val,
            "Failed on INC/DEC absolute, X test"
        );
        g2_flag_test(cpu, carry, zero, negative);

        // Zero Page
        cpu.load_and_reset(vec![first_half + g2_op::ZP, 0xAB]);
        cpu.memory[0xAB] = load_mem;
        cpu.run();
        assert!(
            cpu.memory[0xAB] == expected_val,
            "Failed on INC/DEC Zero Page test"
        );
        g2_flag_test(cpu, carry, zero, negative);

        // Zero Page, X
        cpu.load_and_reset(vec![second_half + g2_op::ZP, 0xAB]);
        cpu.x = 1;
        cpu.memory[0xAC] = load_mem;
        cpu.run();
        assert!(
            cpu.memory[0xAC] == expected_val,
            "Failed on INC/DEC Zero Page, X test"
        );
        g2_flag_test(cpu, carry, zero, negative);

        // Accumulator tests
        if accum {
            cpu.load_and_reset(vec![first_half + g2_op::A]);
            cpu.a = load_mem;
            cpu.run();
            assert!(
                cpu.a == expected_val,
                "Failed on Accumulator test, accumulator value is {:#b}, expected value is {:#b}",
                cpu.a,
                expected_val
            );
            g2_flag_test(cpu, carry, zero, negative);
        }
    }

    #[test]
    fn test_dec() {
        let mut cpu = CPU::new();
        // Zero Flag
        memory_modifier(
            &mut cpu,
            false,
            g2_op::FIRST_DEC,
            g2_op::SECOND_DEC,
            0x1,
            0x0,
            false,
            true,
            false,
        );
        // Negative Flag
        memory_modifier(
            &mut cpu,
            false,
            g2_op::FIRST_DEC,
            g2_op::SECOND_DEC,
            0xFF,
            0xFE,
            false,
            false,
            true,
        );
        // No Flags set
        memory_modifier(
            &mut cpu,
            false,
            g2_op::FIRST_DEC,
            g2_op::SECOND_DEC,
            0x0F,
            0xe,
            false,
            false,
            false,
        );
    }

    #[test]
    fn test_inc() {
        let mut cpu = CPU::new();
        // Zero Flag
        memory_modifier(
            &mut cpu,
            false,
            g2_op::FIRST_INC,
            g2_op::SECOND_INC,
            0xFF,
            0x0,
            false,
            true,
            false,
        );
        // Negative Flag
        memory_modifier(
            &mut cpu,
            false,
            g2_op::FIRST_INC,
            g2_op::SECOND_INC,
            0xFE,
            0xFF,
            false,
            false,
            true,
        );
        // No Flags set
        memory_modifier(
            &mut cpu,
            false,
            g2_op::FIRST_INC,
            g2_op::SECOND_INC,
            0x0F,
            0x10,
            false,
            false,
            false,
        );
    }

    #[test]
    fn test_asl() {
        let mut cpu = CPU::new();
        let first_half = g2_op::FIRST_ASL;
        let second_half = g2_op::SECOND_ASL;
        let mut load_mem = 0b0000_0001;
        let mut expected_val = 0b0000_0010;
        let accum = true;
        let mut carry = false;
        let mut zero = false;
        let mut negative = false;

        // No Flags
        memory_modifier(
            &mut cpu,
            accum,
            first_half,
            second_half,
            load_mem,
            expected_val,
            carry,
            zero,
            negative,
        );

        // Carry flag
        load_mem = 0b1011_0000;
        expected_val = 0b0110_0000;
        carry = true;
        zero = false;
        negative = false;
        memory_modifier(
            &mut cpu,
            accum,
            first_half,
            second_half,
            load_mem,
            expected_val,
            carry,
            zero,
            negative,
        );

        // Negative Flag
        load_mem = 0b0100_0000;
        expected_val = 0b1000_0000;
        carry = false;
        zero = false;
        negative = true;
        memory_modifier(
            &mut cpu,
            accum,
            first_half,
            second_half,
            load_mem,
            expected_val,
            carry,
            zero,
            negative,
        );

        // Zero flag
        load_mem = 0b0000_0000;
        expected_val = 0b0000_0000;
        carry = false;
        zero = true;
        negative = false;
        memory_modifier(
            &mut cpu,
            accum,
            first_half,
            second_half,
            load_mem,
            expected_val,
            carry,
            zero,
            negative,
        );

        // Carry + Negative Flag
        load_mem = 0b1100_0000;
        expected_val = 0b1000_0000;
        carry = true;
        zero = false;
        negative = true;
        memory_modifier(
            &mut cpu,
            accum,
            first_half,
            second_half,
            load_mem,
            expected_val,
            carry,
            zero,
            negative,
        );

        // Carry + Zero Flag
        load_mem = 0b1000_0000;
        expected_val = 0b0000_0000;
        carry = true;
        zero = true;
        negative = false;
        memory_modifier(
            &mut cpu,
            accum,
            first_half,
            second_half,
            load_mem,
            expected_val,
            carry,
            zero,
            negative,
        );
    }

    #[test]
    fn test_rol() {
        let mut cpu = CPU::new();
        let first_half = g2_op::FIRST_ROL;
        let second_half = g2_op::SECOND_ROL;
        let mut load_mem = 0b0000_0001;
        let mut expected_val = 0b0000_0010;
        let accum = true;
        let mut carry = false;
        let mut zero = false;
        let mut negative = false;

        // No Flags
        memory_modifier(
            &mut cpu,
            accum,
            first_half,
            second_half,
            load_mem,
            expected_val,
            carry,
            zero,
            negative,
        );

        // Carry flag
        load_mem = 0b1011_0000;
        expected_val = 0b0110_0000;
        carry = true;
        zero = false;
        negative = false;
        memory_modifier(
            &mut cpu,
            accum,
            first_half,
            second_half,
            load_mem,
            expected_val,
            carry,
            zero,
            negative,
        );

        // Negative Flag
        load_mem = 0b0100_0000;
        expected_val = 0b1000_0000;
        carry = false;
        zero = false;
        negative = true;
        memory_modifier(
            &mut cpu,
            accum,
            first_half,
            second_half,
            load_mem,
            expected_val,
            carry,
            zero,
            negative,
        );

        // Zero flag
        load_mem = 0b0000_0000;
        expected_val = 0b0000_0000;
        carry = false;
        zero = true;
        negative = false;
        memory_modifier(
            &mut cpu,
            accum,
            first_half,
            second_half,
            load_mem,
            expected_val,
            carry,
            zero,
            negative,
        );

        // Carry + Negative Flag
        load_mem = 0b1100_0000;
        expected_val = 0b1000_0000;
        carry = true;
        zero = false;
        negative = true;
        memory_modifier(
            &mut cpu,
            accum,
            first_half,
            second_half,
            load_mem,
            expected_val,
            carry,
            zero,
            negative,
        );

        // Carry + Zero Flag
        load_mem = 0b1000_0000;
        expected_val = 0b0000_0000;
        carry = true;
        zero = true;
        negative = false;
        memory_modifier(
            &mut cpu,
            accum,
            first_half,
            second_half,
            load_mem,
            expected_val,
            carry,
            zero,
            negative,
        );
    }

    #[test]
    fn test_lsr() {
        let mut cpu = CPU::new();
        let first_half = g2_op::FIRST_LSR;
        let second_half = g2_op::SECOND_LSR;
        let mut load_mem = 0b0000_0010;
        let mut expected_val = 0b0000_0001;
        let accum = true;
        let mut carry = false;
        let mut zero = false;
        let mut negative = false;

        // No Flags
        memory_modifier(
            &mut cpu,
            accum,
            first_half,
            second_half,
            load_mem,
            expected_val,
            carry,
            zero,
            negative,
        );

        // Carry flag
        load_mem = 0b1100_0101;
        expected_val = 0b0110_0010;
        carry = true;
        zero = false;
        negative = false;
        memory_modifier(
            &mut cpu,
            accum,
            first_half,
            second_half,
            load_mem,
            expected_val,
            carry,
            zero,
            negative,
        );

        // Zero flag
        load_mem = 0b0000_0000;
        expected_val = 0b0000_0000;
        carry = false;
        zero = true;
        negative = false;
        memory_modifier(
            &mut cpu,
            accum,
            first_half,
            second_half,
            load_mem,
            expected_val,
            carry,
            zero,
            negative,
        );

        load_mem = 0b1100_0001;
        expected_val = 0b0110_0000;
        carry = true;
        zero = false;
        negative = false;
        memory_modifier(
            &mut cpu,
            accum,
            first_half,
            second_half,
            load_mem,
            expected_val,
            carry,
            zero,
            negative,
        );

        // Carry + Zero Flag
        load_mem = 0b0000_0001;
        expected_val = 0b0000_0000;
        carry = true;
        zero = true;
        negative = false;
        memory_modifier(
            &mut cpu,
            accum,
            first_half,
            second_half,
            load_mem,
            expected_val,
            carry,
            zero,
            negative,
        );
    }
}
