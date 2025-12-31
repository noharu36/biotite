use std::{collections::HashMap, path::PathBuf};

#[derive(Debug, Clone, PartialEq)]
pub struct MdDocument<'a> {
    pub path: &'a PathBuf,
    pub front_matter: Option<HashMap<String, String>>,
    pub body: Option<Document>
}

#[derive(Debug, Clone, PartialEq)]
pub struct Document {
    pub blocks: Vec<Block>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Block {
    Heading {
        level: u8, // 1 to 6
        content: Vec<Inline>,
    },
    Paragraph(Vec<Inline>),
    Blockquote(Vec<Block>),
    List(ListType),
    FencedCodeBlock {
        language: Option<String>,
        code: String,
    },
    HorizontalRule,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ListType {
    Unordered(Vec<ListItem>),
    Ordered(Vec<ListItem>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ListItem {
    pub indent: usize,
    pub checked: Option<bool>,
    pub content: Vec<Inline>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Inline {
    Text(String),
    Link {
        text: Vec<Inline>,
        url: String,
    },
    Image {
        alt: String,
        url: String,
    },
    Strong(Vec<Inline>),
    Italic(Vec<Inline>),
    Strikethrough(Vec<Inline>),
    Code(String),
}
