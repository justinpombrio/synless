use std::collections::HashMap;
use std::ops::Index;

use crate::layout::Bounds;
use crate::notation::*;
use crate::pretty::pretty_doc::PrettyDocument;
use crate::style::{Color, Style};

use self::TestNode::{Branch, Leaf};

// TODO: test horz concat

pub struct TestTree {
    node: TestNode,
    notation: Notation,
    bounds: Bounds,
}

pub enum TestNode {
    Branch(Vec<TestTree>),
    Leaf(String),
}

#[derive(Clone)]
pub struct TestTreeRef<'t> {
    root: &'t TestTree,
    path: Vec<usize>,
}

impl TestTree {
    pub fn new_branch(notation: Notation, children: Vec<TestTree>) -> TestTree {
        let mut tree = TestTree {
            node: Branch(children),
            bounds: Bounds::empty(),
            notation: notation,
        };
        tree.bounds = Bounds::compute(&tree.as_ref());
        tree
    }

    pub fn new_leaf(notation: Notation, contents: &str) -> TestTree {
        let mut tree = TestTree {
            node: Leaf(contents.to_string()),
            bounds: Bounds::empty(),
            notation: notation,
        };
        tree.bounds = Bounds::compute(&tree.as_ref());
        tree
    }

    pub fn as_ref(&self) -> TestTreeRef {
        TestTreeRef {
            root: self,
            path: vec![],
        }
    }
}

impl<'a> Index<&'a [usize]> for TestTree {
    type Output = TestTree;
    fn index(&self, path: &[usize]) -> &TestTree {
        match &path {
            &[] => self,
            &[i, path..] => match &self.node {
                TestNode::Branch(children) => children[*i].index(path),
                TestNode::Leaf(_) => panic!("leaf node"),
            },
        }
    }
}

impl<'t> TestTreeRef<'t> {
    fn tree(&self) -> &TestTree {
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

impl<'t> PrettyDocument for TestTreeRef<'t> {
    type TextRef = String;

    fn parent(&self) -> Option<TestTreeRef<'t>> {
        if self.path.is_empty() {
            None
        } else {
            Some(TestTreeRef {
                root: self.root,
                path: shrink_path(self.path.clone()),
            })
        }
    }

    // TODO: panic if index out of bounds
    fn child(&self, i: usize) -> TestTreeRef<'t> {
        match &self.tree().node {
            TestNode::Branch(_) => TestTreeRef {
                root: self.root,
                path: extend_path(self.path.clone(), i),
            },
            TestNode::Leaf(_) => panic!("leaf node"),
        }
    }

    fn children(&self) -> Vec<TestTreeRef<'t>> {
        match &self.tree().node {
            TestNode::Branch(children) => children
                .iter()
                .enumerate()
                .map(|(i, _)| TestTreeRef {
                    root: self.root,
                    path: extend_path(self.path.clone(), i),
                })
                .collect(),
            TestNode::Leaf(_) => panic!("leaf node"),
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
            TestNode::Branch(_) => None,
            TestNode::Leaf(s) => Some(s.to_string()),
        }
    }
}

fn make_test_notation() -> HashMap<String, Notation> {
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
    let note = repeat(Repeat {
        empty: empty(),
        lone: child(0),
        join: child(0) + punct(", ") + child(1),
        surround: child(0),
    }) | repeat(Repeat {
        empty: empty(),
        lone: child(0),
        join: child(0) + punct(",") ^ child(1),
        surround: child(0),
    });
    map.insert("args".to_string(), note);

    let note = repeat(Repeat {
        empty: punct("[]"),
        lone: punct("[") + child(0) + punct("]"),
        join: child(0) + punct(", ") + child(1),
        surround: punct("[") + child(0) + punct("]"),
    }) | repeat(Repeat {
        empty: punct("[]"),
        lone: punct("[") + child(0) + punct("]"),
        join: child(0) + punct(",") ^ child(1),
        surround: punct("[") + child(0) + punct("]"),
    }) | repeat(Repeat {
        empty: punct("[]"),
        lone: punct("[") + child(0) + punct("]"),
        join: child(0) + punct(", ") + child(1) | child(0) + punct(",") ^ child(1),
        surround: punct("[") + child(0) + punct("]"),
    });
    map.insert("list".to_string(), note);

    let note =
        word("func ") + child(0) + punct("(") + child(1) + punct(") { ") + child(2) + punct(" }")
            | word("func ") + child(0) + punct("(") + child(1) + punct(") {")
                ^ empty() + word("  ") + child(2)
                ^ empty() + punct("}")
            | word("func ") + child(0) + punct("(")
                ^ empty() + word("  ") + child(1) + punct(")")
                ^ empty() + punct("{")
                ^ empty() + word("  ") + child(2)
                ^ empty() + punct("}");
    map.insert("function".to_string(), note);

    let note = child(0) + punct(" + ") + child(1)
        | child(0) ^ punct("+ ") + child(1)
        | child(0) ^ punct("+") ^ child(1);
    map.insert("add".to_string(), note);

    let note = if_empty_text(txt() + punct("·"), txt());
    map.insert("id".to_string(), note);

    let note = punct("'") + txt() + punct("'");
    map.insert("string".to_string(), note);

    map
}

pub fn make_test_tree() -> TestTree {
    let notations = make_test_notation();

    let leaf = |construct: &str, contents: &str| -> TestTree {
        let note = notations.get(construct).unwrap().clone();
        TestTree::new_leaf(note, contents)
    };

    let branch = |construct: &str, children: Vec<TestTree>| -> TestTree {
        let note = notations.get(construct).unwrap().clone();
        TestTree::new_branch(note, children)
    };

    branch(
        "function",
        vec![
            leaf("id", "foo"),
            branch("args", vec![leaf("id", "abc"), leaf("id", "def")]),
            branch(
                "add",
                vec![leaf("string", "abcdef"), leaf("string", "abcdef")],
            ),
        ],
    )
}
