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
    fn gen_test(
        cpu: &mut CPU,
        first_half: u8,
        second_half: u8,
        load_a: u8,
        load_mem: u8,
        expected_val: u8,
    ) {
        // Indirect testing, two byte only!
        cpu.load_and_reset(vec![first_half + g1_op::INDIRECT, 0xA1]);
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
        if test_name(first_half) != "STA" {
            // Immediate testing
            print_title!("Immediate Test");
            cpu.load_and_reset(vec![first_half + g1_op::IMMEDIATE_Y, load_mem]);

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
        }

        // Absolute testing
        // Note 0xFE is first due to little endian
        cpu.load_and_reset((vec![first_half + g1_op::ABSOLUTE_X, 0xFE, 0x01]));
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
        gen_test(&mut cpu, fh, sh, 0b10101010, 0b01010101, 0xFF);

        assert!(cpu.flags.contains(CpuFlags::NEGATIVE) && !cpu.flags.contains(CpuFlags::ZERO));

        // Test ora if both are 0b1111_1111
        gen_test(&mut cpu, fh, sh, 0xFF, 0xFF, 0xFF);
        assert!(cpu.flags.contains(CpuFlags::NEGATIVE) && !cpu.flags.contains(CpuFlags::ZERO));

        // Testing ora if both are 0
        gen_test(&mut cpu, fh, sh, 0, 0, 0);
        assert!(!cpu.flags.contains(CpuFlags::NEGATIVE) && cpu.flags.contains(CpuFlags::ZERO));

        // Testing for no flags
        gen_test(&mut cpu, fh, sh, 0b01110000, 0b01110000, 0b01110000);

        assert!(!cpu.flags.contains(CpuFlags::NEGATIVE) && !cpu.flags.contains(CpuFlags::ZERO));
    }

    #[test]
    fn test_and() {
        // Similar tests to ORA
        let mut cpu = CPU::new();
        let fh = g1_op::FIRST_AND;
        let sh = g1_op::SECOND_AND;
        gen_test(&mut cpu, fh, sh, 0b10101010, 0b01010101, 0x00);

        assert!(!cpu.flags.contains(CpuFlags::NEGATIVE) && cpu.flags.contains(CpuFlags::ZERO));

        // Test and if both are 0b1111_1111
        gen_test(&mut cpu, fh, sh, 0xFF, 0xFF, 0xFF);
        assert!(cpu.flags.contains(CpuFlags::NEGATIVE) && !cpu.flags.contains(CpuFlags::ZERO));

        // Testing and if both are 0
        gen_test(&mut cpu, fh, sh, 0, 0, 0);
        assert!(!cpu.flags.contains(CpuFlags::NEGATIVE) && cpu.flags.contains(CpuFlags::ZERO));

        // Testing for no flags
        gen_test(&mut cpu, fh, sh, 0b01110000, 0b01001100, 0b0100_0000);
        assert!(!cpu.flags.contains(CpuFlags::NEGATIVE) && !cpu.flags.contains(CpuFlags::ZERO));
    }

    #[test]
    fn test_eor() {
        let mut cpu = CPU::new();
        let fh = g1_op::FIRST_EOR;
        let sh = g1_op::SECOND_EOR;

        gen_test(&mut cpu, fh, sh, 0b10101010, 0b01010101, 0xFF);

        assert!(cpu.flags.contains(CpuFlags::NEGATIVE) && !cpu.flags.contains(CpuFlags::ZERO));

        // Test and if both are 0b1111_1111
        gen_test(&mut cpu, fh, sh, 0xFF, 0xFF, 0x00);
        assert!(!cpu.flags.contains(CpuFlags::NEGATIVE) && cpu.flags.contains(CpuFlags::ZERO));

        // Testing and if both are 0
        gen_test(&mut cpu, fh, sh, 0, 0, 0);
        assert!(!cpu.flags.contains(CpuFlags::NEGATIVE) && cpu.flags.contains(CpuFlags::ZERO));

        gen_test(&mut cpu, fh, sh, 0b1001_1001, 0b0110_1001, 0xF0);
        assert!(cpu.flags.contains(CpuFlags::NEGATIVE) && !cpu.flags.contains(CpuFlags::ZERO));
    }

    fn test_adc_flag_check(carry: bool, zero: bool, overflow: bool, negative: bool){

    }

    #[test]
    fn test_adc() {
        let mut cpu = CPU::new();
        let fh = g1_op::FIRST_ADC;
        let sh = g1_op::SECOND_ADC;

        // TODO We will simulate carry flags by subtracting 1 from given values and adding them back with the carry flag
        // Set ending to 2 just to do one loop
        for i in 1..2{
            // 2 Positive
            gen_test(&mut cpu, fh, sh, 0x01, 0x02, 0x03);

            // 2 Negative
            gen_test(&mut cpu, fh, sh, 0xff, 0xff, 0xfe);

            // Zero
            gen_test(&mut cpu, fh, sh, 0xff, 0x01, 0x00);

            // Negative
            gen_test(&mut cpu, fh, sh, 0b1100_0000, 0b0000_0001, 0b1100_0001);

            // Zero(All)
            gen_test(&mut cpu, fh, sh, 0, 0, 0);

            // Carry with Zero sum
            gen_test(&mut cpu, fh, sh, 0xff, 0x01, 0x00);

            // Overflow
            gen_test(&mut cpu, fh, sh, 0x7f, 0x02, 0x81);

            // Underflow
            gen_test(&mut cpu, fh, sh, 0x80, 0x81, 0x01);
        }
    }

    #[test]
    fn test_sta() {
        let mut cpu = CPU::new();
        let fh = g1_op::FIRST_STA;
        let sh = g1_op::SECOND_STA;

    }

    #[test]
    fn test_lda() {}

    #[test]
    fn test_cmp() {}

    #[test]
    fn test_sbc() {}
}
