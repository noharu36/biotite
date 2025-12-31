use std::path::PathBuf;
use std::fs::File;
use std::io::Write;

pub fn write(output_dir: &PathBuf, file_name: String, html: String) -> Result<(), std::io::Error> {
    let file_path = file_name + ".html";
    let output_path = output_dir.join(file_path);

    let mut file = File::create(&output_path)?;
    file.write_all(html.as_bytes())?;

    println!("Successfully generated HTML file at: {:?}", output_path);

    Ok(())
}
