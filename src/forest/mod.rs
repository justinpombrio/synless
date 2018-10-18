//! A general represenation of trees.

mod tree;
mod subtree_ref;
mod subtree_mut;

pub use self::tree::{Tree, Forest, ReadLeaf, WriteLeaf, ReadData, WriteData};
pub use self::subtree_ref::SubtreeRef;
pub use self::subtree_mut::SubtreeMut;

use std::collections::HashMap;
use std::mem;
use uuid::Uuid;

use self::NodeContents::*;


// INVARIANTS:
// - children and parents agree

type Id = Uuid;
fn fresh() -> Uuid {
    Uuid::new_v4()
}

struct RawForest<Data, Leaf>{
    map: HashMap<Id, Node<Data, Leaf>>,
    #[cfg(test)]
    refcount: usize
}

struct Node<Data, Leaf> {
    parent: Option<Id>,
    contents: NodeContents<Data, Leaf>
}

enum NodeContents<Data, Leaf> {
    Leaf(Leaf),
    Branch(Data, Vec<Id>)
}

impl<D, L> RawForest<D, L> {

    fn new() -> RawForest<D, L> {
        RawForest {
            map: HashMap::new(),
            #[cfg(test)]
            refcount: 0
        }
    }
    
    // Navigation //

    fn parent(&self, id: Id) -> Option<Id> {
        self.get(id).parent
    }
    
    fn children(&self, id: Id) -> &Vec<Id> {
        match &self.get(id).contents {
            Leaf(_) => panic!("Forest - leaf node has no children!"),
            Branch(_, children) => children
        }
    }

