use crate::ppu_reg::scrollreg::ScrollRegister;
use crate::ppu_reg::statusreg::StatusRegister;
use crate::ppu_reg::{addrreg::AddrRegister, controlreg::ControlRegister, maskreg::MaskRegister};
use crate::rom::Mirroring;

pub struct PPU {
    pub chr_rom: Vec<u8>,
    pub palette_table: [u8; 32],
    pub vram: [u8; 2048],
    pub oam_data: [u8; 256],
    pub oam_addr: u8,
    pub internal_data_buf: u8,

    pub addr: AddrRegister,
    pub status: StatusRegister,
    pub ctrl: ControlRegister,
    pub mask: MaskRegister,
    pub scroll: ScrollRegister,

    pub mirroring: Mirroring,

    scanline: u16, // Which scanline should be drawn
    cycles: usize // Location of current cycle
}

impl PPU {
    // For testing purposes
    pub fn new_empty_rom() -> Self {
        PPU::new(vec![0; 2048], Mirroring::HORIZONTAL)
    }
    pub fn new(chr_rom: Vec<u8>, mirroring: Mirroring) -> Self {
        PPU {
            chr_rom: chr_rom,
            palette_table: [0; 32],
            vram: [0; 2048],
            oam_data: [0; 256],
            oam_addr: 0,
            internal_data_buf: 0, // Emulating internal data buffer
            addr: AddrRegister::new(),
            status: StatusRegister::new(),
            ctrl: ControlRegister::new(),
            mask: MaskRegister::new(),
            scroll: ScrollRegister::new(),
            mirroring: mirroring,
            scanline: 0,
            cycles: 0
        }
    }

    pub fn tick(&mut self, cycles: u8) -> bool{
        self.cycles += cycles as usize;
        // Scanline lasts for 341 PPU cycles
        if self.cycles >= 341{
            self.cycles -= 341;
            self.scanline += 1;

            // Scanline is on vBlank line
            if self.scanline == 241 {
                // Enabling causes NMI interrupt to be called at start of vblank
                if self.ctrl.generate_vblank_nmi(){
                    self.status.set_vblank_status(true);
                    todo!("Triggers nmi interrupt")
                }
            }

            if self.scanline == 262{
                self.scanline = 0;
                self.status.reset_vblank_status();
                return true; // Indicating reset and full render is done
            }
        }
        return false; // Full render is not finished
    }

    // 0x2000 write, PPUCTRL(Flags)
    pub fn write_to_ctrl(&mut self, val: u8) {
        // Val should represent the bit flags of the control registers
        self.ctrl.update(val);
    }

    // 0x2001 write, PPUMASK
    pub fn write_to_mask(&mut self, val: u8) {
        self.mask.update(val);
    }

    // 0x2002 read, PPUSTATUS
    pub fn read_status(&mut self) -> u8 {
        // Flags are read, Vblank and w register should be cleared after read
        let ret = self.status.get_status();
        self.status.clear_vblank();
        self.addr.reset_latch();
        ret
    }

    // 0x2003 write, OAMADDR
    pub fn write_to_oam_addr(&mut self, val: u8) {
        self.oam_addr = val;
    }

    // 0x2004 read, OAMDATA
    pub fn read_oam_data(&self) -> u8 {
        self.oam_data[self.oam_addr as usize]
    }

    pub fn write_to_oam_data(&mut self, val: u8) {
        self.oam_data[self.oam_addr as usize] = val;
        self.oam_addr = self.oam_addr.wrapping_add(1);
    }

    // 0x2005 write, PPUSCROLL
    pub fn write_to_ppuscroll(&mut self, val: u8) {
        self.scroll.write(val);
    }

    // 0x2006 write, PPUADDR
    pub fn write_to_ppu_addr(&mut self, val: u8) {
        self.addr.update(val);
    }

    // 0x2007 read/write, PPUDATA(VRAM read/write data register)
    // Writes to data
    pub fn write_to_data(&mut self, val: u8) {
        let addr = self.addr.get();
        match addr {
            0..=0x1fff => println!("Attempt to write to character rom space!"),
            0x2000..=0x2FFF => {
                // Name tables
                self.vram[self.mirror_vram_addr(addr) as usize] = val;
            }
            0x3000..=0x3EFF => unimplemented!("Unused ppu memory attempted to write {}", addr),
            // Scales down to palette RAM
            0x3F10 | 0x3F14 | 0x3F18 | 0x3F1C => {
                let mirr_addr = addr - 0x3F10; // BUG may be copies of 0x3F00 instead?
                self.palette_table[mirr_addr as usize] = val;
            }
            // BUG should be scaling down by mirroring
            0x3F00..=0x3FFF => self.palette_table[((addr - 0x3f00) % 32) as usize] = val,
            _ => panic!("Unknown write access to mirrored space {}", addr),
        }

        self.increment_vram_addr();
    }

