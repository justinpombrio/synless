use super::ParseError;
use crate::util::{SynlessBug, SynlessError};
use partial_pretty_printer as ppp;
use ppp::Pos;
use regex::Regex;
use std::cell::LazyCell;

thread_local! {
    static NUMBER_REGEX: LazyCell<Regex> =
        LazyCell::new(|| Regex::new("^-?(?:0|[1-9]\\d*)(?:\\.\\d+)?(?:[eE][+-]?\\d+)?").bug());
    static PLAIN_STRING_REGEX: LazyCell<Regex> =
        LazyCell::new(|| Regex::new(r#"^"[^"\\\x00-\x1F]*""#).bug());
}

#[derive(Debug, PartialEq, Eq)]
pub enum Token<'j> {
    True,
    False,
    Null,
    Number(&'j str),
    PlainString(&'j str),
    EscapedString(String),
    StartArray,
    EndArray,
    StartObject,
    EndObject,
    Comma,
    Colon,
}

// INVARIANT: offset always points at the correct source position to blame in an error message. (So
//   make sure not to advance it prematurely.)
pub struct Tokenizer<'j> {
    file_name: &'j str,
    source: &'j str,
    offset: usize,
}

impl<'j> Tokenizer<'j> {
    pub fn new(file_name: &'j str, source: &'j str) -> Tokenizer<'j> {
        Tokenizer {
            file_name,
            source,
            offset: 0,
        }
    }

    pub fn error(&self, message: &str, label: &str) -> SynlessError {
        ParseError::with_location(
            self.file_name.to_owned(),
            message.to_owned(),
            self.source,
            self.pos(),
            label.to_owned(),
        )
        .into()
    }

    fn escape_string(&mut self) -> Option<String> {
        let mut string = String::new();
        self.offset += 1; // Consume opening quote.
        while let Some(ch) = self.peek_char() {
            match ch {
                '\x00'..='\x1F' => return None,
                '"' => {
                    self.offset += 1;
                    return Some(string);
                }
                '\\' => {
                    self.offset += 1;
                    match self.peek_char() {
                        None => return None,
                        Some('u') => {
                            // Apparently there's some messiness around "surrogate pairs",
                            // where a single unicode code point can be encoded using two
                            // `\u` escape sequences even though it would fit in one.
                            // We don't handle that.
                            self.offset += 1;
                            if self.source.len() >= self.offset + 4 {
                                let hex = &self.source[self.offset..self.offset + 4];
                                let hex_u32 = match u32::from_str_radix(hex, 16) {
                                    Ok(hex_u32) => hex_u32,
                                    Err(_) => return None,
                                };
                                let escaped_char = char::from_u32(hex_u32)?;
                                self.offset += 4;
                                string.push(escaped_char);
                            } else {
                                return None;
                            }
                        }
                        Some(escape_char) => {
                            let escaped_char = match escape_char {
                                '"' => '"',
                                '\\' => '\\',
                                '/' => '/',
                                'b' => '\x08',
                                'f' => '\x0c',
                                'n' => '\n',
                                'r' => '\r',
                                't' => '\t',
                                _ => return None,
                            };
                            self.offset += 1;
                            string.push(escaped_char);
                        }
                    }
                }
                ch => {
                    self.offset += ch.len_utf8();
                    string.push(ch);
                }
            }
        }
        None
    }

    fn remaining(&self) -> &'j str {
        &self.source[self.offset..]
    }

    fn peek_char(&self) -> Option<char> {
        self.remaining().chars().next()
    }

    fn consume_whitespace(&mut self) {
        fn is_whitespace(ch: char) -> bool {
            ch == ' ' || ch == '\n' || ch == '\r' || ch == '\t'
        }

        while self.peek_char().map(is_whitespace).unwrap_or(false) {
            self.offset += 1;
        }
    }

    fn consume_char(&mut self, token: Token<'j>) -> Token<'j> {
        self.offset += 1;
        token
    }

    fn consume_constant(
        &mut self,
        constant: &str,
        token: Token<'j>,
    ) -> Result<Token<'j>, SynlessError> {
        if self.remaining().starts_with(constant) {
            self.offset += constant.len();
            Ok(token)
        } else {
            Err(self.error("Expected JSON value.", "invalid"))
        }
    }

    // Get the current position. Has to scan the source up to this point!
    fn pos(&self) -> Pos {
        let mut row = 0;
        let mut col: u16 = 0;
        let mut offset = 0;
        for ch in self.source.chars() {
            if offset >= self.offset {
                break;
            }
            let len = ch.len_utf8();
            if ch == '\n' {
                offset += len;
                row += 1;
                col = 0;
            } else {
                offset += len;
                col += len as u16;
            }
        }
        Pos { row, col }
    }
}

impl<'j> Iterator for Tokenizer<'j> {
    type Item = Result<Token<'j>, SynlessError>;