    fn child(&self, id: Id, index: usize) -> Id {
        match self.children(id).get(index) {
            None => panic!("Forest - child index out of bounds. id={}, i={}", id, index),
            Some(child) => *child
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

    fn is_valid(&self, id: Id) -> bool {
        self.map.get(&id).is_some()
    }

    // Data Access //

    fn is_leaf(&self, id: Id) -> bool {
        match &self.get(id).contents {
            Leaf(_)      => true,
            Branch(_, _) => false
        }
    }

    fn data(&self, id: Id) -> &D {
        match &self.get(id).contents {
            Leaf(_) => panic!("Forest - leaf node has no data!"),
            Branch(data, _) => data
        }
    }

    fn leaf(&self, id: Id) -> &L {
        match &self.get(id).contents {
            Leaf(leaf) => leaf,
            Branch(_, _) => panic!("Forest - branch node has no leaf!")
        }
    }

    // Data Mutation //

    fn data_mut(&mut self, id: Id) -> &mut D {
        match &mut self.get_mut(id).contents {
            Leaf(_) => panic!("Forest - leaf node has no data!"),
            Branch(data, _) => data
        }
    }

    fn leaf_mut(&mut self, id: Id) -> &mut L {
        match &mut self.get_mut(id).contents {
            Leaf(leaf) => leaf,
            Branch(_, _) => panic!("Forest - branch node has no leaf!")
        }
    }

    fn children_mut(&mut self, id: Id) -> &mut Vec<Id> {
        match &mut self.get_mut(id).contents {
            Leaf(_) => panic!("Forest - leaf node has no children!"),
            Branch(_, children) => children
        }
    }

    // Forest Mutation //

    fn create_branch(&mut self, data: D, children: Vec<Id>) -> Id {
        let id = fresh();
        #[cfg(test)] (self.refcount += 1);
        for child in &children {
            self.get_mut(*child).parent = Some(id);
        }
        let node = Node {
            parent: None,
            contents: Branch(data, children)
        };
        self.map.insert(id, node);
        id
    }

    fn create_leaf(&mut self, leaf: L) -> Id {
        let id = fresh();
        #[cfg(test)] (self.refcount += 1);
        let node = Node {
            parent: None,
            contents: Leaf(leaf)
        };
        self.map.insert(id, node);
        id
    }
    
    fn replace_child(&mut self, parent: Id, index: usize, new_child: Id) -> Id {
        self.get_mut(new_child).parent = Some(parent);
        let old_child = match self.children_mut(parent).get_mut(index) {
            None => panic!("Forest::replace - child index out of bounds. id={}, i={}", parent, index),
            Some(child) => {
                let old_child = *child;
                *child = new_child;
                old_child
            }
        };
        self.get_mut(old_child).parent = None;
        old_child
    }

    fn insert_child(&mut self, parent: Id, index: usize, new_child: Id) {
        self.get_mut(new_child).parent = Some(parent);
        let children = self.children_mut(parent);
        if index > children.len() {
            panic!("Forest::insert - child index out of bounds. id={}, i={}", parent, index);
        }
        children.insert(index, new_child);
    }

    fn remove_child(&mut self, parent: Id, index: usize) -> Id {
        let child = {
            let children = self.children_mut(parent);
            if index >= children.len() {
                panic!("Forest::remove - child index out of bounds. id={}, i={}", parent, index);
            }
            children.remove(index)
        };
        self.get_mut(child).parent = None;
        child
    }

    fn delete_tree(&mut self, id: Id) {
        let node = self.remove(id);
        #[cfg(test)] (self.refcount -= 1);
        match node.contents {
            Leaf(leaf) => {
                mem::drop(leaf);
            }
            Branch(data, children) => {
                mem::drop(data);
                children.into_iter().for_each(|child| self.delete_tree(child));
            }
        };
    }

    // Private //

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

    // For Testing //

    #[cfg(test)]
    fn tree_count(&self) -> usize {
        if self.refcount != self.map.len() {
            panic!("Forest - lost track of trees! Refcount: {}, Hashcount: {}",
                   self.refcount, self.map.len());
        }
        self.refcount
    }
}



#[cfg(test)]
mod test {
    use super::*;

    fn family<'f>(forest: &'f Forest<&'static str, &'static str>)
                  -> Tree<'f, &'static str, &'static str>
    {
        let leaves = vec!(forest.new_leaf("elder"),
                          forest.new_leaf("younger"));
        forest.new_branch("parent", leaves)
    }

    fn mirror<'f>(forest: &Forest<u32, u32>, height: u32, id: u32) -> Tree<u32, u32> {
        if height == 0 {
            forest.new_leaf(id)
        } else {
            let mut children = vec!();
            for i in 0..height {
                children.push(mirror(forest, i, id + 2_u32.pow(i)));
            }
            forest.new_branch(id, children)
        }
    }

    impl<'f> SubtreeRef<'f, u32, u32> {
        fn sum(&self) -> u32 {
            if self.is_leaf() {
                *self.leaf()
            } else {
                let mut sum = *self.data();
                for child in self.children() {
                    sum += child.sum();
                }
                sum
            }
        }
    }

    #[test]
    fn test_leaves() {
        let forest: Forest<(), u32> = Forest::new();
        // Begin with a leaf of 2
        let mut tree = forest.new_leaf(2);
        assert!(tree.as_mut().is_leaf()); // check SubtreeMut
        assert_eq!(*tree.as_mut().leaf(), 2);
        // Mutate it to be 3
        *tree.as_mut().leaf_mut() = 3;
        assert!(tree.as_ref().is_leaf()); // check SubtreeRef
        assert_eq!(*tree.as_ref().leaf(), 3);
    }

    #[test]
    fn test_data() {
        let forest: Forest<u32, ()> = Forest::new();
        // Begin with data of 2
        let mut tree = forest.new_branch(2, vec!());
        assert!(!tree.as_ref().is_leaf()); // check SubtreeRef
        assert_eq!(*tree.as_ref().data(), 2);
        // Mutate it to be 3
        *tree.as_mut().data_mut() = 3;
        assert!(!tree.as_mut().is_leaf()); // check SubtreeMut
        assert_eq!(*tree.as_mut().data(), 3);
    }

