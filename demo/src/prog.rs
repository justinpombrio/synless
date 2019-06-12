use std::fmt;

use editor::Ast;

#[derive(Clone)]
pub struct Prog<'l> {
    pub words: Vec<Thing<'l>>,
    pub name: Option<String>,
}

#[allow(dead_code)]
#[derive(Clone)]
pub enum Thing<'l> {
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
    pub fn single(thing: Thing<'l>) -> Self {
        Prog {
            words: vec![thing],
            name: None,
        }
    }
    pub fn named(name: &str, things: &[Thing<'l>]) -> Self {
        Prog {
            words: things.into(),
            name: Some(name.to_owned()),
        }
    }
}

impl<'l> fmt::Display for Thing<'l> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Thing::Tree(..) => write!(f, "Tree"),
            Thing::Usize(n) => write!(f, "{}", n),
            Thing::MapName(s) => write!(f, "MapName(\"{}\")", s),
            Thing::NodeName(s) => write!(f, "NodeName(\"{}\")", s),
            Thing::Message(s) => write!(f, "Message(\"{}\")", s),
            Thing::Echo => write!(f, "Echo"),
            Thing::SelectNode => write!(f, "SelectNode"),
            Thing::NodeByName => write!(f, "NodeByName"),
            Thing::PushMap => write!(f, "PushMap"),
            Thing::PopMap => write!(f, "PopMap"),
            Thing::Remove => write!(f, "Remove"),
            Thing::InsertAfter => write!(f, "InsertAfter"),
            Thing::InsertBefore => write!(f, "InsertBefore"),
            Thing::InsertPrepend => write!(f, "InsertPrepend"),
            Thing::InsertPostpend => write!(f, "InsertPostpend"),
            Thing::Replace => write!(f, "Replace"),
            Thing::Left => write!(f, "Left"),
            Thing::Right => write!(f, "Right"),
            Thing::Parent => write!(f, "Parent"),
            Thing::Child => write!(f, "Child"),
            Thing::Undo => write!(f, "Undo"),
            Thing::Redo => write!(f, "Redo"),
        }
    }
}
