//! A custom Json parser.
//!
//! Why not use an existing one? We haven't found a Json parser written in Rust that has the
//! properties required to work well with an editor (as opposed to being read as data):
//!
//! - It must preserve the order of object keys.
//! - It must not de-duplicate object keys.
//! - For full fidelity, it should read numbers as strings (f64s have limited precision).

use super::json_tokenizer::{Token, Tokenizer};
use super::Parse;
use crate::language::{Construct, Language, Storage};
use crate::tree::Node;
use crate::util::{bug_assert, error, SynlessBug, SynlessError};

const LANGUAGE_NAME: &str = "json";
const PARSER_NAME: &str = "builtin_json_parser";

/*************
 * Interface *
 *************/

#[derive(Debug)]
pub struct JsonParser {
    // Must be lazily loaded, because the JSON language doesn't exist when `new()` is called!
    constructs: Option<JsonConstructs>,
}

impl JsonParser {
    pub fn new() -> JsonParser {
        JsonParser { constructs: None }
    }

    fn constructs(&mut self, s: &Storage) -> Result<&JsonConstructs, SynlessError> {
        if self.constructs.is_none() {
            let lang = s.language(LANGUAGE_NAME)?;
            let constructs = JsonConstructs::new(s, lang)?;
            self.constructs = Some(constructs);
        }
        Ok(self.constructs.as_ref().bug())
    }

}

impl Default for JsonParser {
    fn default() -> JsonParser {
        JsonParser::new()
    }
}

impl Parse for JsonParser {
    fn name(&self) -> &str {
        PARSER_NAME
    }

    fn parse(
        &mut self,
        s: &mut Storage,
        file_name: &str,
        source: &str,
    ) -> Result<Node, SynlessError> {
        let constructs = self.constructs(s)?;
        let tokens = Tokenizer::new(file_name, source);
        let mut builder = JsonBuilder { tokens };
        let json = builder.parse()?;
        let mut converter = JsonConverter {
            storage: s,
            constructs,
        };
        Ok(converter.json_to_tree(json))
    }
}

/******************
 * JSON Converter *
 ******************/

enum Json {
    Null,
    True,
    False,
    Number(String),
    String(String),
    Array(Vec<Json>),
    Object(Vec<(String, Json)>),
}

struct JsonConverter<'a> {
    constructs: &'a JsonConstructs,
    storage: &'a mut Storage,
}

impl<'a> JsonConverter<'a> {
    fn json_to_tree(&mut self, json: Json) -> Node {
        let node = self.convert(json);
        Node::with_children(self.storage, self.constructs.root, [node])
            .bug_msg("Wrong arity in json Root")
    }

    fn convert(&mut self, json: Json) -> Node {
        let c = self.constructs;

        match json {
            Json::Null => Node::new(self.storage, c.null),
            Json::True => Node::new(self.storage, c.c_true),
            Json::False => Node::new(self.storage, c.c_false),
            Json::Number(string) => {
                let node = Node::new(self.storage, c.number);
                node.text_mut(self.storage).bug().set(string);
                node
            }
            Json::String(string) => {
                let node = Node::new(self.storage, c.string);
                node.text_mut(self.storage).bug().set(string);
                node
            }
            Json::Array(elems) => {
                let array = Node::new(self.storage, c.array);
                for elem in elems {
                    let node = self.convert(elem);
                    bug_assert!(
                        array.insert_last_child(self.storage, node),
                        "Wrong arity in json Array");
                }
                array
            }
            Json::Object(pairs) => {
                let object = Node::new(self.storage, c.object);
                for (key, val) in pairs {
                    let key_node = Node::new(self.storage, c.key);
                    key_node.text_mut(self.storage).bug().set(key);
                    let val_node = self.convert(val);
                    let pair_node =
                        Node::with_children(self.storage, c.pair, [key_node, val_node])
                            .bug_msg("Wrong arity in json ObjectPair");
                    bug_assert!(
                       object.insert_last_child(self.storage, pair_node),
                     "Wrong arity in json Object");
                }
                object
            }
        }
    }
}

/**************
 * Constructs *
 **************/

