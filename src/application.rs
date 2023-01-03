use std::any::Any;
use std::time::Duration;

use ocl::{flags, Platform, Device, Context, Queue, CommandQueueProperties, Buffer, Program, Kernel, SpatialDims};

use crate::types::ct3d_error::CT3DError;
use crate::types::application_state::ApplicationState;

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

    
    application_state.opencl_state.screen_dimensions_buffer = Some(Buffer::builder()
        .queue(application_state.opencl_state.queue.as_ref().unwrap().clone())
        .flags(ocl::core::MEM_READ_ONLY)
        .len(2)
        .build()
        .unwrap()
    );


    let source_code = crate::kernels::render::RENDER;

    application_state.opencl_state.program = Some(Program::builder()
        .src(source_code)
        .build(&application_state.opencl_state.context.as_ref().unwrap().clone())
        .unwrap()
    );

    application_state.opencl_state.kernel = Some(Kernel::builder()
        .program(&application_state.opencl_state.program.as_ref().unwrap())
        .queue(application_state.opencl_state.queue.as_ref().unwrap().clone())
        .arg(application_state.opencl_state.screen_dimensions_buffer.as_ref().unwrap())
        .arg(application_state.opencl_state.output_buffer.as_ref().unwrap())
        .name("render")
        .build()
        .unwrap()
    );

    Ok(())

}

pub fn main(application_state: &mut ApplicationState, delta_time: Duration) -> Result<(), CT3DError>{

    let screen_dimensions_vec = vec![application_state.width as i32, application_state.height as i32];

    application_state.opencl_state.screen_dimensions_buffer.as_mut().unwrap().write(&screen_dimensions_vec).enq().unwrap();

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