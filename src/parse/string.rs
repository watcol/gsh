extern crate unindent;

use combine::parser::char;
use combine::{attempt, choice, count_min_max, many, many1, one_of, parser, satisfy, token, value};
use combine::{ParseError, Parser, Stream};
use unindent::unindent;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpecialStr(Vec<StrKind>);

#[derive(Clone, Debug, PartialEq, Eq)]
enum StrKind {
    String(String),
    Var(String),
}

impl From<String> for SpecialStr {
    fn from(s: String) -> Self {
        Self(vec![StrKind::String(s)])
    }
}

impl SpecialStr {
    pub fn new() -> Self {
        SpecialStr(Vec::new())
    }

    pub fn parse<I: Stream<Token = char>>() -> impl Parser<I, Output = Self> {
        choice((
            attempt(raw_unindent()).map(|s| Self::from(s)),
            raw_str().map(|s| Self(vec![StrKind::String(s)])),
            attempt(lit_unindent()),
            lit(),
            direct(),
        ))
    }

    pub fn eval(&self) -> anyhow::Result<String> {
        Ok(self
            .0
            .iter()
            .map(|kind| match kind {
                StrKind::String(s) => Ok(s.clone()),
                StrKind::Var(key) => std::env::var(key),
            })
            .collect::<Result<Vec<_>, _>>()?
            .join(""))
    }
}

fn direct<I: Stream<Token = char>>() -> impl Parser<I, Output = SpecialStr> {
    many1(
        env()
            .map(|s| StrKind::Var(s))
            .or(direct_str().map(|s| StrKind::String(s))),
    )
    .map(|strs| SpecialStr(strs))
}

fn direct_str<I: Stream<Token = char>>() -> impl Parser<I, Output = String> {
    many1(satisfy(|c: char| {
        !c.is_whitespace() && "#|&;${}()".chars().all(|d| c != d)
    }))
}

fn lit_unindent<I: Stream<Token = char>>() -> impl Parser<I, Output = SpecialStr> {
    char::string("\"\"\"").with(
    parser(|input: &mut I| {
        let (s, commited) = lit_str().parse_stream(input).into_result()?;
        let s = unindent(&s);
        let res = lit_reparse().parse_stream(&mut s.as_str()).into_result();

        match res {
            Ok((special, _)) => Ok((special, commited)),
            Err(_) => Err(combine::error::Commit::Peek(combine::error::Tracked::from(
                I::Error::empty(input.position()),
            ))
            .into()),
        }
    })).skip(char::string("\"\"\""))
}

fn lit<I: Stream<Token = char>>() -> impl Parser<I, Output = SpecialStr> {
    token('"').with(
    parser(|input: &mut I| {
        let (s, commited) = lit_str().parse_stream(input).into_result()?;
        let res = lit_reparse().parse_stream(&mut s.as_str()).into_result();

        match res {
            Ok((special, _)) => Ok((special, commited)),
            Err(_) => Err(combine::error::Commit::Peek(combine::error::Tracked::from(
                I::Error::empty(input.position()),
            ))
            .into()),
        }
    })).skip(token('"'))
}

fn lit_str<I: Stream<Token = char>>() -> impl Parser<I, Output = String> {
    use std::convert::TryFrom;

    many1(satisfy(|c| c != '"').then(|c| {
        if c == '\\' {
            choice((
                one_of("abefnrtv\\\"".chars()).map(|seq| match seq {
                    'a' => '\x07',
                    'b' => '\x08',
                    'e' => '\x1b',
                    'f' => '\x0c',
                    'n' => '\n',
                    'r' => '\r',
                    't' => '\t',
                    'v' => '\x0b',
                    '\\' => '\\',
                    '"' => '"',
                    _ => unreachable!(),
                }),
                token('x')
                    .with(count_min_max(2, 2, char::hex_digit()))
                    .map(|s: String| u8::from_str_radix(s.as_str(), 16).unwrap() as char),
                one_of("uU".chars())
                    .and(token('{'))
                    .with(many1(char::hex_digit()).map(|s: String| {
                        char::try_from(u32::from_str_radix(s.as_str(), 16).unwrap()).unwrap()
                    }))
                    .skip(token('}')),
            ))
            .left()
        } else {
            value(c).right()
        }
    }))
}

fn lit_reparse<I: Stream<Token = char>>() -> impl Parser<I, Output = SpecialStr> {
    many1(
        env()
            .map(|s| StrKind::Var(s))
            .or(many1(satisfy(|c| c != '$')).map(|s| StrKind::String(s))),
    )
    .map(|strs| SpecialStr(strs))
}

fn raw_unindent<I: Stream<Token = char>>() -> impl Parser<I, Output = String> {
    char::string("''")
        .with(raw_str())
        .skip(char::string("''"))
        .map(|s| unindent(&s))
}

fn raw_str<I: Stream<Token = char>>() -> impl Parser<I, Output = String> {
    token('\'')
        .with(many(choice((
            attempt(char::string("\\\\")).map(|_| '\\'),
            attempt(char::string("\\\'")).map(|_| '\''),
            satisfy(|c| c != '\''),
        ))))
        .skip(token('\''))
}

fn env<I: Stream<Token = char>>() -> impl Parser<I, Output = String> {
    token('$')
        .with(many1(satisfy(|c| c != ';')))
        .skip(token(';'))
}
