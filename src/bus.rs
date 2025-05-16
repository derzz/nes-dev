use core::panic;

use crate::controller::Controller;
use crate::cpu::Mem;
use crate::ppu::PPU;
use crate::rom::Rom;

pub struct Bus<'call> {
    cpu_vram: [u8; 2048],
    prg_rom: Vec<u8>,
    pub ppu: PPU,
    pub cycles: usize, // Contains total amount of cpu cycles
    gameloop_callback: Box<dyn FnMut(&PPU, &mut Controller) + 'call>, // Box, pointer to heap ddata is managed by the box
    controller1: Controller,
}

impl<'a> Bus<'a> {
    pub fn new<'call, F>(rom: Rom, gameloop_callback: F) -> Bus<'call>
    where
        F: FnMut(&PPU, &mut Controller) + 'call,
    {
        let ppu = PPU::new(rom.chr_rom, rom.screen_mirroring);
        Bus {
            cpu_vram: [0; 2048],
            prg_rom: rom.prg_rom,
            ppu: ppu,
            cycles: 7, // Starting with 7 clock cycles
            gameloop_callback: Box::from(gameloop_callback),
            controller1: Controller::new(),
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
        let new_frame = self.ppu.tick(cycles * 3);
        if new_frame {
            (self.gameloop_callback)(&self.ppu, &mut self.controller1);
        }
    }

    // Polling for NMI Interrupt
    pub fn poll_nmi_status(&mut self) -> Option<u8> {
        self.ppu.nmi_interrupt.take()
    }
}

const RAM: u16 = 0x0000;
const RAM_MIRRORS_END: u16 = 0x1FFF; // 0x800- 0x1FFF mirrors of 0000-07FF
                                     // const PPU_REGISTERS: u16 = 0x2000; // 0x2000- 0x2007 NES PPU Registers(Communication with PPU)
const PPU_REGISTERS_MIRRORS_END: u16 = 0x3FFF; // Mirrors of above for every 8 bytes
                                               // Cartride ROM and mapper registers
const PROGRAM_RAM: u16 = 0x8000;
const PROGRAM_RAM_END: u16 = 0xFFFF;

impl Mem for Bus<'_> {
    // Used for the CPU
    fn mem_read(&mut self, addr: u16) -> u8 {
        match addr {
            RAM..=RAM_MIRRORS_END => {
                let mirror_down_addr = addr & 0b00000111_11111111;
                self.cpu_vram[mirror_down_addr as usize]
            }
            0x2000 | 0x2001 | 0x2003 | 0x2005 | 0x2006 | 0x4014 => {
                // panic!("Attempt to read from write-only PPU address {:x}", addr);
                0
            }
            0x2002 => self.ppu.read_status(),
            0x2004 => self.ppu.read_oam_data(),
            0x2007 => self.ppu.read_data(),
            0x2008..=PPU_REGISTERS_MIRRORS_END => {
                // Mirroring down to 0x2000 to 0x2007
                let mirror_down_addr = addr & 0x2007;
                self.mem_read(mirror_down_addr)
            }
            0x4000..=0x4015 => {
                // Ignoring APU
                0
            }

            0x4016 => self.controller1.read(),

            0x4017 => {
                // Controller 2
                0
            }
            PROGRAM_RAM..=PROGRAM_RAM_END => self.read_prg_rom(addr),
            _ => {
                println!("Ignoring mem access at 0x{:4X}", addr);
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
            0x2001 => self.ppu.write_to_mask(data),
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
            }
            PROGRAM_RAM..=PROGRAM_RAM_END => {
                panic!("Attempt to write to program CHR ROM space {:x}!", addr);
            }

            0x4014 => {
                let mut buffer: [u8; 256] = [0; 256];
                let hi: u16 = (data as u16) << 8;
                for i in 0..256u16 {
                    buffer[i as usize] = self.mem_read(hi + i);
                }

                self.ppu.write_oam_dma(&buffer);
            }

            0x4000..=0x4013 | 0x4015 => {
                //ignore APU
            }

            0x4016 => {
                self.controller1.write(data);
            }

            0x4017 => {
                // ignore controller 2
            }
            _ => {
                println!("Ignoring mem write-access at {}", addr);
            }
        }
    }
}
