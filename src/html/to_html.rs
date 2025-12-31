use crate::{
    html::{convert::ToHtml, template::wrap_template},
    parser::document::ast::MdDocument,
};

#[derive(Debug, Clone, PartialEq)]
pub struct HTMLDocument<'a> {
    pub path: &'a str,
    pub title: &'a str,
    pub tags: Option<Vec<&'a str>>,
    pub content: String,
}

pub fn md_to_html<'a>(md_doc: &'a MdDocument) -> HTMLDocument<'a> {
    let original_path = md_doc
        .path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or_else(|| "".into());

    let path = md_doc
        .front_matter
        .as_ref()
        .and_then(|fm| fm.get("slug"))
        .map_or(original_path, |s| s);

    let title = if let Some(title) = md_doc.front_matter.as_ref().and_then(|fm| fm.get("title"))
        && title != ""
    {
        title
    } else {
        original_path
    };

    let tags = if let Some(tags) = md_doc.front_matter.as_ref().and_then(|fm| fm.get("tags"))
        && tags != ""
    {
        Some(tags.split(", ").collect())
    } else {
        None
    };

    // mainでis_someを使って確認してるのでここはunwrapしてOK
    let content = wrap_template(title, md_doc.body.as_ref().unwrap().to_html().as_str());

    HTMLDocument {
        path,
        title,
        tags,
        content,
    }
}
