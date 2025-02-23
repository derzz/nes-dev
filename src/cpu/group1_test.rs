#[cfg(test)]
mod group1_test {
    use crate::cpu::op::*;
    use crate::cpu::test_fn::*;
    use crate::cpu::CpuFlags;
    use crate::cpu::CPU;
    use crate::print_title;

    fn test_name(code: u8) -> &'static str {
        match code {
            0x00 | 0x10 => "ORA",
            0x20 | 0x30 => "AND",
            0x40 | 0x50 => "EOR",
            0x60 | 0x70 => "ADC",
            0x80 | 0x90 => "STA",
            0xA0 | 0xB0 => "LDA",
            0xC0 | 0xD0 => "CMP",
            0xE0 | 0xF0 => "SBC",
            _ => "UNKNOWN",
        }
    }

    // This will test the expected value of address a with addressing modes
    // This does not tests flags- all the expected values are the same so flags can be tested afterwards
    // NOT TO BE USED WITH CMP or STA
    fn gen_test_flag(cpu: &mut CPU, carry: bool){
        if carry {
            cpu.flags.insert(CpuFlags::CARRY);
        } else {
            cpu.flags.remove(CpuFlags::CARRY);
        }
    }


    fn gen_test(
        cpu: &mut CPU,
        first_half: u8,
        second_half: u8,
        load_a: u8,
        load_mem: u8,
        expected_val: u8,
        carry: bool,
    ) {

        // Indirect testing, two byte only!
        cpu.load_and_reset(vec![first_half + g1_op::INDIRECT, 0xA1]);
        gen_test_flag(cpu, carry);
        cpu.x = 0xB;
        cpu.memory[0xAC] = 0xDF; // This is the address where it goes to as 0xA1 + 0xB = 0xAC
        cpu.memory[0xDF] = load_mem;
        cpu.a = load_a;
        cpu.run();
        assert!(
            cpu.a == expected_val,
            "Failed on Indirect test with {}. cpu.a is {:#b}, expected_val is {:#b}",
            test_name(first_half),
            cpu.a,
            expected_val
        );

        // Zero Page Testing
        print_title!("Zero Page Test");
        cpu.load_and_reset((vec![first_half + g1_op::ZP, 0xFF]));
        gen_test_flag(cpu, carry);
        cpu.memory[0xFF] = load_mem;
        cpu.a = load_a;
        cpu.run();
        assert!(
            cpu.a == expected_val,
            "Failed on zero page test with {}. cpu.a is {:#b}, expected_val is {:#b}",
            test_name(first_half),
            cpu.a,
            expected_val
        );
        // Since STA Immediate doesn't make sense...
        // Immediate testing
        print_title!("Immediate Test");
        cpu.load_and_reset(vec![first_half + g1_op::IMMEDIATE_Y, load_mem]);
        gen_test_flag(cpu, carry);
        cpu.memory[0xFF] = load_mem;
        cpu.a = load_a;
        cpu.run();
        assert!(
            cpu.a == expected_val,
            "Failed on Immediate test with {}. cpu.a is {:#b}, expected_val is {:#b}",
            test_name(first_half),
            cpu.a,
            expected_val
        );

        // Absolute testing
        // Note 0xFE is first due to little endian
        cpu.load_and_reset((vec![first_half + g1_op::ABSOLUTE_X, 0xFE, 0x01]));
        gen_test_flag(cpu, carry);
        cpu.memory[0x01FE] = load_mem;
        cpu.a = load_a;
        cpu.run();
        assert!(
            cpu.a == expected_val,
            "Failed on Absolute test with {}. cpu.a is {:#b}, expected_val is {:#b}",
            test_name(first_half),
            cpu.a,
            expected_val
        );

        // Indirect Indexed: ($c0), Y
        // Can look at four bytes
        print_title!("Indirect Indexed Test");
        cpu.load_and_reset(vec![second_half + g1_op::INDIRECT, 0xA1]);
        gen_test_flag(cpu, carry);
        cpu.memory[0xA1] = 0xE1; // LSB
        cpu.memory[0xA2] = 0x05; // MSB
        cpu.y = 0x12;
        cpu.memory[0x05F3] = load_mem;
        cpu.a = load_a;
        cpu.run();

        assert!(
            cpu.a == expected_val,
            "Failed on indirect indexed[($c0), Y] with {}. cpu.a is {:#b}, expected_val is {:#b}",
            test_name(first_half),
            cpu.a,
            expected_val
        );

        // Zero Page, X
        cpu.load_and_reset(vec![second_half + g1_op::ZP, 0xFE]);
        gen_test_flag(cpu, carry);
        cpu.memory[0xFF] = load_mem;
        cpu.x = 0x01;
        cpu.a = load_a;
        cpu.run();

        assert!(
            cpu.a == expected_val,
            "Failed on Zero Page, X with {}. cpu.a is {:#b}, expected_val is {:#b}",
            test_name(first_half),
            cpu.a,
            expected_val
        );

        // Absolyte, Y
        cpu.load_and_reset(vec![second_half + g1_op::IMMEDIATE_Y, 0x00, 0x02]);
        gen_test_flag(cpu, carry);
        cpu.y = 0x01;
        cpu.memory[0x0201] = load_mem;
        cpu.a = load_a;
        cpu.run();

        assert!(
            cpu.a == expected_val,
            "Failed on Absolute, Y with {}. cpu.a is {:#b}, expected_val is {:#b}",
            test_name(first_half),
            cpu.a,
            expected_val
        );

        // Absolyte, X
        cpu.load_and_reset(vec![second_half + g1_op::ABSOLUTE_X, 0x00, 0x02]);
        gen_test_flag(cpu, carry);
        cpu.x = 0x01;
        cpu.memory[0x0201] = load_mem;
        cpu.a = load_a;
        cpu.run();

        assert!(
            cpu.a == expected_val,
            "Failed on Absolute, X with {}. cpu.a is {:#b}, expected_val is {:#b}",
            test_name(first_half),
            cpu.a,
            expected_val
        );
    }

    #[test]
    fn test_ora() {
        let mut cpu = CPU::new();
        // Testing on alternating bits
        let fh = g1_op::FIRST_ORA;
        let sh = g1_op::SECOND_ORA;
        gen_test(&mut cpu, fh, sh, 0b10101010, 0b01010101, 0xFF, false);

        assert!(cpu.flags.contains(CpuFlags::NEGATIVE) && !cpu.flags.contains(CpuFlags::ZERO));

        // Test ora if both are 0b1111_1111
        gen_test(&mut cpu, fh, sh, 0xFF, 0xFF, 0xFF, false);
        assert!(cpu.flags.contains(CpuFlags::NEGATIVE) && !cpu.flags.contains(CpuFlags::ZERO));

        // Testing ora if both are 0
        gen_test(&mut cpu, fh, sh, 0, 0, 0, false);
        assert!(!cpu.flags.contains(CpuFlags::NEGATIVE) && cpu.flags.contains(CpuFlags::ZERO));

        // Testing for no flags
        gen_test(&mut cpu, fh, sh, 0b01110000, 0b01110000, 0b01110000, false);

        assert!(!cpu.flags.contains(CpuFlags::NEGATIVE) && !cpu.flags.contains(CpuFlags::ZERO));
    }

    #[test]
    fn test_and() {
        // Similar tests to ORA
        let mut cpu = CPU::new();
        let fh = g1_op::FIRST_AND;
        let sh = g1_op::SECOND_AND;
        gen_test(&mut cpu, fh, sh, 0b10101010, 0b01010101, 0x00, false);

        assert!(!cpu.flags.contains(CpuFlags::NEGATIVE) && cpu.flags.contains(CpuFlags::ZERO));

        // Test and if both are 0b1111_1111
        gen_test(&mut cpu, fh, sh, 0xFF, 0xFF, 0xFF, false);
        assert!(cpu.flags.contains(CpuFlags::NEGATIVE) && !cpu.flags.contains(CpuFlags::ZERO));

        // Testing and if both are 0
        gen_test(&mut cpu, fh, sh, 0, 0, 0, false);
        assert!(!cpu.flags.contains(CpuFlags::NEGATIVE) && cpu.flags.contains(CpuFlags::ZERO));

        // Testing for no flags
        gen_test(&mut cpu, fh, sh, 0b01110000, 0b01001100, 0b0100_0000, false);
        assert!(!cpu.flags.contains(CpuFlags::NEGATIVE) && !cpu.flags.contains(CpuFlags::ZERO));
    }

    #[test]
    fn test_eor() {
        let mut cpu = CPU::new();
        let fh = g1_op::FIRST_EOR;
        let sh = g1_op::SECOND_EOR;

        gen_test(&mut cpu, fh, sh, 0b10101010, 0b01010101, 0xFF, false);

        assert!(cpu.flags.contains(CpuFlags::NEGATIVE) && !cpu.flags.contains(CpuFlags::ZERO));

        // Test and if both are 0b1111_1111
        gen_test(&mut cpu, fh, sh, 0xFF, 0xFF, 0x00, false);
        assert!(!cpu.flags.contains(CpuFlags::NEGATIVE) && cpu.flags.contains(CpuFlags::ZERO));

        // Testing and if both are 0
        gen_test(&mut cpu, fh, sh, 0, 0, 0, false);
        assert!(!cpu.flags.contains(CpuFlags::NEGATIVE) && cpu.flags.contains(CpuFlags::ZERO));

        gen_test(&mut cpu, fh, sh, 0b1001_1001, 0b0110_1001, 0xF0, false);
        assert!(cpu.flags.contains(CpuFlags::NEGATIVE) && !cpu.flags.contains(CpuFlags::ZERO));
    }

