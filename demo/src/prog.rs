use editor::Ast;

use language::{ConstructName, LanguageName, Sort};

use crate::error::Error;

#[derive(Clone)]
pub struct Prog<'l> {
    pub words: Vec<Word<'l>>,
    pub name: Option<String>,
}

pub struct Stack<'l>(Vec<Word<'l>>);

#[derive(Clone, Debug)]
pub struct KmapSpec {
    pub name: String,
    pub required_sort: Sort,
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub enum Word<'l> {
    // data:
    Tree(Ast<'l>),
    Usize(usize),
    Char(char),
    MapName(String),
    Sort(Sort),
    LangConstruct(LanguageName, ConstructName),
    Message(String),
    Quote(Box<Word<'l>>),

    // stack manipulation:
    Swap,
    Apply,
    Pop,

    // editor-specific:
    PushMap,
    PopMap,
    SelfSort,
    ChildSort,
    SiblingSort,
    AnySort,
    NodeByName,
    Echo,

    // tree/text commands:
    InsertChar,
    InsertHoleAfter,
    InsertHoleBefore,
    InsertHolePrepend,
    InsertHolePostpend,
    Replace,
    Remove,
    Left,
    Right,
    Parent,
    Child,
    Cut,
    Copy,
    PasteReplace,
    Undo,
    Redo,
}

impl<'l> Prog<'l> {
    pub fn single(word: Word<'l>) -> Self {
        Prog {
            words: vec![word],
            name: None,
        }
    }
    pub fn named(name: &str, words: &[Word<'l>]) -> Self {
        Prog {
            words: words.into(),
            name: Some(name.to_owned()),
        }
    }
}

impl<'l> Stack<'l> {
    pub fn new() -> Self {
        Stack(Vec::new())
    }

    pub fn push(&mut self, word: Word<'l>) {
        self.0.push(word);
    }

    pub fn pop(&mut self) -> Result<Word<'l>, Error> {
        self.0.pop().ok_or(Error::EmptyStack)
    }

    pub fn swap(&mut self) -> Result<(), Error> {
        let first = self.pop()?;
        let second = self.pop()?;
        self.push(first);
        self.push(second);
        Ok(())
    }

    pub fn pop_tree(&mut self) -> Result<Ast<'l>, Error> {
        if let Word::Tree(tree) = self.pop()? {
            Ok(tree)
        } else {
            Err(Error::ExpectedWord("Tree".into()))
        }
    }

    pub fn pop_usize(&mut self) -> Result<usize, Error> {
        if let Word::Usize(num) = self.pop()? {
            Ok(num)
        } else {
            Err(Error::ExpectedWord("Usize".into()))
        }
    }

    pub fn pop_map_name(&mut self) -> Result<String, Error> {
        if let Word::MapName(s) = self.pop()? {
            Ok(s)
        } else {
            Err(Error::ExpectedWord("MapName".into()))
        }
    }

    pub fn pop_sort(&mut self) -> Result<Sort, Error> {
        if let Word::Sort(s) = self.pop()? {
            Ok(s)
        } else {
            Err(Error::ExpectedWord("Sort".into()))
        }
    }

    pub fn pop_lang_construct(&mut self) -> Result<(LanguageName, ConstructName), Error> {
        if let Word::LangConstruct(lang_name, construct_name) = self.pop()? {
            Ok((lang_name, construct_name))
        } else {
            Err(Error::ExpectedWord("LangConstruct".into()))
        }
    }

    pub fn pop_message(&mut self) -> Result<String, Error> {
        if let Word::Message(s) = self.pop()? {
            Ok(s)
        } else {
            Err(Error::ExpectedWord("Message".into()))
        }
    }

    pub fn pop_char(&mut self) -> Result<char, Error> {
        if let Word::Char(ch) = self.pop()? {
            Ok(ch)
        } else {
            Err(Error::ExpectedWord("Char".into()))
        }
    }

    pub fn pop_quote(&mut self) -> Result<Word<'l>, Error> {
        if let Word::Quote(word) = self.pop()? {
            Ok(*word)
        } else {
            Err(Error::ExpectedWord("Quote".into()))
        }
    }
}

impl<'l> Word<'l> {
    pub fn quote(self) -> Self {
        Word::Quote(Box::new(self))
    }
}