    #[test]
    fn test_num_children() {
        let forest: Forest<(), ()> = Forest::new();
        let leaves = vec!(forest.new_leaf(()),
                          forest.new_leaf(()),
                          forest.new_leaf(()));
        println!("before");
        let mut tree = forest.new_branch((), leaves);
        println!("after");
        assert_eq!(tree.as_ref().num_children(), 3);
        assert_eq!(tree.as_mut().num_children(), 3);
    }

    #[test]
    fn test_navigation_ref() {
        let forest: Forest<&'static str, &'static str> = Forest::new();
        let tree = family(&forest);
        assert_eq!(*tree.as_ref().child(0).leaf(), "elder");
        assert_eq!(*tree.as_ref().child(1).leaf(), "younger");
        assert_eq!(*tree.as_ref().child(0).parent().unwrap().data(), "parent");
        assert!(tree.as_ref().child(0).parent().unwrap().parent().is_none());
        let children: Vec<&'static str> = tree.as_ref()
            .children()
            .map(|child| *child.leaf())
            .collect();
        assert_eq!(children, vec!("elder", "younger"));
    }

    #[test]
    fn test_at_root_mut() {
        let forest: Forest<&'static str, &'static str> = Forest::new();
        let mut tree = family(&forest);
        {
            let mut cursor = tree.as_mut();
            assert!(cursor.at_root());
            cursor.goto_child(1);
            assert!(!cursor.at_root());
        }
    }

    #[test]
    fn test_navigation_mut() {
        let forest: Forest<&'static str, &'static str> = Forest::new();
        let mut tree = family(&forest);
        {
            let mut cursor = tree.as_mut();
            cursor.goto_child(1);
            assert_eq!(*cursor.leaf(), "younger");
            cursor.goto_parent();
            assert_eq!(*cursor.data(), "parent");
            cursor.goto_child(0);
            assert_eq!(*cursor.leaf(), "elder");
        }
    }

    #[test]
    fn test_bookmark_ref() {
        let forest: Forest<&'static str, &'static str> = Forest::new();
        let mut tree = family(&forest);
        let mut other_tree = forest.new_leaf("stranger");
        let bookmark = tree.as_ref().child(1).bookmark();
        assert!(other_tree.as_ref().lookup_bookmark(bookmark).is_none());
        assert!(!other_tree.as_mut().goto_bookmark(bookmark));
        assert_eq!(*tree.as_ref()
                   .lookup_bookmark(bookmark).unwrap()
                   .leaf(),
                   "younger");
        {
            let mut cursor = tree.as_mut();
            cursor.goto_child(0);
            assert!(cursor.goto_bookmark(bookmark));
            assert_eq!(*cursor.leaf(), "younger");
        }
    }

    #[test]
    fn test_bookmark_mut() {
        let forest: Forest<&'static str, &'static str> = Forest::new();
        let mut tree = family(&forest);
        let mut other_tree = forest.new_leaf("stranger");
        let bookmark = {
            let mut cursor = tree.as_mut();
            cursor.goto_child(1);
            cursor.bookmark()
        };
        assert!(other_tree.as_ref().lookup_bookmark(bookmark).is_none());
        assert!(!other_tree.as_mut().goto_bookmark(bookmark));
        assert_eq!(*tree.as_ref()
                   .lookup_bookmark(bookmark).unwrap()
                   .leaf(),
                   "younger");
        {
            let mut cursor = tree.as_mut();
            cursor.goto_child(0);
            assert!(cursor.goto_bookmark(bookmark));
            assert_eq!(*cursor.leaf(), "younger");
        }
    }

