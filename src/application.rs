use std::any::Any;
use std::io::Write;
use std::time::Duration;
use std::fs::File;

use ocl::{flags, Platform, Device, Context, Queue, CommandQueueProperties, Buffer, Program, Kernel, SpatialDims};

use crate::types::ct3d_error::CT3DError;
use crate::types::application_state::ApplicationState;

const INPUT_DATA_BUFFER_SIZE_BYTES: u32 = 1024*1024*512; // 0.5 GB of Storage
const DRAG_RADIANS_PER_SCREEN_X: f32=2.0*2.0*(std::f64::consts::PI as f32);
const DRAG_RADIANS_PER_SCREEN_Y: f32=2.0*2.0*(std::f64::consts::PI as f32);
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

    application_state.opencl_state.axes_buffer = Some(Buffer::builder()
    .queue(application_state.opencl_state.queue.as_ref().unwrap().clone())
    .flags(ocl::core::MEM_READ_ONLY)
    .len(9)
    .build()
    .unwrap()
    );

    let source_code =
    crate::kernel_helpers::color::COLOR.to_owned() +
    &crate::kernel_helpers::math::MATH.to_owned() +
    &crate::kernels::render::RENDER.to_owned();

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
        .name("render")
        .build()
        .unwrap()
    );

    Ok(())

}

pub fn main(application_state: &mut ApplicationState, delta_time: Duration) -> Result<(), CT3DError>{

    let screen_dimensions_vec = vec![application_state.width as i32, application_state.height as i32];

    application_state.opencl_state.screen_dimensions_buffer.as_mut().unwrap().write(&screen_dimensions_vec).enq().unwrap();

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
        kernel.set_default_global_work_size(SpatialDims::One(work_size as usize)).set_default_local_work_size(SpatialDims::One(1usize)).enq().unwrap();
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
        
    }

    Ok(())
}