pub mod document;
pub mod lex;
pub mod parse;

use crate::parser::lex::{character, string};

pub trait Parser<'a, T>: Fn(&'a str) -> Option<(T, &'a str)> + Clone {
    fn map<U>(self, f: impl Fn(T) -> U + Clone) -> impl Parser<'a, U>;
    fn or(self, other: impl Parser<'a, T>) -> impl Parser<'a, T>;
    fn and<U>(self, other: impl Parser<'a, U>) -> impl Parser<'a, (T, U)>;
    fn parse(&self, input: &'a str) -> Option<(T, &'a str)>;
}

impl<'a, T, F> Parser<'a, T> for F
where
    F: Fn(&'a str) -> Option<(T, &'a str)> + Clone,
{
    fn map<U>(self, f: impl Fn(T) -> U + Clone) -> impl Parser<'a, U> {
        move |input: &'a str| self(input).map(|(t, rest)| (f(t), rest))
    }

    fn or(self, other: impl Parser<'a, T>) -> impl Parser<'a, T> {
        move |input: &'a str| self(input).or_else(|| other(input))
    }

    fn and<U>(self, other: impl Parser<'a, U>) -> impl Parser<'a, (T, U)> {
        move |input: &'a str| {
            self(input).and_then(|(t, rest)| other(rest).map(|(u, rest)| ((t, u), rest)))
        }
    }
    fn parse(&self, input: &'a str) -> Option<(T, &'a str)> {
        self(input)
    }
}

// zero or more
pub fn many<'a, T>(parser: impl Parser<'a, T>) -> impl Parser<'a, Vec<T>> {
    move |mut input: &'a str| {
        let mut result = Vec::new();
        while let Some((t, rest)) = parser(input) {
            result.push(t);
            input = rest;
        }
        Some((result, input))
    }
}

// one or more
pub fn some<'a, T>(parser: impl Parser<'a, T>) -> impl Parser<'a, Vec<T>> {
    move |input: &'a str| {
        let (t, rest) = parser(input)?;
        let mut result = vec![t];
        let mut input = rest;
        while let Some((t, rest)) = parser(input) {
            result.push(t);
            input = rest;
        }
        Some((result, input))
    }
}

pub fn id<'a, T>() -> impl Parser<'a, T>
where
    T: Default + 'a,
{
    move |input: &'a str| Some((T::default(), input))
}

pub fn take_until<'a>(target: &'a str) -> impl Parser<'a, &'a str> {
    move |input: &'a str| match input.find(target) {
        Some(index) => Some((&input[..index], &input[index..])),
        _ => None,
    }
}

pub fn newline<'a>() -> impl Parser<'a, &'a str> {
    string("\n")
}

pub fn parse_line<'a>() -> impl Parser<'a, &'a str> {
    move |input: &'a str| match input.find('\n') {
        Some(index) => Some((&input[..index], &input[index + 1..])),
        _ => Some((input, "")),
    }
}

pub fn blank_line<'a>() -> impl Parser<'a, ()> {
    many(character(|c| c == ' ' || c == '\t'))
        .and(newline())
        .map(|_| ())
}

#[macro_export]
macro_rules! choice {
    ($parser0:expr, $($parser:expr),*) => {{
        let p = $parser0;
        $(
            let p = p.or($parser);
        )*
        p
    }};
}

#[cfg(test)]
mod tests {
    use crate::parser::lex::{character, string};
    use crate::parser::{Parser, many};

    #[test]
    fn test_map() {
        let parser = character(|c| c == 'a').map(|_| 1);

        assert_eq!(parser("abc"), Some((1, "bc")));
        assert_eq!(parser("def"), None);
        assert_eq!(parser(""), None);
    }

    #[test]
    fn test_or() {
        let parser = character(|c| c == 'a').or(character(|c| c == 'b'));

        assert_eq!(parser("abc"), Some(('a', "bc")));
        assert_eq!(parser("bcd"), Some(('b', "cd")));
        assert_eq!(parser("def"), None);
        assert_eq!(parser(""), None);
    }

    #[test]
    fn test_and() {
        let parser = character(|c| c == 'a').and(character(|c| c == 'b'));

        assert_eq!(parser("abc"), Some((('a', 'b'), "c")));
        assert_eq!(parser("def"), None);
        assert_eq!(parser(""), None);
    }

    #[test]
    fn test_many() {
        fn triming<'a, T>(parser: impl Parser<'a, T>) -> impl Parser<'a, T> {
            move |input: &'a str| parser(input.trim_start())
        }
        let parser = many(triming(character(|c| c == 'a')));
        assert_eq!(parser("a a a"), Some((vec!['a', 'a', 'a'], "")));
        assert_eq!(parser(""), Some((vec![], "")));
        assert_eq!(parser("  a aabc"), Some((vec!['a', 'a', 'a'], "bc")));
    }

    #[test]
    fn test_choice_macro() {
        let parser = choice![
            string("zero").map(|_| 0),
            string("one").map(|_| 1),
            string("two").map(|_| 2),
            string("three").map(|_| 3)
        ];

        assert_eq!(parser("zero"), Some((0, "")));
        assert_eq!(parser("one"), Some((1, "")));
        assert_eq!(parser("two"), Some((2, "")));
        assert_eq!(parser("three"), Some((3, "")));
        assert_eq!(parser("hoge"), None);
    }
}
