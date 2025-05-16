use bitflags::bitflags;

bitflags! {
// 7  bit  0
// ---- ----
// BGRs bMmG
// |||| ||||
// |||| |||+- Greyscale (0: normal color, 1: greyscale)
// |||| ||+-- 1: Show background in leftmost 8 pixels of screen, 0: Hide
// |||| |+--- 1: Show sprites in leftmost 8 pixels of screen, 0: Hide
// |||| +---- 1: Enable background rendering
// |||+------ 1: Enable sprite rendering
// ||+------- Emphasize red (green on PAL/Dendy)
// |+-------- Emphasize green (red on PAL/Dendy)
// +--------- Emphasize blue
    pub struct MaskRegister: u8{
        const BLUE = 0b1000_0000;
        const GREEN = 0b0100_0000;
        const RED = 0b0010_0000;
        const SPRITE = 0b0001_0000;
        const BACKGROUND = 0b0000_1000;
        const LEFTSPRITE = 0b0000_0100;
        const LEFTBG = 0b0000_0010;
        const GREY = 0b0000_00001;
    }
}

impl MaskRegister {
    pub fn new() -> Self {
        MaskRegister::from_bits_truncate(0x00)
    }

    pub fn update(&mut self, data: u8) {
        *self = MaskRegister::from_bits_truncate(data);
    }

    pub fn show_sprites(&self) -> bool {
        self.contains(MaskRegister::SPRITE)
    }
}
