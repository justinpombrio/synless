use forest::{Forest, Tree, TreeRef};

fn make_family(forest: &Forest<&'static str, &'static str>) -> Tree<&'static str, &'static str> {
    let leaves = vec![
        forest.new_leaf("sister", "elder"),
        forest.new_leaf("sister", "younger"),
    ];
    forest.new_branch("parent", leaves)
}

fn make_mirror(forest: &Forest<u32, u32>, height: u32, id: u32) -> Tree<u32, u32> {
    if height == 0 {
        forest.new_leaf(42, id)
    } else {
        let mut children = vec![];
        for i in 0..height {
            children.push(make_mirror(forest, i, id + 2_u32.pow(i)));
        }
        forest.new_branch(id, children)
    }
}

fn sum(tree_ref: TreeRef<u32, u32>) -> u32 {
    if tree_ref.is_leaf() {
        tree_ref.with_leaf(|l| *l)
    } else {
        let mut total = tree_ref.with_data(|d| *d);
        for child in tree_ref.children() {
            total += sum(child);
        }
        total
    }
}

#[test]
fn test_leaves() {
    let forest: Forest<u32, u32> = Forest::new();
    // Begin with a leaf of 2
    let mut tree = forest.new_leaf(4, 2);
    assert!(tree.is_leaf());
    assert_eq!(tree.with_leaf(|l| *l), 2);
    assert_eq!(tree.borrow().with_data(|d| *d), 4);
    // Mutate it to be 3
    tree.with_leaf_mut(|l| *l = 3);
    tree.with_data_mut(|d| *d = 5);
    assert!(tree.borrow().is_leaf());
    assert_eq!(tree.borrow().with_leaf(|l| *l), 3);
    assert_eq!(tree.borrow().with_data(|d| *d), 5);
}

#[test]
fn test_data() {
    let forest: Forest<u32, ()> = Forest::new();
    // Begin with data of 2
    let mut tree = forest.new_branch(2, vec![]);
    assert!(!tree.borrow().is_leaf());
    assert_eq!(tree.borrow().with_data(|d| *d), 2);
    // Mutate it to be 3
    tree.with_data_mut(|d| *d = 3);
    assert!(!tree.is_leaf());
    assert_eq!(tree.with_data(|d| *d), 3);
}

#[test]
fn test_num_children() {
    let forest: Forest<(), ()> = Forest::new();
    let leaves = vec![
        forest.new_leaf((), ()),
        forest.new_leaf((), ()),
        forest.new_leaf((), ()),
    ];
    println!("before");
    let tree = forest.new_branch((), leaves);
    println!("after");
    assert_eq!(tree.borrow().num_children(), 3);
    assert_eq!(tree.num_children(), 3);
}

#[test]
fn test_navigation_ref() {
    let forest: Forest<&'static str, &'static str> = Forest::new();
    let tree = make_family(&forest);
    assert_eq!(tree.borrow().child(0).with_leaf(|l| *l), "elder");
    assert_eq!(tree.borrow().child(1).with_leaf(|l| *l), "younger");
    assert_eq!(
        tree.borrow().child(0).parent().unwrap().with_data(|d| *d),
        "parent"
    );
    assert!(tree.borrow().child(0).parent().unwrap().parent().is_none());
    let children: Vec<&'static str> = tree
        .borrow()
        .children()
        .into_iter()
        .map(|child| child.with_leaf(|l| *l))
        .collect();
    assert_eq!(children, vec!("elder", "younger"));
}

#[test]
fn test_at_root_mut() {
    let forest: Forest<&'static str, &'static str> = Forest::new();
    let mut tree = make_family(&forest);
    assert!(tree.is_at_root());
    tree.goto_child(1);
    assert!(!tree.is_at_root());
}

#[test]
fn test_navigation_mut() {
    let forest: Forest<&'static str, &'static str> = Forest::new();
    let mut tree = make_family(&forest);
    tree.goto_child(1);
    assert_eq!(tree.with_leaf(|l| *l), "younger");
    tree.goto_parent();
    assert_eq!(tree.with_data(|d| *d), "parent");
    tree.goto_child(0);
    assert_eq!(tree.with_leaf(|l| *l), "elder");
    tree.goto_root();
    assert_eq!(tree.with_data(|d| *d), "parent");
    assert_eq!(tree.borrow().child(1).with_leaf(|l| *l), "younger");
}

#[test]
fn test_bookmark_ref() {
    let forest: Forest<&'static str, &'static str> = Forest::new();
    let mut tree = make_family(&forest);
    let mut other_tree = forest.new_leaf("mister", "stranger");
    let bookmark = tree.borrow().child(1).bookmark();
    assert!(other_tree.borrow().lookup_bookmark(bookmark).is_none());
    assert!(!other_tree.goto_bookmark(bookmark));
    assert_eq!(
        tree.borrow()
            .lookup_bookmark(bookmark)
            .unwrap()
            .with_leaf(|l| *l),
        "younger"
    );
    tree.goto_child(0);
    assert!(tree.goto_bookmark(bookmark));
    assert_eq!(tree.with_leaf(|l| *l), "younger");
}