    #[test]
    fn test_bookmark_deleted() {
        let forest: Forest<&'static str, &'static str> = Forest::new();
        let mut tree = family(&forest);
        let bookmark = tree.as_ref().child(1).bookmark();
        let _ = tree.as_mut().remove_child(1);
        assert!(tree.as_ref().lookup_bookmark(bookmark).is_none());
        assert!(!tree.as_mut().goto_bookmark(bookmark));
    }

    #[test]
    fn test_replace_child() {
        let forest: Forest<&'static str, &'static str> = Forest::new();
        let mut tree = family(&forest);
        let old_imposter = forest.new_leaf("oldImposter");
        let young_imposter = forest.new_leaf("youngImposter");
        let elder = tree.as_mut().replace_child(0, old_imposter);
        let younger = tree.as_mut().replace_child(1, young_imposter);
        assert_eq!(*elder.as_ref().leaf(), "elder");
        assert_eq!(*younger.as_ref().leaf(), "younger");
        assert_eq!(tree.as_ref().num_children(), 2);
        assert_eq!(*tree.as_ref().child(0).leaf(), "oldImposter");
        assert_eq!(*tree.as_ref().child(1).leaf(), "youngImposter");
    }

    #[test]
    fn test_remove_child() {
        let forest: Forest<&'static str, &'static str> = Forest::new();
        let mut tree = family(&forest);
        // Remove elder child from the family
        let elder = tree.as_mut().remove_child(0);
        assert_eq!(*elder.as_ref().leaf(), "elder");
        assert!(elder.as_ref().parent().is_none());
        assert_eq!(tree.as_ref().num_children(), 1);
        assert_eq!(*tree.as_ref().child(0).leaf(), "younger");
        assert_eq!(*tree.as_ref().child(0).parent().unwrap().data(), "parent");
        // Remove younger child from the family
        let younger = tree.as_mut().remove_child(0);
        assert_eq!(*younger.as_ref().leaf(), "younger");
        assert!(younger.as_ref().parent().is_none());
        assert_eq!(tree.as_ref().num_children(), 0);
    }

    #[test]
    fn test_insert_child() {
        let forest: Forest<&'static str, &'static str> = Forest::new();
        let mut tree = family(&forest);
        let malcolm = forest.new_leaf("Malcolm");
        let reese = forest.new_leaf("Reese");
        let dewey = forest.new_leaf("Dewey");
        tree.as_mut().insert_child(1, malcolm); // Malcolm is in the middle
        tree.as_mut().insert_child(0, reese);
        tree.as_mut().insert_child(4, dewey);
        let children: Vec<&'static str> = tree.as_ref()
            .children()
            .map(|child| *child.leaf())
            .collect();
        assert_eq!(children, vec!("Reese", "elder", "Malcolm", "younger", "Dewey"));
        assert_eq!(*tree.as_ref().child(0).parent().unwrap().data(), "parent");
        assert_eq!(*tree.as_ref().child(1).parent().unwrap().data(), "parent");
    }

