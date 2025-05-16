use nes::controller::{self, ControllerButton};
use nes::cpu::*;
use std::collections::HashMap;

use nes::frame::Frame;
use nes::ppu::PPU;
use nes::render;
use nes::rom::Rom;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
// use nes::trace::trace;

use nes::bus::Bus;

fn main() {
    // Setting up screen and scaling
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window("NEXie", (256.0 * 3.0) as u32, (240.0 * 3.0) as u32)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().present_vsync().build().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();
    canvas.set_scale(3.0, 3.0).unwrap();

    let creator = canvas.texture_creator();
    let mut texture = creator
        .create_texture_target(PixelFormatEnum::RGB24, 256, 240)
        .unwrap();
    canvas.set_scale(3.0, 3.0).unwrap();

    // Note: Reading from the right file, why is it starting on C004?
    // let log_file = std::fs::File::create("debug.log").unwrap();
    // env_logger::Builder::new()
    //     .target(env_logger::Target::Pipe(Box::new(log_file)))
    //     .filter_level(log::LevelFilter::Debug)
    //     .init();

    // Setting up controllers
    let mut input_map = HashMap::new();
    input_map.insert(Keycode::Down, ControllerButton::DOWN);
    input_map.insert(Keycode::Up, ControllerButton::UP);
    input_map.insert(Keycode::Right, ControllerButton::RIGHT);
    input_map.insert(Keycode::Left, ControllerButton::LEFT);
    input_map.insert(Keycode::X, ControllerButton::B);
    input_map.insert(Keycode::Z, ControllerButton::A);
    input_map.insert(Keycode::Return, ControllerButton::START);
    input_map.insert(Keycode::Space, ControllerButton::SELECT);

    // Game loading and CPU setup
    let game_bytes = std::fs::read("PATH GOES HERE").unwrap();
    let rom = Rom::new(&game_bytes).unwrap();

    let mut frame = Frame::new();

    let bus = Bus::new(
        rom,
        move |ppu: &PPU, controller: &mut controller::Controller| {
            render::render(ppu, &mut frame);
            texture.update(None, &frame.data, 256 * 3).unwrap();

            canvas.copy(&texture, None, None).unwrap();

            canvas.present();
            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit { .. }
                    | Event::KeyDown {
                        keycode: Some(Keycode::Escape),
                        ..
                    } => std::process::exit(0),
                    Event::KeyDown { keycode, .. } => {
                        if let Some(key) = input_map.get(&keycode.unwrap_or(Keycode::Ampersand)) {
                            // println!("Pressed button!");
                            controller.set_button_pressed_status(*key, true);
                        }
                    }
                    Event::KeyUp { keycode, .. } => {
                        if let Some(key) = input_map.get(&keycode.unwrap_or(Keycode::Ampersand)) {
                            // println!("Released button!");
                            controller.set_button_pressed_status(*key, false);
                        }
                    }

                    _ => { /* do nothing */ }
                }
            }
        },
    );
    let mut cpu = CPU::new(bus);
    cpu.reset();
    // let mut screen_state = [0 as u8; 32 * 3 * 32];
    // let mut rng = rand::thread_rng();
    // cpu.run_with_callback(move |cpu| {
    //     println!("{}", trace(cpu));
    // });
    cpu.run();
}
