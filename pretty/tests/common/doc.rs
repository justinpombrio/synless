use std::ops::Index;

use pretty::{Bounds, Notation, PlainText, PrettyDocument};

// TODO: rename `Doc` to `Tree`. Likewise `DocRef`.

use Node::{Branch, Leaf};

pub struct Doc {
    node: Node,
    notation: Notation,
    bounds: Bounds,
}

pub enum Node {
    Branch(Vec<Doc>),
    Leaf(String),
}

#[derive(Clone)]
pub struct DocRef<'t> {
    root: &'t Doc,
    path: Vec<usize>,
}

impl Doc {
    pub fn new_branch(notation: Notation, children: Vec<Doc>) -> Doc {
        let mut tree = Doc {
            node: Branch(children),
            bounds: Bounds::empty(),
            notation: notation,
        };
        tree.bounds = Bounds::compute(&tree.as_ref());
        tree
    }

    pub fn new_leaf(notation: Notation, contents: &str) -> Doc {
        let mut tree = Doc {
            node: Leaf(contents.to_string()),
            bounds: Bounds::empty(),
            notation: notation,
        };
        tree.bounds = Bounds::compute(&tree.as_ref());
        tree
    }

    pub fn as_ref(&self) -> DocRef {
        DocRef {
            root: self,
            path: vec![],
        }
    }

    pub fn write(&self, width: usize) -> String {
        let mut screen = PlainText::new(width);
        self.as_ref().pretty_print(&mut screen).unwrap();
        format!("{}", screen)
    }
}

impl<'a> Index<&'a [usize]> for Doc {
    type Output = Doc;
    fn index(&self, path: &[usize]) -> &Doc {
        match &path {
            [] => self,
            [i, path..] => match &self.node {
                Node::Branch(children) => children[*i].index(path),
                Node::Leaf(_) => panic!("leaf node"),
            },
        }
    }
}

impl<'t> DocRef<'t> {
    fn tree(&self) -> &Doc {
        &self.root[&self.path]
    }
}

fn pop_path(mut path: Vec<usize>) -> (Vec<usize>, usize) {
    let i = path.pop().expect("pretty::example: could not pop");
    (path, i)
}

fn extend_path(mut path: Vec<usize>, i: usize) -> Vec<usize> {
    path.push(i);
    path
}

impl<'t> PrettyDocument for DocRef<'t> {
    type TextRef = String;

    fn parent(&self) -> Option<(DocRef<'t>, usize)> {
        if self.path.is_empty() {
            None
        } else {
            let (path, i) = pop_path(self.path.clone());
            let doc = DocRef {
                root: self.root,
                path: path,
            };
            Some((doc, i))
        }
    }

    fn child(&self, i: usize) -> DocRef<'t> {
        match &self.tree().node {
            Node::Branch(_) => DocRef {
                root: self.root,
                path: extend_path(self.path.clone(), i),
            },
            Node::Leaf(_) => panic!("leaf node"),
        }
    }

    fn children(&self) -> Vec<DocRef<'t>> {
        match &self.tree().node {
            Node::Branch(children) => children
                .iter()
                .enumerate()
                .map(|(i, _)| DocRef {
                    root: self.root,
                    path: extend_path(self.path.clone(), i),
                })
                .collect(),
            Node::Leaf(_) => panic!("leaf node"),
        }
    }

    fn notation(&self) -> &Notation {
        &self.tree().notation
    }

    fn bounds(&self) -> Bounds {
        self.tree().bounds.clone()
    }

    fn text(&self) -> Option<String> {
        match &self.tree().node {
            Node::Branch(_) => None,
            Node::Leaf(s) => Some(s.to_string()),
        }
    }
}
