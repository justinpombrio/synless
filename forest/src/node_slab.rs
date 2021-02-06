use super::node::{Key, Node};
use slab::Slab;
use std::ops::{Index, IndexMut};

pub struct NodeSlab<D, L> {
    slab: Slab<Node<D, L>>,
}

impl<D, L> Index<Key> for NodeSlab<D, L> {
    type Output = Node<D, L>;

    fn index(&self, key: Key) -> &Node<D, L> {
        match self.slab.get(key) {
            Some(node) => node,
            None => panic!("Forest - key {} not found!", key),
        }
    }
}

impl<D, L> IndexMut<Key> for NodeSlab<D, L> {
    fn index_mut(&mut self, key: Key) -> &mut Node<D, L> {
        match self.slab.get_mut(key) {
            Some(node) => node,
            None => panic!("Forest - key {} not found!", key),
        }
    }
}

impl<D, L> NodeSlab<D, L> {
    pub fn new() -> NodeSlab<D, L> {
        NodeSlab { slab: Slab::new() }
    }

    pub fn insert(&mut self, node: Node<D, L>) -> Key {
        self.slab.insert(node)
    }

    pub fn remove(&mut self, key: Key) -> Node<D, L> {
        if !self.slab.contains(key) {
            panic!("Forest - key {} not found!", key);
        }
        self.slab.remove(key)
    }

    pub fn contains(&self, key: Key) -> bool {
        self.slab.contains(key)
    }

    pub fn free_tree(&mut self, key: Key) {
        let mut to_free = vec![self.root_key(key)];
        while let Some(key) = to_free.pop() {
            let node = self.remove(key);
            if !node.is_leaf() {
                for child in node.children() {
                    to_free.push(*child);
                }
            }
        }
    }

    pub fn root_key(&self, key: Key) -> Key {
        let mut key = key;
        while let Some(parent_key) = self.slab[key].parent {
            key = parent_key;
        }
        key
    }

    pub fn root(&self, key: Key) -> &Node<D, L> {
        &self.slab[self.root_key(key)]
    }

    pub fn len(&self) -> usize {
        self.slab.len()
    }
}
