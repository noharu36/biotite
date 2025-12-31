use std::{fs, path::PathBuf};
use super::document::{parse_document, front_matter};
use super::document::ast::MdDocument;

pub fn parse<'a>(path: &'a PathBuf) -> Result<MdDocument<'a>, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    let (front_matter, markdown_body) = front_matter::parse_front_matter(&content);
    let parser = parse_document();
    if let Some((doc, _rest)) = parser(markdown_body) {
        Ok(MdDocument {
            path,
            front_matter,
            body: Some(doc)
        })
    } else {
        println!("Markdown本文の解析に失敗しました at: {:?}", path);
        Ok(MdDocument {
            path,
            front_matter,
            body: None
        })
    }
}
