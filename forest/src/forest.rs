use std::collections::HashMap;
use std::mem;
use uuid::Uuid;

use self::NodeContents::*;

// INVARIANTS:
// - children and parents agree

pub type Id = Uuid;
fn fresh() -> Uuid {
    Uuid::new_v4()
}

pub struct RawForest<Data, Leaf> {
    map: HashMap<Id, Node<Data, Leaf>>,
    #[cfg(test)]
    refcount: usize,
}

struct Node<Data, Leaf> {
    parent: Option<Id>,
    contents: NodeContents<Data, Leaf>,
}

enum NodeContents<Data, Leaf> {
    Leaf(Leaf),
    Branch(Data, Vec<Id>),
}

impl<D, L> RawForest<D, L> {
    pub fn new() -> RawForest<D, L> {
        RawForest {
            map: HashMap::new(),
            #[cfg(test)]
            refcount: 0,
        }
    }

    // Navigation //

    pub fn parent(&self, id: Id) -> Option<Id> {
        self.get(id).parent
    }

    pub fn children(&self, id: Id) -> impl Iterator<Item = &Id> {
        match &self.get(id).contents {
            Leaf(_) => panic!("Forest - leaf node has no children!"),
            Branch(_, children) => children.iter(),
        }
    }

    pub fn child(&self, id: Id, index: usize) -> Id {
        match self.children(id).nth(index) {
            None => panic!("Forest - child index out of bounds. id={}, i={}", id, index),
            Some(child) => *child,
        }
    }

    pub fn root(&self, mut id: Id) -> Id {
        loop {
            match self.get(id).parent {
                None => return id,
                Some(parent) => {
                    id = parent;
                }
            }
        }
    }

    pub fn index(&self, id: Id) -> usize {
        match self.get(id).parent {
            None => return 0,
            Some(parent_id) => {
                for (index, &child_id) in self.children(parent_id).enumerate() {
                    if child_id == id {
                        return index;
                    }
                }
            }
        }
        panic!("Forest::index - id {} not found", id)
    }

    pub fn is_valid(&self, id: Id) -> bool {
        self.map.get(&id).is_some()
    }

    // Data Access //

    pub fn is_leaf(&self, id: Id) -> bool {
        match &self.get(id).contents {
            Leaf(_) => true,
            Branch(_, _) => false,
        }
    }

    pub fn data(&self, id: Id) -> &D {
        match &self.get(id).contents {
            Leaf(_) => panic!("Forest - leaf node has no data!"),
            Branch(data, _) => data,
        }
    }

    pub fn leaf(&self, id: Id) -> &L {
        match &self.get(id).contents {
            Leaf(leaf) => leaf,
            Branch(_, _) => panic!("Forest - branch node has no leaf!"),
        }
    }

    // Data Mutation //

    pub fn data_mut(&mut self, id: Id) -> &mut D {
        match &mut self.get_mut(id).contents {
            Leaf(_) => panic!("Forest - leaf node has no data!"),
            Branch(data, _) => data,
        }
    }

    pub fn leaf_mut(&mut self, id: Id) -> &mut L {
        match &mut self.get_mut(id).contents {
            Leaf(leaf) => leaf,
            Branch(_, _) => panic!("Forest - branch node has no leaf!"),
        }
    }

    pub fn children_mut(&mut self, id: Id) -> &mut Vec<Id> {
        match &mut self.get_mut(id).contents {
            Leaf(_) => panic!("Forest - leaf node has no children!"),
            Branch(_, children) => children,
        }
    }

    // Forest Mutation //

    pub fn create_branch(&mut self, data: D, children: Vec<Id>) -> Id {
        let id = fresh();
        #[cfg(test)]
        (self.refcount += 1);
        for child in &children {
            self.get_mut(*child).parent = Some(id);
        }
        let node = Node {
            parent: None,
            contents: Branch(data, children),
        };
        self.map.insert(id, node);
        id
    }

    pub fn create_leaf(&mut self, leaf: L) -> Id {
        let id = fresh();
        #[cfg(test)]
        (self.refcount += 1);
        let node = Node {
            parent: None,
            contents: Leaf(leaf),
        };
        self.map.insert(id, node);
        id
    }

    pub fn replace_child(&mut self, parent: Id, index: usize, new_child: Id) -> Id {
        self.get_mut(new_child).parent = Some(parent);
        let old_child = match self.children_mut(parent).get_mut(index) {
            None => panic!(
                "Forest::replace - child index out of bounds. id={}, i={}",
                parent, index
            ),
            Some(child) => {
                let old_child = *child;
                *child = new_child;
                old_child
            }
        };
        self.get_mut(old_child).parent = None;
        old_child
    }

    pub fn insert_child(&mut self, parent: Id, index: usize, new_child: Id) {
        self.get_mut(new_child).parent = Some(parent);
        let children = self.children_mut(parent);
        if index > children.len() {
            panic!(
                "Forest::insert - child index out of bounds. id={}, i={}",
                parent, index
            );
        }
        children.insert(index, new_child);
    }

    pub fn remove_child(&mut self, parent: Id, index: usize) -> Id {
        let child = {
            let children = self.children_mut(parent);
            if index >= children.len() {
                panic!(
                    "Forest::remove - child index out of bounds. id={}, i={}",
                    parent, index
                );
            }
            children.remove(index)
        };
        self.get_mut(child).parent = None;
        child
    }

    pub fn delete_tree(&mut self, id: Id) {
        let node = self.remove(id);
        #[cfg(test)]
        (self.refcount -= 1);
        match node.contents {
            Leaf(leaf) => {
                mem::drop(leaf);
            }
            Branch(data, children) => {
                mem::drop(data);
                children
                    .into_iter()
                    .for_each(|child| self.delete_tree(child));
            }
        };
    }

    // Private //

    fn get(&self, id: Id) -> &Node<D, L> {
        match self.map.get(&id) {
            Some(node) => node,
            None => panic!("Forest - id {} not found!", id),
        }
    }

    fn get_mut(&mut self, id: Id) -> &mut Node<D, L> {
        match self.map.get_mut(&id) {
            Some(node) => node,
            None => panic!("Forest - id {} not found!", id),
        }
    }

    fn remove(&mut self, id: Id) -> Node<D, L> {
        match self.map.remove(&id) {
            Some(node) => node,
            None => panic!("Forest - id {} not found!", id),
        }
    }

    // For Testing //

    #[cfg(test)]
    pub fn tree_count(&self) -> usize {
        if self.refcount != self.map.len() {
            panic!(
                "Forest - lost track of trees! Refcount: {}, Hashcount: {}",
                self.refcount,
                self.map.len()
            );
        }
        self.refcount
    }
}
