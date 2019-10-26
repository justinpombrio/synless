use editor::Ast;

use language::{ConstructName, LanguageName, Sort};

use crate::error::ServerError;
use crate::keymaps::{MenuName, ModeName};

#[derive(Clone, Debug)]
pub struct Prog<'l> {
    /// Optional display name for this program.
    pub name: Option<String>,
    /// A stack of words to execute, starting at the highest index.
    words: Vec<Word<'l>>,
}

pub struct DataStack<'l>(Vec<Value<'l>>);
pub struct CallStack<'l>(Vec<Prog<'l>>);

#[derive(Clone, Debug)]
pub enum Value<'l> {
    Tree(Ast<'l>),
    Usize(usize),
    Char(char),
    ModeName(ModeName),
    MenuName(MenuName),
    Sort(Sort),
    LangConstruct(LanguageName, ConstructName),
    String(String),
    Quote(Prog<'l>),
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
    PushMode,
    PopMode,
    ActivateMenu,
    SelfSort,
    ChildSort,
    SiblingSort,
    NodeByName,
    Print,

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
    /// Construct a program in which the given words will be executed in order,
    /// starting from index 0 of the slice.
    pub fn new(forward_words: Vec<Word<'l>>) -> Self {
        Prog {
            words: forward_words.into_iter().rev().collect(),
            name: None,
        }
    }

    /// Construct a program containing one word. Use the word's debug
    /// representation as the program name.
    pub fn new_single(word: Word<'l>) -> Self {
        Prog {
            name: Some(format!("{:?}", &word)),
            words: vec![word],
        }
    }

    /// Set the optional program name.
    pub fn with_name<T>(self, name: T) -> Self
    where
        T: ToString,
    {
        Prog {
            name: Some(name.to_string()),
            ..self
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

    /// Produce a literal value containing this program.
    pub fn quote(self) -> Value<'l> {
        Value::Quote(self)
    }
}

impl<'l> From<Word<'l>> for Prog<'l> {
    /// Unlike `Prog::single()`, this does not give the program a name.
    fn from(word: Word<'l>) -> Prog<'l> {
        Prog {
            name: None,
            words: vec![word],
        }
    }
}

impl<'l> CallStack<'l> {
    /// Construct a new empty callstack.
    pub fn new() -> Self {
        Self(Vec::new())
    }

    /// Push a program onto the call stack.
    pub fn push(&mut self, prog: Prog<'l>) {
        self.0.push(prog)
    }

    /// Return the next word to execute, removing it from the call stack. Or
    /// return None if the call stack is empty.
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
    /// Construct a new empty data stack.
    pub fn new() -> Self {
        Self(Vec::new())
    }

    /// Push this value onto the data stack.
    pub fn push(&mut self, value: Value<'l>) {
        self.0.push(value);
    }

    /// Pop a value from the data stack, returning it. If the stack is empty,
    /// return an error.
    pub fn pop(&mut self) -> Result<Value<'l>, ServerError<'l>> {
        self.0.pop().ok_or(ServerError::EmptyDataStack)
    }

    /// Swap the order of the two top-most values on the data stack. Return an
    /// error if there are less than 2 values on the data stack.
    pub fn swap(&mut self) -> Result<(), ServerError<'l>> {
        let first = self.pop()?;
        let maybe_second = self.pop();
        self.push(first);
        self.push(maybe_second?);
        Ok(())
    }

    /// If there is a `Value::Tree` on top of the stack, pop it and return it.
    /// Otherwise return an error.
    pub fn pop_tree(&mut self) -> Result<Ast<'l>, ServerError<'l>> {
        match self.pop()? {
            Value::Tree(tree) => Ok(tree),
            other => {
                self.push(other);
                Err(ServerError::ExpectedValue("Tree".into()))
            }
        }
    }

    /// If there is a `Value::Usize` on top of the stack, pop it and return it.
    /// Otherwise return an error.
    pub fn pop_usize(&mut self) -> Result<usize, ServerError<'l>> {
        match self.pop()? {
            Value::Usize(num) => Ok(num),
            other => {
                self.push(other);
                Err(ServerError::ExpectedValue("Usize".into()))
            }
        }
    }

    /// If there is a `Value::ModeName` on top of the stack, pop it and return it.
    /// Otherwise return an error.
    pub fn pop_mode_name(&mut self) -> Result<ModeName, ServerError<'l>> {
        match self.pop()? {
            Value::ModeName(s) => Ok(s),
            other => {
                self.push(other);
                Err(ServerError::ExpectedValue("ModeName".into()))
            }
        }
    }

    /// If there is a `Value::MenuName` on top of the stack, pop it and return
    /// it. Otherwise return an error.
    pub fn pop_menu_name(&mut self) -> Result<MenuName, ServerError<'l>> {
        match self.pop()? {
            Value::MenuName(s) => Ok(s),
            other => {
                self.push(other);
                Err(ServerError::ExpectedValue("MenuName".into()))
            }
        }
    }

    /// If there is a `Value::LangConstruct` on top of the stack, pop it and
    /// return it. Otherwise return an error.
    pub fn pop_lang_construct(&mut self) -> Result<(LanguageName, ConstructName), ServerError<'l>> {
        match self.pop()? {
            Value::LangConstruct(lang_name, construct_name) => Ok((lang_name, construct_name)),
            other => {
                self.push(other);
                Err(ServerError::ExpectedValue("LangConstruct".into()))
            }
        }
    }

    /// If there is a `Value::String` on top of the stack, pop it and return it.
    /// Otherwise return an error.
    pub fn pop_string(&mut self) -> Result<String, ServerError<'l>> {
        match self.pop()? {
            Value::String(s) => Ok(s),
            other => {
                self.push(other);
                Err(ServerError::ExpectedValue("String".into()))
            }
        }
    }

    /// If there is a `Value::Char` on top of the stack, pop it and return it.
    /// Otherwise return an error.
    pub fn pop_char(&mut self) -> Result<char, ServerError<'l>> {
        match self.pop()? {
            Value::Char(ch) => Ok(ch),
            other => {
                self.push(other);
                Err(ServerError::ExpectedValue("Char".into()))
            }
        }
    }

    /// If there is a `Value::Quote` on top of the stack, pop it and return it.
    /// Otherwise return an error.
    pub fn pop_quote(&mut self) -> Result<Prog<'l>, ServerError<'l>> {
        match self.pop()? {
            Value::Quote(prog) => Ok(prog),
            other => {
                self.push(other);
                Err(ServerError::ExpectedValue("Quote".into()))
            }
        }
    }
}
