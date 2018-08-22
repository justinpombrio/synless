use std::collections::HashMap;
use std::mem;
use std::iter::Iterator;
use uuid::Uuid;

use self::NodeContents::*;

// TODO: Reorganize into four files:
//       - mod
//       - forest: merges Forest&Node. Everything needed for unsafe tree ops.
//       - subtree_ref
//       - subtree_mut

// TODO: Note that it's up to the user to make sure that Trees are
// kept with the Forest they came from.


// Ids

type Id = Uuid;
fn fresh() -> Uuid {
    Uuid::new_v4()
}

// Forests

pub struct Forest<Data, Leaf>{
    map: HashMap<Id, Node<Data, Leaf>>
}

struct Node<Data, Leaf> {
    parent: Option<Id>,
    contents: NodeContents<Data, Leaf>
}

enum NodeContents<Data, Leaf> {
    Leaf(Leaf),
    Branch(Data, Vec<Id>)
}

impl<D, L> Node<D, L> {
    fn new_leaf(leaf: L) -> Node<D, L> {
        Node {
            parent: None,
            contents: Leaf(leaf)
        }
    }

    fn new_branch(data: D, children: Vec<Tree>) -> Node<D, L> {
        let children = children.into_iter().map(|child| child.id).collect();
        Node {
            parent: None,
            contents: Branch(data, children)
        }
    }

    fn is_leaf(&self) -> bool {
        match &self.contents {
            Leaf(_)      => true,
            Branch(_, _) => false
        }
    }

    fn data(&self) -> &D {
        match &self.contents {
            Leaf(_) => panic!("Forest - leaf node has no data!"),
            Branch(data, _) => data
        }
    }

    fn data_mut(&mut self) -> &mut D {
        match &mut self.contents {
            Leaf(_) => panic!("Forest - leaf node has no data!"),
            Branch(data, _) => data
        }
    }

    fn leaf(&self) -> &L {
        match &self.contents {
            Leaf(leaf) => leaf,
            Branch(_, _) => panic!("Forest - branch node has no leaf!")
        }
    }

    fn leaf_mut(&mut self) -> &mut L {
        match &mut self.contents {
            Leaf(leaf) => leaf,
            Branch(_, _) => panic!("Forest - branch node has no leaf!")
        }
    }

    fn children(&self) -> &Vec<Id> {
        match &self.contents {
            Leaf(_) => panic!("Forest - leaf node has no children!"),
            Branch(_, children) => children
        }
    }

    fn children_mut(&mut self) -> &mut Vec<Id> {
        match &mut self.contents {
            Leaf(_) => panic!("Forest - leaf node has no children!"),
            Branch(_, children) => children
        }
    }
}

impl<D, L> Forest<D, L> {
    fn get(&self, id: Id) -> &Node<D, L> {
        match self.map.get(&id) {
            Some(node) => node,
            None => panic!("Forest - id {} not found!", id)
        }
    }

    fn get_mut(&mut self, id: Id) -> &mut Node<D, L> {
        match self.map.get_mut(&id) {
            Some(node) => node,
            None => panic!("Forest - id {} not found!", id)
        }
    }

    fn remove(&mut self, id: Id) -> Node<D, L> {
        match self.map.remove(&id) {
            Some(node) => node,
            None => panic!("Forest - id {} not found!", id)
        }
    }

    fn root(&self, mut id: Id) -> Id {
        loop {
            match self.get(id).parent {
                None => return id,
                Some(parent) => {
                    id = parent;
                }
            }
        }
    }
}

// Trees

pub struct Tree {
    id: Id
}

pub struct Bookmark {
    id: Id
}

impl Tree {
    pub fn new_leaf<D, L>(f: &mut Forest<D, L>, leaf: L) -> Tree {
        let id = fresh();
        let node = Node::new_leaf(leaf);
        f.map.insert(id, node);
        Tree{id: id}
    }

    pub fn new_branch<D, L>(f: &mut Forest<D, L>, data: D, children: Vec<Tree>) -> Tree {
        let id = fresh();
        let node = Node::new_branch(data, children);
        f.map.insert(id, node);
        Tree{id: id}
    }

    pub fn as_ref<D, L>(&self) -> SubtreeRef {
        SubtreeRef {
            id: self.id,
            root: self
        }
    }

    pub fn as_mut<D, L>(&mut self) -> SubtreeMut {
        SubtreeMut {
            id: self.id,
            root: self
        }
    }

    pub fn delete<D, L>(self, f: &mut Forest<D, L>) {
        let node = f.remove(self.id);
        match node.contents {
            Leaf(leaf) => {
                mem::drop(leaf);
            }
            Branch(data, children) => {
                mem::drop(data);
                children.into_iter().for_each(|child| Tree{id: child}.delete(f));
            }
        };
        mem::forget(self)
    }
}

impl Drop for Tree {
    fn drop(&mut self) {
        panic!("Forest - a tree was not recycled! id:{}", self.id);
    }
}

// SubtreeRefs

pub struct SubtreeRef<'a> {
    root: &'a Tree,
    id: Id
}

pub struct RefChildrenIter<'a> {
    root: &'a Tree,
    children: Vec<Id>,
    index: usize
}

impl<'a> Iterator for RefChildrenIter<'a> {
    type Item = SubtreeRef<'a>;
    fn next(&mut self) -> Option<SubtreeRef<'a>> {
        if self.index >= self.children.len() {
            None
        } else {
            let subtree = SubtreeRef {
                root: self.root,
                id: self.children[self.index]
            };
            self.index += 1;
            Some(subtree)
        }
    }
}

impl<'a> SubtreeRef<'a> {

    // Data Access //

    pub fn is_leaf<D, L>(&self, f: &Forest<D, L>) -> bool {
        self.node(f).is_leaf()
    }
    