    #[test]
    fn comprehensive_exam() {
        let forest: Forest<u32, u32> = Forest::new();
        {
            let mut tree = mirror(&forest, 3, 0);
            let mut canada = forest.new_branch(721, vec!());
            let mut mexico = forest.new_leaf(3767);
            assert_eq!(forest.read_lock().tree_count(), 8+1+1);
            // tree:
            //       0
            //     / |  \
            //    1  2    4
            //       |   / \
            //       3  5   6
            //              |
            //              7

            // Test SubtreeRef
            let (mark2, mark4) = {
                // Data Access
                assert_eq!(tree.as_ref().sum(), 28);
                assert_eq!(tree.as_ref().num_children(), 3);

                // Navigation, Data Access
                let node5 = tree.as_ref().child(2).child(0);
                assert!(node5.is_leaf());
                assert_eq!(*node5.leaf(), 5);
                let node4 = node5.parent().unwrap();
                assert_eq!(*node4.data(), 4);
                assert!(node5
                        .parent().unwrap()
                        .parent().unwrap()
                        .parent()
                        .is_none());

                // Bookmarks: successful lookup
                let subtree = tree.as_ref().child(1);
                let mark5 = node5.bookmark();
                assert_eq!(*subtree
                           .lookup_bookmark(mark5).unwrap()
                           .parent().unwrap()
                           .data(), 4);
                let mark4 = node4.bookmark();
                assert_eq!(*node5
                           .lookup_bookmark(mark4).unwrap()
                           .parent().unwrap()
                           .child(1)
                           .data(), 2);
                
                // Bookmarks: failing lookup
                assert!(canada.as_ref().lookup_bookmark(mark5).is_none());
                let mark_mexico = mexico.as_ref().bookmark();
                assert!(node4.lookup_bookmark(mark_mexico).is_none());
                
                // Save some bookmarks for later testing
                let mark2 = tree.as_ref().child(1).bookmark();
                let mark4 = node4.bookmark();
                (mark2, mark4)
            };

            // Test SubtreeMut
            
            // To start
            let mut cursor = tree.as_mut();
            //  tree: 0
            //      / |  \
            //     1  2*   4*
            //        |   / \
            //        3  5   6
            //               |
            //               7
            //  canada: 721
            //  mexico: 3767

            // Navigate
            assert!(!cursor.is_leaf());
            cursor.goto_child(1);
            assert_eq!(*cursor.data(), 2);
            // Data Mutation
            *cursor.data_mut() = 22;
            assert_eq!(*cursor.data(), 22);
            assert_eq!(cursor.num_children(), 1);
            // Navigate
            assert!(!cursor.at_root());
            cursor.goto_parent();
            let mark0 = cursor.bookmark();
            assert!(cursor.at_root());
            
            // Cut
            let mut snip = cursor.remove_child(1);
            //  tree: 0+
            //       / \
            //      1    4*
            //          / \
            //         5   6
            //             |
            //             7
            //  snip: 2*
            //        |
            //        3
            //  canada: 721
            //  mexico: 3767

            assert_eq!(*snip.as_ref().data(), 22);
            assert_eq!(cursor.as_ref().sum(), 23);
            assert_eq!(forest.read_lock().tree_count(), 10);
            
            // Paste
            cursor.goto_child(1);
            cursor.insert_child(1, snip);
            cursor.insert_child(3, mexico);
            //  tree: 0+
            //       /  \
            //      1     4* _
            //          / | \  \
            //         5 22* 6 3767
            //            |  |
            //            3  7
            //  canada: 721

            // Leaf Mutation
            cursor.goto_child(3);
            assert!(cursor.is_leaf());
            assert_eq!(*cursor.leaf(), 3767);
            let mark3767 = cursor.bookmark();
            *cursor.leaf_mut() = 376;
            assert_eq!(*cursor.leaf(), 376);
            assert!(!cursor.at_root());
            cursor.goto_parent();
            assert!(!cursor.is_leaf());
            //  tree: 0+
            //       /  \
            //      1     4* _
            //          / | \  \
            //         5 22* 6 376+
            //            |  |
            //            3  7
            //  canada: 721

            // Replace
            snip = cursor.replace_child(1, canada);
            assert!(snip.as_ref().parent().is_none());
            cursor.goto_child(1);
            assert_eq!(*cursor.data(), 721);
            cursor.goto_parent();
            assert_eq!(*cursor.data(), 4);
            // Further mucking
            mexico = cursor.remove_child(3);
            assert!(mexico.as_ref().parent().is_none());
            snip.as_mut().insert_child(0, mexico);
            canada = snip;
            cursor.goto_child(2);
            //  tree: 0+
            //       / \
            //      1   4*
            //        / | \
            //       5 721 6
            //             |
            //             7
            // canada: 22*
            //        / \
            //       3  376+

            // Bookmarks after mutation
            assert!( ! cursor.goto_bookmark(mark2));
            assert_eq!(*cursor.data(), 6);
            assert!(cursor.goto_bookmark(mark4));
            assert_eq!(*cursor.data(), 4);
            assert_eq!(*canada.as_ref()
                       .lookup_bookmark(mark3767).unwrap()
                       .leaf(),
                       376);
            assert!( ! canada.as_mut().goto_bookmark(mark0));

            // Some final bookmark checks
            assert!(tree.as_ref().child(0).lookup_bookmark(mark2).is_none());
            assert_eq!(tree.as_ref()
                       .child(0)
                       .lookup_bookmark(mark4).unwrap()
                       .sum(),
                       743);
            // Summation checks
            assert_eq!(tree.as_ref().sum(), 744);
            assert_eq!(canada.as_ref().sum(), 401);
        }

        // Check for leaks
        assert_eq!(forest.read_lock().tree_count(), 0);
    }

