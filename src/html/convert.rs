use crate::parser::document::ast::{Document, Block, ListItem, ListType, Inline};

fn escape_html(s: &str) -> String {
    s.replace("&", "&amp;")
     .replace("<", "&lt;")
     .replace(">", "&gt;")
     .replace("\"", "&quot;")
     .replace("'", "&#39;")
}

pub trait ToHtml {
    fn to_html(&self) -> String;
}

impl ToHtml for Inline {
    fn to_html(&self) -> String {
        match self {
            Inline::Text(s) => escape_html(s),
            Inline::Strong(inlines) => format!("<strong>{}</strong>", inlines.to_html()),
            Inline::Italic(inlines) => format!("<em>{}</em>", inlines.to_html()),
            Inline::Strikethrough(inlines) => format!("<del>{}</del>", inlines.to_html()),
            Inline::Code(s) => format!("<code>{}</code>", escape_html(s)),
            Inline::Link { text, url } => format!("<a href=\"{}\">{}</a>", escape_html(url), text.to_html()),
            Inline::Image { alt, url } => {
                format!("<img src=\"{}\" alt=\"{}\" />", escape_html(url), escape_html(alt))
            },
        }
    }
}

impl ToHtml for Vec<Inline> {
    fn to_html(&self) -> String {
        self.iter().map(|i| i.to_html()).collect::<String>()
    }
}

impl ToHtml for Block {
    fn to_html(&self) -> String {
        match self {
            Block::Heading { level, content } => {
                format!("<h{}>{}</h{}>", level, content.to_html(), level)
            },
            Block::Paragraph(content) => {
                format!("<p>{}</p>", content.to_html())
            },
            Block::Blockquote(blocks) => {
                let inner_html = blocks.iter().map(|b| b.to_html()).collect::<String>();
                format!("<blockquote>{}</blockquote>", inner_html)
            },
            Block::FencedCodeBlock { language, code } => {
                let lang_class = match language {
                    Some(l) => format!(" class=\"language-{}\"", escape_html(l)),
                    None => "".to_string(),
                };
                format!("<pre><code{}>{}</code></pre>", lang_class, escape_html(code))
            },
            Block::HorizontalRule => "<hr />".to_string(),
            Block::List(list_type) => list_type.to_html(),
        }
    }
}

impl ToHtml for ListType {
    fn to_html(&self) -> String {
        let (tag, items) = match self {
            ListType::Unordered(items) => ("ul", items),
            ListType::Ordered(items) => ("ol", items),
        };

        if items.is_empty() {
            return String::new();
        }

        let initial_indent = items[0].indent;

        let (mut html, _) = items.iter().fold((format!("<{}>", tag), initial_indent), |(mut html, last_indent), item| {
            if item.indent > last_indent {
                html.push_str(&format!("\n<{}>", tag));
            } else if item.indent < last_indent {
                html.push_str(&format!("</{}>", tag));
            }

            html.push_str(&item.to_html());

            (html, item.indent)
        });

        html.push_str(&format!("</{}>", tag));
        html
    }
}

impl ToHtml for ListItem {
    fn to_html(&self) -> String {
        let checkbox_html = match self.checked {
            Some(true) => "<input type=\"checkbox\" checked disabled> ",
            Some(false) => "<input type=\"checkbox\" disabled> ",
            None => "",
        };
        
        format!("<li>{}{}\n</li>", checkbox_html, self.content.to_html())
    }
}

impl ToHtml for Document {
    fn to_html(&self) -> String {
        self.blocks.iter().map(|b| b.to_html()).collect::<String>()
    }
}
