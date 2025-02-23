// CPY and CPX, steal cmp testing from group 1(only need immediate, zp, and absolute)
#[cfg(test)]
mod group3_test {
    use crate::cpu::op::*;
    use crate::cpu::CpuFlags;
    use crate::cpu::CPU;
    use crate::print_title;

    fn test_name(code: u8) -> &'static str {
        match code {
            0xA | 0xB => "LDX/LDY",
            _ => "UNKNOWN",
        }
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
        first_half: u8,
        x: bool,
        carry: bool,
        zero: bool,
        negative: bool,
    ) {
        // Zero Page Testing
        print_title!("Zero Page Test");
        cpu.load_and_reset(vec![first_half + g3_op::ZP, 0xFF]);
        cpu.memory[0xFF] = load_mem;
        cpu.x = load_a;
        if !x {
            cpu.x = 0;
            cpu.y = load_a;
        }
        cpu.run();
        cmp_flag_tester(cpu, carry, zero, negative);

        // Immediate testing
        print_title!("Immediate Test");
        cpu.load_and_reset(vec![first_half + g3_op::IMMEDIATE, load_mem]);
        cpu.x = load_a;
        if !x {
            cpu.x = 0;
            cpu.y = load_a;
        }
        cpu.run();
        cmp_flag_tester(cpu, carry, zero, negative);

        // Absolute testing
        // Note 0xFE is first due to little endian
        cpu.load_and_reset(vec![first_half + g3_op::ABS, 0xFE, 0x01]);
        cpu.memory[0x01FE] = load_mem;
        cpu.x = load_a;
        if !x {
            cpu.x = 0;
            cpu.y = load_a;
        }
        cpu.run();
        cmp_flag_tester(cpu, carry, zero, negative);
    }

    #[test]
    fn test_cpy_cpx() {
        // Set x to be true for cpx, false for cpy
        for i in 0..2 {
            let mut cpu = CPU::new();
            let mut fh = g3_op::CPX;
            let mut x = true;
            if i == 1 {
                x = false;
                fh = g3_op::CPY;
            }
            test_cmp_helper(&mut cpu, 0x10, 0x10, fh, x, true, true, false); // A == M
            test_cmp_helper(&mut cpu, 0x20, 0x10, fh, x, true, false, false); // A > M
            test_cmp_helper(&mut cpu, 0x10, 0x20, fh, x, false, false, true); // A < M
        }
    }

    fn reg_loader(cpu: &mut CPU, x: bool, val: u8) {
        if x {
            cpu.x = val
        } else {
            cpu.y = val
        }
    }

    fn reg_output(cpu: &CPU, x: bool) -> u8 {
        if x {
            return cpu.x;
        }
        return cpu.y;
    }

    // Yes this is copied from group1_test, but modified to allow x and a switching
    fn gen_test(cpu: &mut CPU, x: bool, load_reg: u8, load_mem: u8, expected_val: u8) {
        let first_half = g3_op::FIRST_LDY;
        let second_half = g3_op::SECOND_LDY;
        let mut zp_end = g3_op::ZP;
        let mut im_end = g3_op::IMMEDIATE;
        let mut abs_end = g3_op::ABS;

        if x {
            zp_end = g2_op::ZP;
            im_end = g2_op::IMMEDIATE;
            abs_end = g2_op::ABS;
        }

        // Zero Page Testing
        print_title!("Zero Page Test");
        cpu.load_and_reset((vec![first_half + zp_end, 0xFF]));
        cpu.memory[0xFF] = load_mem;
        reg_loader(cpu, x, load_reg);
        cpu.run();
        assert!(
            reg_output(cpu, x) == expected_val,
            "Failed on Immediate test with {}. cpu.a is {:#b}, expected_val is {:#b}",
            test_name(first_half),
            cpu.a,
            expected_val
        );

        // Immediate testing
        print_title!("Immediate Test");
        cpu.load_and_reset(vec![first_half + im_end, load_mem]);
        cpu.memory[0xFF] = load_mem;
        reg_loader(cpu, x, load_reg);
        cpu.run();
        assert!(
            reg_output(cpu, x) == expected_val,
            "Failed on Immediate test with {}. cpu.a is {:#b}, expected_val is {:#b}",
            test_name(first_half),
            cpu.a,
            expected_val
        );

        // Absolute testing
        // Note 0xFE is first due to little endian
        cpu.load_and_reset((vec![first_half + abs_end, 0xFE, 0x01]));
        cpu.memory[0x01FE] = load_mem;
        reg_loader(cpu, x, load_reg);
        cpu.run();
        assert!(
            reg_output(cpu, x) == expected_val,
            "Failed on Absolute test with {}. cpu.a is {:#b}, expected_val is {:#b}",
            test_name(first_half),
            cpu.a,
            expected_val
        );

        // Zero Page, X
        cpu.load_and_reset(vec![second_half + zp_end, 0xA0]);
        reg_loader(cpu, x, load_reg);
        cpu.memory[0xA0 + cpu.x as usize] = load_mem;
        cpu.run();

        assert!(
            reg_output(cpu, x) == expected_val,
            "Failed on Zero Page, X with {}. cpu reg is {:#b}, expected_val is {:#b}",
            test_name(first_half),
            if x { cpu.x } else { cpu.y },
            expected_val
        );

        // Absolyte, X
        cpu.load_and_reset(vec![second_half + abs_end, 0x00, 0x02]);
        reg_loader(cpu, x, load_reg);
        cpu.memory[0x0200 + cpu.x as usize] = load_mem;
        cpu.run();

        assert!(
            reg_output(cpu, x) == expected_val,
            "Failed on Absolute, X with {}. cpu.a is {:#b}, expected_val is {:#b}",
            test_name(first_half),
            cpu.a,
            expected_val
        );
    }

    // LDY and LDX, steal from LDA
    #[test]
    fn ldx_ldy_test() {
        // Set x to be true for cpx, false for cpy
        for i in 0..2 {
            let mut cpu = CPU::new();
            let mut x = i == 1;
            gen_test(&mut cpu, x, 0, 0x05, 0x05);
            assert!(!cpu.flags.contains(CpuFlags::NEGATIVE) && !cpu.flags.contains(CpuFlags::ZERO));

            // Zero Test
            gen_test(&mut cpu, x, 0x20, 0, 0);
            assert!(!cpu.flags.contains(CpuFlags::NEGATIVE) && cpu.flags.contains(CpuFlags::ZERO));

            // Negative test
            gen_test(&mut cpu, x, 0x20, 0xFF, 0xFF);
            assert!(cpu.flags.contains(CpuFlags::NEGATIVE) && !cpu.flags.contains(CpuFlags::ZERO));
        }
    }

    // STY done first, STX done second
    #[test]
    fn test_stx_sty() {
        let mut cpu = CPU::new();
        let first_half = g3_op::FIRST_STY;
        let second_half = g3_op::SECOND_STY; // Ok this is worded badly but is the MSB switched for zp, X
        let mut second_half_zp = g3_op::ZP;
        let mut second_half_abs = g3_op::ABS;
        let load_reg: u8 = 0x0E;

        for i in 0..2 {
            // Zero Page Testing
            print_title!("Zero Page Test");
            let x = i == 1;
            if x {
                second_half_zp = g2_op::ZP;
                second_half_abs = g2_op::ABS;
            }
            cpu.load_and_reset((vec![first_half + second_half_zp, 0xA1]));
            reg_loader(&mut cpu, x, load_reg);
            cpu.run();
            assert!(
                cpu.memory[0xA1] == reg_output(&cpu, x),
                "Failed on zero page test for test_sta"
            );

            // Absolute testing
            // Note 0xFE is first due to little endian
            cpu.load_and_reset((vec![first_half + second_half_abs, 0xFE, 0x01]));
            reg_loader(&mut cpu, x, load_reg);
            cpu.run();
            assert!(
                cpu.memory[0x01FE] == reg_output(&cpu, x),
                "Failed on absolute test for test_sta. Cpu.a is {}, cpu.memory[0x01FE] is {}",
                cpu.a,
                cpu.memory[0x01FE]
            );

            // Zero Page, X
            cpu.load_and_reset(vec![second_half + second_half_zp, 0xA0]);
            reg_loader(&mut cpu, x, load_reg);
            cpu.run();

            assert!(
                reg_output(&cpu, x) == cpu.memory[0xA0 + cpu.x as usize],
                "Failed on zero page, X for test_sta"
            );
        }
    }

    // JMP and JMP(), one is absolute, other is indirect
    // Just test for pc
    #[test]
    fn test_jump() {
        let mut cpu = CPU::new();
        cpu.load_and_reset(vec![g3_op::JMP, 0x04, 0x06]);
        cpu.memory[0x0604] = 0xE8; // INX instruction
        cpu.run();
        assert!(cpu.x == 1, "Jump failed, cpu.x value is {}", cpu.x);

        // JMP() test
        print_title!("JMP() Test");
        cpu.memory[0x0600..0x0605].fill(0);
        cpu.load_and_reset(vec![g3_op::JMP_REL, 0x04, 0x06]);
        cpu.memory[0x0604] = 0xAA;
        cpu.memory[0x0605] = 0x01;
        // Should jump to 0x01AA
        cpu.memory[0x01AA] = 0xE8;
        cpu.memory[0x01AB] = 0x00; // BRK
        cpu.run();
        assert!(cpu.x == 1, "Relative jump failed, cpu.x value is {}", cpu.x);
        // JMP test with page bug
        print_title!("JMP() with page bug");
        cpu.load_and_reset(vec![g3_op::JMP_REL, 0xFF, 0x06]);
        cpu.memory[0x06FF] = 0xAA;
        cpu.memory[0x0600] = 0x01;
        cpu.memory[0x01AA] = 0xE8;
        cpu.run();
        assert!(
            cpu.x == 1,
            "Relative jump failed with page bug, cpu.x value is {}",
            cpu.x
        );
    }

    // Check BIT for zp and abs
    // Just need to check zero, overflow, and negative flags
    // Similar to CMP flag checking procedure

    fn bit_flag_tester(cpu: &CPU, overflow: bool, zero: bool, negative: bool){
        if overflow {
            assert!(
                cpu.flags.contains(CpuFlags::OVERFLOW),
                "Expected overflow flag to be set"
            );
        } else {
            assert!(
                !cpu.flags.contains(CpuFlags::OVERFLOW),
                "Expected overflow flag to be clear"
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

    fn test_bit_helper(
        cpu: &mut CPU,
        load_a: u8,
        load_mem: u8,
        zero: bool,
        overflow:bool,
        negative: bool,
    ) {
        let first_half = g3_op::BIT;

        // Zero Page Testing
        print_title!("Zero Page Test");
        cpu.load_and_reset(vec![first_half + g3_op::ZP, 0xFF]);
        cpu.memory[0xFF] = load_mem;
        cpu.a = load_a;
        cpu.run();
        bit_flag_tester(cpu, overflow, zero, negative);


        // Absolute testing
        // Note 0xFE is first due to little endian
        print_title!("Absolute test");
        cpu.load_and_reset(vec![first_half + g3_op::ABS, 0xFE, 0x01]);
        cpu.memory[0x01FE] = load_mem;
        cpu.a = load_a;
        cpu.run();
        bit_flag_tester(cpu, overflow, zero, negative);
    }

    #[test]
    fn test_bit() {
        // Call modified gen_test but instead of testing if cpu.a value, test the flags
        // Test cases for BIT instruction
        let mut cpu = CPU::new();
        // Zero Test
        test_bit_helper(&mut cpu, 0b11111111, 0b00000000, true, false, false);
        // Overflow Test
        test_bit_helper(&mut cpu, 0xFF, 0b0100_0000, false, true, false);
        // Negative Test
        test_bit_helper(&mut cpu, 0xFF, 0b1000_0000, false, false, true)
        
    }
}
