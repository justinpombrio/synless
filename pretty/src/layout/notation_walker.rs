use crate::notation::{Notation, RepeatInner};
use crate::style::Style;

use self::NotationCase::*;

pub struct Bounds();

pub struct NotationWalker<'a> {
    children: &'a [Bounds],
    is_empty_text: bool,
}

pub enum NotationCase<'a> {
    Empty,
    Literal(&'a str, Style),
    Text(Style),
    Child(usize),
    Nest(Vec<NotationNode<'a>>),
    Vert(Vec<NotationNode<'a>>),
    IfFlat(NotationNode<'a>, NotationNode<'a>),
    Join {
        left: NotationNode<'a>,
        right: NotationNode<'a>,
        join: NotationNode<'a>,
    },
    Left,
    Right,
    Surround {
        surrounded: NotationNode<'a>,
        surround: NotationNode<'a>,
    },
    Surrounded,
}

pub struct NotationNode<'a>(Node<'a>);

enum Node<'a> {
    Basic(&'a Notation),
    Child(usize),
    Join(&'a Notation, usize),
}

impl<'a> NotationNode<'a> {
    pub fn new(notation: &'a Notation) -> NotationNode<'a> {
        NotationNode(Node::Basic(notation))
    }
}

impl<'a> NotationWalker<'a> {
    pub fn new(children: &'a [Bounds], is_empty_text: bool) -> NotationWalker<'a> {
        NotationWalker {
            children,
            is_empty_text,
        }
    }

    pub fn walk(&self, node: NotationNode<'a>) -> NotationCase<'a> {
        match &node.0 {
            Node::Join(_, 0) => Child(0),
            Node::Join(join, i) => Join {
                left: NotationNode(Node::Join(join, i - 1)),
                right: NotationNode(Node::Child(*i)),
                join: NotationNode::new(join),
            },
            Node::Child(i) => Child(*i),
            Node::Basic(notation) => match notation {
                Notation::Empty => Empty,
                Notation::Literal(string, style) => Literal(string, *style),
                Notation::Text(style) => Text(*style),
                Notation::Child(i) => Child(*i),
                Notation::Nest(notations) => {
                    Nest(notations.iter().map(|n| NotationNode::new(n)).collect())
                }
                Notation::Vert(notations) => {
                    Vert(notations.iter().map(|n| NotationNode::new(n)).collect())
                }
                Notation::IfEmptyText(notation1, notation2) => {
                    if self.is_empty_text {
                        self.walk(NotationNode::new(notation1))
                    } else {
                        self.walk(NotationNode::new(notation2))
                    }
                }
                Notation::IfFlat(notation1, notation2) => {
                    IfFlat(NotationNode::new(notation1), NotationNode::new(notation2))
                }
                Notation::Empty => Empty,
                Notation::Surrounded => Surrounded,
                Notation::Left => Left,
                Notation::Right => Right,
                Notation::Repeat(box RepeatInner {
                    empty,
                    lone,
                    join,
                    surround,
                }) => match self.children.len() {
                    0 => self.walk(NotationNode::new(empty)),
                    1 => self.walk(NotationNode::new(lone)),
                    n => Surround {
                        surround: NotationNode::new(surround),
                        surrounded: NotationNode(Node::Join(join, n - 1)),
                    },
                },
            },
        }
    }
}
