use super::node::{Key, Node};
use slab::Slab;
use std::ops::{Index, IndexMut};
use std::sync::atomic::{AtomicUsize, Ordering};
use uuid::Uuid;

pub struct NodeSlab<D, L> {
    slab: Slab<Node<D, L>>,
    //#[cfg(test)]
    refcount: AtomicUsize,
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
        NodeSlab {
            slab: Slab::new(),
            refcount: AtomicUsize::new(0),
        }
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

    pub fn delete_tree(&mut self, key: Key) {
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

    // For Testing //

    //#[cfg(test)]
    pub fn inc_refcount(&self) {
        self.refcount.fetch_add(1, Ordering::SeqCst);
    }

    //#[cfg(test)]
    pub fn dec_refcount(&self) {
        self.refcount.fetch_sub(1, Ordering::SeqCst);
    }

    //#[cfg(test)]
    pub fn tree_count(&self) -> usize {
        let len = self.slab.len();
        let refcount = self.refcount.load(Ordering::SeqCst);
        if refcount != len {
            panic!(
                "Forest - lost track of trees! Refcount: {}, Slabcount: {}",
                refcount, len
            );
        }
        refcount
    }
}
