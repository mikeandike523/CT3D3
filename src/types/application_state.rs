use ocl::{flags, Platform, Device, Context, Queue, Buffer, Program, Kernel};

use crate::types::rgb_image::RGBImage;

pub struct OpenCLState {
    pub device: Option<Device>,
    pub context: Option<Context>,
    pub queue: Option<Queue>,
    pub screen_dimensions_buffer: Option<Buffer<i32>>,
    pub output_buffer: Option<Buffer<f32>>,
    pub program: Option<Program>,
    pub kernel: Option<Kernel>
}

pub struct ApplicationState {
    pub width: u32,
    pub height: u32,
    pub screen_buffer: RGBImage,
    pub opencl_state: OpenCLState
}

impl OpenCLState {
    pub fn new() -> Self {
        Self {
            device: None,
            context: None,
            queue: None,
            screen_dimensions_buffer: None,
            output_buffer: None,
            program: None,
            kernel: None
        }
    }
}

impl ApplicationState {
    pub fn new(width: u32, height:u32) -> ApplicationState {
        ApplicationState {
            width: width,
            height: height,
            screen_buffer: RGBImage::new(width as usize, height as usize),
            opencl_state: OpenCLState::new()
        }
    }
}