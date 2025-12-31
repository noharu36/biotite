use crate::choice;
use super::{Parser, some};
use super::ast::Inline;
use crate::parser::{take_until, id};
use crate::parser::lex::{string, character};

fn parse_text_context<'a>() -> impl Parser<'a, Vec<char>> {
    let escaped = string("\\").and(character(|_| true)).map(|(_, c)| c);
    let normal = character(|c| c != '\\' && c != '\n' && c != '*' && c != '`' && c != '~' && c != '[' && c != ']' && c != '(' && c != ')' && c != '!');

    let parser = choice![
        escaped,
        normal
    ];
    
    some(parser)
}

fn parse_text_inline<'a>() -> impl Parser<'a, Inline> {
    parse_text_context().map(|chars| Inline::Text(chars.into_iter().collect()))
}

// 太字
fn parse_bold_inline<'a>() -> impl Parser<'a, Inline> {
    let asterisk = string("**");

    // ((("**", "text"), "**"), rest)
    asterisk.clone()
        .and(take_until("**"))
        .and(asterisk)
        .map(|((_open_tag, text), _close_tag)| {
            // restは絶対空になるはず
            if let Some((inlines, _rest)) = parse_inlines().parse(text) {
                Inline::Strong(inlines)
            } else {
                Inline::Strong(vec![Inline::Text(text.to_string())])
            }
        })
}

// 取り消し線
fn parse_strikethru_inline<'a>() -> impl Parser<'a, Inline> {
    let tilde = string("~~");

    // ((("~~", "text"), "~~"), rest)
    tilde.clone()
        .and(take_until("~~"))
        .and(tilde)
        .map(|((_open_tag, text), _close_tag)| {
            // restは絶対空になるはず
            if let Some((inlines, _rest)) = parse_inlines().parse(text) {
                Inline::Strikethrough(inlines)
            } else {
                Inline::Strikethrough(vec![Inline::Text(text.to_string())])
            }
        })
}

// 斜体(アンダースコアで始まる構文は無視)
fn parse_italic_inline<'a>() -> impl Parser<'a, Inline> {
    let delimiter = character(|c| c == '*');

    // ((('*', "text"), '*'), rest)
    delimiter.clone()
        .and(take_until("*"))
        .and(delimiter)
        .map(|((_open_tag, text), _close_tag)| {
            // restは絶対空になるはず
            if let Some((inlines, _rest)) = parse_inlines().parse(text) {
                Inline::Italic(inlines)
            } else {
                Inline::Italic(vec![Inline::Text(text.to_string())])
            }
        })
}

// インラインコード
fn parse_code_inline<'a>() -> impl Parser<'a, Inline> {
    let delimiter = character(|c| c == '`');
    let text = parse_text_context().map(|chars| chars.into_iter().collect());

    // ((('`', "text"), '`'), rest)
    delimiter.clone()
        .and(text)
        .and(delimiter)
        .map(|((_open_tag, text), _close_tag)| Inline::Code(text))
}

// 画像
fn parse_image_inline<'a>() -> impl Parser<'a, Inline> {
    let start = string("![");
    let mid = string("](");
    let end = character(|c| c == ')');
    let alt = parse_text_context().map(|chars| chars.into_iter().collect());
    let url = some(character(|c| c != ')')).map(|chars| chars.into_iter().collect());

    // ((((("![", "alt"), "]("), url), end) rest)
    start
        .and(alt)
        .and(mid)
        .and(url)
        .and(end)
        .map(|((((_start, alt), _mid), url), _end)| Inline::Image { alt, url })
}

// リンク
fn parse_link_inline<'a>() -> impl Parser<'a, Inline> {
    let start = string("[");
    let mid = string("](");
    let end = character(|c| c == ')');
    let url = some(character(|c| c != ')')).map(|chars| chars.into_iter().collect());

    // ((((("[", "text"), "]("), url), end) rest)
    start
        .and(take_until("]("))
        .and(mid)
        .and(url)
        .and(end)
        .map(|((((_start, text), _mid), url), _end)| {
            // restは絶対空になるはず
            if let Some((inlines, _rest)) = parse_inlines().parse(text) {
                Inline::Link { text: inlines, url}
            } else {
                Inline::Link { text: vec![Inline::Text(text.to_string())], url}
            }
        })
}

