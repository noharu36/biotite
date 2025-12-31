use crate::file::{self, image, scan, write};
use crate::html::to_html::md_to_html;
use crate::parser::document::ast::MdDocument;
use crate::parser::parse;
use std::path::PathBuf;

pub fn build(dir: &PathBuf, output_dir: &PathBuf) -> Result<(), std::io::Error> {
    let images_dir = output_dir.clone().join("images");
    file::clear(output_dir, &images_dir)?;

    let files = scan::scan_dir(dir);
    println!("Starting analysis of {} files...", files.len());

    let document: Vec<MdDocument> = files
        .iter()
        .filter_map(|path| parse::parse(path).ok())
        .filter(|doc| {
            doc.front_matter
                .as_ref()
                .and_then(|fm| fm.get("publish"))
                .map(|v| v == "true")
                .unwrap_or(false)
                && doc.body.is_some()
        })
        .filter_map(|mut doc| {
            image::copy_document_images(&mut doc, &images_dir)
                .ok()
                .map(|_| doc)
        })
        .collect();

    println!(
        "{} HTML files are being written to the {:#?} directory.",
        document.len(),
        output_dir
    );

    document.iter().map(|doc| md_to_html(doc)).try_for_each(
        |html_doc| -> Result<(), std::io::Error> {
            println!("path: {}", html_doc.path);
            println!("title: {}", html_doc.title);
            println!("tags: {:?}", html_doc.tags);
            write::write(output_dir, html_doc.path.to_string(), html_doc.content)?;

            Ok(())
        },
    )?;

    Ok(())
}