    fn increment_vram_addr(&mut self) {
        self.addr.increment(self.ctrl.vram_addr_increment());
    }

    pub fn read_data(&mut self) -> u8 {
        let addr = self.addr.get();
        self.increment_vram_addr();

        match addr {
            0..=0x1fff => {
                // Pattern tables 0 and 1
                let result = self.internal_data_buf;
                self.internal_data_buf = self.chr_rom[addr as usize];
                result
            }
            0x2000..=0x2fff => {
                // Nametables 0-3
                let result = self.internal_data_buf;
                self.internal_data_buf = self.vram[self.mirror_vram_addr(addr) as usize];
                result
            }
            0x3000..=0x3eff => panic!(
                // Unused
                "0x3000 to 0x3eff is not expected to be used, the requested address is {}",
                addr
            ),
            0x3f00..=0x3fff => self.palette_table[(addr - 0x3f00) as usize], // Palette RAM, 0x3F20-0x3FFF are mirrors of 0x3F00-0x3F1F
            _ => panic!("unexpected access to mirrored space {}", addr),
        }
    }

    // Mirrors vram

    fn mirror_vram_addr(&mut self, addr: u16) -> u16 {
        let mirrored_vram = addr & !0x1000; // Bring down 0x3000.. 0x3eff to 0x2000.. 0x2eff by subtracting 0x1000(If it's 0x2000 or 0x2fff, nothing changes)
        let vram_index = mirrored_vram - 0x2000; // Bring down to 0x0.. 0x0eff(to vram vector)
        let name_table = vram_index / 0x400; // Determines what nametable to access
        match (&self.mirroring, name_table) {
            (Mirroring::VERTICAL, 2) | (Mirroring::VERTICAL, 3) => vram_index - 0x800,
            (Mirroring::HORIZONTAL, 2) => vram_index - 0x400,
            (Mirroring::HORIZONTAL, 1) => vram_index - 0x400,
            (Mirroring::HORIZONTAL, 3) => vram_index - 0x800,
            _ => vram_index,
        }
    }

    fn write_oam_dma(&mut self, data: &[u8; 256]) {
        for x in data.iter() {
            self.oam_data[self.oam_addr as usize] = *x;
            self.oam_addr = self.oam_addr.wrapping_add(1);
        }
    }
}

#[cfg(test)]
pub mod test {
    use super::*;

    #[test]
    fn test_ppu_vram_writes() {
        let mut ppu = PPU::new_empty_rom();
        ppu.write_to_ppu_addr(0x23);
        ppu.write_to_ppu_addr(0x05);
        ppu.write_to_data(0x66);

        assert_eq!(ppu.vram[0x0305], 0x66);
    }

    #[test]
    fn test_ppu_vram_reads() {
        let mut ppu = PPU::new_empty_rom();
        ppu.write_to_ctrl(0);
        ppu.vram[0x0305] = 0x66;

        ppu.write_to_ppu_addr(0x23);
        ppu.write_to_ppu_addr(0x05);

        ppu.read_data(); //load_into_buffer
        assert_eq!(ppu.addr.get(), 0x2306);
        assert_eq!(ppu.read_data(), 0x66);
    }

    #[test]
    fn test_ppu_vram_reads_cross_page() {
        let mut ppu = PPU::new_empty_rom();
        ppu.write_to_ctrl(0);
        ppu.vram[0x01ff] = 0x66;
        ppu.vram[0x0200] = 0x77;

        ppu.write_to_ppu_addr(0x21);
        ppu.write_to_ppu_addr(0xff);

        ppu.read_data(); //load_into_buffer
        assert_eq!(ppu.read_data(), 0x66);
        assert_eq!(ppu.read_data(), 0x77);
    }

    #[test]
    fn test_ppu_vram_reads_step_32() {
        let mut ppu = PPU::new_empty_rom();
        ppu.write_to_ctrl(0b100);
        ppu.vram[0x01ff] = 0x66;
        ppu.vram[0x01ff + 32] = 0x77;
        ppu.vram[0x01ff + 64] = 0x88;

        ppu.write_to_ppu_addr(0x21);
        ppu.write_to_ppu_addr(0xff);

        ppu.read_data(); //load_into_buffer
        assert_eq!(ppu.read_data(), 0x66);
        assert_eq!(ppu.read_data(), 0x77);
        assert_eq!(ppu.read_data(), 0x88);
    }