    // panics if this is not a branch node
    pub fn data<D, L>(&self, f: &'a Forest<D, L>) -> &'a D {
        &self.node(f).data()
    }

    // panics if this is not a leaf node
    pub fn leaf<D, L>(&self, f: &'a Forest<D, L>) -> &'a L {
        self.node(f).leaf()
    }

    // panics if this is not a branch node
    pub fn num_children<D, L>(&self, f: &Forest<D, L>) -> usize {
        self.node(f).children().len()
    }

    // Bookmarks //

    pub fn bookmark<D, L>(&self, _f: &Forest<D, L>) -> Bookmark {
        Bookmark {
            id: self.id
        }
    }

    pub fn lookup_bookmark<D, L>(&self, f: &Forest<D, L>, mark: Bookmark) -> Option<SubtreeRef<'a>> {
        if f.root(mark.id) == self.root.id {
            Some(SubtreeRef {
                root: self.root,
                id: mark.id
            })
        } else {
            None
        }
    }

    // Navigation //
    
    pub fn parent<D, L>(&self, f: &Forest<D, L>) -> Option<SubtreeRef<'a>> {
        match self.node(f).parent {
            None => None,
            Some(parent) => Some(SubtreeRef {
                root: self.root,
                id: parent
            })
        }
    }

    // panics if size is out of bounds, or if this isn't a leaf
    pub fn child<D, L>(&self, f: &Forest<D, L>, i: usize) -> SubtreeRef<'a> {
        match self.node(f).children().get(i) {
            None => panic!("Forest - child index out of bounds. id={}, i={}", self.id, i),
            Some(child) => SubtreeRef {
                root: self.root,
                id: *child
            }
        }
    }

    pub fn children<D, L>(&self, f: &Forest<D, L>) -> RefChildrenIter {
        let children = self.node(f).children();
        RefChildrenIter {
            root: self.root,
            children: children.clone(), // TODO: avoid clone?
            index: 0
        }
    }

    // Private //

    fn node<D, L>(&self, f: &'a Forest<D, L>) -> &'a Node<D, L> {
        &f.get(self.id)
    }
}


// SubtreeMuts

pub struct SubtreeMut<'a> {
    root: &'a mut Tree,
    id: Id
}

impl<'a> SubtreeMut<'a> {
    
    // Data Access //

    pub fn is_leaf<D, L>(&self, f: &Forest<D, L>) -> bool {
        self.node(f).is_leaf()
    }
    
    pub fn data<D, L>(&self, f: &'a Forest<D, L>) -> &'a D {
        &self.node(f).data()
    }

    // panics if this is not a leaf node
    pub fn leaf<D, L>(&self, f: &'a Forest<D, L>) -> &'a L {
        self.node(f).leaf()
    }

    // panics if this is not a branch node
    pub fn num_children<D, L>(&self, f: &Forest<D, L>) -> usize {
        self.node(f).children().len()
    }

    // Data Mutation //

    pub fn data_mut<D, L>(&mut self, f: &'a mut Forest<D, L>) -> &'a mut D {
        self.node_mut(f).data_mut()
    }

    // panics if this is not a leaf node
    pub fn leaf_mut<D, L>(&mut self, f: &'a mut Forest<D, L>) -> &'a L {
        self.node_mut(f).leaf_mut()
    }

    pub fn replace<D, L>(&mut self, f: &'a mut Forest<D, L>, i: usize, tree: Tree) -> Tree {
        let children = self.node_mut(f).children_mut();
        match children.get_mut(i) {
            None => panic!("Forest::replace - index out of bounds. id={}, i={}", self.id, i),
            Some(child) => {
                let result = Tree { id: *child };
                *child = tree.id;
                result
            }
        }
    }

    pub fn insert<D, L>(&mut self, f: &'a mut Forest<D, L>, i: usize, tree: Tree) {
        let children = self.node_mut(f).children_mut();
        if i > children.len() {
            panic!("Forest::insert - index out of bounds. id={}, i={}", self.id, i);
        }
        children.insert(i, tree.id);
    }

    pub fn remove<D, L>(&mut self, f: &'a mut Forest<D, L>, i: usize) -> Tree {
        let children = self.node_mut(f).children_mut();
        if i >= children.len() {
            panic!("Forest::remove - index out of bounds. id={}, i={}", self.id, i);
        }
        let child = children.remove(i);
        Tree { id: child }
    }

    // Bookmarks //

    pub fn bookmark<D, L>(&mut self, _f: &mut Forest<D, L>) -> Bookmark {
        Bookmark {
            id: self.id
        }
    }

    pub fn goto_bookmark<D, L>(&mut self, f: &mut Forest<D, L>, mark: Bookmark) -> bool {
        if f.root(mark.id) == self.root.id {
            self.id = mark.id;
            true
        } else {
            false
        }
    }

    // Navigation //

    pub fn goto_parent<D, L>(&mut self, f: &mut Forest<D, L>) -> bool {
        match self.node(f).parent {
            None => false,
            Some(parent) => {
                self.id = parent;
                true
            }
        }
    }

    // panics if size is out of bounds, or if this isn't a leaf
    pub fn goto_child<D, L>(&mut self, f: &mut Forest<D, L>, i: usize) {
        match self.node(f).children().get(i) {
            None => panic!("Forest - child index out of bounds. id={}, i={}", self.id, i),
            Some(child) => {
                self.id = *child;
            }
        }
    }

    // Private //

    fn node<D, L>(&self, f: &'a Forest<D, L>) -> &'a Node<D, L> {
        f.get(self.id)
    }

    fn node_mut<D, L>(&mut self, f: &'a mut Forest<D, L>) -> &'a mut Node<D, L> {
        f.get_mut(self.id)
    }
}
