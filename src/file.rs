pub mod image;
pub mod scan;
pub mod write;

use std::fs;
use std::path::PathBuf;

pub fn clear(output_dir: &PathBuf, image_dir: &PathBuf) -> Result<(), std::io::Error> {
    if output_dir.exists() {
        fs::remove_dir_all(output_dir)?;
    }

    if image_dir.exists() {
        fs::remove_dir_all(image_dir)?;
    }

    fs::create_dir_all(output_dir)?;
    fs::create_dir_all(image_dir)?;

    Ok(())
}
