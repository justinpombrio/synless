use editor::Ast;

#[derive(Clone)]
pub struct Prog<'l> {
    pub words: Vec<Thing<'l>>,
    pub name: Option<String>,
}

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
    pub fn multi(things: &[Thing<'l>]) -> Self {
        Prog {
            words: things.into(),
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
