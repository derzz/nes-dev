use crate::ppu_reg::statusreg::StatusRegister;
use crate::ppu_reg::{addrreg::AddrRegister, controlreg::ControlRegister};
use crate::rom::Mirroring;

pub struct PPU {
    pub chr_rom: Vec<u8>,
    pub palette_table: [u8; 32],
    pub vram: [u8; 2048],
    pub oam_data: [u8; 256],
    pub internal_data_buf: u8,
    
    pub addr: AddrRegister,
    pub status: StatusRegister,
    pub ctrl: ControlRegister,

    pub mirroring: Mirroring,
}

impl PPU {
    pub fn new(chr_rom: Vec<u8>, mirroring: Mirroring) -> Self {
        PPU {
            chr_rom: chr_rom,
            palette_table: [0; 32],
            vram: [0; 2048],
            oam_data: [0; 256],
            internal_data_buf: 0, // Emulating internal data buffer
            addr: AddrRegister::new(),
            status: StatusRegister::new(),
            ctrl: ControlRegister::new(),
            mirroring: mirroring,
        }
    }

    // 0x2000 write, PPUCTRL(Flags)
    pub fn write_to_ctrl(&mut self, val: u8) {
        // Val should represent the bit flags of the control registers
        self.ctrl.update(val);
    }

    // 0x2002 read, PPUSTATUS
    pub fn read_status(&mut self) -> u8{
        // Flags are read, Vblank and w register should be cleared after read
        let ret = self.status.get_status();
        self.status.clear_vblank();
        ret
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
}
