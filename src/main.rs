use nes::cpu::CPU;
// NOTE DO NOT RUN MAIN AT THE MOMENT
// IMPLEMENTATION IS NOT FINISHED(Added to just get rid of never used errors)
fn main() {
    let mut cpu = CPU::new();
    cpu.load_and_run(vec![0x00]);
}
