//! Module for performing lexical analysis on source code.
use crate::parsers::error::{ParseError, ParseErrorKind, ParseResult};

#[derive(Debug, PartialEq)]
pub enum TokenKind {
    Integer(usize),
    Decimal(f64),
    Identifier(String),
    QuotedString(String),
    Asterisk,
    At,
    Carat,
    CloseParen,
    CloseSquare,
    Colon,
    Dot,
    End,
    Equals,
    Minus,
    OpenParen,
    OpenSquare,
    Plus,
    Semicolon,
    Slash,
}

impl From<String> for TokenKind {
    fn from(other: String) -> TokenKind {
        TokenKind::Identifier(other)
    }
}

impl From<&str> for TokenKind {
    fn from(other: &str) -> TokenKind {
        TokenKind::Identifier(other.to_string())
    }
}

impl From<usize> for TokenKind {
    fn from(other: usize) -> TokenKind {
        TokenKind::Integer(other)
    }
}

impl From<f64> for TokenKind {
    fn from(other: f64) -> TokenKind {
        TokenKind::Decimal(other)
    }
}

struct Tokenizer<'a> {
    cur_idx: usize,
    data: &'a str,
}

impl<'a> Tokenizer<'a> {
    fn new(data: &str) -> Tokenizer {
        Tokenizer { cur_idx: 0, data }
    }

    fn next_token(&mut self) -> ParseResult<Option<(TokenKind, usize, usize)>> {
        self.skip_whitespace();

        if self.data.is_empty() {
            Ok(None)
        } else {
            let start = self.cur_idx;
            let token = self.fetch_next_token()?;
            let end = self.cur_idx;

            Ok(Some((token, start, end)))
        }
    }

    fn fetch_next_token(&mut self) -> ParseResult<TokenKind> {
        let response = tokenize_next_token(self.data);

        if response.is_err() {
            Err(response.unwrap_err().index(self.cur_idx))
        } else {
            let (token, bytes_read) = response.unwrap();
            self.chomp(bytes_read);

            Ok(token)
        }
    }

    fn skip_whitespace(&mut self) {
        self.chomp(skip(self.data));
    }

    fn chomp(&mut self, num_bytes: usize) {
        self.data = &self.data[num_bytes..];
        self.cur_idx += num_bytes;
    }
}

/// Turn a string of valid Delphi code into a list of tokens, including the
/// location of that token's start and end point in the original source code.
///
/// Note the token indices represent the half-open interval `[start, end)`,
/// equivalent to `start .. end` in Rust.
pub fn tokenize(data: &str) -> ParseResult<Vec<(TokenKind, usize, usize)>> {
    let mut tokenizer = Tokenizer::new(data);
    let mut tokens = vec![];

    while let Some(token) = tokenizer.next_token()? {
        tokens.push(token);
    }

    Ok(tokens)
}

fn tokenize_next_token(data: &str) -> ParseResult<(TokenKind, usize)> {
    let next = match data.chars().next() {
        Some(c) => c,
        None => return Err(ParseError::kind(ParseErrorKind::UnexpectedEOF)),
    };

    Ok(match next {
        '.' => (TokenKind::Dot, 1),
        '=' => (TokenKind::Equals, 1),
        '+' => (TokenKind::Plus, 1),
        '-' => (TokenKind::Minus, 1),
        '*' => (TokenKind::Asterisk, 1),
        '/' => (TokenKind::Slash, 1),
        '@' => (TokenKind::At, 1),
        '^' => (TokenKind::Carat, 1),
        '(' => (TokenKind::OpenParen, 1),
        ')' => (TokenKind::CloseParen, 1),
        '[' => (TokenKind::OpenSquare, 1),
        ']' => (TokenKind::CloseSquare, 1),
        ':' => (TokenKind::Colon, 1),
        ';' => (TokenKind::Semicolon, 1),
        '0'..'9' => tokenize_number(data)?,
        c @ '_' | c if c.is_alphabetic() => tokenize_identifier(data)?,
        other => return Err(ParseError::kind(ParseErrorKind::UnknownCharacter(other))),
    })
}

fn tokenize_identifier(data: &str) -> ParseResult<(TokenKind, usize)> {
    validate_idenifier_char(data.chars().next())?;

    let (got, bytes_read) = take_while(data, |ch| ch == '_' || ch.is_alphanumeric())?;

    // TODO: Recognise keywords using a `match` statement here.

    Ok((TokenKind::Identifier(got.to_string()), bytes_read))
}

fn validate_idenifier_char(ch: Option<char>) -> ParseResult<()> {
    match ch {
        Some(ch) => {
            if ch.is_digit(10) {
                Err(ParseError::kind(ParseErrorKind::InvalidSymbol)
                    .msg("Identifiers can't start with numbers".to_string()))
            } else {
                Ok(())
            }
        }
        None => Err(ParseError::kind(ParseErrorKind::UnexpectedEOF)
            .msg("Invalid EOF while parsing file".to_string())),
    }
}

