use super::{spaces, spaces_line, string, Redirect};
use combine::{attempt, eof, optional, sep_end_by, token};
use combine::{Parser, Stream};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Command {
    pub name: String,
    pub args: Vec<Arg>,
    pub pipe: Option<Box<Command>>,
    pub bg: bool,
}

impl Command {
    fn empty() -> Self {
        Self {
            name: String::new(),
            args: Vec::new(),
            pipe: None,
            bg: false,
        }
    }

    pub fn parse<I: Stream<Token = char>>() -> impl Parser<I, Output = Self> {
        command()
    }

    fn parse_<I: Stream<Token = char>>() -> impl Parser<I, Output = Self> {
        spaces_line().with(
            eof().map(|_| Self::empty()).or((
                string().skip(spaces()),
                sep_end_by(Arg::parse(), spaces()),
                optional(token('|').with(Self::parse())),
                optional(spaces().with(token('&')).skip(spaces())),
            )
                .map(|(name, args, pipe, bg)| Self {
                    name,
                    args,
                    pipe: pipe.map(|c| Box::new(c)),
                    bg: bg.is_some(),
                })),
        )
    }
}

combine::parser! {
    fn command[I]()(I) -> Command
    where [I: Stream<Token = char>]
    {
        Command::parse_()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Arg {
    Arg(String),
    Redirect(Redirect),
}

impl Arg {
    pub fn parse<I: Stream<Token = char>>() -> impl Parser<I, Output = Self> {
        attempt(Redirect::parse().map(|r| Self::Redirect(r))).or(string().map(|s| Self::Arg(s)))
    }
}
