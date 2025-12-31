use super::ast::{Block, ListItem, ListType};
use super::inline::parse_inlines;
use super::{Parser, many, some};
use crate::choice;
use crate::parser::lex::{character, digit, string};
use crate::parser::{id, newline, parse_line, take_until};

// 水平線
fn parse_horizontal_rule<'a>() -> impl Parser<'a, Block> {
    let tag = choice![string("---"), string("***"), string("___")];

    tag.and(many(character(|c| c == ' ')))
        .and(newline())
        .map(|_| Block::HorizontalRule)
}

// 見出し
fn parse_heading<'a>() -> impl Parser<'a, Block> {
    let hashes = some(character(|c| c == '#'));

    move |input: &'a str| match hashes(input) {
        Some((hashes_vec, content)) => {
            let level = hashes_vec.len();
            if level > 6 {
                return None;
            }

            if let Some(((_blank, next), rest)) = string(" ").and(parse_line()).parse(content)
                && let Some((inlines, _)) = parse_inlines().parse(next)
            {
                Some((
                    Block::Heading {
                        level: level as u8,
                        content: inlines,
                    },
                    rest,
                ))
            } else {
                None
            }
        }
        _ => None,
    }
}

// コードブロック
fn parse_fenced_code_block<'a>() -> impl Parser<'a, Block> {
    let start = string("```");
    let end_marker = "```";
    let language = many(character(|c| c != '\n')).map(|chars| {
        if chars.is_empty() {
            None
        } else {
            Some(chars.into_iter().collect())
        }
    });

    start
        .and(language)
        .and(newline())
        .and(take_until(end_marker))
        .and(string("```\n").or(string(end_marker)))
        .map(
            |((((_start, lang), _), code), _end)| Block::FencedCodeBlock {
                language: lang,
                code: code.to_string(),
            },
        )
}

