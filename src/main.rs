use std::time::{Duration, Instant};

use subprocess::{Exec, Redirection};

use sdl2::mouse::MouseButton;
use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

use crate::types::rgb_image::RGBImage;
use crate::types::application_state::ApplicationState;

mod types {
    pub mod rgb_image;
    pub mod ct3d_error;
    pub mod application_state;
    pub mod volume;
}

mod tools {
    pub mod resources;
}

mod application;

mod content {
    pub mod generate_initial_volume;
}

const SCREEN_WIDTH: u32 = 640;
const SCREEN_HEIGHT: u32 = 640;

fn main() {
    // Initialize SDL2
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    // Create the window
    let window = video_subsystem.window("CT3D", SCREEN_WIDTH, SCREEN_HEIGHT)
        .position_centered()
        .build()
        .unwrap();

    // Create a canvas for the screen
    let mut canvas = window.into_canvas().build().unwrap();

    // Set the background color to white
    canvas.set_draw_color(Color::RGB(255, 255, 255));
    canvas.clear();
    canvas.present();

    // Create the event pump
    let mut event_pump = sdl_context.event_pump().unwrap();
    
    let texture_creator = canvas.texture_creator();

    let mut screen_texture = texture_creator.create_texture_streaming(PixelFormatEnum::RGB888,SCREEN_WIDTH,SCREEN_HEIGHT).unwrap();

    // Not exactly just the app state. Also is the owner of any variables that need to passed around by reference
    let mut application_state = ApplicationState::new(SCREEN_WIDTH, SCREEN_HEIGHT);

    crate::application::init(&mut application_state).unwrap(); 

    // Initialize the previous frame time to the current time
    let mut prev_frame_time = Instant::now();

    // Run the event loop
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                Event::MouseButtonDown {
                    mouse_btn: MouseButton::Right,
                    x,
                    y,
                    ..
                } =>{
                    crate::application::rmb_down(x, y, &mut application_state).unwrap();
                },
                Event::MouseButtonUp {
                    mouse_btn: MouseButton::Right,
                    x,
                    y,
                    ..
                } =>{
                    crate::application::rmb_up(x, y, &mut application_state).unwrap();
                }
                Event::MouseMotion {
                    x,
                    y,
                    ..
                } => {
                    crate::application::mouse_move(x, y, &mut application_state).unwrap();
                },
                Event::MouseWheel {y, ..} => {
                    crate::application::wheel(y, &mut application_state).unwrap();
                },
                Event::DropFile {filename, ..} =>{
                    crate::application::drop_file(filename, &mut application_state).unwrap();
                },
                Event::KeyDown { timestamp, window_id, keycode, scancode, keymod, repeat } => {
                    crate::application::key_down(scancode, &mut application_state).unwrap();
                },
                Event::KeyUp { timestamp, window_id, keycode, scancode, keymod, repeat } => {
                    crate::application::key_up(scancode, &mut application_state).unwrap();
                },
                _ => {}
            }
        }

        // Measure the delta time by subtracting the previous frame time from the current time
        let delta_time = Instant::now() - prev_frame_time;
        prev_frame_time = Instant::now();

        crate::application::main(&mut application_state, delta_time).unwrap(); 

        application_state.screen_buffer.copy_to_texture(&mut screen_texture);

        canvas.copy(&screen_texture, sdl2::rect::Rect::new(0,0,application_state.width, application_state.height), sdl2::rect::Rect::new(0,0,application_state.width, application_state.height)).unwrap();

        canvas.present();
    }

    crate::application::quit(&mut application_state).unwrap(); 

}