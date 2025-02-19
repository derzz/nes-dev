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
    // NOT TO BE USED WITH CMP
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
    }

    #[test]
    fn test_ora() {
        let mut cpu = CPU::new();
        gen_test(
            &mut cpu,
            g1_op::FIRST_ORA,
            g1_op::SECOND_ORA,
            0b10101010,
            0b01010101,
            0xFF,
        );
    }

    #[test]
    fn test_and() {}

    #[test]
    fn test_eor() {}
}
