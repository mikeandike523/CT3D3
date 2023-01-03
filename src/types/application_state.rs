use ocl::{flags, Platform, Device, Context, Queue, Buffer, Program, Kernel};
use glam::{Vec3};

use crate::types::rgb_image::RGBImage;

pub struct DragState {
    pub dragging: bool,

    // todo: change "init" to last
    pub init_x: i32,
    pub init_y: i32,
    pub init_RIGHT: Vec3,
    pub init_UP: Vec3,
    pub init_FORWARD: Vec3
}
pub struct OpenCLState {
    pub device: Option<Device>,
    pub context: Option<Context>,
    pub queue: Option<Queue>,
    pub screen_dimensions_buffer: Option<Buffer<i32>>,
    pub output_buffer: Option<Buffer<f32>>,
    pub input_data_buffer: Option<Buffer<f32>>,
    pub axes_buffer: Option<Buffer<f32>>,
    pub program: Option<Program>,
    pub kernel: Option<Kernel>
}

pub struct ApplicationState {
    pub width: u32,
    pub height: u32,
    pub screen_buffer: RGBImage,
    pub opencl_state: OpenCLState,
    pub RIGHT: Vec3,
    pub UP: Vec3,
    pub FORWARD: Vec3,
    pub drag_state: DragState
}

impl DragState {
    pub fn new() -> DragState {
        DragState {
            dragging: false,
            init_x: 0,
            init_y: 0,
            init_RIGHT: Vec3::new(1.0, 0.0, 0.0),
            init_UP: Vec3::new(0.0, 1.0, 0.0),
            init_FORWARD: Vec3::new(0.0, 0.0, 1.0)
        }
    }
}

impl OpenCLState {
    pub fn new() -> Self {
        Self {
            device: None,
            context: None,
            queue: None,
            screen_dimensions_buffer: None,
            output_buffer: None,
            input_data_buffer: None,
            axes_buffer: None,
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
            opencl_state: OpenCLState::new(),
            RIGHT: Vec3::new(1.0, 0.0, 0.0),
            UP: Vec3::new(0.0, 1.0, 0.0),
            FORWARD: Vec3::new(0.0, 0.0, 1.0),
            drag_state: DragState::new()
        }
    }
}