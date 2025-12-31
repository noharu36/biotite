pub mod ast;
pub mod front_matter;
pub mod inline;
pub mod block;

use super::{Parser, many, some, blank_line};
use ast::Document;
use block::parse_block;

pub fn parse_document<'a>() -> impl Parser<'a, Document> {
    let block = many(blank_line()).and(parse_block()).map(|(_, block)| block);

    many(block).map(|blocks| Document { blocks })
}

#[cfg(test)]
mod tests {
    use super::*;
    use ast::Block;

    #[test]
    fn test_parse_document() {
        let parser = parse_document();
        let input = 
r#"
# Heading 1

This is a paragraph.

- List item 1
- List item 2

```rust
fn code() {}
```
"#;

        match parser.parse(input) {
            Some((doc, rest)) => {
                assert_eq!(rest, "");
                assert_eq!(doc.blocks.len(), 4);

                match &doc.blocks[0] {
                    Block::Heading { level: 1, .. } => {},
                    _ => panic!("First block should be Heading"),
                }

                match &doc.blocks[1] {
                    Block::Paragraph(_) => {},
                    _ => panic!("Second block should be Paragraph"),
                }

                match &doc.blocks[2] {
                    Block::List(_) => {},
                    _ => panic!("Third block should be List"),
                }

                match &doc.blocks[3] {
                    Block::FencedCodeBlock { .. } => {},
                    _ => panic!("Fourth block should be CodeBlock"),
                }
            },
            _ => panic!("Document parsing failed"),
        }
    }
}
