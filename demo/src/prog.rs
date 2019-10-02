use editor::Ast;

use language::{ConstructName, LanguageName, Sort};

use crate::error::Error;

#[derive(Clone)]
pub struct Prog<'l> {
    /// Optional display name for this program.
    pub name: Option<String>,
    /// A stack of words to execute, starting at the highest index.
    words: Vec<Word<'l>>,
}

pub struct DataStack<'l>(Vec<Value<'l>>);

pub struct CallStack<'l>(Vec<Prog<'l>>);

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
    /// Construct a program containing one word. Use the word's debug
    /// representation as the program name.
    pub fn single(word: Word<'l>) -> Self {
        Prog {
            name: Some(format!("{:?}", word)),
            words: vec![word],
        }
    }

    /// Construct a program in which the given words will be executed in order,
    /// starting from index 0 of the slice.
    pub fn named<T: ToString>(name: T, forward_words: &[Word<'l>]) -> Self {
        Prog {
            words: forward_words.iter().cloned().rev().collect(),
            name: Some(name.to_string()),
        }
    }

    /// Pop the next word from the program.
    pub fn pop(&mut self) -> Option<Word<'l>> {
        self.words.pop()
    }

    /// True if there are no words in the program
    pub fn is_empty(&self) -> bool {
        self.words.is_empty()
    }

    /// The display name of the program, if it has one.
    pub fn name(&self) -> Option<&str> {
        self.name.as_ref().map(String::as_str)
    }
}

impl<'l> CallStack<'l> {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    /// Push a program onto the call stack.
    pub fn push(&mut self, prog: Prog<'l>) {
        self.0.push(prog)
    }

    /// Return the next word to execute, removing it from the call stack. Or
    /// None if the call stack is empty.
    pub fn next(&mut self) -> Option<Word<'l>> {
        loop {
            let prog = self.0.last_mut()?;
            let word = prog.pop();
            if word.is_some() {
                if prog.is_empty() {
                    // Do a sort of tail-call optimization, immediately removing
                    // the empty program from the stack.
                    self.0.pop().expect("call stack shouldn't have been empty");
                }
                return word; // Success!
            }
            // Remove empty program and try again
            self.0.pop().expect("call stack shouldn't have been empty");
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
