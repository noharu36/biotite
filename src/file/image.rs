use crate::parser::document::ast::{Block, Inline, ListType, MdDocument};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

pub fn copy_document_images(
    doc: &mut MdDocument,
    image_dir: &PathBuf,
) -> Result<(), std::io::Error> {
    let base_dir = doc.path.parent().unwrap_or(Path::new("."));

    for block in &mut doc.body.as_mut().unwrap().blocks {
        process_block_images(block, base_dir, image_dir)?;
    }

    Ok(())
}

fn process_block_images(
    block: &mut Block,
    base_dir: &Path,
    image_dir: &PathBuf,
) -> Result<(), std::io::Error> {
    match block {
        Block::Paragraph(inlines)
        | Block::Heading {
            content: inlines, ..
        } => process_inlines(inlines, base_dir, image_dir),
        Block::Blockquote(blocks) => {
            for b in blocks {
                process_block_images(b, base_dir, image_dir)?;
            }

            Ok(())
        }
        Block::List(list_type) => {
            let items = match list_type {
                ListType::Ordered(items) => items,
                ListType::Unordered(items) => items,
            };
            for item in items {
                process_inlines(&mut item.content, base_dir, image_dir)?;
            }

            Ok(())
        }
        _ => Ok(()),
    }
}

fn process_inlines(
    inlines: &mut Vec<Inline>,
    base_dir: &Path,
    image_dir: &PathBuf,
) -> Result<(), std::io::Error> {
    for inline in inlines {
        match inline {
            Inline::Image { url, .. } => {
                handle_image_copy(url, base_dir, image_dir)?;
            }
            Inline::Strong(children)
            | Inline::Italic(children)
            | Inline::Strikethrough(children)
            | Inline::Link { text: children, .. } => {
                process_inlines(children, base_dir, image_dir)?;
            }
            _ => {}
        }
    }

    Ok(())
}

fn handle_image_copy(
    url: &mut String,
    base_dir: &Path,
    image_dir: &PathBuf,
) -> Result<(), std::io::Error> {
    if url.starts_with("http://") || url.starts_with("https://") {
        return Ok(());
    }

    let source_path = if url.starts_with("/") {
        Path::new(url).to_path_buf()
    } else if url.starts_with("~/") {
        let home = env::var("HOME").unwrap_or_else(|_| ".".to_string());
        Path::new(&home).join(&url[2..])
    } else {
        base_dir.join(&url)
    };

    if source_path.exists() {
        if let Some(file_name) = source_path.file_name() {
            let copied_path = image_dir.join(file_name);

            fs::copy(&source_path, &copied_path)?;

            let new_url = format!("/images/{}", file_name.to_string_lossy());
            *url = new_url;
        }
    } else {
        println!("Warning: Image not found at {:?}", source_path);
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Warning: Image not found at {:?}", source_path),
        ));
    }

    Ok(())
}
