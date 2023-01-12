use std::any::Any;
use std::io::Write;
use std::time::Duration;
use std::fs::File;

use ocl::{flags, Platform, Device, Context, Queue, CommandQueueProperties, Buffer, Program, Kernel, SpatialDims};
use glam::{Vec3, Quat};
use subprocess::{Exec, Redirection};

use crate::types::ct3d_error::CT3DError;
use crate::types::application_state::ApplicationState;
use crate::types::volume::Volume;
use crate::tools::resources::read_resource_file_as_text;

const INPUT_DATA_BUFFER_SIZE_BYTES: u32 = 1024*1024*1024; // 1 GB of Storage
const DRAG_RADIANS_PER_SCREEN_X: f32=1.0*2.0*(std::f64::consts::PI as f32); // One rotation per half screen
const DRAG_RADIANS_PER_SCREEN_Y: f32=1.0*2.0*(std::f64::consts::PI as f32); // One rotation per half screen
const MIN_CAMERA_Z: f32 = -10.0;
const MAX_CAMERA_Z: f32 = -0.75;
const ZOOM_SPEED: f32 = 0.25;
const LOCAL_SIZE: usize = 512;
const LOW_CUTOFF_CHANGE_SPEED: f32 = 0.010;

pub fn init(application_state: &mut ApplicationState ) -> Result<(), CT3DError>{
    let platform = Platform::default();
    // Query for devices of type GPU
    let devices = Device::list_all(platform).unwrap().into_iter().filter(|device| device.info(ocl::enums::DeviceInfo::Type).unwrap().to_string().to_ascii_lowercase().contains("gpu")).collect::<Vec<Device>>();
            // Filter the list of devices by vendor
    let nvidia_devices = devices
    .into_iter()
    .filter(|device| device.info(ocl::enums::DeviceInfo::Vendor).unwrap().to_string().starts_with("NVIDIA"))
    .collect::<Vec<Device>>();

    if nvidia_devices.len() == 0 {
        panic!("No NVIDIA GPU devices found.");
    }

    let nvidia_device = nvidia_devices[0];

    application_state.opencl_state.device = Some(nvidia_device);

    application_state.opencl_state.context = Some(Context::builder().platform(platform).devices(application_state.opencl_state.device.unwrap()).build().unwrap());

    application_state.opencl_state.queue = Some(Queue::new(
        application_state.opencl_state.context.as_ref().unwrap(), application_state.opencl_state.device.unwrap(), None).unwrap());

    application_state.opencl_state.output_buffer = Some(Buffer::builder()
        .queue(application_state.opencl_state.queue.as_ref().unwrap().clone())
        .flags(ocl::core::MEM_WRITE_ONLY)
        .len(application_state.width*application_state.height*3)
        .build()
        .unwrap()
    );

    application_state.opencl_state.input_data_buffer = Some(Buffer::builder()
        .queue(application_state.opencl_state.queue.as_ref().unwrap().clone())
        .flags(ocl::core::MEM_READ_ONLY)
        .len(INPUT_DATA_BUFFER_SIZE_BYTES/4)
        .build()
        .unwrap()
    );

    // Zero the input data buffer
    let zeros = vec![0.0; (INPUT_DATA_BUFFER_SIZE_BYTES/4) as usize];
    application_state.opencl_state.input_data_buffer.as_ref().unwrap().write(&zeros).enq().unwrap();
    
    application_state.opencl_state.screen_dimensions_buffer = Some(Buffer::builder()
        .queue(application_state.opencl_state.queue.as_ref().unwrap().clone())
        .flags(ocl::core::MEM_READ_ONLY)
        .len(2)
        .build()
        .unwrap()
    );

    application_state.opencl_state.general_parameters_buffer = Some(Buffer::builder()
    .queue(application_state.opencl_state.queue.as_ref().unwrap().clone())
    .flags(ocl::core::MEM_READ_ONLY)
    .len(2) // Remember to update if new parameters are added
    .build()
    .unwrap()
    );

    application_state.opencl_state.axes_buffer = Some(Buffer::builder()
    .queue(application_state.opencl_state.queue.as_ref().unwrap().clone())
    .flags(ocl::core::MEM_READ_ONLY)
    .len(9)
    .build()
    .unwrap()
    );

    // let source_code =
    // crate::kernel_helpers::color::COLOR.to_owned() +
    // &crate::kernel_helpers::math::MATH.to_owned() +
    // &crate::kernel_helpers::raycasting::RAYCASTING.to_owned() +
    // &crate::kernels::render::RENDER.to_owned();

    let source_code = read_resource_file_as_text("kernel_helpers/math.cl".to_owned())? +
    &read_resource_file_as_text("kernel_helpers/raycasting.cl".to_owned())? +
    &read_resource_file_as_text("kernels/render.cl".to_owned())?;

    File::create("debug/kernel_source.cl").unwrap().write(source_code.as_bytes()).unwrap();


    application_state.opencl_state.program = Some(Program::builder()
        .src(source_code.as_str())
        .build(&application_state.opencl_state.context.as_ref().unwrap().clone())
        .unwrap()
    );

    application_state.opencl_state.kernel = Some(Kernel::builder()
        .program(&application_state.opencl_state.program.as_ref().unwrap())
        .queue(application_state.opencl_state.queue.as_ref().unwrap().clone())
        .arg(application_state.opencl_state.screen_dimensions_buffer.as_ref().unwrap())
        .arg(application_state.opencl_state.output_buffer.as_ref().unwrap())
        .arg(application_state.opencl_state.input_data_buffer.as_ref().unwrap())
        .arg(application_state.opencl_state.axes_buffer.as_ref().unwrap())
        .arg(application_state.opencl_state.general_parameters_buffer.as_ref().unwrap())
        .name("render")
        .build()
        .unwrap()
    );

    change_volume(application_state, Box::new(crate::content::generate_initial_volume::generate_initial_volume()));

    Ok(())

}