fn parse_autolink_inline<'a>() -> impl Parser<'a, Inline> {
    let http = string("http://");
    let https = string("https://");
    let protocol = https.or(http);

    let url_char = character(|c| !c.is_whitespace() && c != '<' && c != '>' && c != '(' && c != ')' && c != '[' && c != ']' && c != '{' && c != '}' && c != '"' && c != '\'');
    let body = some(url_char).map(|chars| chars.into_iter().collect::<String>());

    protocol
        .and(body)
        .map(|(proto, mut domain)| {
            let trim_chars = ['.', ',', ';', ':', '!', '?'];

            let raw = format!("{}{}", proto, domain);

            while let Some(last) = domain.chars().last() {
                if trim_chars.contains(&last) {
                    domain.pop();
                } else {
                    break;
                }
            }

            let url = format!("{}{}", proto, domain);

            Inline::Link { text: vec![Inline::Text(raw.to_string())], url}
        })
}

// Wikiリンク
fn parse_wikilink_inline<'a>() -> impl Parser<'a, Inline> {
    let start = string("[[");
    let end = string("]]");
    let pipe = string("|");

    let content_char = character(|c| c != '|' && c != ']' && c != '\n');
    let text = some(content_char).map(|chars| chars.into_iter().collect::<String>());

    // ((((("[", "text"), "]("), url), end) rest)
    start
        .and(text.clone())
        .and((pipe.and(text)).or(id()))
        .and(end)
        .map(|(((_start, link), (_pipe, text)), _end)| {
            if text == "" {
                Inline::Link { text: vec![Inline::Text(link.clone())], url: link }
            } else {
                Inline::Link { text: vec![Inline::Text(text)], url: link }
            }
        })
}

// どの構文にもマッチしない記号
fn parse_symbol_as_text<'a>() -> impl Parser<'a, Inline> {
    character(|c| c != '\n').map(|c| Inline::Text(c.to_string()))
}

