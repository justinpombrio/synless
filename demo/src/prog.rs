use editor::Ast;

#[derive(Clone)]
pub struct Prog<'l> {
    pub words: Vec<Word<'l>>,
    pub name: Option<String>,
}

pub struct Stack<'l>(Vec<Word<'l>>);

#[derive(Clone, Debug)]
pub enum Word<'l> {
    Tree(Ast<'l>),
    Usize(usize),
    Char(char),
    MapName(String),
    NodeName(String),
    Message(String),
    InsertChar,
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
    Cut,
    Copy,
    PasteReplace,
    PasteBefore,
    PasteAfter,
    PastePrepend,
    PastePostpend,
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

impl<'l> Stack<'l> {
    pub fn new() -> Self {
        Stack(Vec::new())
    }

    pub fn push(&mut self, word: Word<'l>) {
        self.0.push(word);
    }

    pub fn pop_tree(&mut self) -> Ast<'l> {
        if let Some(Word::Tree(tree)) = self.0.pop() {
            tree
        } else {
            panic!("expected tree on stack")
        }
    }

    pub fn pop_usize(&mut self) -> usize {
        if let Some(Word::Usize(num)) = self.0.pop() {
            num
        } else {
            panic!("expected usize on stack")
        }
    }

    pub fn pop_map_name(&mut self) -> String {
        if let Some(Word::MapName(s)) = self.0.pop() {
            s
        } else {
            panic!("expected map name on stack")
        }
    }

    pub fn pop_node_name(&mut self) -> String {
        if let Some(Word::NodeName(s)) = self.0.pop() {
            s
        } else {
            panic!("expected node name on stack")
        }
    }

    pub fn pop_message(&mut self) -> String {
        if let Some(Word::Message(s)) = self.0.pop() {
            s
        } else {
            panic!("expected message on stack")
        }
    }
    pub fn pop_char(&mut self) -> char {
        if let Some(Word::Char(ch)) = self.0.pop() {
            ch
        } else {
            panic!("expected char on stack")
        }
    }
}
