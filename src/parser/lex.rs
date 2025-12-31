use crate::parser::some;

use super::Parser;

pub fn character<'a, F>(p: F) -> impl Parser<'a, char>
where
    F: Fn(char) -> bool + 'a + Clone,
{
    move |input: &'a str| match input.chars().next() {
        Some(c) if p(c) => Some((c, &input[c.len_utf8()..])),
        _ => None,
    }
}

pub fn string<'a>(s: &'a str) -> impl Parser<'a, &'a str> {
    move |input: &'a str| {
        if input.starts_with(s) {
            Some((s, &input[s.len()..]))
        } else {
            None
        }
    }
}

pub fn digit<'a>() -> impl Parser<'a, i32> {
    let digits = some(character(|c| c.is_ascii_digit()));

    digits.map(|chars| chars.into_iter().collect::<String>().parse().unwrap())
}
#[cfg(test)]
mod tests {
    use crate::parser::lex::character;
    use crate::parser::lex::string;

    #[test]
    fn char_works() {
        let parser = character(|c| c == 'a');
        let parser_utf8 = character(|c| c == 'あ');

        assert_eq!(parser("abc"), Some(('a', "bc")));
        assert_eq!(parser("def"), None);
        assert_eq!(parser_utf8("あいう"), Some(('あ', "いう")));
        assert_eq!(parser_utf8("えおか"), None);
    }

    #[test]
    fn test_string() {
        let parser = string("abc");
        let parser_utf8 = string("あいう");

        assert_eq!(parser("abcdef"), Some(("abc", "def")));
        assert_eq!(parser("def"), None);
        assert_eq!(parser_utf8("あいう"), Some(("あいう", "")));
        assert_eq!(parser_utf8("あいうえお"), Some(("あいう", "えお")));
    }
}
