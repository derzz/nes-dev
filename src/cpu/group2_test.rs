// NOTE: STX and LDX are in group3_tests

// ASL, ROL, LSR, and LSR can be done in conjunction
// Note a separate test for accumulator must be done

// DEC and INC need to check memory vlaues

#[cfg(test)]
mod group2_test{
    use crate::cpu::op::*;
    use crate::cpu::CpuFlags;
    use crate::cpu::CPU;
    use crate::print_title;

    fn zero_negative_flag_test(cpu: &CPU, zero: bool, negative: bool) {
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
    fn memory_modifier(cpu: &mut CPU, first_half: u8, second_half: u8, load_mem: u8, expected_val: u8, zero: bool, negative: bool){
        // Absolute
        cpu.load_and_reset(vec![first_half + g2_op::ABS, 0x00, 0x02]);
        cpu.memory[0x200] = load_mem;
        cpu.run();
        assert!(cpu.memory[0x200] == expected_val, "Failed on INC/DEC absolute test, provided value is {:#x}, the contents of 0x200 is {:#x}",load_mem, cpu.memory[0x200]);
        zero_negative_flag_test(cpu, zero, negative);

        // Absolute + x
        cpu.load_and_reset(vec![second_half + g2_op::ABS, 0x00, 0x02]);
        cpu.x = 1;
        cpu.memory[0x201] = load_mem;
        cpu.run();
        assert!(cpu.memory[0x201] == expected_val, "Failed on INC/DEC absolute, X test");
        zero_negative_flag_test(cpu, zero, negative);

        // Zero Page
        cpu.load_and_reset(vec![first_half + g2_op::ZP, 0xAB]);
        cpu.memory[0xAB] = load_mem;
        cpu.run();
        assert!(cpu.memory[0xAB] == expected_val, "Failed on INC/DEC Zero Page test");
        zero_negative_flag_test(cpu, zero, negative);

        // Zero Page, X
        cpu.load_and_reset(vec![second_half + g2_op::ZP, 0xAB]);
        cpu.x = 1;
        cpu.memory[0xAC] = load_mem;
        cpu.run();
        assert!(cpu.memory[0xAC] == expected_val, "Failed on INC/DEC Zero Page, X test");
        zero_negative_flag_test(cpu, zero, negative);
    }

   #[test]
   fn test_dec(){
        let mut cpu = CPU::new();
        // Zero Flag
        memory_modifier(&mut cpu, g2_op::FIRST_DEC, g2_op::SECOND_DEC, 0x1, 0x0, true, false); 
        // Negative Flag
        memory_modifier(&mut cpu, g2_op::FIRST_DEC, g2_op::SECOND_DEC, 0xFF, 0xFE, false, true);
        // No Flags set
        memory_modifier(&mut cpu, g2_op::FIRST_DEC, g2_op::SECOND_DEC, 0x0F, 0xe, false, false);
   }

   #[test]
   fn test_inc(){
    let mut cpu = CPU::new();
        // Zero Flag
        memory_modifier(&mut cpu, g2_op::FIRST_INC, g2_op::SECOND_INC, 0xFF, 0x0, true, false);
        // Negative Flag
        memory_modifier(&mut cpu, g2_op::FIRST_INC, g2_op::SECOND_INC, 0xFE, 0xFF, false, true);
        // No Flags set
        memory_modifier(&mut cpu, g2_op::FIRST_INC, g2_op::SECOND_INC, 0x0F, 0x10, false, false);
   }


}