pub fn change_volume(application_state: &mut ApplicationState, volume: Box<Volume>){
    application_state.volume = Some(volume);
    application_state.volume.as_ref().unwrap().as_ref().to_ocl_buffer(application_state.opencl_state.input_data_buffer.as_mut().unwrap())
}

pub fn main(application_state: &mut ApplicationState, delta_time: Duration) -> Result<(), CT3DError>{

    if *application_state.keymap.get(&sdl2::keyboard::Scancode::A) {
        application_state.low_cutoff = (application_state.low_cutoff - LOW_CUTOFF_CHANGE_SPEED).max(0.0f32);
    }

    if *application_state.keymap.get(&sdl2::keyboard::Scancode::D) {
        application_state.low_cutoff = (application_state.low_cutoff + LOW_CUTOFF_CHANGE_SPEED).min(1.0f32);
    }

    if *application_state.keymap.get(&sdl2::keyboard::Scancode::Q) {
        application_state.low_cutoff = (application_state.low_cutoff - LOW_CUTOFF_CHANGE_SPEED/4.0).max(0.0f32);
    }

    if *application_state.keymap.get(&sdl2::keyboard::Scancode::E) {
        application_state.low_cutoff = (application_state.low_cutoff + LOW_CUTOFF_CHANGE_SPEED/4.0).min(1.0f32);
    }

    let screen_dimensions_vec = vec![application_state.width as i32, application_state.height as i32];

    application_state.opencl_state.screen_dimensions_buffer.as_mut().unwrap().write(&screen_dimensions_vec).enq().unwrap();

    let general_parameters_vec =vec![application_state.camera_z, application_state.low_cutoff];

    application_state.opencl_state.general_parameters_buffer.as_mut().unwrap().write(&general_parameters_vec).enq().unwrap();

    let mut axes_vec = vec![0.0; 9];
    axes_vec[0*3+0] = application_state.RIGHT.x;
    axes_vec[0*3+1] = application_state.RIGHT.y;
    axes_vec[0*3+2] = application_state.RIGHT.z;
    axes_vec[1*3+0] = application_state.UP.x;
    axes_vec[1*3+1] = application_state.UP.y;
    axes_vec[1*3+2] = application_state.UP.z;
    axes_vec[2*3+0] = application_state.FORWARD.x;
    axes_vec[2*3+1] = application_state.FORWARD.y;
    axes_vec[2*3+2] = application_state.FORWARD.z;

    application_state.opencl_state.axes_buffer.as_mut().unwrap().write(&axes_vec).enq().unwrap();

    let work_size = application_state.width*application_state.height;

    unsafe {
        let kernel = application_state.opencl_state.kernel.as_mut().unwrap();
        kernel.set_default_global_work_size(SpatialDims::One(work_size as usize)).set_default_local_work_size(SpatialDims::One(LOCAL_SIZE)).enq().unwrap();
    }

    application_state.screen_buffer.extract_from_buffer(&application_state.opencl_state.output_buffer.as_ref().unwrap())?;

    Ok(())
}