    // Horizontal: https://wiki.nesdev.com/w/index.php/Mirroring
    //   [0x2000 A ] [0x2400 a ]
    //   [0x2800 B ] [0x2C00 b ]
    #[test]
    fn test_vram_horizontal_mirror() {
        let mut ppu = PPU::new_empty_rom();
        ppu.write_to_ppu_addr(0x24);
        ppu.write_to_ppu_addr(0x05);

        ppu.write_to_data(0x66); //write to a

        ppu.write_to_ppu_addr(0x28);
        ppu.write_to_ppu_addr(0x05);

        ppu.write_to_data(0x77); //write to B

        ppu.write_to_ppu_addr(0x20);
        ppu.write_to_ppu_addr(0x05);

        ppu.read_data(); //load into buffer
        assert_eq!(ppu.read_data(), 0x66); //read from A

        ppu.write_to_ppu_addr(0x2C);
        ppu.write_to_ppu_addr(0x05);

        ppu.read_data(); //load into buffer
        assert_eq!(ppu.read_data(), 0x77); //read from b
    }

    // Vertical: https://wiki.nesdev.com/w/index.php/Mirroring
    //   [0x2000 A ] [0x2400 B ]
    //   [0x2800 a ] [0x2C00 b ]
    #[test]
    fn test_vram_vertical_mirror() {
        let mut ppu = PPU::new(vec![0; 2048], Mirroring::VERTICAL);

        ppu.write_to_ppu_addr(0x20);
        ppu.write_to_ppu_addr(0x05);

        ppu.write_to_data(0x66); //write to A

        ppu.write_to_ppu_addr(0x2C);
        ppu.write_to_ppu_addr(0x05);

        ppu.write_to_data(0x77); //write to b

        ppu.write_to_ppu_addr(0x28);
        ppu.write_to_ppu_addr(0x05);

        ppu.read_data(); //load into buffer
        assert_eq!(ppu.read_data(), 0x66); //read from a

        ppu.write_to_ppu_addr(0x24);
        ppu.write_to_ppu_addr(0x05);

        ppu.read_data(); //load into buffer
        assert_eq!(ppu.read_data(), 0x77); //read from B
    }

    #[test]
    fn test_read_status_resets_latch() {
        let mut ppu = PPU::new_empty_rom();
        ppu.vram[0x0305] = 0x66;

        ppu.write_to_ppu_addr(0x21);
        ppu.write_to_ppu_addr(0x23);
        ppu.write_to_ppu_addr(0x05);

        ppu.read_data(); //load_into_buffer
        assert_ne!(ppu.read_data(), 0x66);

        ppu.read_status();

        ppu.write_to_ppu_addr(0x23);
        ppu.write_to_ppu_addr(0x05);

        ppu.read_data(); //load_into_buffer
        assert_eq!(ppu.read_data(), 0x66);
    }

    #[test]
    fn test_ppu_vram_mirroring() {
        let mut ppu = PPU::new_empty_rom();
        ppu.write_to_ctrl(0);
        ppu.vram[0x0305] = 0x66;

        ppu.write_to_ppu_addr(0x63); //0x6305 -> 0x2305
        ppu.write_to_ppu_addr(0x05);

        ppu.read_data(); //load into_buffer
        assert_eq!(ppu.read_data(), 0x66);
        // assert_eq!(ppu.addr.read(), 0x0306)
    }

    #[test]
    fn test_read_status_resets_vblank() {
        let mut ppu = PPU::new_empty_rom();
        ppu.status.set_vblank_status(true);

        let status = ppu.read_status();

        assert_eq!(status >> 7, 1);
        assert_eq!(ppu.status.snapshot() >> 7, 0);
    }

    #[test]
    fn test_oam_read_write() {
        let mut ppu = PPU::new_empty_rom();
        ppu.write_to_oam_addr(0x10);
        ppu.write_to_oam_data(0x66);
        ppu.write_to_oam_data(0x77);

        ppu.write_to_oam_addr(0x10);
        assert_eq!(ppu.read_oam_data(), 0x66);

        ppu.write_to_oam_addr(0x11);
        assert_eq!(ppu.read_oam_data(), 0x77);
    }

    #[test]
    fn test_oam_dma() {
        let mut ppu = PPU::new_empty_rom();

        let mut data = [0x66; 256];
        data[0] = 0x77;
        data[255] = 0x88;

        ppu.write_to_oam_addr(0x10);
        ppu.write_oam_dma(&data);

        ppu.write_to_oam_addr(0xf); //wrap around
        assert_eq!(ppu.read_oam_data(), 0x88);

        ppu.write_to_oam_addr(0x10);
        assert_eq!(ppu.read_oam_data(), 0x77);

        ppu.write_to_oam_addr(0x11);
        assert_eq!(ppu.read_oam_data(), 0x66);
    }
}
