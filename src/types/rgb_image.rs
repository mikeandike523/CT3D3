use sdl2::render::Texture;

use ocl::{Buffer, Queue};

use super::ct3d_error::CT3DError;

pub struct RGBImage {
    width: usize,
    height: usize,
    pixel_data: Vec<u8>,
}

impl RGBImage {
    pub fn new(width: usize, height: usize) -> RGBImage {
        let num_pixels = width * height;
        let mut pixel_data = vec![0u8; num_pixels * 3];

        RGBImage { width, height, pixel_data }
    }

    pub fn get_pixel(&self, x: usize, y: usize) -> (u8, u8, u8) {
        let index = (y * self.width + x) * 3;
        (self.pixel_data[index], self.pixel_data[index + 1], self.pixel_data[index + 2])
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, value: (u8, u8, u8)) {
        let index = (y * self.width + x) * 3;
        self.pixel_data[index] = value.0;
        self.pixel_data[index + 1] = value.1;
        self.pixel_data[index + 2] = value.2;
    }


    // What the hell is going on here? Is it sld2 or my code that is causing this function to need to be so wonky?
    pub fn copy_to_texture(&self, texture: &mut Texture){
        texture.with_lock(sdl2::rect::Rect::new(0,0,self.width as u32, self.height as u32), |buffer: &mut [u8], pitch: usize| {
            for x in 0..self.width {
                for y in 0..self.height {
                    let pixel = self.get_pixel(x, y);
                    let tid = (y*self.width + x);
                    buffer[(tid*4)+2] =  pixel.0;
                    buffer[(tid*4)+1] =  pixel.1;
                    buffer[(tid*4)+0] =  pixel.2;
                    buffer[(tid*4)+3] =  255;
 
                }
            }
        }).unwrap();
    }

    pub fn extract_from_buffer(&mut self, buffer: &Buffer<f32>) -> Result<(), CT3DError>{
        let mut data = vec![0.0f32; self.width*self.height*3];
        buffer.read(&mut data).enq().unwrap();
        for x in 0..self.width {
            for y in 0..self.height {
                for channel in 0..3 {
                    self.pixel_data[((y*self.width+x)*3)+channel] = (255.0 * data[((y*self.width+x)*3)+channel].max(0.0).min(1.0)) as u8;
                }
            }
        }

        Ok(())
    }

}