fn test_adc_flag_check(cpu: &CPU, carry: bool, zero: bool, overflow: bool, negative: bool, name: &str) {
    if carry {
        assert!(
            cpu.flags.contains(CpuFlags::CARRY),
            "{}: Expected carry flag to be set, but it was not",
            name
        );
    } else {
        assert!(
            !cpu.flags.contains(CpuFlags::CARRY),
            "{}: Expected carry flag to be clear, but it was set",
            name
        );
    }

    if zero {
        assert!(
            cpu.flags.contains(CpuFlags::ZERO),
            "{}: Expected zero flag to be set, but it was not",
            name
        );
    } else {
        assert!(
            !cpu.flags.contains(CpuFlags::ZERO),
            "{}: Expected zero flag to be clear, but it was set",
            name
        );
    }

    if overflow {
        assert!(
            cpu.flags.contains(CpuFlags::OVERFLOW),
            "{}: Expected overflow flag to be set, but it was not",
            name
        );
    } else {
        assert!(
            !cpu.flags.contains(CpuFlags::OVERFLOW),
            "{}: Expected overflow flag to be clear, but it was set",
            name
        );
    }

    if negative {
        assert!(
            cpu.flags.contains(CpuFlags::NEGATIVE),
            "{}: Expected negative flag to be set, but it was not",
            name
        );
    } else {
        assert!(
            !cpu.flags.contains(CpuFlags::NEGATIVE),
            "{}: Expected negative flag to be clear, but it was set",
            name
        );
    }
}

