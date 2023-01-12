use std::env;
use std::path;
use std::path::Path;
use std::fs;

fn copy_directory(src: &Path, dst: &Path) -> std::io::Result<()> {
    if src.is_dir() {
        fs::create_dir_all(dst)?;
        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                copy_directory(&path, &dst.join(entry.file_name()))?;
            } else {
                fs::copy(&path, &dst.join(entry.file_name()))?;
            }
        }
    } else {
        fs::copy(src, dst)?;
    }
    Ok(())
}

fn main() {

    let targets: [&str; 2] = ["debug", "release"];

    for target in targets.iter() {
        let PROJECT_ROOT = env::var("CARGO_MANIFEST_DIR").unwrap();
        let CARGO_TARGET_DIR = Path::new(&PROJECT_ROOT.clone()).join("target").join(String::from(*target)).to_str().unwrap().to_owned();
        println!("{}", CARGO_TARGET_DIR);
        let kernels_path = Path::new("").join(PROJECT_ROOT.clone()).join("src").join("kernels");
        let kernel_helpers_path = Path::new("").join(PROJECT_ROOT.clone()).join("src").join("kernel_helpers");
        let output_kernels_path = Path::new("").join(CARGO_TARGET_DIR.clone()).join("kernels");
        let output_kernel_helpers_path = Path::new("").join(CARGO_TARGET_DIR.clone()).join("kernel_helpers");
        copy_directory(&kernels_path, &output_kernels_path).unwrap();
        copy_directory(&kernel_helpers_path, &output_kernel_helpers_path).unwrap();
    }

}