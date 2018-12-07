use std::ops::Index;
use std::collections::HashMap;

use crate::notation::*;
use crate::style::{Style, Color};
use crate::layout::Bounds;
use crate::pretty::pretty_doc::PrettyDocument;

use self::ExampleNode::{Branch, Leaf};

// TODO: test horz concat


pub struct ExampleTree {
    arity: usize,
    node: ExampleNode,
    notation: Notation,
    bounds: Bounds
}

pub enum ExampleNode {
    Branch(Vec<ExampleTree>),
    Leaf(String)
}

#[derive(Clone)]
pub struct ExampleTreeRef<'t> {
    root: &'t ExampleTree,
    path: Vec<usize>
}

impl ExampleTree {
    pub fn new_branch(arity: usize, notation: Notation, children: Vec<ExampleTree>)
                      -> ExampleTree
    {
        let mut tree = ExampleTree {
            arity: arity,
            node: Branch(children),
            bounds: Bounds::empty(),
            notation: notation
        };
        tree.bounds = Bounds::compute(&tree.as_ref());
        tree
    }

    pub fn new_leaf(notation: Notation, contents: &str) -> ExampleTree {
        let mut tree = ExampleTree {
            arity: 0,
            node: Leaf(contents.to_string()),
            bounds: Bounds::empty(),
            notation: notation
        };
        tree.bounds = Bounds::compute(&tree.as_ref());
        tree
    }

    pub fn as_ref(&self) -> ExampleTreeRef {
        ExampleTreeRef {
            root: self,
            path: vec!()
        }
    }
}

impl<'a> Index<&'a [usize]> for ExampleTree {
    type Output = ExampleTree;
    fn index(&self, path: &[usize]) -> &ExampleTree {
        match &path {
            &[] => self,
            &[i, path..] => match &self.node {
                ExampleNode::Branch(children) => children[*i].index(path),
                ExampleNode::Leaf(_) => panic!("leaf node")
            }
        }
    }
}

impl<'t> ExampleTreeRef<'t> {
    fn tree(&self) -> &ExampleTree {
        &self.root[&self.path]
    }
}

fn shrink_path(mut path: Vec<usize>) -> Vec<usize> {
    path.pop();
    path
}

fn extend_path(mut path: Vec<usize>, i: usize) -> Vec<usize> {
    path.push(i);
    path
}

impl<'t> PrettyDocument for ExampleTreeRef<'t> {
    fn arity(&self) -> usize {
        self.tree().arity
    }

    fn parent(&self) -> Option<ExampleTreeRef<'t>> {
        if self.path.is_empty() {
            None
        } else {
            Some(ExampleTreeRef {
                root: self.root,
                path: shrink_path(self.path.clone())
            })
        }
    }

    // TODO: panic if index out of bounds
    fn child(&self, i: usize) -> ExampleTreeRef<'t> {
        match &self.tree().node {
            ExampleNode::Branch(_) => {
                ExampleTreeRef {
                    root: self.root,
                    path: extend_path(self.path.clone(), i)
                }
            }
            ExampleNode::Leaf(_) => panic!("leaf node")
        }
    }
    
    fn children(&self) -> Vec<ExampleTreeRef<'t>> {
        match &self.tree().node {
            ExampleNode::Branch(children) => {
                children.iter().enumerate().map(|(i, _)| {
                    ExampleTreeRef {
                        root: self.root,
                        path: extend_path(self.path.clone(), i)
                    }
                }).collect()
            }
            ExampleNode::Leaf(_) => panic!("leaf node")
        }
    }

    fn notation(&self) -> &Notation {
        &self.tree().notation
    }
    
    fn bounds(&self) -> Bounds {
        self.tree().bounds.clone()
    }

    fn text(&self) -> Option<&str> {
        match &self.tree().node {
            ExampleNode::Branch(_) => None,
            ExampleNode::Leaf(s)   => Some(s)
        }
    }
}



fn example_notation() -> HashMap<String, Notation> {
    fn punct(text: &str) -> Notation {
        literal(text, Style::color(Color::Base0A))
    }
    fn word(text: &str) -> Notation {
        literal(text, Style::color(Color::Base0D))
    }
    fn txt() -> Notation {
        text(Style::plain())
    }

    let mut map = HashMap::new();
    let note = repeat(Repeat{
        empty:  empty(),
        lone:   star(),
        first:  star() + punct(", "),
        middle: star() + punct(", "),
        last:   star()
    }) | repeat(Repeat{
        empty:  empty(),
        lone:   star(),
        first:  star() + punct(",") ^ empty(),
        middle: star() + punct(",") ^ empty(),
        last:   star()
    });
    map.insert("args".to_string(), note);

    let note = repeat(Repeat{
        empty:  punct("[]"),
        lone:   punct("[") + star() + punct("]"),
        first:  punct("[") + star() + punct(", "),
        middle: star() + punct(", "),
        last:   star() + punct("]")
    })| repeat(Repeat{
        empty:  punct("[]"),
        lone:   punct("[") + star() + punct("]"),
        first:  star() + punct(",") ^ empty(),
        middle: star() + punct(",") ^ empty(),
        last:   star() + punct("]")
    })| repeat(Repeat{
        empty:  punct("[]"),
        lone:   punct("[") + star() + punct("]"),
        first:  punct("[")
            + (star() + punct(", ") | star() + punct(",") ^ empty()),
        middle: star() + punct(", ") | star() + punct(",") ^ empty(),
        last:   star() + punct("]")
    });
    map.insert("list".to_string(), note);

    let note =
        word("func ") + child(0)
        + punct("(") + child(1) + punct(") { ") + child(2) + punct(" }")
        | word("func ") + child(0) + punct("(") + child(1) + punct(") {") ^ empty()
        + word("  ") + child(2) ^ empty()
        + punct("}")
        | word("func ") + child(0) + punct("(") ^ empty()
        + word("  ") + child(1) + punct(")") ^ empty()
        + punct("{") ^ empty()
        + word("  ") + child(2) ^ empty()
        + punct("}");
    map.insert("function".to_string(), note);

    let note =
        child(0) + punct(" + ") + child(1)
        | child(0) ^ punct("+ ") + child(1)
        | child(0) ^ punct("+") ^ child(1);
    map.insert("add".to_string(), note);

    let note = if_empty_text(txt() + punct("·"), txt());
    map.insert("id".to_string(), note);

    let note = punct("'") + txt() + punct("'");
    map.insert("string".to_string(), note);

    map
}

pub fn make_example_tree() -> ExampleTree {

    let notations = example_notation();

    let leaf = |construct: &str, contents: &str| -> ExampleTree {
        let note = notations.get(construct).unwrap().clone();
        ExampleTree::new_leaf(note, contents)
    };

    let branch = |construct: &str, children: Vec<ExampleTree>| -> ExampleTree {
        let note = notations.get(construct).unwrap().clone();
        ExampleTree::new_branch(children.len(), note, children)
    };

    branch("function", vec!(
        leaf("id", "foo"),
        branch("args", vec!(
            leaf("id", "abc"),
            leaf("id", "def"))),
        branch("add", vec!(
            leaf("string", "abcdef"),
            leaf("string", "abcdef")))))
}
