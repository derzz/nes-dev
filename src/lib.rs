pub mod bus;
pub mod cpu;
pub mod op;
pub mod ppu;
pub mod ppu_reg;
pub mod rom;
pub mod trace;
pub mod frame;
pub mod palette;
pub mod render;
pub mod controller;

use cpu::*;

use rom::Rom;
use sdl2::{event::Event, keyboard::Keycode, pixels::PixelFormatEnum};
use trace::trace;
use frame::Frame;
use ppu::PPU;

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


// Used with NES test, removing for now due to polishing state of ppu and controllers
// pub fn run_nestest_and_capture() -> Vec<String> {
//     let sdl_context = sdl2::init().unwrap();
//     let video_subsystem = sdl_context.video().unwrap();
//     let window = video_subsystem
//         .window("Snake game", (32.0 * 10.0) as u32, (32.0 * 10.0) as u32)
//         .position_centered()
//         .build()
//         .unwrap();

//     let mut canvas = window.into_canvas().present_vsync().build().unwrap();
//     let mut event_pump = sdl_context.event_pump().unwrap();
//     canvas.set_scale(3.0, 3.0).unwrap();

//     let creator = canvas.texture_creator();
//     let mut texture = creator
//         .create_texture_target(PixelFormatEnum::RGB24, 32, 32)
//         .unwrap();
//     canvas.set_scale(3.0, 3.0).unwrap();
//     let game_bytes = std::fs::read("nestest.nes").unwrap();
//     let rom = Rom::new(&game_bytes).unwrap();
//     let mut frame = Frame::new();
//     let bus = Bus::new(rom, move |ppu:&PPU|{
//         render::render(ppu, &mut frame);
//         texture.update(None, &frame.data, 265 * 3).unwrap();

//         canvas.copy(&texture, None, None).unwrap();

//         canvas.present();
//        for event in event_pump.poll_iter() {
//            match event {
//              Event::Quit { .. }
//              | Event::KeyDown {
//                  keycode: Some(Keycode::Escape),
//                  ..
//              } => std::process::exit(0),
//              _ => { /* do nothing */ }
//            }
//         }
//     });
//     let mut cpu = CPU::new(bus);
//     cpu.reset();
//     let mut output = Vec::new();
//     cpu.run_with_callback(|cpu| {
//         output.push(trace(cpu));
//     });
//     output
// }