pub fn quit(application_state: &mut ApplicationState ) -> Result<(), CT3DError>{

    Ok(())
}


// Event handlers
pub fn rmb_down(x: i32, y: i32, application_state: &mut ApplicationState) -> Result<(), CT3DError> {

    application_state.drag_state.init_x = x;
    application_state.drag_state.init_y = y;
    application_state.drag_state.init_RIGHT = application_state.RIGHT;
    application_state.drag_state.init_UP = application_state.UP;
    application_state.drag_state.init_FORWARD = application_state.FORWARD;
    application_state.drag_state.dragging = true;

    Ok(())
}
pub fn rmb_up(x: i32, y: i32, application_state: &mut ApplicationState) -> Result<(), CT3DError> {

    application_state.drag_state.dragging = false;

    Ok(())
}
pub fn mouse_move(x: i32, y: i32, application_state: &mut ApplicationState) -> Result<(), CT3DError> {

    if(application_state.drag_state.dragging){
        
        let dx = x - application_state.drag_state.init_x;
        let dy = y - application_state.drag_state.init_y;
        // application_state.drag_state.init_x = x;
        // application_state.drag_state.init_y = y;
    
        let fraction_dx = (dx as f32) /(application_state.width as f32);
        let fraction_dy = (dy as f32) /(application_state.height as f32);
    
        let mut dx_radians = DRAG_RADIANS_PER_SCREEN_X * fraction_dx * 2.0;
        let mut dy_radians = DRAG_RADIANS_PER_SCREEN_Y * fraction_dy * 2.0;
    
        // change dx_radians and dy_radians as necessary to make sure it makes sense to user
        // account for the fact that this application uses an LHS coord system
        dx_radians *= -1.0;
        dy_radians *= -1.0;

        let dx_rotation_matrix = Quat::from_axis_angle(application_state.drag_state.init_UP, dx_radians);
        let dy_rotation_matrix = Quat::from_axis_angle(Vec3::new(1.0,0.0,0.0),dy_radians);
        
        let rotation_matrix = dy_rotation_matrix * dx_rotation_matrix;
        
        application_state.RIGHT = rotation_matrix * application_state.drag_state.init_RIGHT;
        application_state.UP = rotation_matrix * application_state.drag_state.init_UP;
        application_state.FORWARD = rotation_matrix * application_state.drag_state.init_FORWARD;
    }

    Ok(())
}

pub fn wheel(delta: i32, application_state: &mut ApplicationState) -> Result<(), CT3DError> {

    let old_camera_z = application_state.camera_z;
    let mut new_camera_z = old_camera_z + (delta.signum() as f32) * ZOOM_SPEED;
    new_camera_z = new_camera_z.max(MIN_CAMERA_Z).min(MAX_CAMERA_Z);
    application_state.camera_z = new_camera_z;
    
    Ok(())
}

pub fn drop_file(filename: String, application_state: &mut ApplicationState) -> Result<(), CT3DError> {

    println!("{}", filename);

    let capture_result = Exec::cmd("python").arg("ct3d3-python/dicom_to_volume.py").arg("--dropped-file").arg(filename).capture();

    match capture_result {
        Ok(captured_data) =>{
            if(subprocess::ExitStatus::success(captured_data.exit_status)){
                println!("Python subprocess run successfully.");
                change_volume(application_state, Box::new(crate::content::generate_initial_volume::generate_initial_volume()));
            }else{
                println!("Python subprocess exited with non-zero code.");
            }
        }
        Err(e)=>{
            println!("Python could not be started or run successfully.");
            println!("{}", e);
        }
    }

    Ok(())

}

pub fn key_down(scancode: Option<sdl2::keyboard::Scancode>, application_state: &mut ApplicationState) -> Result<(), CT3DError> {

    if let Some(scancode) = scancode {
        application_state.keymap.insert(scancode, true);
    };

    Ok(())

}
pub fn key_up(scancode: Option<sdl2::keyboard::Scancode>, application_state: &mut ApplicationState) -> Result<(), CT3DError> {

    if let Some(scancode) = scancode {
        application_state.keymap.insert(scancode, false);
    };

    Ok(())

}