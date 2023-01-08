use noise::{Perlin};
use noise::NoiseFn;
use glam::{Vec3, IVec3};
use crate::types::volume::Volume;
use std::fs;

///! Generate a volume to test the basic rendering with
///!Use perline noise
pub fn construct_initial_volume() -> Volume {
    
    println!("Generating volume...");

    let perlin = Perlin::new(1999);
    let octaves: usize = 3;
    let scale = 0.75;
    let persistence:f64 = 0.90;
    let mut volume = Volume::new(
        Vec3::new(0.75,0.75,0.75),
        IVec3::new(128,128,128)
    );

    for x in 0..128 {
        for y in 0..128 {
            for z in 0..128 { 

                let mut total_noise: f64 = 0.0;

                let nx: f64 = (x as f64)/128.0*scale;
                let ny: f64 = (y as f64)/128.0*scale;
                let nz: f64 = (z as f64)/128.0*scale;

                for octave in 0..octaves{
                    let noise = perlin.get([nx*(octave as f64), ny*(octave as f64), nz*(octave as f64)]);
                    total_noise += noise*persistence.powf(octave as f64);
                }

                volume.set(IVec3::new(x,y,z),total_noise as f32);

            }}}


    volume.normalize();

    println!("Done.");

    volume
}

pub fn generate_initial_volume() -> Volume {
    let meta = fs::metadata("temp/initial_volume.txt");
    match meta {
        Ok(_) => {
            println!("Loading pre-generatedd initial volume...");
            let volume = Volume::deserialize_from_file("temp/initial_volume.txt".to_owned()).unwrap();
            print!("Done!");
            volume
        },
        Err(_) => {
            println!("Regenerating initial volume...");
            let result = construct_initial_volume();
            result.serialize_to_file("temp/initial_volume.txt".to_owned()).unwrap();
            println!("Done!");
            result
        }
    }

}