// 引用
fn parse_blockquote<'a>() -> impl Parser<'a, Block> {
    let marker = string(">").and(many(string(" ")));
    let quote_lines = some(marker.and(parse_line()).map(|(_, content)| content));

    move |input: &'a str| {
        match quote_lines.parse(input) {
            Some((lines, rest)) => {
                let capacity = lines.iter().map(|s| s.len() + 1).sum();
                let mut content = String::with_capacity(capacity);
                lines.iter().for_each(|s| {
                    content.push_str(s);
                    content.push('\n');
                });
                // restは絶対空になるはず
                if let Some((blocks, _rest)) = some(parse_block()).parse(&content) {
                    Some((Block::Blockquote(blocks), rest))
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

// リスト
fn parse_list_item<'a>(marker_parser: impl Parser<'a, usize>) -> impl Parser<'a, ListItem> {
    marker_parser
        .and(string(" "))
        .and(parse_checkbox().or(id()))
        .and(parse_inlines())
        .and(newline())
        .map(|((((indent, _), checked), inlines), _)| ListItem {
            indent,
            checked,
            content: inlines,
        })
}

fn parse_checkbox<'a>() -> impl Parser<'a, Option<bool>> {
    let checked = string("[x]").and(string(" ")).map(|_| Some(true));
    let unchecked = string("[ ]").and(string(" ")).map(|_| Some(false));

    choice![checked, unchecked]
}

fn parse_unorderd_list<'a>() -> impl Parser<'a, Block> {
    let indent = many(character(|c| c == ' ' || c == '\t')).map(|chars| chars.len());
    let marker_char = choice![string("*"), string("-"), string("+")];
    let marker = indent.and(marker_char).map(|(len, _)| len);

    some(parse_list_item(marker))
        .map(ListType::Unordered)
        .map(Block::List)
}

fn parse_orderd_list<'a>() -> impl Parser<'a, Block> {
    let indent = many(character(|c| c == ' ' || c == '\t')).map(|chars| chars.len());
    let marker_content = digit().and(string("."));
    let marker = indent.and(marker_content).map(|(len, _)| len);

    some(parse_list_item(marker))
        .map(ListType::Ordered)
        .map(Block::List)
}

// 段落
fn parse_paragraph<'a>() -> impl Parser<'a, Block> {
    parse_inlines()
        // .and(newline())
        .map(|inlines| Block::Paragraph(inlines))
}

pub fn parse_block<'a>() -> impl Parser<'a, Block> {
    choice![
        parse_horizontal_rule(),
        parse_fenced_code_block(),
        parse_heading(),
        parse_blockquote(),
        parse_unorderd_list(),
        parse_orderd_list(),
        parse_paragraph()
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::document::ast::Inline;

    #[test]
    fn test_parse_heading() {
        let parser = parse_heading();
        let input = "# Title One\n";
        let input2 = "### Title **Three**\n";

        assert_eq!(
            parser(input),
            Some((
                Block::Heading {
                    level: 1,
                    content: vec![Inline::Text("Title One".to_string())]
                },
                ""
            ))
        );
        assert_eq!(
            parser(input2),
            Some((
                Block::Heading {
                    level: 3,
                    content: vec![
                        Inline::Text("Title ".to_string()),
                        Inline::Strong(vec![Inline::Text("Three".to_string())])
                    ]
                },
                ""
            ))
        )
    }

    #[test]
    fn test_parse_horizontal_rule() {
        let parser = parse_horizontal_rule();

        assert_eq!(parser("---\n"), Some((Block::HorizontalRule, "")));
        assert_eq!(parser("***\n"), Some((Block::HorizontalRule, "")));
        assert_eq!(parser("___\n"), Some((Block::HorizontalRule, "")));
    }

    #[test]
    fn test_parse_fenced_code_block() {
        let parser = parse_fenced_code_block();
        let input = "```rust\nfn main() {}\n```";
        let input2 = "```\nline 1\nline 2\n```";

        assert_eq!(
            parser(input),
            Some((
                Block::FencedCodeBlock {
                    language: Some("rust".to_string()),
                    code: "fn main() {}\n".to_string()
                },
                ""
            ))
        );
        assert_eq!(
            parser(input2),
            Some((
                Block::FencedCodeBlock {
                    language: None,
                    code: "line 1\nline 2\n".to_string()
                },
                ""
            ))
        )
    }

    #[test]
    fn test_parse_unordered_list() {
        let parser = parse_unorderd_list();
        let input = "- Item 1\n- Item **2**\n";

        match parser(input) {
            Some((Block::List(ListType::Unordered(items)), rest)) => {
                assert_eq!(rest, "");
                assert_eq!(items.len(), 2);
                assert_eq!(items[0].content, vec![Inline::Text("Item 1".to_string())]);
                match &items[1].content[1] {
                    Inline::Strong(_) => assert!(true),
                    _ => panic!("Expected Strong inline"),
                }
            }
            _ => panic!("Failed to parse unordered list"),
        }
    }

    #[test]
    fn test_parse_ordered_list() {
        let parser = parse_orderd_list();
        let input = "1. First\n2. Second\n";

        match parser(input) {
            Some((Block::List(ListType::Ordered(items)), rest)) => {
                assert_eq!(rest, "");
                assert_eq!(items.len(), 2);
                assert_eq!(items[0].content, vec![Inline::Text("First".to_string())]);
            }
            _ => panic!("Failed to parse ordered list"),
        }
    }

    #[test]
    fn test_parse_check_list() {
        let parser = parse_unorderd_list();
        let input = "- [x] Done task\n- [ ] Pending task\n- Normal item\n";

        match parser(input) {
            Some((Block::List(ListType::Unordered(items)), rest)) => {
                assert_eq!(rest, "");
                assert_eq!(items.len(), 3);

                assert_eq!(items[0].checked, Some(true));
                assert_eq!(items[0].content[0], Inline::Text("Done task".to_string()));

                assert_eq!(items[1].checked, Some(false));
                assert_eq!(
                    items[1].content[0],
                    Inline::Text("Pending task".to_string())
                );

                assert_eq!(items[2].checked, None);
                assert_eq!(items[2].content[0], Inline::Text("Normal item".to_string()));
            }
            _ => panic!("Failed to parse ordered list"),
        }
    }

    #[test]
    fn test_parse_blockquote() {
        let parser = parse_blockquote();
        let input = "> Simple quote\n";
        let input2 = "> # Heading inside\n> Normal text\n";
        let input3 = "> > Double nested\n";

        match parser(input) {
            Some((Block::Blockquote(blocks), rest)) => {
                assert_eq!(rest, "");
                assert_eq!(blocks.len(), 1);

                if let Block::Paragraph(inlines) = &blocks[0] {
                    assert_eq!(inlines[0], Inline::Text("Simple quote".to_string()));
                } else {
                    panic!("Expected Paragraph inside Blockquote");
                }
            }
            _ => panic!("Failed to parse simple blockquote"),
        }

        match parser(input2) {
            Some((Block::Blockquote(blocks), rest)) => {
                assert_eq!(rest, "");
                assert_eq!(blocks.len(), 2);

                if let Block::Heading { level, content } = &blocks[0] {
                    assert_eq!(*level, 1);
                    assert_eq!(content[0], Inline::Text("Heading inside".to_string()));
                } else {
                    panic!("Expected Heading as first block");
                }

                if let Block::Paragraph(inlines) = &blocks[1] {
                    assert_eq!(inlines[0], Inline::Text("Normal text".to_string()));
                } else {
                    panic!("Expected Paragraph as second block");
                }
            }
            _ => panic!("Failed to parse nested blockquote"),
        }

        match parser(input3) {
            Some((Block::Blockquote(outer_blocks), rest)) => {
                assert_eq!(rest, "");
                if let Block::Blockquote(inner_blocks) = &outer_blocks[0] {
                    if let Block::Paragraph(inlines) = &inner_blocks[0] {
                        assert_eq!(inlines[0], Inline::Text("Double nested".to_string()));
                    } else {
                        panic!("Expected Paragraph inside inner Blockquote");
                    }
                } else {
                    panic!("Expected Blockquote inside Blockquote");
                }
            }
            _ => panic!("Failed to parse double nested blockquote"),
        }
    }
}