#[test]
fn test_bookmark_mut() {
    let forest: Forest<&'static str, &'static str> = Forest::new();
    let mut tree = make_family(&forest);
    let mut other_tree = forest.new_leaf("mister", "stranger");
    tree.goto_child(1);
    let bookmark = tree.bookmark();
    tree.goto_parent();
    assert!(other_tree.borrow().lookup_bookmark(bookmark).is_none());
    assert!(!other_tree.goto_bookmark(bookmark));
    assert_eq!(
        tree.borrow()
            .lookup_bookmark(bookmark)
            .unwrap()
            .with_leaf(|l| *l),
        "younger"
    );
    tree.goto_child(0);
    assert!(tree.goto_bookmark(bookmark));
    assert_eq!(tree.with_leaf(|l| *l), "younger");
}

#[test]
fn test_bookmark_deleted() {
    let forest: Forest<&'static str, &'static str> = Forest::new();
    let mut tree = make_family(&forest);
    let bookmark = tree.borrow().child(1).bookmark();
    let _ = tree.remove_child(1);
    let _ = make_family(&forest); // Reuse the bookmark key to be tricksy
    assert!(tree.borrow().lookup_bookmark(bookmark).is_none());
    assert!(!tree.goto_bookmark(bookmark));
}

#[test]
fn test_replace_child() {
    let forest: Forest<&'static str, &'static str> = Forest::new();
    let mut tree = make_family(&forest);
    let old_imposter = forest.new_leaf("imposter", "old");
    let young_imposter = forest.new_leaf("imposter", "young");
    let elder = tree.replace_child(0, old_imposter);
    let younger = tree.replace_child(1, young_imposter);
    assert_eq!(elder.borrow().with_leaf(|l| *l), "elder");
    assert_eq!(younger.borrow().with_leaf(|l| *l), "younger");
    assert_eq!(tree.borrow().num_children(), 2);
    assert_eq!(tree.borrow().child(0).with_leaf(|l| *l), "old");
    assert_eq!(tree.borrow().child(0).with_data(|d| *d), "imposter");
    assert_eq!(tree.borrow().child(1).with_leaf(|l| *l), "young");
    assert_eq!(tree.borrow().child(1).with_data(|d| *d), "imposter");
}

#[test]
fn test_remove_child() {
    let forest: Forest<&'static str, &'static str> = Forest::new();
    let mut tree = make_family(&forest);
    // Remove elder child from the family
    let elder = tree.remove_child(0);
    assert_eq!(elder.borrow().with_leaf(|l| *l), "elder");
    assert!(elder.borrow().parent().is_none());
    assert_eq!(tree.borrow().num_children(), 1);
    assert_eq!(tree.borrow().child(0).with_leaf(|l| *l), "younger");
    assert_eq!(
        tree.borrow().child(0).parent().unwrap().with_data(|d| *d),
        "parent"
    );
    // Remove younger child from the family
    let younger = tree.remove_child(0);
    assert_eq!(younger.borrow().with_leaf(|l| *l), "younger");
    assert!(younger.borrow().parent().is_none());
    assert_eq!(tree.borrow().num_children(), 0);
}

#[test]
fn test_insert_child() {
    let forest: Forest<&'static str, &'static str> = Forest::new();
    let mut tree = make_family(&forest);
    let malcolm = forest.new_leaf("Middle", "Malcolm");
    let reese = forest.new_leaf("Middle", "Reese");
    let dewey = forest.new_leaf("Middle", "Dewey");
    tree.insert_child(1, malcolm); // Malcolm is in the middle
    tree.insert_child(0, reese);
    tree.insert_child(4, dewey);
    let children: Vec<&'static str> = tree
        .borrow()
        .children()
        .into_iter()
        .map(|child| child.with_leaf(|l| *l))
        .collect();
    assert_eq!(
        children,
        vec!("Reese", "elder", "Malcolm", "younger", "Dewey")
    );
    assert_eq!(
        tree.borrow().child(0).parent().unwrap().with_data(|d| *d),
        "parent"
    );
    assert_eq!(
        tree.borrow().child(1).parent().unwrap().with_data(|d| *d),
        "parent"
    );
}