    // Error Testing //
    
    #[test]
    #[should_panic(expected="leaf node has no children")]
    fn test_num_chilren_panic() {
        let forest: Forest<(), ()> = Forest::new();
        let tree = forest.new_leaf(());
        tree.as_ref().num_children();
    }

    #[test]
    #[should_panic(expected="leaf node has no data")]
    fn test_data_panic() {
        let forest: Forest<(), ()> = Forest::new();
        let tree = forest.new_leaf(());
        *tree.as_ref().data();
    }

    #[test]
    #[should_panic(expected="branch node has no leaf")]
    fn test_leaf_panic() {
        let forest: Forest<(), ()> = Forest::new();
        let mut tree = forest.new_branch((), vec!());
        *tree.as_mut().leaf_mut();
    }

    #[test]
    #[should_panic(expected="leaf node has no children")]
    fn test_navigation_panic_leaf_ref() {
        let forest: Forest<(), ()> = Forest::new();
        let tree = forest.new_leaf(());
        tree.as_ref().child(0);
    }

    #[test]
    #[should_panic(expected="leaf node has no children")]
    fn test_navigation_panic_leaf_mut() {
        let forest: Forest<(), ()> = Forest::new();
        let mut tree = forest.new_leaf(());
        tree.as_mut().goto_child(0);
    }

    #[test]
    #[should_panic(expected="child index out of bounds")]
    fn test_navigation_panic_oob_ref() {
        let forest: Forest<(), ()> = Forest::new();
        let tree = forest.new_branch((), vec!());
        tree.as_ref().child(0);
    }

    #[test]
    #[should_panic(expected="child index out of bounds")]
    fn test_navigation_panic_oob_mut() {
        let forest: Forest<(), ()> = Forest::new();
        let mut tree = forest.new_branch((), vec!());
        tree.as_mut().goto_child(0);
    }

    #[test]
    #[should_panic(expected="child index out of bounds")]
    fn test_insert_panic_oob() {
        let forest: Forest<&'static str, &'static str> = Forest::new();
        let mut tree = family(&forest);
        let leaf = forest.new_leaf("");
        tree.as_mut().insert_child(3, leaf);
    }

    #[test]
    #[should_panic(expected="child index out of bounds")]
    fn test_remove_panic_oob() {
        let forest: Forest<&'static str, &'static str> = Forest::new();
        let mut tree = family(&forest);
        tree.as_mut().remove_child(2);
    }

    #[test]
    #[should_panic(expected="child index out of bounds")]
    fn test_replace_panic_oob() {
        let forest: Forest<&'static str, &'static str> = Forest::new();
        let mut tree = family(&forest);
        let leaf = forest.new_leaf("");
        tree.as_mut().replace_child(2, leaf);
    }

    #[test]
    #[should_panic(expected="root node has no parent")]
    fn test_parent_of_root_panic() {
        let forest: Forest<&'static str, &'static str> = Forest::new();
        let mut tree = family(&forest);
        tree.as_mut().goto_parent();
    }
}
