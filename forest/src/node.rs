use utility::spanic;
use uuid::Uuid;

use self::NodeContents::*;

pub type Key = usize;

#[derive(Clone, Copy, Debug)]
pub struct Bookmark {
    pub(super) key: Key,
    pub(super) uuid: Uuid,
}

fn fresh_uuid() -> Uuid {
    Uuid::new_v4()
}

pub struct Node<D, L> {
    pub uuid: Uuid,
    pub parent: Option<Key>,
    contents: NodeContents<D, L>,
}

enum NodeContents<D, L> {
    Leaf(L),
    Branch(D, Vec<Key>),
}

impl<D, L> Node<D, L> {
    pub fn new_leaf(leaf: L) -> Node<D, L> {
        Node {
            uuid: fresh_uuid(),
            parent: None,
            contents: Leaf(leaf),
        }
    }

    pub fn new_branch(data: D, child_keys: Vec<Key>) -> Node<D, L> {
        Node {
            uuid: fresh_uuid(),
            parent: None,
            contents: Branch(data, child_keys),
        }
    }

    pub fn is_leaf(&self) -> bool {
        match &self.contents {
            Leaf(_) => true,
            Branch(_, _) => false,
        }
    }

    pub fn data(&self) -> &D {
        match &self.contents {
            Leaf(_) => spanic!("Forest - leaf node has no data!"),
            Branch(data, _) => data,
        }
    }

    pub fn data_mut(&mut self) -> &mut D {
        match &mut self.contents {
            Leaf(_) => spanic!("Forest - leaf node has no data!"),
            Branch(data, _) => data,
        }
    }

    pub fn leaf(&self) -> &L {
        match &self.contents {
            Leaf(leaf) => leaf,
            Branch(_, _) => spanic!("Forest - branch node has no leaf!"),
        }
    }

    pub fn leaf_mut(&mut self) -> &mut L {
        match &mut self.contents {
            Leaf(leaf) => leaf,
            Branch(_, _) => spanic!("Forest - branch node has no leaf!"),
        }
    }

    pub fn children(&self) -> &[Key] {
        match &self.contents {
            Leaf(_) => spanic!("Forest - leaf node has no children!"),
            Branch(_, children) => children,
        }
    }

    pub fn children_mut(&mut self) -> &mut Vec<Key> {
        match &mut self.contents {
            Leaf(_) => spanic!("Forest - leaf node has no children!"),
            Branch(_, children) => children,
        }
    }
}