    fn next(&mut self) -> Option<Result<Token<'j>, SynlessError>> {
        use Token::*;

        self.consume_whitespace();
        match self.peek_char()? {
            'n' => Some(self.consume_constant("null", Null)),
            't' => Some(self.consume_constant("true", True)),
            'f' => Some(self.consume_constant("false", False)),
            '"' => {
                // Fast path for strings without escapes or control characters.
                if let Some(matched) = PLAIN_STRING_REGEX.with(|regex| regex.find(self.remaining()))
                {
                    self.offset += matched.len();
                    // Trim quotes.
                    Some(Ok(PlainString(&matched.as_str()[1..matched.len() - 1])))
                } else if let Some(string) = self.escape_string() {
                    // escape_string() advances self.offset.
                    Some(Ok(EscapedString(string)))
                } else {
                    Some(Err(self.error("Invalid string.", "invalid")))
                }
            }
            '0'..='9' | '.' | '-' => {
                if let Some(matched) = NUMBER_REGEX.with(|regex| regex.find(self.remaining())) {
                    self.offset += matched.len();
                    Some(Ok(Number(matched.as_str())))
                } else {
                    Some(Err(self.error("Invalid number.", "invalid")))
                }
            }
            '{' => Some(Ok(self.consume_char(StartObject))),
            '}' => Some(Ok(self.consume_char(EndObject))),
            '[' => Some(Ok(self.consume_char(StartArray))),
            ']' => Some(Ok(self.consume_char(EndArray))),
            ',' => Some(Ok(self.consume_char(Comma))),
            ':' => Some(Ok(self.consume_char(Colon))),
            _ => Some(Err(self.error("Expected JSON value.", "expected value"))),
        }
    }
}

#[test]
fn test_json_tokenizer() {
    #[track_caller]
    fn tokenize(source: &str) -> Vec<Token<'_>> {
        let tokenizer = Tokenizer::new("[test_json_tokenizer]", source);
        let mut tokens = Vec::new();
        for result in tokenizer {
            match result {
                Ok(token) => tokens.push(token),
                Err(err) => panic!("Tokenization test -- unexpected error:\n{}", err),
            }
        }
        tokens
    }

    #[track_caller]
    fn assert_tokens(source: &str, expected: &[Token]) {
        let tokens = tokenize(source);
        if tokens.len() < expected.len() {
            panic!("Tokenization test -- too few tokens");
        }
        if tokens.len() > expected.len() {
            panic!("Tokenization test -- too many tokens");
        }
        for (found_token, expected_token) in tokens.iter().zip(expected.iter()) {
            if found_token != expected_token {
                panic!(
                    "Tokenization test -- wrong token. Found {:?}, expected {:?}.",
                    found_token, expected_token
                )
            }
        }
    }

    #[track_caller]
    fn assert_invalid(source: &str, expected_message: &str) {
        let tokenizer = Tokenizer::new("[test_json_tokenizer]", source);
        for result in tokenizer {
            match result {
                Ok(_) => (),
                Err(err) => {
                    let actual_message = err.to_string();
                    if actual_message.contains(expected_message) {
                        return;
                    } else {
                        panic!("Tokenization test -- expected error message to contain '{expected_message}', but found error:\n{actual_message}");
                    }
                }
            }
        }
        panic!("Tokenization test -- expected error, but tokenization succeeded.")
    }

    // Punctuation and keywords
    assert_tokens(
        "   true[,  false}   {:]null   ",
        &[
            Token::True,
            Token::StartArray,
            Token::Comma,
            Token::False,
            Token::EndObject,
            Token::StartObject,
            Token::Colon,
            Token::EndArray,
            Token::Null,
        ],
    );

    // Valid numbers
    assert_tokens(
        "7 2.3e5 -555",
        &[
            Token::Number("7"),
            Token::Number("2.3e5"),
            Token::Number("-555"),
        ],
    );

    // Invalid numbers
    assert_invalid(".5", "Invalid number");
    assert_invalid("1.2.3", "Invalid number");
    assert_invalid("3e4.5", "Invalid number");

    // Error pos
    assert_invalid("[2\n3\n4\n     five]", "at 4:6: Expected JSON value");

    // Plain strings
    assert_tokens(
        "\"Hello, 'world'!\"",
        &[Token::PlainString("Hello, 'world'!")],
    );

    // Escaped strings
    assert_tokens(
        r#""NL \n" "\\" "\tTab""#,
        &[
            Token::EscapedString("NL \n".to_owned()),
            Token::EscapedString("\\".to_owned()),
            Token::EscapedString("\tTab".to_owned()),
        ],
    );
    assert_tokens(
        r#""\u12345""#,
        &[Token::EscapedString("\u{1234}5".to_owned())],
    );

    // Invalid strings
    assert_invalid("\"Hello\nWorld\"", "at 1:7: Invalid string.");
}
