use super::{spaces_line, Command, SpecialStr};

use combine::parser::char;
use combine::{attempt, choice, many, many1, optional, satisfy, sep_by, Parser, Stream};
use combine::{sep_end_by, token};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Line {
    Single(Command),
    Multi(Vec<Line>),
    If(SpecialStr, Box<Line>, Option<Box<Line>>),
    While(SpecialStr, Box<Line>),
    Case(SpecialStr, Vec<(Vec<SpecialStr>, Line)>),
    For(String, SpecialStr, Box<Line>),
    Break,
    Continue,
}

impl Line {
    pub fn parse<I: Stream<Token = char>>() -> impl Parser<I, Output = Self> {
        line()
    }

    fn parse_<I: Stream<Token = char>>() -> impl Parser<I, Output = Self> {
        choice((
            attempt(char::string("break")).map(|_| Self::Break),
            attempt(char::string("continue")).map(|_| Self::Continue),
            if_().map(|(cond, first, second)| Self::If(cond, first, second)),
            while_().map(|(cond, block)| Self::While(cond, block)),
            case().map(|(cond, blocks)| Self::Case(cond, blocks)),
            for_().map(|(c, iter, block)| Self::For(c, iter, block)),
            multi().map(|lines| Self::Multi(lines)),
            Command::parse().map(|cmd| Self::Single(cmd)),
        ))
    }
}

combine::parser! {
    fn line[I]()(I) -> Line
    where [I: Stream<Token = char>]
    {
        Line::parse_()
    }
}

fn multi<I: Stream<Token = char>>() -> impl Parser<I, Output = Vec<Line>> {
    token('{')
        .skip(spaces_line())
        .with(sep_end_by(
            Line::parse(),
            token('\n').or(token(';')).with(spaces_line()),
        ))
        .skip(token('}'))
}

fn if_<I: Stream<Token = char>>(
) -> impl Parser<I, Output = (SpecialStr, Box<Line>, Option<Box<Line>>)> {
    (
        attempt(char::string("if")),
        spaces_line(),
        SpecialStr::parse(),
        spaces_line(),
        Line::parse().map(|line| Box::new(line)),
        spaces_line(),
        optional(
            (
                char::string("else"),
                spaces_line(),
                Line::parse().map(|line| Box::new(line)),
            )
                .map(|(_, _, line)| line),
        ),
    )
        .map(|(_, _, cond, _, first, _, second)| (cond, first, second))
}

fn while_<I: Stream<Token = char>>() -> impl Parser<I, Output = (SpecialStr, Box<Line>)> {
    (
        attempt(char::string("while")),
        spaces_line(),
        SpecialStr::parse(),
        spaces_line(),
        Line::parse().map(|line| Box::new(line)),
    )
        .map(|(_, _, cond, _, block)| (cond, block))
}

fn case<I: Stream<Token = char>>(
) -> impl Parser<I, Output = (SpecialStr, Vec<(Vec<SpecialStr>, Line)>)> {
    (
        attempt(char::string("case")),
        spaces_line(),
        SpecialStr::parse(),
        spaces_line(),
        token('{'),
        spaces_line(),
        many(
            (
                sep_by(
                    SpecialStr::parse().skip(spaces_line()),
                    token('|').skip(spaces_line()),
                ),
                char::string("=>"),
                spaces_line(),
                Line::parse(),
                spaces_line(),
            )
                .map(|(pats, _, _, block, _)| (pats, block)),
        ),
        token('}'),
    )
        .map(|(_, _, cond, _, _, _, blocks, _)| (cond, blocks))
}

fn for_<I: Stream<Token = char>>() -> impl Parser<I, Output = (String, SpecialStr, Box<Line>)> {
    (
        attempt(char::string("for")),
        spaces_line(),
        many1(satisfy(|c: char| !c.is_whitespace())),
        spaces_line(),
        char::string("in"),
        spaces_line(),
        SpecialStr::parse(),
        spaces_line(),
        Line::parse().map(|line| Box::new(line)),
    )
        .map(|(_, _, c, _, _, _, iter, _, block)| (c, iter, block))
}
