use editor::Ast;

use language::{ConstructName, LanguageName, Sort};

use crate::error::Error;

#[derive(Clone)]
pub struct Prog<'l> {
    pub words: Vec<Word<'l>>,
    pub name: Option<String>,
}

pub struct DataStack<'l>(Vec<Value<'l>>);

#[derive(Clone, Debug)]
pub struct KmapSpec {
    pub name: String,
    pub required_sort: Sort,
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub enum Value<'l> {
    Tree(Ast<'l>),
    Usize(usize),
    Char(char),
    MapName(String),
    Sort(Sort),
    LangConstruct(LanguageName, ConstructName),
    Message(String),
    Quote(Box<Word<'l>>), // TODO vec, not box
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub enum Word<'l> {
    // data-stack manipulation:
    Swap,
    Apply,
    Pop,
    Literal(Value<'l>),

    // editor-specific:
    PushMap,
    PopMap,
    SelfSort,
    ChildSort,
    SiblingSort,
    AnySort,
    NodeByName,
    Echo,

    // tree commands:
    InsertHoleAfter,
    InsertHoleBefore,
    InsertHolePrepend,
    InsertHolePostpend,
    Replace,
    Remove,
    Clear,
    Left,
    Right,
    Parent,
    Child,
    Cut,
    Copy,
    PasteSwap,
    PopClipboard,
    Undo,
    Redo,
    GotoBookmark,
    SetBookmark,

    // text commands:
    InsertChar,
    TreeMode,
    DeleteCharBackward,
    DeleteCharForward,
    TextLeft,
    TextRight,
}

impl<'l> Prog<'l> {
    pub fn single(word: Word<'l>) -> Self {
        Prog {
            words: vec![word],
            name: None,
        }
    }
    pub fn named<T: ToString>(name: T, words: &[Word<'l>]) -> Self {
        Prog {
            words: words.into(),
            name: Some(name.to_string()),
        }
    }
}

impl<'l> DataStack<'l> {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn push(&mut self, value: Value<'l>) {
        self.0.push(value);
    }

    pub fn pop(&mut self) -> Result<Value<'l>, Error> {
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
        if let Value::Tree(tree) = self.pop()? {
            Ok(tree)
        } else {
            Err(Error::ExpectedValue("Tree".into()))
        }
    }

    pub fn pop_usize(&mut self) -> Result<usize, Error> {
        if let Value::Usize(num) = self.pop()? {
            Ok(num)
        } else {
            Err(Error::ExpectedValue("Usize".into()))
        }
    }

    pub fn pop_map_name(&mut self) -> Result<String, Error> {
        if let Value::MapName(s) = self.pop()? {
            Ok(s)
        } else {
            Err(Error::ExpectedValue("MapName".into()))
        }
    }

    pub fn pop_sort(&mut self) -> Result<Sort, Error> {
        if let Value::Sort(s) = self.pop()? {
            Ok(s)
        } else {
            Err(Error::ExpectedValue("Sort".into()))
        }
    }

    pub fn pop_lang_construct(&mut self) -> Result<(LanguageName, ConstructName), Error> {
        if let Value::LangConstruct(lang_name, construct_name) = self.pop()? {
            Ok((lang_name, construct_name))
        } else {
            Err(Error::ExpectedValue("LangConstruct".into()))
        }
    }

    pub fn pop_message(&mut self) -> Result<String, Error> {
        if let Value::Message(s) = self.pop()? {
            Ok(s)
        } else {
            Err(Error::ExpectedValue("Message".into()))
        }
    }

    pub fn pop_char(&mut self) -> Result<char, Error> {
        if let Value::Char(ch) = self.pop()? {
            Ok(ch)
        } else {
            Err(Error::ExpectedValue("Char".into()))
        }
    }

    pub fn pop_quote(&mut self) -> Result<Word<'l>, Error> {
        if let Value::Quote(word) = self.pop()? {
            Ok(*word)
        } else {
            Err(Error::ExpectedValue("Quote".into()))
        }
    }
}

impl<'l> Word<'l> {
    pub fn quote(self) -> Self {
        Word::Literal(Value::Quote(Box::new(self)))
    }
}
