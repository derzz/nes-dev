#[cfg(test)]
// Similar to JMP testing, and just branch further into the code
// Eg. Set [BRANCH 0x02 0x00 INX]
// And just test for x and see if branch has been or hasn't been done
mod branch_test {
    use crate::cpu::op::*;
    use crate::cpu::CpuFlags;
    use crate::cpu::CPU;
    use crate::cpu::PROGRAM_START;

    // All this will do is run [op 0x01 0x00 INX]
    // Specify clear if the flag should be clear(This will enable all flags except the chosen flag)
    // It will initalize all flags as empty, specify in flags to enable flags
    // If x == 1, branch is successful, if not, branch is unsuccessful
    fn jump(cpu: &mut CPU, op: u8, clear: bool, flag: CpuFlags) {
        cpu.load_and_reset(vec![op, 0x01, 0x00, op::INX]);
        if clear {
            // println!("branch_test: Allocating flags to clear!");
            cpu.flags = CpuFlags::from_bits_truncate(0b1111_1111);
            cpu.flags.remove(flag);
        } else {
            cpu.flags.insert(flag);
        }
        cpu.run();
    }

    // Literally same as above but just testing if branch can branch negative
    fn jump_neg(cpu: &mut CPU, op: u8, clear: bool, flag: CpuFlags) {
        cpu.load_and_reset(vec![op, 0xFC, 0x00]);
        cpu.memory[PROGRAM_START - 2] = op::INX;
        cpu.memory[PROGRAM_START - 1] = 0x00;
        if clear {
            // println!("branch_test: Allocating flags to clear!");
            cpu.flags = CpuFlags::from_bits_truncate(0b1111_1111);
            cpu.flags.remove(flag);
        } else {
            cpu.flags.insert(flag);
        }
        cpu.run();
    }

    // Each test will test when branch should occur and branch should not occur
    // Clear tests if flag is clear
    fn helper_test(op: u8, clear: bool, flag: CpuFlags) {
        let mut cpu = CPU::new();
        // First test should succeed
        jump(&mut cpu, op, clear, flag);
        assert!(
            cpu.x == 1,
            "Helper_test positive failed on {:#x}, cpu.x is {}",
            op,
            cpu.x
        );

        jump_neg(&mut cpu, op, clear, flag);
        assert!(
            cpu.x == 1,
            "jump neg Helper_test positive failed on {:#x}, cpu.x is {}",
            op,
            cpu.x
        );

        // Second test should fail
        jump(&mut cpu, op, !clear, flag);
        assert!(
            cpu.x == 0,
            "Helper_test negative failed on {:#x}, cpu.x is {}",
            op,
            cpu.x
        );

        jump_neg(&mut cpu, op, !clear, flag);
        assert!(
            cpu.x == 0,
            "jump neg Helper_test negative failed on {:#x}, cpu.x is {}",
            op,
            cpu.x
        );
    }

    #[test]
    fn bpl_test() {
        helper_test(branch_op::BPL, true, CpuFlags::NEGATIVE);
    }

    #[test]
    fn bmi_test() {
        helper_test(branch_op::BMI, false, CpuFlags::NEGATIVE);
    }

    #[test]
    fn bvc_test() {
        helper_test(branch_op::BVC, true, CpuFlags::OVERFLOW);
    }

    #[test]
    fn bvs_test() {
        helper_test(branch_op::BVS, false, CpuFlags::OVERFLOW);
    }

    #[test]
    fn bcc_test() {
        helper_test(branch_op::BCC, true, CpuFlags::CARRY);
    }

    #[test]
    fn bcs_test() {
        helper_test(branch_op::BCS, false, CpuFlags::CARRY);
    }

    #[test]
    fn bne_test() {
        helper_test(branch_op::BNE, true, CpuFlags::ZERO);
    }

    #[test]
    fn beq_test() {
        helper_test(branch_op::BEQ, false, CpuFlags::ZERO);
    }
}
