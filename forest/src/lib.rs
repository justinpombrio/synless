//! A general represenation of trees.

mod forest;
mod tree;
mod tree_ref;

pub use self::tree::{Tree, Forest, Bookmark,
                     ReadLeaf, WriteLeaf, ReadData, WriteData};
pub use self::tree_ref::TreeRef;


#[cfg(test)]
mod forest_tests {
    use super::*;

    fn family(forest: &Forest<&'static str, &'static str>)
                  -> Tree<&'static str, &'static str>
    {
        let leaves = vec!(forest.new_leaf("elder"),
                          forest.new_leaf("younger"));
        forest.new_branch("parent", leaves)
    }

    fn mirror(forest: &Forest<u32, u32>, height: u32, id: u32) -> Tree<u32, u32> {
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

    impl<'f> TreeRef<'f, u32, u32> {
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
        assert!(tree.is_leaf());
        assert_eq!(*tree.leaf(), 2);
        // Mutate it to be 3
        *tree.leaf_mut() = 3;
        assert!(tree.borrow().is_leaf());
        assert_eq!(*tree.borrow().leaf(), 3);
    }

    #[test]
    fn test_data() {
        let forest: Forest<u32, ()> = Forest::new();
        // Begin with data of 2
        let mut tree = forest.new_branch(2, vec!());
        assert!(!tree.borrow().is_leaf());
        assert_eq!(*tree.borrow().data(), 2);
        // Mutate it to be 3
        *tree.data_mut() = 3;
        assert!(!tree.is_leaf());
        assert_eq!(*tree.data(), 3);
    }

    #[test]
    fn test_num_children() {
        let forest: Forest<(), ()> = Forest::new();
        let leaves = vec!(forest.new_leaf(()),
                          forest.new_leaf(()),
                          forest.new_leaf(()));
        println!("before");
        let tree = forest.new_branch((), leaves);
        println!("after");
        assert_eq!(tree.borrow().num_children(), 3);
        assert_eq!(tree.num_children(), 3);
    }

    #[test]
    fn test_navigation_ref() {
        let forest: Forest<&'static str, &'static str> = Forest::new();
        let tree = family(&forest);
        assert_eq!(*tree.borrow().child(0).leaf(), "elder");
        assert_eq!(*tree.borrow().child(1).leaf(), "younger");
        assert_eq!(*tree.borrow().child(0).parent().unwrap().data(), "parent");
        assert!(tree.borrow().child(0).parent().unwrap().parent().is_none());
        let children: Vec<&'static str> = tree.borrow()
            .children()
            .map(|child| *child.leaf())
            .collect();
        assert_eq!(children, vec!("elder", "younger"));
    }

    #[test]
    fn test_at_root_mut() {
        let forest: Forest<&'static str, &'static str> = Forest::new();
        let mut tree = family(&forest);
        assert!(tree.at_root());
        tree.goto_child(1);
        assert!(!tree.at_root());
    }

    #[test]
    fn test_navigation_mut() {
        let forest: Forest<&'static str, &'static str> = Forest::new();
        let mut tree = family(&forest);
        tree.goto_child(1);
        assert_eq!(*tree.leaf(), "younger");
        tree.goto_parent();
        assert_eq!(*tree.data(), "parent");
        tree.goto_child(0);
        assert_eq!(*tree.leaf(), "elder");
        tree.goto_root();
        assert_eq!(*tree.data(), "parent");
        assert_eq!(*tree.borrow().child(1).leaf(), "younger");
    }

    #[test]
    fn test_bookmark_ref() {
        let forest: Forest<&'static str, &'static str> = Forest::new();
        let mut tree = family(&forest);
        let mut other_tree = forest.new_leaf("stranger");
        let bookmark = tree.borrow().child(1).bookmark();
        assert!(other_tree.borrow().lookup_bookmark(bookmark).is_none());
        assert!(!other_tree.goto_bookmark(bookmark));
        assert_eq!(*tree.borrow()
                   .lookup_bookmark(bookmark).unwrap()
                   .leaf(),
                   "younger");
        tree.goto_child(0);
        assert!(tree.goto_bookmark(bookmark));
        assert_eq!(*tree.leaf(), "younger");
    }

    #[test]
    fn test_bookmark_mut() {
        let forest: Forest<&'static str, &'static str> = Forest::new();
        let mut tree = family(&forest);
        let mut other_tree = forest.new_leaf("stranger");
        tree.goto_child(1);
        let bookmark = tree.bookmark();
        tree.goto_parent();
        assert!(other_tree.borrow().lookup_bookmark(bookmark).is_none());
        assert!(!other_tree.goto_bookmark(bookmark));
        assert_eq!(*tree.borrow().lookup_bookmark(bookmark).unwrap().leaf(), "younger");
        tree.goto_child(0);
        assert!(tree.goto_bookmark(bookmark));
        assert_eq!(*tree.leaf(), "younger");
    }

    #[test]
    fn test_bookmark_deleted() {
        let forest: Forest<&'static str, &'static str> = Forest::new();
        let mut tree = family(&forest);
        let bookmark = tree.borrow().child(1).bookmark();
        let _ = tree.remove_child(1);
        assert!(tree.borrow().lookup_bookmark(bookmark).is_none());
        assert!(!tree.goto_bookmark(bookmark));
    }

    #[test]
    fn test_replace_child() {
        let forest: Forest<&'static str, &'static str> = Forest::new();
        let mut tree = family(&forest);
        let old_imposter = forest.new_leaf("oldImposter");
        let young_imposter = forest.new_leaf("youngImposter");
        let elder = tree.replace_child(0, old_imposter);
        let younger = tree.replace_child(1, young_imposter);
        assert_eq!(*elder.borrow().leaf(), "elder");
        assert_eq!(*younger.borrow().leaf(), "younger");
        assert_eq!(tree.borrow().num_children(), 2);
        assert_eq!(*tree.borrow().child(0).leaf(), "oldImposter");
        assert_eq!(*tree.borrow().child(1).leaf(), "youngImposter");
    }

    #[test]
    fn test_remove_child() {
        let forest: Forest<&'static str, &'static str> = Forest::new();
        let mut tree = family(&forest);
        // Remove elder child from the family
        let elder = tree.remove_child(0);
        assert_eq!(*elder.borrow().leaf(), "elder");
        assert!(elder.borrow().parent().is_none());
        assert_eq!(tree.borrow().num_children(), 1);
        assert_eq!(*tree.borrow().child(0).leaf(), "younger");
        assert_eq!(*tree.borrow().child(0).parent().unwrap().data(), "parent");
        // Remove younger child from the family
        let younger = tree.remove_child(0);
        assert_eq!(*younger.borrow().leaf(), "younger");
        assert!(younger.borrow().parent().is_none());
        assert_eq!(tree.borrow().num_children(), 0);
    }

    #[test]
    fn test_insert_child() {
        let forest: Forest<&'static str, &'static str> = Forest::new();
        let mut tree = family(&forest);
        let malcolm = forest.new_leaf("Malcolm");
        let reese = forest.new_leaf("Reese");
        let dewey = forest.new_leaf("Dewey");
        tree.insert_child(1, malcolm); // Malcolm is in the middle
        tree.insert_child(0, reese);
        tree.insert_child(4, dewey);
        let children: Vec<&'static str> = tree.borrow()
            .children()
            .map(|child| *child.leaf())
            .collect();
        assert_eq!(children, vec!("Reese", "elder", "Malcolm", "younger", "Dewey"));
        assert_eq!(*tree.borrow().child(0).parent().unwrap().data(), "parent");
        assert_eq!(*tree.borrow().child(1).parent().unwrap().data(), "parent");
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

            // Test TreeRef
            let (mark2, mark4) = {
                // Data Access
                assert_eq!(tree.borrow().sum(), 28);
                assert_eq!(tree.borrow().num_children(), 3);

                // Navigation, Data Access
                let node5 = tree.borrow().child(2).child(0);
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
                let subtree = tree.borrow().child(1);
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
                assert!(canada.borrow().lookup_bookmark(mark5).is_none());
                let mark_mexico = mexico.borrow().bookmark();
                assert!(node4.lookup_bookmark(mark_mexico).is_none());
                
                // Save some bookmarks for later testing
                let mark2 = tree.borrow().child(1).bookmark();
                let mark4 = node4.bookmark();
                (mark2, mark4)
            };

            // Test Tree
            
            // To start:
            //
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
            assert!(!tree.is_leaf());
            tree.goto_child(1);
            assert_eq!(*tree.data(), 2);
            // Data Mutation
            *tree.data_mut() = 22;
            assert_eq!(*tree.data(), 22);
            assert_eq!(tree.num_children(), 1);
            // Navigate
            assert!(!tree.at_root());
            tree.goto_parent();
            let mark0 = tree.bookmark();
            assert!(tree.at_root());
            
            // Cut
            let mut snip = tree.remove_child(1);
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

            assert_eq!(*snip.borrow().data(), 22);
            assert_eq!(tree.borrow().sum(), 23);
            assert_eq!(forest.read_lock().tree_count(), 10);
            
            // Paste
            tree.goto_child(1);
            tree.insert_child(1, snip);
            tree.insert_child(3, mexico);
            //  tree: 0+
            //       /  \
            //      1     4* _
            //          / | \  \
            //         5 22* 6 3767
            //            |  |
            //            3  7
            //  canada: 721

            // Leaf Mutation
            tree.goto_child(3);
            assert!(tree.is_leaf());
            assert_eq!(*tree.leaf(), 3767);
            let mark3767 = tree.bookmark();
            *tree.leaf_mut() = 376;
            assert_eq!(*tree.leaf(), 376);
            assert!(!tree.at_root());
            tree.goto_parent();
            assert!(!tree.is_leaf());
            //  tree: 0+
            //       /  \
            //      1     4* _
            //          / | \  \
            //         5 22* 6 376+
            //            |  |
            //            3  7
            //  canada: 721

            // Replace
            snip = tree.replace_child(1, canada);
            assert!(snip.borrow().parent().is_none());
            tree.goto_child(1);
            assert_eq!(*tree.data(), 721);
            tree.goto_parent();
            assert_eq!(*tree.data(), 4);
            // Further mucking
            mexico = tree.remove_child(3);
            assert!(mexico.borrow().parent().is_none());
            snip.insert_child(0, mexico);
            canada = snip;
            tree.goto_child(2);
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
            assert!( ! tree.goto_bookmark(mark2));
            assert_eq!(*tree.data(), 6);
            assert!(tree.goto_bookmark(mark4));
            assert_eq!(*tree.data(), 4);
            assert_eq!(*canada.borrow()
                       .lookup_bookmark(mark3767).unwrap()
                       .leaf(),
                       376);
            assert!( ! canada.goto_bookmark(mark0));

            // Some final bookmark checks
            assert!(tree.borrow().child(0).lookup_bookmark(mark2).is_none());
            assert_eq!(tree.borrow()
                       .child(0)
                       .lookup_bookmark(mark4).unwrap()
                       .sum(),
                       743);
            // Summation checks
            tree.goto_parent();
            assert_eq!(tree.borrow().sum(), 744);
            assert_eq!(canada.borrow().sum(), 401);
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
        tree.borrow().num_children();
    }

    #[test]
    #[should_panic(expected="leaf node has no data")]
    fn test_data_panic() {
        let forest: Forest<(), ()> = Forest::new();
        let tree = forest.new_leaf(());
        *tree.borrow().data();
    }

    #[test]
    #[should_panic(expected="branch node has no leaf")]
    fn test_leaf_panic() {
        let forest: Forest<(), ()> = Forest::new();
        let mut tree = forest.new_branch((), vec!());
        *tree.leaf_mut();
    }

    #[test]
    #[should_panic(expected="leaf node has no children")]
    fn test_navigation_panic_leaf_ref() {
        let forest: Forest<(), ()> = Forest::new();
        let tree = forest.new_leaf(());
        tree.borrow().child(0);
    }

    #[test]
    #[should_panic(expected="leaf node has no children")]
    fn test_navigation_panic_leaf_mut() {
        let forest: Forest<(), ()> = Forest::new();
        let mut tree = forest.new_leaf(());
        tree.goto_child(0);
    }

    #[test]
    #[should_panic(expected="child index out of bounds")]
    fn test_navigation_panic_oob_ref() {
        let forest: Forest<(), ()> = Forest::new();
        let tree = forest.new_branch((), vec!());
        tree.borrow().child(0);
    }

    #[test]
    #[should_panic(expected="child index out of bounds")]
    fn test_navigation_panic_oob_mut() {
        let forest: Forest<(), ()> = Forest::new();
        let mut tree = forest.new_branch((), vec!());
        tree.goto_child(0);
    }

    #[test]
    #[should_panic(expected="child index out of bounds")]
    fn test_insert_panic_oob() {
        let forest: Forest<&'static str, &'static str> = Forest::new();
        let mut tree = family(&forest);
        let leaf = forest.new_leaf("");
        tree.insert_child(3, leaf);
    }

    #[test]
    #[should_panic(expected="child index out of bounds")]
    fn test_remove_panic_oob() {
        let forest: Forest<&'static str, &'static str> = Forest::new();
        let mut tree = family(&forest);
        tree.remove_child(2);
    }

    #[test]
    #[should_panic(expected="child index out of bounds")]
    fn test_replace_panic_oob() {
        let forest: Forest<&'static str, &'static str> = Forest::new();
        let mut tree = family(&forest);
        let leaf = forest.new_leaf("");
        tree.replace_child(2, leaf);
    }

    #[test]
    #[should_panic(expected="root node has no parent")]
    fn test_parent_of_root_panic() {
        let forest: Forest<&'static str, &'static str> = Forest::new();
        let mut tree = family(&forest);
        tree.goto_parent();
    }
}
