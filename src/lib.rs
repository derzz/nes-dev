pub mod bus;
pub mod cpu;
pub mod op;
pub mod ppu;
pub mod ppu_reg;
pub mod rom;
pub mod trace;

use cpu::*;

use rom::Rom;
use trace::trace;

use crate::bus::Bus;
#[macro_export]
macro_rules! print_title {
    ($title:expr) => {
        println!(
            "===================== {} =====================",
            $title.to_uppercase()
        );
    };
}

pub fn run_nestest_and_capture() -> Vec<String> {
    let game_bytes = std::fs::read("nestest.nes").unwrap();
    let rom = Rom::new(&game_bytes).unwrap();
    let bus = Bus::new(rom);
    let mut cpu = CPU::new(bus);
    cpu.reset();
    let mut output = Vec::new();
    cpu.run_with_callback(|cpu| {
        output.push(trace(cpu));
    });
    output
}
