mod mos;
mod execute;


    fn main(){
        let mut emulator = mos::Mos::initalize(); // Initalizes memory and registers
        execute::instruction::execute(&mut emulator, 0x08);
    }
