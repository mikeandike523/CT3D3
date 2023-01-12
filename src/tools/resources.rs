use std::env;
use std::path::Path;
use std::fs::File;
use std::io::Read;

use crate::types::ct3d_error::CT3DError;

pub fn get_resource_path(relative_path: String) -> Result<String, CT3DError> {
    let mut exe_dir = env::current_exe()?;
    exe_dir.pop();
    let exe_dir_path = exe_dir.to_str().unwrap().to_owned();
    let resource_path = Path::new(&exe_dir_path).join(relative_path.clone());
    Ok(resource_path.to_str().unwrap().to_owned())
}

pub fn read_resource_file_as_text(relative_path: String) -> Result<String, CT3DError> {
    let resource_path = get_resource_path(relative_path)?;
    let mut file = File::open(&resource_path)?;
    let mut file_contents = String::new();
    file.read_to_string(&mut file_contents)?;
    Ok(file_contents)
}