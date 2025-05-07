use core::panic;

use crate::ppu::PPU;
use crate::frame::Frame;
use crate::palette;

// Renders the palette for a background tile
fn bg_palette(ppu: &PPU, tile_col: usize, tile_row: usize) -> [u8;4]{
    let attr_table_idx = tile_row / 4 * 8 + tile_col / 4;  // 8 columns in attribute table to get index
    let attr_byte = ppu.vram[0x3C0 + attr_table_idx]; // Using nametable 0

    // Extracting pallet index quadrant in 4 x 4 tile
    let pallet_idx = match (tile_col % 4 / 2, tile_row % 4 / 2){
        (0, 0) => attr_byte & 0b11,
        (1, 0) => (attr_byte >> 2) & 0b11,
        (0, 1) => (attr_byte >> 4) & 0b11,
        (1, 1) => (attr_byte >> 6) & 0b11,
        _ => panic!("bg_palette, shouldn't reach here!")
    };
    let pallete_start: usize = 1 + (pallet_idx as usize) * 4; // Take index and determine palette for the quadrant
    // 4 palletes extracted with 1st one being universal
    [ppu.palette_table[0], ppu.palette_table[pallete_start], ppu.palette_table[pallete_start+1], ppu.palette_table[pallete_start+2]]
}

// Renders the sprites for a given pallete index
fn sprite_palette(ppu: &PPU, pallete_idx: u8) -> [u8; 4] {
    let start = 0x11 + (pallete_idx * 4) as usize;
    [
        0,
        ppu.palette_table[start],
        ppu.palette_table[start + 1],
        ppu.palette_table[start + 2],
    ]
}

pub fn render(ppu: &PPU, frame: &mut Frame) {
   let bank = ppu.ctrl.bknd_pattern_addr();


   // Background
   for i in 0..0x03c0 { // just for now, lets use the first nametable
       let tile = ppu.vram[i] as u16;
       let tile_col = i % 32;
       let tile_row = i / 32;
       let tile = &ppu.chr_rom[(bank + tile * 16) as usize..=(bank + tile * 16 + 15) as usize];
       let palette = bg_palette(ppu, tile_col, tile_row);

       for y in 0..=7 {
           let mut upper = tile[y];
           let mut lower = tile[y + 8];

        for x in (0..=7).rev() {
               let value = (1 & lower) << 1 | (1 & upper);
               upper = upper >> 1;
               lower = lower >> 1;
               let rgb = match value {
                   0 => palette::SYSTEM_PALLETE[ppu.palette_table[0] as usize],
                   1 => palette::SYSTEM_PALLETE[palette[1] as usize],
                   2 => palette::SYSTEM_PALLETE[palette[2] as usize],
                   3 => palette::SYSTEM_PALLETE[palette[3] as usize],
                   _ => panic!("can't be"),
               };
               frame.set_pixel(tile_col * 8 + x, tile_row * 8 + y, rgb)
           }
        }
   }

   // Sprites
   // Iterates through the OAM to list up to 64 bytes
    for i in (0..ppu.oam_data.len()).step_by(4).rev() {
       let tile_idx = ppu.oam_data[i + 1] as u16; // Byte 1
       let tile_x = ppu.oam_data[i + 3] as usize; // Byte 3
       let tile_y = ppu.oam_data[i] as usize; //Byte 0

       let flip_vertical = if ppu.oam_data[i + 2] >> 7 & 1 == 1 {
           true
       } else {
           false
       };
       let flip_horizontal = if ppu.oam_data[i + 2] >> 6 & 1 == 1 {
           true
       } else {
           false
       };
       let pallette_idx = ppu.oam_data[i + 2] & 0b11;
       let sprite_palette = sprite_palette(ppu, pallette_idx);
      
       let bank: u16 = ppu.ctrl.sprt_pattern_addr();

       let tile = &ppu.chr_rom[(bank + tile_idx * 16) as usize..=(bank + tile_idx * 16 + 15) as usize];


       for y in 0..=7 {
           let mut upper = tile[y];
           let mut lower = tile[y + 8];
           'ololo: for x in (0..=7).rev() {
               let value = (1 & lower) << 1 | (1 & upper);
               upper = upper >> 1;
               lower = lower >> 1;
               let rgb = match value {
                   0 => continue 'ololo, // skip coloring the pixel
                   1 => palette::SYSTEM_PALLETE[sprite_palette[1] as usize],
                   2 => palette::SYSTEM_PALLETE[sprite_palette[2] as usize],
                   3 => palette::SYSTEM_PALLETE[sprite_palette[3] as usize],
                   _ => panic!("can't be"),
               };
               match (flip_horizontal, flip_vertical) {
                   (false, false) => frame.set_pixel(tile_x + x, tile_y + y, rgb),
                   (true, false) => frame.set_pixel(tile_x + 7 - x, tile_y + y, rgb),
                   (false, true) => frame.set_pixel(tile_x + x, tile_y + 7 - y, rgb),
                   (true, true) => frame.set_pixel(tile_x + 7 - x, tile_y + 7 - y, rgb),
               }
           }
       }
   }
}
