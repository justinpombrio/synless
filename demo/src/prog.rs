use editor::Ast;

use language::{ConstructName, LanguageName, Sort};

use crate::error::ShellError;
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
    Message(String),
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
    /// Construct a program in which the given words will be executed in order,
    /// starting from index 0 of the slice.
    pub fn new(forward_words: &[Word<'l>]) -> Self {
        Prog {
            words: forward_words.iter().cloned().rev().collect(),
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
    pub fn pop(&mut self) -> Result<Value<'l>, ShellError> {
        self.0.pop().ok_or(ShellError::EmptyStack)
    }

    /// Swap the order of the two top-most values on the data stack. Return an
    /// error if there are less than 2 values on the data stack.
    pub fn swap(&mut self) -> Result<(), ShellError> {
        let first = self.pop()?;
        let maybe_second = self.pop();
        self.push(first);
        self.push(maybe_second?);
        Ok(())
    }

    /// Pop a value from the stack. If it has type `Value::Tree`, return it.
    /// Otherwise return an error.
    pub fn pop_tree(&mut self) -> Result<Ast<'l>, ShellError> {
        if let Value::Tree(tree) = self.pop()? {
            Ok(tree)
        } else {
            Err(ShellError::ExpectedValue("Tree".into()))
        }
    }

    /// Pop a value from the stack. If it has type `Value::Usize`, return it.
    /// Otherwise return an error.
    pub fn pop_usize(&mut self) -> Result<usize, ShellError> {
        if let Value::Usize(num) = self.pop()? {
            Ok(num)
        } else {
            Err(ShellError::ExpectedValue("Usize".into()))
        }
    }

    /// Pop a value from the stack. If it has type `Value::ModeName`, return it.
    /// Otherwise return an error.
    pub fn pop_mode_name(&mut self) -> Result<ModeName, ShellError> {
        if let Value::ModeName(s) = self.pop()? {
            Ok(s)
        } else {
            Err(ShellError::ExpectedValue("ModeName".into()))
        }
    }

    /// Pop a value from the stack. If it has type `Value::MenuName`, return it.
    /// Otherwise return an error.
    pub fn pop_menu_name(&mut self) -> Result<MenuName, ShellError> {
        if let Value::MenuName(s) = self.pop()? {
            Ok(s)
        } else {
            Err(ShellError::ExpectedValue("MenuName".into()))
        }
    }

    /// Pop a value from the stack. If it has type `Value::LangConstruct`, return it.
    /// Otherwise return an error.
    pub fn pop_lang_construct(&mut self) -> Result<(LanguageName, ConstructName), ShellError> {
        if let Value::LangConstruct(lang_name, construct_name) = self.pop()? {
            Ok((lang_name, construct_name))
        } else {
            Err(ShellError::ExpectedValue("LangConstruct".into()))
        }
    }

    /// Pop a value from the stack. If it has type `Value::Message`, return it.
    /// Otherwise return an error.
    pub fn pop_message(&mut self) -> Result<String, ShellError> {
        if let Value::Message(s) = self.pop()? {
            Ok(s)
        } else {
            Err(ShellError::ExpectedValue("Message".into()))
        }
    }

    /// Pop a value from the stack. If it has type `Value::Char`, return it.
    /// Otherwise return an error.
    pub fn pop_char(&mut self) -> Result<char, ShellError> {
        if let Value::Char(ch) = self.pop()? {
            Ok(ch)
        } else {
            Err(ShellError::ExpectedValue("Char".into()))
        }
    }

    /// Pop a value from the stack. If it has type `Value::Quote`, return it.
    /// Otherwise return an error.
    pub fn pop_quote(&mut self) -> Result<Prog<'l>, ShellError> {
        if let Value::Quote(prog) = self.pop()? {
            Ok(prog)
        } else {
            Err(ShellError::ExpectedValue("Quote".into()))
        }
    }
}