#[test]
fn test_adc() {
    let mut cpu = CPU::new();
    let fh = g1_op::FIRST_ADC;
    let sh = g1_op::SECOND_ADC;

    for i in 0..2 {
        let carry = if i == 0 { false } else { true };

        // 2 Positive
        gen_test(&mut cpu, fh, sh, 0x01, 0x02_u8.wrapping_sub(i), 0x03, carry);
        test_adc_flag_check(&cpu, false, false, false, false, "2 Positive");

        // 2 Negative
        gen_test(&mut cpu, fh, sh, 0xff, 0xff_u8.wrapping_sub(i), 0xfe, carry);
        test_adc_flag_check(&cpu, true, false, false, true, "2 Negative");

        // Zero
        gen_test(&mut cpu, fh, sh, 0xff, 0x01_u8.wrapping_sub(i), 0x00, carry);
        test_adc_flag_check(&cpu, true, true, false, false, "Zero");

        // Negative
        gen_test(&mut cpu, fh, sh, 0b1100_0000, 0b0000_0001_u8.wrapping_sub(i), 0b1100_0001, carry);
        test_adc_flag_check(&cpu, false, false, false, true, "Negative");

        // Zero(All)
        gen_test(&mut cpu, fh, sh, 0, 0_u8.wrapping_sub(i), 0, carry);
        test_adc_flag_check(&cpu, carry, true, false, false, "Zero(All)");

        // Carry with Zero sum
        gen_test(&mut cpu, fh, sh, 0xff, 0x01_u8.wrapping_sub(i), 0x00, carry);
        test_adc_flag_check(&cpu, true, true, false, false, "Carry with Zero sum");

        // Overflow
        gen_test(&mut cpu, fh, sh, 0x7f, 0x02_u8.wrapping_sub(i), 0x81, carry);
        test_adc_flag_check(&cpu, false, false, true, true, "Overflow");

        // Underflow
        gen_test(&mut cpu, fh, sh, 0x80, 0x81_u8.wrapping_sub(i), 0x01, carry);
        test_adc_flag_check(&cpu, true, false, true, false, "Underflow");
    }
}

    #[test]
    fn test_sta() {
        let mut cpu = CPU::new();
        let first_half = g1_op::FIRST_STA;
        let second_half = g1_op::SECOND_STA;
        let load_a: u8 = 0xAB;

        // Indirect testing, two byte only!
        cpu.load_and_reset(vec![first_half + g1_op::INDIRECT, 0xA1]);
        cpu.x = 0xB;
        cpu.memory[0xAC] = 0xDF; // This is the address where it goes to as 0xA1 + 0xB = 0xAC
        cpu.a = load_a;
        cpu.run();
        assert!(
            cpu.memory[0xDF] == cpu.a,
            "Failed on Indirect test for test_sta"
        );

        // Zero Page Testing
        print_title!("Zero Page Test");
        cpu.load_and_reset((vec![first_half + g1_op::ZP, 0xA1]));
        cpu.a = load_a;
        cpu.run();
        assert!(
            cpu.memory[0xA1] == cpu.a,
            "Failed on zero page test for test_sta"
        );

        // Absolute testing
        // Note 0xFE is first due to little endian
        cpu.load_and_reset((vec![first_half + g1_op::ABSOLUTE_X, 0xFE, 0x01]));
        cpu.a = load_a;
        cpu.run();
        assert!(
            cpu.memory[0x01FE] == cpu.a,
            "Failed on absolute test for test_sta. Cpu.a is {}, cpu.memory[0x01FE] is {}",
            cpu.a,
            cpu.memory[0x01FE]
        );

        // Indirect Indexed: ($c0), Y
        // Can look at four bytes
        print_title!("Indirect Indexed Test");
        cpu.load_and_reset(vec![second_half + g1_op::INDIRECT, 0xA1]);
        cpu.memory[0xA1] = 0xE1; // LSB
        cpu.memory[0xA2] = 0x05; // MSB
        cpu.y = 0x12;
        cpu.a = load_a;
        cpu.run();

        assert!(
            cpu.a == cpu.memory[0x05F3],
            "Failed on Indirect Indexed Test for test_sta"
        );

        // Zero Page, X
        cpu.load_and_reset(vec![second_half + g1_op::ZP, 0xFE]);
        cpu.x = 0x01;
        cpu.a = load_a;
        cpu.run();

        assert!(
            cpu.a == cpu.memory[0xFF],
            "Failed on zero page, X for test_sta"
        );

        // Absolute, Y
        cpu.load_and_reset(vec![second_half + g1_op::IMMEDIATE_Y, 0x00, 0x02]);
        cpu.y = 0x01;
        cpu.a = load_a;
        cpu.run();

        assert!(
            cpu.a == cpu.memory[0x0201],
            "Failed on Absolute for test_sta, cpu.a is {}, cpu.memory[0x201] is {}",
            cpu.a,
            cpu.memory[0x201]
        );

        // Absolute, X
        cpu.load_and_reset(vec![second_half + g1_op::ABSOLUTE_X, 0x00, 0x02]);
        cpu.x = 0x01;
        cpu.a = load_a;
        cpu.run();

        assert!(
            cpu.a == cpu.memory[0x201],
            "Failed on Absolute, X for test_sta"
        );
    }

    #[test]
    fn test_lda() {
        let mut cpu = CPU::new();
        let fh = g1_op::FIRST_LDA;
        let sh = g1_op::SECOND_LDA;
        gen_test(&mut cpu, fh, sh, 0, 0x05, 0x05, false);
        assert!(!cpu.flags.contains(CpuFlags::NEGATIVE) && !cpu.flags.contains(CpuFlags::ZERO));

        // Zero Test
        gen_test(&mut cpu, fh, sh, 0x20, 0x0, 0x0, false);
        assert!(!cpu.flags.contains(CpuFlags::NEGATIVE) && cpu.flags.contains(CpuFlags::ZERO));

        // Negative test
        gen_test(&mut cpu, fh, sh, 0x20, 0xFF, 0xFF, false);
        assert!(cpu.flags.contains(CpuFlags::NEGATIVE) && !cpu.flags.contains(CpuFlags::ZERO));
    }

    fn cmp_flag_tester(cpu: &CPU, carry: bool, zero: bool, negative: bool) {
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
    fn test_cmp_helper(
        cpu: &mut CPU,
        load_a: u8,
        load_mem: u8,
        carry: bool,
        zero: bool,
        negative: bool,
    ) {
        let first_half = g1_op::FIRST_CMP;
        let second_half = g1_op::SECOND_CMP;

        // Indirect testing, two byte only!
        cpu.load_and_reset(vec![first_half + g1_op::INDIRECT, 0xA1]);
        cpu.x = 0xB;
        cpu.memory[0xAC] = 0xDF; // This is the address where it goes to as 0xA1 + 0xB = 0xAC
        cpu.memory[0xDF] = load_mem;
        cpu.a = load_a;
        cpu.run();
        cmp_flag_tester(cpu, carry, zero, negative);

        // Zero Page Testing
        print_title!("Zero Page Test");
        cpu.load_and_reset(vec![first_half + g1_op::ZP, 0xFF]);
        cpu.memory[0xFF] = load_mem;
        cpu.a = load_a;
        cpu.run();
        cmp_flag_tester(cpu, carry, zero, negative);

        // Immediate testing
        print_title!("Immediate Test");
        cpu.load_and_reset(vec![first_half + g1_op::IMMEDIATE_Y, load_mem]);
        cpu.memory[0xFF] = load_mem;
        cpu.a = load_a;
        cpu.run();
        cmp_flag_tester(cpu, carry, zero, negative);

        // Absolute testing
        // Note 0xFE is first due to little endian
        cpu.load_and_reset(vec![first_half + g1_op::ABSOLUTE_X, 0xFE, 0x01]);
        cpu.memory[0x01FE] = load_mem;
        cpu.a = load_a;
        cpu.run();
        cmp_flag_tester(cpu, carry, zero, negative);

        // Indirect Indexed: ($c0), Y
        // Can look at four bytes
        print_title!("Indirect Indexed Test");
        cpu.load_and_reset(vec![second_half + g1_op::INDIRECT, 0xA1]);
        cpu.memory[0xA1] = 0xE1; // LSB
        cpu.memory[0xA2] = 0x05; // MSB
        cpu.y = 0x12;
        cpu.memory[0x05F3] = load_mem;
        cpu.a = load_a;
        cpu.run();
        cmp_flag_tester(cpu, carry, zero, negative);

        // Zero Page, X
        cpu.load_and_reset(vec![second_half + g1_op::ZP, 0xFE]);
        cpu.memory[0xFF] = load_mem;
        cpu.x = 0x01;
        cpu.a = load_a;
        cpu.run();
        cmp_flag_tester(cpu, carry, zero, negative);

        // Absolute, Y
        cpu.load_and_reset(vec![second_half + g1_op::IMMEDIATE_Y, 0x00, 0x02]);
        cpu.y = 0x01;
        cpu.memory[0x0201] = load_mem;
        cpu.a = load_a;
        cpu.run();
        cmp_flag_tester(cpu, carry, zero, negative);

        // Absolute, X
        cpu.load_and_reset(vec![second_half + g1_op::ABSOLUTE_X, 0x00, 0x02]);
        cpu.x = 0x01;
        cpu.memory[0x0201] = load_mem;
        cpu.a = load_a;
        cpu.run();
        cmp_flag_tester(cpu, carry, zero, negative);
    }

    #[test]
    fn test_cmp() {
        // Call modified gen_test but instead of testing if cpu.a value, test the flags
        // Test cases for CMP instruction
        let mut cpu = CPU::new();
        test_cmp_helper(&mut cpu, 0x10, 0x10, true, true, false); // A == M
        test_cmp_helper(&mut cpu, 0x20, 0x10, true, false, false); // A > M
        test_cmp_helper(&mut cpu, 0x10, 0x20, false, false, true); // A < M
    }

    fn set_negative(val: u8) -> u8 {
        let ret = (val as i8).wrapping_neg() as u8;
        println!("old value is {:#b}, new value is {:#b}", val, ret);
        ret
    }

    #[test]
    fn test_sbc() {
        let mut cpu = CPU::new();
        let fh = g1_op::FIRST_SBC;
        let sh = g1_op::SECOND_SBC;
        for i in 0..2 {
            let carry = if i == 0 { false } else { true };
            // 2 Positive
            gen_test(&mut cpu, fh, sh, 0x01 - i, set_negative(0x02), 0x03, carry);

            // 2 Negative
            gen_test(&mut cpu, fh, sh, 0xff - i, set_negative(0xff), 0xfe, carry);

            // Zero
            gen_test(&mut cpu, fh, sh, 0xff - i, 0xff, 0x00, carry);

            // Negative
            gen_test(
                &mut cpu,
                fh,
                sh,
                0b1100_0000 - i,
                set_negative(0b0000_0001),
                0b1100_0001,
                carry,
            );

            // Zero(All)
            gen_test(&mut cpu, fh, sh, 0_u8.wrapping_sub(i), 0, 0, carry);

            // Carry with Zero sum
            gen_test(&mut cpu, fh, sh, 0xff - i, set_negative(0x01), 0x00, carry);

            // Overflow
            gen_test(&mut cpu, fh, sh, 0x7f - i, set_negative(0x02), 0x81, carry);

            // Underflow
            gen_test(&mut cpu, fh, sh, 0x80 - i, set_negative(0x81), 0x01, carry);

            // Additional tests for SBC
            // Positive result with carry
            gen_test(&mut cpu, fh, sh, 0x50 - i, set_negative(0x30), 0x80, carry);

            // Negative result with carry
            gen_test(&mut cpu, fh, sh, 0x30 - i, set_negative(0x50), 0x80, carry);

            // Positive result without carry
            gen_test(&mut cpu, fh, sh, 0x50 - i, set_negative(0x20), 0x70, carry);

            // Negative result without carry
            gen_test(&mut cpu, fh, sh, 0x20 - i, set_negative(0x50), 0x70, carry);
        }
    }
}