pub fn parse_inlines<'a>() -> impl Parser<'a, Vec<Inline>> {
    let inline = choice![
        parse_image_inline(),
        parse_wikilink_inline(),
        parse_link_inline(),
        parse_autolink_inline(),
        parse_bold_inline(),
        parse_strikethru_inline(),
        parse_italic_inline(), // boldよりも後ろに書かないとダメ
        parse_code_inline(),
        parse_text_inline(),

        parse_symbol_as_text()
    ];

    some(inline)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_text_inline() {
        let parser = parse_text_inline();
        let input = "HelloRust";
        let input2 = "Hello**World";
        let input3 = "Line1\nLine2";
        let input4 = "**Bold";
        //let input5 = r"not\\*Bold";

        assert_eq!(parser(input), Some((Inline::Text("HelloRust".to_string()), "")));
        assert_eq!(parser(input2), Some((Inline::Text("Hello".to_string()), "**World")));
        assert_eq!(parser(input3), Some((Inline::Text("Line1".to_string()), "\nLine2")));
        assert_eq!(parser(input4), None);
        //assert_eq!(parser(input5), Some((Inline::Text(r"not\*Bold".to_string()), "")));
    }

    #[test]
    fn test_parse_bold_inline() {
        let parser = parse_bold_inline();
        let input = "**BoldText**";
        let input2 = "**Rust** is cool";
        let input3 = "**Unclosed";
        let input4 = "**Bold and *Italic* mixed**";
        let input5 = "NotBold";

        assert_eq!(parser(input), Some((Inline::Strong(vec![Inline::Text("BoldText".to_string())]), "")));
        assert_eq!(parser(input2), Some((Inline::Strong(vec![Inline::Text("Rust".to_string())]), " is cool")));
        assert_eq!(parser(input3), None);

        match parser(input4) {
            Some((Inline::Strong(content), _)) => {
                assert_eq!(content.len(), 3);
                assert_eq!(content[0], Inline::Text("Bold and ".to_string()));
                match &content[1] {
                    Inline::Italic(inner) => {
                        assert_eq!(inner[0], Inline::Text("Italic".to_string()));
                    },
                    _ => panic!("Expected Emphasis inside Strong"),
                }
                assert_eq!(content[2], Inline::Text(" mixed".to_string()));
            },
            _ => panic!("Recursive parse failed"),
        };

        assert_eq!(parser(input5), None);
    }

    #[test]
    fn test_parse_italic_inline() {
        let parser = parse_italic_inline();
        let input = "*ItalicText*";
        let input2 = "*Rust* is cool";
        let input3 = "*Unclosed";
        let input4 = "NotItalic";

        assert_eq!(parser(input), Some((Inline::Italic(vec![Inline::Text("ItalicText".to_string())]), "")));
        assert_eq!(parser(input2), Some((Inline::Italic(vec![Inline::Text("Rust".to_string())]), " is cool")));
        assert_eq!(parser(input3), None);
        assert_eq!(parser(input4), None);
    }

    #[test]
    fn test_parse_code_inline() {
        let parser = parse_code_inline();
        let input = "`let x = 5;`";

        assert_eq!(parser(input), Some((Inline::Code("let x = 5;".to_string()), "")));
    }

    #[test]
    fn test_parse_strikethru_inline() {
        let parser = parse_strikethru_inline();
        let input = "~~deleted~~";

        assert_eq!(parser(input), Some((Inline::Strikethrough(vec![Inline::Text("deleted".to_string())]), "")));
    }

    #[test]
    fn test_parse_image_inline() {
        let parser = parse_image_inline();
        let input = "![Rust Logo](https://rust-lang.org/logo.png)";

        assert_eq!(parser(input), Some((Inline::Image{ alt: "Rust Logo".to_string(), url: "https://rust-lang.org/logo.png".to_string() }, "")));
    }

    #[test]
    fn test_parse_link_inline() {
        let parser = parse_link_inline();
        let input = "[Click here](https://google.com)";

        assert_eq!(parser(input), Some((Inline::Link{ text: vec![Inline::Text("Click here".to_string())], url: "https://google.com".to_string() }, "")));
    }

    #[test]
    fn test_parse_wikilink_inline() {
        let parser = parse_wikilink_inline();
        let input = "[[Obsidian Note]]";
        let input2 = "[[File Name|Custom Label]]";

        assert_eq!(parser(input), Some((Inline::Link{ text: vec![Inline::Text("Obsidian Note".to_string())], url: "Obsidian Note".to_string() }, "")));
        assert_eq!(parser(input2), Some((Inline::Link{ text: vec![Inline::Text("Custom Label".to_string())], url: "File Name".to_string() }, "")));
    }

    #[test]
    fn test_parse_symbol_inline() {
        let parser = parse_symbol_as_text();
        let input = "!";

        assert_eq!(parser(input), Some((Inline::Text("!".to_string()), "")));
    }

    #[test]
    fn test_parse_inlines() {
        let parser = parse_inlines();
        let input = "Hello! **Bold** and `code`";
        let input2 = r"This is \*not bold\*";

        assert_eq!(
            parser(input),
            Some((vec![
                    Inline::Text("Hello".to_string()),
                    Inline::Text("!".to_string()),
                    Inline::Text(" ".to_string()),
                    Inline::Strong(vec![Inline::Text("Bold".to_string())]),
                    Inline::Text(" and ".to_string()),
                    Inline::Code("code".to_string()),
            ], ""))
        );
        assert_eq!(parser(input2), Some((vec![Inline::Text("This is *not bold*".to_string())], "")))
    }
}