/// Tokenize a numeric literal.
fn tokenize_number(data: &str) -> ParseResult<(TokenKind, usize)> {
    let mut seen_dot = false;
    let (decimal, bytes_read) = take_while(data, |c| {
        if c.is_digit(10) {
            true
        } else if c == '.' {
            if !seen_dot {
                seen_dot = true;
                true
            } else {
                false
            }
        } else {
            false
        }
    })?;

    if seen_dot {
        Ok((TokenKind::Decimal(decimal.parse::<f64>()?), bytes_read))
    } else {
        Ok((TokenKind::Integer(decimal.parse::<usize>()?), bytes_read))
    }
}

fn skip(data: &str) -> usize {
    let mut remaining = data;

    loop {
        let mut skipped = 0;
        let skippers: Vec<&dyn Fn(&str) -> usize> = vec![&skip_whitespace, &skip_comments];
        for skipper in skippers {
            let to_skip = (skipper)(remaining);
            remaining = &remaining[to_skip..];
            skipped += to_skip;
        }

        if skipped == 0 {
            return data.len() - remaining.len();
        }
    }
}

fn skip_whitespace(data: &str) -> usize {
    match take_while(data, |c| c.is_whitespace()) {
        Ok((_, bytes_skipped)) => bytes_skipped,
        _ => 0,
    }
}

fn skip_comments(data: &str) -> usize {
    let comment_blocks = vec![("//", "\n"), ("{", "}"), ("(*", "*)")];

    for (start, end) in comment_blocks {
        if data.starts_with(start) {
            return data.len() - skip_untill(data, end).len();
        }
    }

    0
}

fn skip_untill<'a>(mut data: &'a str, pattern: &str) -> &'a str {
    while !data.is_empty() && !data.starts_with(pattern) {
        let next_char_size = data
            .chars()
            .next()
            .expect("The string isn't empty")
            .len_utf8();
        data = &data[next_char_size..];
    }
    &data[pattern.len()..]
}

/// Consumes bytes while predicate evaluates to true.
fn take_while<F>(data: &str, mut pred: F) -> ParseResult<(&str, usize)>
where
    F: FnMut(char) -> bool,
{
    let mut cur_idx = 0;

    for ch in data.chars() {
        if pred(ch) {
            cur_idx += ch.len_utf8();
        } else {
            break;
        }
    }

    if cur_idx == 0 {
        Err(ParseError::kind(ParseErrorKind::NoMatches))
    } else {
        Ok((&data[..cur_idx], cur_idx))
    }
}

#[allow(unused_macros)]
macro_rules! lexer_test {
    (FAIL: $name:ident, $func:ident, $input:expr) => {
        #[test]
        fn $name() {
            let response = $func($input);

            assert!(response.is_err(), "{:?} should be an error", response);
        }
    };
    ($name:ident, $func:ident, $input:expr => $expected:expr) => {
        #[test]
        fn $name() {
            let input: &str = $input;
            let expected = TokenKind::from($expected);

            let (response, _) = $func(input).unwrap();

            assert_eq!(expected, response, "Input was {:?}", input);
        }
    };
}

#[allow(unused_macros)]
macro_rules! comment_test {
    ($name:ident, $input:expr => $expected:expr) => {
        #[test]
        fn $name() {
            assert_eq!($expected, skip_comments($input));
        }
    };
}

#[allow(unused_macros)]
macro_rules! skip_test {
    ($name:ident, $input:expr => $expected:expr) => {
        #[test]
        fn $name() {
            assert_eq!($expected, skip($input));
        }
    };
}

#[cfg(test)]
mod tokenize_identifier_tests {
    use super::tokenize_identifier;
    use crate::parsers::delphi::lexer::TokenKind;

    lexer_test!(tokenize_a_single_letter, tokenize_identifier, "F" => "F");
    lexer_test!(tokenize_an_identifer, tokenize_identifier, "Foo" => "Foo");
    lexer_test!(tokenize_ident_containing_an_underscore, tokenize_identifier, "Foo_bar" => "Foo_bar");
    lexer_test!(tokenize_ident_can_end_with_number, tokenize_identifier, "Foo_bar7" => "Foo_bar7");
    lexer_test!(
        FAIL: tokenize_ident_cant_start_with_number,
        tokenize_identifier,
        "7Foo_bar"
    );
    lexer_test!(
        FAIL: tokenize_ident_cant_start_with_dot,
        tokenize_identifier,
        ".Foo_bar"
    );
}

#[cfg(test)]
mod tokenize_number_tests {
    use super::tokenize_number;
    use crate::parsers::delphi::lexer::TokenKind;

    lexer_test!(tokenize_a_single_digit_integer, tokenize_number, "1" => 1);
    lexer_test!(tokenize_a_longer_integer, tokenize_number, "1234567890" => 1234567890);
    lexer_test!(tokenize_basic_decimal, tokenize_number, "12.3" => 12.3);
    lexer_test!(tokenize_decimal_starting_with_dot, tokenize_number, ".3" => 0.3);
    lexer_test!(tokenize_decimal_ending_with_dot, tokenize_number, "3.0" => 3.0);
    lexer_test!(tokenize_string_with_multiple_decimal_points, tokenize_number, "12.3.456" => 12.3);
    lexer_test!(
        FAIL: cant_tokenize_a_string_as_a_decimal,
        tokenize_number,
        "asdfghj"
    );
    lexer_test!(tokenizing_decimal_stops_at_alpha, tokenize_number, "123.4asdfghj" => 123.4);
}

