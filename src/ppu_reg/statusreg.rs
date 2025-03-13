use bitflags::bitflags;

bitflags! {
//     7  bit  0
// ---- ----
// VSOx xxxx
// |||| ||||
// |||+-++++- (PPU open bus or 2C05 PPU identifier)
// ||+------- Sprite overflow flag
// |+-------- Sprite 0 hit flag
// +--------- Vblank flag, cleared on read. Unreliable; see below.

pub struct StatusRegister: u8{
    const VBLANK = 0b1000_0000;
    const ZEROHIT = 0b0100_0000;
    const OVERFLOW = 0b0010_0000;
    // Rest are not needed
}
}

impl StatusRegister{
    pub fn new() -> Self{
        StatusRegister::from_bits_truncate(0x00) //BUG may not be initalized to this value
    }

    pub fn clear_vblank(&mut self){
        self.remove(StatusRegister::VBLANK);
    }

    pub fn get_status(&self) -> u8{
        self.bits()
    }


}
