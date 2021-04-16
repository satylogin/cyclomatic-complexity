use std::error::Error;
use std::fmt;
use std::num::{ParseFloatError, ParseIntError};

#[derive(Debug, PartialEq)]
pub enum ParseErrorKind {
    InvalidSymbol,
    UnexpectedEOF,
    NoMatches,
    ConversionError,
    UnknownCharacter(char),
}

impl fmt::Display for ParseErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug)]
pub struct ParseError {
    pub kind: ParseErrorKind,
    pub msg: Option<String>,
    pub index: Option<usize>,
    pub source: Option<Box<dyn Error>>,
}

impl ParseError {
    pub fn kind(kind: ParseErrorKind) -> ParseError {
        ParseError {
            kind,
            msg: None,
            index: None,
            source: None,
        }
    }

    pub fn msg(mut self, msg: String) -> ParseError {
        self.msg = Some(msg);
        self
    }

    pub fn source(mut self, source: Box<dyn Error>) -> ParseError {
        self.source = Some(source);
        self
    }

    pub fn index(mut self, index: usize) -> ParseError {
        self.index = Some(index);
        self
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut response = write!(f, "Error: {}", self.kind);
        if self.msg.is_some() {
            response = write!(f, ", Message: {}", self.msg.clone().unwrap());
        }
        response
    }
}

impl Error for ParseError {}

impl From<ParseFloatError> for ParseError {
    fn from(other: ParseFloatError) -> ParseError {
        ParseError::kind(ParseErrorKind::ConversionError)
            .msg("Cannot convert token to float".to_string())
            .source(Box::new(other))
    }
}

impl From<ParseIntError> for ParseError {
    fn from(other: ParseIntError) -> ParseError {
        ParseError::kind(ParseErrorKind::ConversionError)
            .msg("Cannot convert token to int".to_string())
            .source(Box::new(other))
    }
}

pub type ParseResult<T> = Result<T, ParseError>;
