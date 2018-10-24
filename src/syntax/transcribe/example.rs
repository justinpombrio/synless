use std::ops::Index;
use std::iter;

use super::super::syntax::Syntax;
use super::super::layout::BoundSet;
use super::transcribe::Document;


struct ExampleTree {
    contents: String,
    arity: usize,
    node: ExampleNode,
    syntax: Syntax,
    bounds: BoundSet<()>
}

enum ExampleNode {
    Branch(Vec<ExampleTree>),
    Leaf(String)
}

#[derive(Clone)]
struct ExampleTreeRef<'t> {
    root: &'t ExampleTree,
    path: Vec<usize>
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

impl<'t> Document for ExampleTreeRef<'t> {
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

    fn child(&self, i: usize) -> ExampleTreeRef<'t> {
        match &self.tree().node {
            ExampleNode::Branch(children) => {
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
                children.iter().enumerate().map(|(i, child)| {
                    ExampleTreeRef {
                        root: self.root,
                        path: extend_path(self.path.clone(), i)
                    }
                }).collect()
            }
            ExampleNode::Leaf(_) => panic!("leaf node")
        }
    }

    fn syntax(&self) -> &Syntax {
        &self.tree().syntax
    }
    
    fn bounds(&self) -> &BoundSet<()> {
        &self.tree().bounds
    }

    fn text(&self) -> Option<&str> {
        match &self.tree().node {
            ExampleNode::Branch(_) => None,
            ExampleNode::Leaf(s)   => Some(s)
        }
    }
}