use core::panic;

use crate::cpu::Mem;
use crate::ppu::PPU;
use crate::rom::Rom;

pub struct Bus {
    cpu_vram: [u8; 2048],
    prg_rom: Vec<u8>,
    ppu: PPU,
    cycles: usize,
}

impl Bus {
    pub fn new(rom: Rom) -> Self {
        let ppu = PPU::new(rom.chr_rom, rom.screen_mirroring);
        Bus {
            cpu_vram: [0; 2048],
            prg_rom: rom.prg_rom,
            ppu: ppu,
            cycles: 0,
        }
    }
    fn read_prg_rom(&self, mut addr: u16) -> u8 {
        addr -= 0x8000;
        if self.prg_rom.len() == 0x4000 && addr >= 0x4000 {
            //mirror 16 kb for addressible space
            addr = addr % 0x4000;
        }
        // println!("chosen addr {:04X} with values {:04X}!", addr, self.prg_rom[addr as usize]);
        self.prg_rom[addr as usize]
    }

    // Counting ticks
    pub fn tick(&mut self, cycles: u8) {
        self.cycles += cycles as usize;
        self.ppu.tick(cycles * 3); //PPU Cycles are 3 times faster than CPU clock cycles
    }
}

const RAM: u16 = 0x0000;
const RAM_MIRRORS_END: u16 = 0x1FFF; // 0x800- 0x1FFF mirrors of 0000-07FF
                                     // const PPU_REGISTERS: u16 = 0x2000; // 0x2000- 0x2007 NES PPU Registers(Communication with PPU)
const PPU_REGISTERS_MIRRORS_END: u16 = 0x3FFF; // Mirrors of above for every 8 bytes
                                               // Cartride ROM and mapper registers
const PROGRAM_RAM: u16 = 0x8000;
const PROGRAM_RAM_END: u16 = 0xFFFF;

impl Mem for Bus {
    // Used for the CPU
    fn mem_read(&mut self, addr: u16) -> u8 {
        match addr {
            RAM..=RAM_MIRRORS_END => {
                let mirror_down_addr = addr & 0b00000111_11111111;
                self.cpu_vram[mirror_down_addr as usize]
            }
            0x2000 | 0x2001 | 0x2003 | 0x2005 | 0x2006 | 0x4014 => {
                panic!("Attempt to read from write-only PPU address {:x}", addr);
            }
            0x2002 => self.ppu.read_status(),
            0x2004 => self.ppu.read_oam_data(),
            0x2007 => self.ppu.read_data(),
            0x2008..=PPU_REGISTERS_MIRRORS_END => {
                // Mirroring down to 0x2000 to 0x2007
                let mirror_down_addr = addr & 0x2007;
                self.mem_read(mirror_down_addr)
            }
            PROGRAM_RAM..=PROGRAM_RAM_END => self.read_prg_rom(addr),
            _ => {
                println!("Ignoring mem access at {}", addr);
                0
            }
        }
    }

    fn mem_write(&mut self, addr: u16, data: u8) {
        match addr {
            RAM..=RAM_MIRRORS_END => {
                let mirror_down_addr = addr & 0b11111111111;
                self.cpu_vram[mirror_down_addr as usize] = data;
            }
            // PPU Control
            0x2000 => {
                self.ppu.write_to_ctrl(data);
            }
            // PPUMask Rendering
            0x2001 => {
                // When writing to this, should mean write the data to this register
                unimplemented!("PPUMASK")
            }
            0x2002 => panic!("Attempt to write to PPU status register PPUSTATUS"),
            0x2003 => self.ppu.write_to_oam_addr(data),
            0x2004 => self.ppu.write_to_oam_data(data),
            0x2005 => self.ppu.write_to_ppuscroll(data),
            0x2006 => {
                self.ppu.write_to_ppu_addr(data);
            }
            0x2007 => {
                self.ppu.write_to_data(data);
            }
            0x2008..=PPU_REGISTERS_MIRRORS_END => {
                let mirror_addr = addr & 0b00100000_00000111;
                self.mem_write(mirror_addr, data);
                // todo!("PPU is not supported yet");
            }
            PROGRAM_RAM..=PROGRAM_RAM_END => {
                panic!("Attempt to write to program RAM space!");
            }
            _ => {
                println!("Ignoring mem write-access at {}", addr);
            }
        }
    }
}
