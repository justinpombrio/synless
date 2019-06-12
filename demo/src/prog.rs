use std::fmt;

use editor::Ast;

#[derive(Clone)]
pub struct Prog<'l> {
    pub words: Vec<Word<'l>>,
    pub name: Option<String>,
}

#[allow(dead_code)]
#[derive(Clone)]
pub enum Word<'l> {
    Tree(Ast<'l>),
    Usize(usize),
    MapName(String),
    NodeName(String),
    Message(String),
    InsertAfter,
    InsertBefore,
    InsertPrepend,
    InsertPostpend,
    Replace,
    Remove,
    Left,
    Right,
    Parent,
    Child,
    // Cut,
    // Copy,
    // PasteReplace,
    // PasteBefore,
    // PasteAfter,
    // PastePrepend,
    // PastePostpend,
    Undo,
    Redo,
    SelectNode,
    PushMap,
    PopMap,
    Echo,
    NodeByName,
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

impl<'l> fmt::Display for Word<'l> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Word::Tree(..) => write!(f, "Tree"),
            Word::Usize(n) => write!(f, "{}", n),
            Word::MapName(s) => write!(f, "MapName(\"{}\")", s),
            Word::NodeName(s) => write!(f, "NodeName(\"{}\")", s),
            Word::Message(s) => write!(f, "Message(\"{}\")", s),
            Word::Echo => write!(f, "Echo"),
            Word::SelectNode => write!(f, "SelectNode"),
            Word::NodeByName => write!(f, "NodeByName"),
            Word::PushMap => write!(f, "PushMap"),
            Word::PopMap => write!(f, "PopMap"),
            Word::Remove => write!(f, "Remove"),
            Word::InsertAfter => write!(f, "InsertAfter"),
            Word::InsertBefore => write!(f, "InsertBefore"),
            Word::InsertPrepend => write!(f, "InsertPrepend"),
            Word::InsertPostpend => write!(f, "InsertPostpend"),
            Word::Replace => write!(f, "Replace"),
            Word::Left => write!(f, "Left"),
            Word::Right => write!(f, "Right"),
            Word::Parent => write!(f, "Parent"),
            Word::Child => write!(f, "Child"),
            Word::Undo => write!(f, "Undo"),
            Word::Redo => write!(f, "Redo"),
        }
    }
}