/// Loads constructs from the language exactly once, when constructing the parser.
#[derive(Debug)]
struct JsonConstructs {
    root: Construct,
    null: Construct,
    c_false: Construct,
    c_true: Construct,
    string: Construct,
    number: Construct,
    array: Construct,
    object: Construct,
    key: Construct,
    pair: Construct,
}

impl JsonConstructs {
    fn new(s: &Storage, lang: Language) -> Result<JsonConstructs, SynlessError> {
        Ok(JsonConstructs {
            root: Self::lookup_construct(s, lang, "Root")?,
            null: Self::lookup_construct(s, lang, "Null")?,
            c_false: Self::lookup_construct(s, lang, "False")?,
            c_true: Self::lookup_construct(s, lang, "True")?,
            string: Self::lookup_construct(s, lang, "String")?,
            number: Self::lookup_construct(s, lang, "Number")?,
            array: Self::lookup_construct(s, lang, "Array")?,
            object: Self::lookup_construct(s, lang, "Object")?,
            key: Self::lookup_construct(s, lang, "Key")?,
            pair: Self::lookup_construct(s, lang, "ObjectPair")?,
        })
    }

    fn lookup_construct(
        s: &Storage,
        lang: Language,
        name: &str,
    ) -> Result<Construct, SynlessError> {
        match lang.construct(s, name) {
            Some(construct) => Ok(construct),
            None => Err(error!(
                Parse,
                "Construct '{}' missing from json language spec", name
            )),
        }
    }
}

/******************
 * Error Messages *
 ******************/

enum JsonError {
    ExpectedValueFoundComma,
    ExpectedValueFoundColon,
    ExpectedValueFoundEndArray,
    ExpectedValueFoundEndObject,
    ExpectedArrayComma,
    UnclosedArray,
    ExpectedObjectComma,
    ExpectedObjectColon,
    IncompletePair,
    NotAKey,
    UnclosedObject,
    TrailingComma,
    EmptyFile,
    ExtraToken,
}

impl JsonError {
    fn message_and_label(&self) -> (&'static str, &'static str) {
        use JsonError::*;

        match self {
            ExpectedValueFoundComma => ("Expected JSON value, found ','.", "unexpected"),
            ExpectedValueFoundColon => ("Expected JSON value, found ':'.", "unexpected"),
            ExpectedValueFoundEndArray => {
                ("Extra ']' does not match any opening '['.", "unmatched")
            }
            ExpectedValueFoundEndObject => {
                ("Extra '}' does not match any opening '{'.", "unmatched")
            }
            ExpectedArrayComma => (
                "Array elements must be separated by commas.",
                "expected ',' or ']'",
            ),
            UnclosedArray => ("Array not closed.", "expected ']'."),
            ExpectedObjectComma => (
                "Object elements must be separated by commas.",
                "expected ',' or '}'",
            ),
            ExpectedObjectColon => ("Object keys must be followed by ':'.", "expected ':'"),
            IncompletePair => ("Missing value after object key.", "expected JSON value"),
            NotAKey => ("Object key must be a string.", "expected string"),
            UnclosedObject => ("Object not closed.", "expected '}'"),
            TrailingComma => ("Trailing commas aren't allowed in JSON.", "trailing comma"),
            EmptyFile => ("File is empty.", "expected JSON value"),
            ExtraToken => ("Expected end of file, found extra token.", "unexpected"),
        }
    }
}

/***********
 * Builder *
 ***********/

/// Converts a token stream into a `Json` value.
// TODO: This parses recursively, and thus will overflow the stack on deeply nested arrays/objects.
// This is true of many JSON parsers, but we should aim higher.
struct JsonBuilder<'a> {
    tokens: Tokenizer<'a>,
}

impl<'a> JsonBuilder<'a> {
    /// Parse a JSON value, and ensure the file ends afterwards.
    fn parse(&mut self) -> Result<Json, SynlessError> {
        use JsonError::*;

        let token = match self.tokens.next() {
            None => return Err(self.error(EmptyFile)),
            Some(result) => result?,
        };
        let json = self.parse_value(token)?;
        if self.tokens.next().transpose()?.is_some() {
            return Err(self.error(ExtraToken));
        }
        Ok(json)
    }