#[cfg(test)]
mod skip_whitespace_tests {
    use super::skip_whitespace;

    #[test]
    fn skip_past_several_whitespace_chars() {
        let src = " \t\n\r123";
        let should_be = 4;

        let num_skipped = skip_whitespace(src);
        assert_eq!(num_skipped, should_be);
    }

    #[test]
    fn skipping_whitespace_when_first_is_a_letter_returns_zero() {
        let src = "Hello World";
        let should_be = 0;

        let num_skipped = skip_whitespace(src);
        assert_eq!(num_skipped, should_be);
    }
}

#[cfg(test)]
mod comment_tests {
    use super::skip_comments;

    comment_test!(slash_slash_skips_to_end_of_line, "// foo bar { baz }\n 1234" => 19);
    comment_test!(comment_skip_curly_braces, "{ baz \n 1234} hello wor\nld" => 13);
    comment_test!(comment_skip_round_brackets, "(* Hello World *) asd" => 17);
    comment_test!(comment_skip_ignores_alphanumeric, "123 hello world" => 0);
    comment_test!(comment_skip_ignores_whitespace, "   (* *) 123 hello world" => 0);
}

#[cfg(test)]
mod skip_tests {
    use super::skip;

    skip_test!(slash_slash_skips_to_end_of_line, "// foo bar { baz }\n 1234" => 20);
    skip_test!(comment_skip_curly_braces, "{ baz \n 1234} hello wor\nld" => 14);
    skip_test!(comment_skip_round_brackets, "(* Hello World *) asd" => 18);
    skip_test!(comment_skip_ignores_alphanumeric, "123 hello world" => 0);
    skip_test!(comment_skip_ignores_whitespace, "   (* *) 123 hello world" => 9);
}

#[cfg(test)]
mod tokenize_next_token_tests {
    use super::tokenize_next_token;
    use crate::parsers::delphi::lexer::TokenKind;

    lexer_test!(central_tokenizer_integer, tokenize_next_token, "1234" => 1234);
    lexer_test!(central_tokenizer_decimal, tokenize_next_token, "123.4" => 123.4);
    lexer_test!(central_tokenizer_dot, tokenize_next_token, "." => TokenKind::Dot);
    lexer_test!(central_tokenizer_plus, tokenize_next_token, "+" => TokenKind::Plus);
    lexer_test!(central_tokenizer_minus, tokenize_next_token, "-" => TokenKind::Minus);
    lexer_test!(central_tokenizer_asterisk, tokenize_next_token, "*" => TokenKind::Asterisk);
    lexer_test!(central_tokenizer_slash, tokenize_next_token, "/" => TokenKind::Slash);
    lexer_test!(central_tokenizer_at, tokenize_next_token, "@" => TokenKind::At);
    lexer_test!(central_tokenizer_carat, tokenize_next_token, "^" => TokenKind::Carat);
    lexer_test!(central_tokenizer_equals, tokenize_next_token, "=" => TokenKind::Equals);
    lexer_test!(central_tokenizer_open_paren, tokenize_next_token, "(" => TokenKind::OpenParen);
    lexer_test!(central_tokenizer_close_paren, tokenize_next_token, ")" => TokenKind::CloseParen);
    lexer_test!(central_tokenizer_open_square, tokenize_next_token, "[" => TokenKind::OpenSquare);
    lexer_test!(central_tokenizer_close_square, tokenize_next_token, "]" => TokenKind::CloseSquare);
    lexer_test!(central_tokenizer_colon, tokenize_next_token, ":" => TokenKind::Colon);
    lexer_test!(central_tokenizer_semi_colon, tokenize_next_token, ";" => TokenKind::Semicolon);
}

#[cfg(test)]
mod tokenizer_tests {
    use super::tokenize;
    use crate::parsers::delphi::lexer::TokenKind;
    use crate::parsers::error::ParseErrorKind;

    #[test]
    fn tokenize_a_basic_expression() {
        let src = "foo = 1 + 2.34";
        let should_be = vec![
            (TokenKind::from("foo"), 0, 3),
            (TokenKind::Equals, 4, 5),
            (TokenKind::from(1), 6, 7),
            (TokenKind::Plus, 8, 9),
            (TokenKind::from(2.34), 10, 14),
        ];

        let got = tokenize(src).unwrap();
        assert_eq!(got, should_be);
    }

    #[test]
    fn tokenizer_detects_invalid_stuff() {
        let src = "foo bar `%^&\\";
        let index_of_backtick = 8;

        let err = tokenize(src).unwrap_err();
        assert_eq!(ParseErrorKind::UnknownCharacter('`'), err.kind);
        assert_eq!(Some(index_of_backtick), err.index);
    }
}
