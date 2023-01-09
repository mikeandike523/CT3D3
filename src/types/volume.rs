use std::fs::File;
use std::io::{Write, BufRead, BufReader, Read};

use glam::{Vec3, IVec3};
use ocl::Buffer;


use crate::types::ct3d_error::CT3DError;

pub struct Volume {
    pub radii: Vec3, 
    pub res: IVec3,
    pub data: Vec<f32>
}

pub fn text_to_Vec3(text: String) -> Result<Vec3, CT3DError>{
    
    let mut data_vec: Vec<f32> = Vec::<f32>::new();

    for value in text.split_whitespace().into_iter() {
        data_vec.push(value.parse::<f32>().map_err(
            |e| CT3DError::new(Some(Box::new(e)))
        )?);
    }

    if data_vec.len() != 3 {
        return Err(CT3DError::new(None));
    }

    Ok(Vec3::new(data_vec[0], data_vec[1],data_vec[2]))

}

pub fn text_to_IVec3(text: String) -> Result<IVec3, CT3DError>{
    
    let mut data_vec: Vec<i32> = Vec::<i32>::new();

    for value in text.split_whitespace().into_iter() {
        data_vec.push(value.parse::<i32>().map_err(
            |e| CT3DError::new(Some(Box::new(e)))
        )?);
    }

    if data_vec.len() != 3 {
        return Err(CT3DError::new(None));
    }

    Ok(IVec3::new(data_vec[0], data_vec[1],data_vec[2]))

}

impl Volume {
    pub fn new(radii: Vec3, res: IVec3) -> Self {
        Self {
            radii:radii,
            res:res,
            data: vec![0.0;(res.x*res.y*res.z).try_into().unwrap()]
        }
    }
    pub fn set(&mut self, coord: IVec3, value:f32){
        let idx = coord.z*self.res.x*self.res.y + coord.y*self.res.x + coord.x;
        self.data[idx as usize] = value;
    }
    pub fn get(&self, coord: IVec3) -> f32{
        let idx = coord.z*self.res.x*self.res.y + coord.y*self.res.x + coord.x;
        return self.data[idx as usize];
    }
    pub fn to_ocl_buffer(&self, buffer: &mut Buffer<f32>){
        let mut data = Vec::<f32>::new();
        data.push(1.0f32);
        data.push(self.radii.x);
        data.push(self.radii.y);
        data.push(self.radii.z);
        data.push(self.res.x as f32);
        data.push(self.res.y as f32);
        data.push(self.res.z as f32);
        data.extend(self.data.clone());
        buffer.write(&data).enq().unwrap();
    }
    pub fn normalize(&mut self){
        let min = *self.data.iter().min_by(|x, y| x.partial_cmp(y).unwrap()).unwrap();
        let max = *self.data.iter().max_by(|x, y| x.partial_cmp(y).unwrap()).unwrap();
        self.data.iter_mut().for_each(|x| {
            *x = (*x-min)/(max-min);
        });
    }

    pub fn serialize_to_file(&self, path: String) -> Result<(), CT3DError> {
        
        let mut file = File::create(path).map_err(|e| CT3DError::new(Some(Box::new(e))))?;
        
        let line1: String = format!("{} {} {}\n", self.radii.x, self.radii.y, self.radii.z);
        let line2: String = format!("{} {} {}\n", self.res.x, self.res.y, self.res.z);

        file.write_all(line1.as_bytes()).map_err(|e| CT3DError::new(Some(Box::new(e))))?;

        file.write_all(line2.as_bytes()).map_err(|e| CT3DError::new(Some(Box::new(e))))?;

        for value in self.data.iter() {
            let bytes = (*value).to_ne_bytes();
            file.write_all(&bytes).map_err(|e| CT3DError::new(Some(Box::new(e))))?;
        }

        Ok(())

    }
    pub fn deserialize_from_file(path: String) -> Result<Volume,CT3DError> {
        let file = File::open(path).map_err(|e| CT3DError::new(Some(Box::new(e))))?;
        let mut reader = BufReader::new(file);

        let mut line1: String = String::new();

        reader.read_line(&mut line1).map_err(|e| CT3DError::new(Some(Box::new(e))))?;

        let mut line2: String = String::new(); 
        reader.read_line(&mut line2).map_err(|e| CT3DError::new(Some(Box::new(e))))?;

        let radii = text_to_Vec3(line1)?;
        let res = text_to_IVec3(line2)?;

        let mut data_bytes: Vec<u8> = Vec::<u8>::new();
        reader.read_to_end(&mut data_bytes)?;
        
        if data_bytes.len() % 4 != 0 {
            return Err(CT3DError::new(None));
        }

        let mut data = Vec::<f32>::new();

        for chunk in data_bytes.chunks(4) {
            let mut chunk_owned = [0u8;4];
            chunk_owned.clone_from_slice(chunk);
            let value = f32::from_ne_bytes(chunk_owned);
            data.push(value);

        }

        let mut result = Volume::new(radii, res);

        result.data = data;

        Ok(result)
    }

}