    /// Parse a key value pair (like `"key" : 17`) that starts with the given token.
    fn parse_key_value_pair(&mut self, token: Token) -> Result<(String, Json), SynlessError> {
        use JsonError::*;
        use Token::*;

        let key = match token {
            // "key"
            PlainString(token) => token.to_owned(),
            // "key"
            EscapedString(token) => token,
            // 17
            _ => return Err(self.error(NotAKey)),
        };

        if let Some(token) = self.tokens.next().transpose()? {
            if token == Colon {
                // "key" :
                if let Some(token) = self.tokens.next().transpose()? {
                    // "key" : 17
                    let value = self.parse_value(token)?;
                    Ok((key, value))
                } else {
                    // "key": EOF
                    Err(self.error(IncompletePair))
                }
            } else {
                // "key" 17
                Err(self.error(ExpectedObjectColon))
            }
        } else {
            // "key" EOF
            Err(self.error(ExpectedObjectColon))
        }
    }

    /// Parse a JSON value that starts with the given token.
    fn parse_value(&mut self, token: Token) -> Result<Json, SynlessError> {
        use JsonError::*;
        use Token::*;

        match token {
            Null => Ok(Json::Null),
            True => Ok(Json::True),
            False => Ok(Json::False),
            Number(token) => Ok(Json::Number(token.to_owned())),
            PlainString(string) => Ok(Json::String(string.to_owned())),
            EscapedString(string) => Ok(Json::String(string)),
            StartArray => {
                // [
                let mut array = Vec::new();
                let token = match self.tokens.next().transpose()? {
                    Some(token) => token,
                    // [ EOF
                    None => return Err(self.error(UnclosedArray)),
                };
                if token == EndArray {
                    // [ ]
                    return Ok(Json::Array(array));
                }
                // [ 5
                let elem = self.parse_value(token)?;
                array.push(elem);
                while let Some(token) = self.tokens.next().transpose()? {
                    if token == EndArray {
                        // [ 5 ]
                        return Ok(Json::Array(array));
                    }
                    if token != Comma {
                        // [ 5 6
                        return Err(self.error(ExpectedArrayComma));
                    }
                    // [ 5 ,
                    let token = match self.tokens.next().transpose()? {
                        Some(token) => token,
                        // [ 5 , EOF
                        None => return Err(self.error(UnclosedArray)),
                    };
                    if token == EndArray {
                        // [ 5 , ]
                        return Err(self.error(TrailingComma));
                    }
                    // [ 5 , 6
                    let elem = self.parse_value(token)?;
                    array.push(elem);
                }
                // [ 5 EOF
                Err(self.error(UnclosedArray))
            }
            StartObject => {
                // {
                let mut object = Vec::new();
                let token = match self.tokens.next().transpose()? {
                    Some(token) => token,
                    // { EOF
                    None => return Err(self.error(UnclosedObject)),
                };
                if token == EndObject {
                    // { }
                    return Ok(Json::Object(object));
                }
                // { "key" : "value"
                let pair = self.parse_key_value_pair(token)?;
                object.push(pair);
                while let Some(token) = self.tokens.next().transpose()? {
                    if token == EndObject {
                        // { "key" : "value" }
                        return Ok(Json::Object(object));
                    }
                    if token != Comma {
                        // { "key" : "value" 17
                        return Err(self.error(ExpectedObjectComma));
                    }
                    // { "key" : "value" ,
                    let token = match self.tokens.next().transpose()? {
                        Some(token) => token,
                        // { "key" : "value" , EOF
                        None => return Err(self.error(UnclosedObject)),
                    };
                    if token == EndObject {
                        // { "key" : "value" , }
                        return Err(self.error(TrailingComma));
                    }
                    // { "key" : "value", "key2" : "value2"
                    let pair = self.parse_key_value_pair(token)?;
                    object.push(pair);
                }
                // { "key" : "value" EOF
                Err(self.error(UnclosedObject))
            }
            Comma => Err(self.error(ExpectedValueFoundComma)),
            Colon => Err(self.error(ExpectedValueFoundColon)),
            EndArray => Err(self.error(ExpectedValueFoundEndArray)),
            EndObject => Err(self.error(ExpectedValueFoundEndObject)),
        }
    }

    fn error(&self, error: JsonError) -> SynlessError {
        let (message, label) = error.message_and_label();
        self.tokens.error(message, label)
    }
}