#[test]
#[allow(clippy::cognitive_complexity)]
fn comprehensive_exam() {
    let forest: Forest<u32, u32> = Forest::new();
    {
        let mut tree = make_mirror(&forest, 3, 0);
        let mut canada = forest.new_branch(721, vec![]);
        let mut mexico = forest.new_leaf(1430, 3767);
        assert_eq!(forest.node_count(), 8 + 1 + 1);
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
            assert_eq!(sum(tree.borrow()), 28);
            assert_eq!(tree.borrow().num_children(), 3);

            // Navigation, Data Access
            let node5 = tree.borrow().child(2).child(0);
            assert!(node5.is_leaf());
            assert_eq!(node5.with_leaf(|l| *l), 5);
            let node4 = node5.parent().unwrap();
            assert_eq!(node4.with_data(|d| *d), 4);
            assert!(node5.parent().unwrap().parent().unwrap().parent().is_none());

            // Bookmarks: successful lookup
            let subtree = tree.borrow().child(1);
            let mark5 = node5.bookmark();
            assert_eq!(
                subtree
                    .lookup_bookmark(mark5)
                    .unwrap()
                    .parent()
                    .unwrap()
                    .with_data(|d| *d),
                4
            );
            let mark4 = node4.bookmark();
            assert_eq!(
                node5
                    .lookup_bookmark(mark4)
                    .unwrap()
                    .parent()
                    .unwrap()
                    .child(1)
                    .with_data(|d| *d),
                2
            );

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
        assert_eq!(tree.with_data(|d| *d), 2);
        // Data Mutation
        tree.with_data_mut(|d| *d = 22);
        assert_eq!(tree.with_data(|d| *d), 22);
        assert_eq!(tree.num_children(), 1);
        // Navigate
        assert!(!tree.is_at_root());
        tree.goto_parent();
        let mark0 = tree.bookmark();
        assert!(tree.is_at_root());

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

        assert_eq!(snip.borrow().with_data(|d| *d), 22);
        assert_eq!(sum(tree.borrow()), 23);
        assert_eq!(forest.node_count(), 10);

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
        assert_eq!(tree.with_leaf(|l| *l), 3767);
        let mark3767 = tree.bookmark();
        tree.with_leaf_mut(|l| *l = 376);
        assert_eq!(tree.with_leaf(|l| *l), 376);
        assert!(!tree.is_at_root());
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
        assert_eq!(tree.with_data(|d| *d), 721);
        tree.goto_parent();
        assert_eq!(tree.with_data(|d| *d), 4);
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
        assert!(!tree.goto_bookmark(mark2));
        assert_eq!(tree.with_data(|d| *d), 6);
        assert!(tree.goto_bookmark(mark4));
        assert_eq!(tree.with_data(|d| *d), 4);
        assert_eq!(
            canada
                .borrow()
                .lookup_bookmark(mark3767)
                .unwrap()
                .with_leaf(|l| *l),
            376
        );
        assert!(!canada.goto_bookmark(mark0));

        // Some final bookmark checks
        assert!(tree.borrow().child(0).lookup_bookmark(mark2).is_none());
        assert_eq!(
            sum(tree.borrow().child(0).lookup_bookmark(mark4).unwrap()),
            743
        );
        // Summation checks
        tree.goto_parent();
        assert_eq!(sum(tree.borrow()), 744);
        assert_eq!(sum(canada.borrow()), 401);
    }

    // Check for leaks
    assert_eq!(forest.node_count(), 0);
}

// Error Testing //

#[test]
#[should_panic(expected = "leaf node has no children")]
fn test_num_chilren_panic() {
    let forest: Forest<(), ()> = Forest::new();
    let tree = forest.new_leaf((), ());
    tree.borrow().num_children();
}

#[test]
#[should_panic(expected = "branch node has no leaf")]
fn test_leaf_panic() {
    let forest: Forest<(), ()> = Forest::new();
    let mut tree = forest.new_branch((), vec![]);
    tree.with_leaf_mut(|_l| ());
}

#[test]
#[should_panic(expected = "leaf node has no children")]
fn test_navigation_panic_leaf_ref() {
    let forest: Forest<(), ()> = Forest::new();
    let tree = forest.new_leaf((), ());
    tree.borrow().child(0);
}

#[test]
#[should_panic(expected = "leaf node has no children")]
fn test_navigation_panic_leaf_mut() {
    let forest: Forest<(), ()> = Forest::new();
    let mut tree = forest.new_leaf((), ());
    tree.goto_child(0);
}

#[test]
#[should_panic(expected = "index")]
fn test_navigation_panic_oob_ref() {
    let forest: Forest<(), ()> = Forest::new();
    let tree = forest.new_branch((), vec![]);
    tree.borrow().child(0);
}

#[test]
#[should_panic(expected = "index")]
fn test_navigation_panic_oob_mut() {
    let forest: Forest<(), ()> = Forest::new();
    let mut tree = forest.new_branch((), vec![]);
    tree.goto_child(0);
}

#[test]
#[should_panic(expected = "index")]
fn test_insert_panic_oob() {
    let forest: Forest<&'static str, &'static str> = Forest::new();
    let mut tree = make_family(&forest);
    let leaf = forest.new_leaf("", "");
    tree.insert_child(3, leaf);
}

#[test]
#[should_panic(expected = "index")]
fn test_remove_panic_oob() {
    let forest: Forest<&'static str, &'static str> = Forest::new();
    let mut tree = make_family(&forest);
    tree.remove_child(2);
}

#[test]
#[should_panic(expected = "index")]
fn test_replace_panic_oob() {
    let forest: Forest<&'static str, &'static str> = Forest::new();
    let mut tree = make_family(&forest);
    let leaf = forest.new_leaf("", "");
    tree.replace_child(2, leaf);
}

#[test]
#[should_panic(expected = "root node has no parent")]
fn test_parent_of_root_panic() {
    let forest: Forest<&'static str, &'static str> = Forest::new();
    let mut tree = make_family(&forest);
    tree.goto_parent();
}
