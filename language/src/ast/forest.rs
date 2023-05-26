use generational_arena::{Arena, Index};

pub struct Forest<D>(Arena<Node<D>>);

/// A node in a doubly-linked-list representation of a tree that can store some
/// data `D`. All nodes store their parent (except root nodes which don't have
/// a parent).  Parents store only their first child. The siblings are linked to
/// each other in a circle.
#[derive(Debug)]
struct Node<D> {
    parent: Option<Index>,
    prev: Index,
    next: Index,
    first_child: Option<Index>,
    data: D,
}

impl<D> Forest<D> {
    /// Create a new empty forest.
    pub fn new() -> Forest<D> {
        Forest(Arena::new())
    }

    /// Create a new root node containing the given data.
    pub fn new_node(&mut self, data: D) -> Index {
        // Very bad temporary index. Never visible.
        let dummy_index = Index::from_raw_parts(0, 0);

        let id = self.0.insert(Node {
            parent: None,
            prev: dummy_index,
            next: dummy_index,
            first_child: None,
            data,
        });
        self.0[id].prev = id;
        self.0[id].next = id;
        id
    }

    /// Get the parent of `node`, or `None` if it's a root.
    pub fn parent(&self, node: Index) -> Option<Index> {
        self.0[node].parent
    }

    /// Get the root of the tree that `node` belongs to (exactly the node you
    /// would get by calling `parent` repeatedly).
    pub fn root(&self, node: Index) -> Index {
        let mut ancestor = node;
        while let Some(parent) = self.0[ancestor].parent {
            ancestor = parent;
        }
        ancestor
    }

    /// Get the first child of `node`, or `None` if it has no children.
    pub fn first_child(&self, node: Index) -> Option<Index> {
        self.0[node].first_child
    }

    /// Get the `node`'s previous sibling. If `node` is the first sibling, wrap around
    /// and return the last sibling. If `node` has no siblings, return `node`.
    pub fn prev(&self, node: Index) -> Index {
        self.0[node].prev
    }

    /// Get the `node`'s next sibling. If `node` is the last sibling, wrap around
    /// and return the first sibling. If `node` has no siblings, return `node`.
    pub fn next(&self, node: Index) -> Index {
        self.0[node].next
    }

    /// True if `node` is its parent's first child, or if it is a root node.
    pub fn is_first(&self, node: Index) -> bool {
        if let Some(parent) = self.0[node].parent {
            self.0[parent].first_child.unwrap() == node
        } else {
            true
        }
    }

    /// True if `node` is its parent's last (or only) child, or if it is a root node.
    pub fn is_last(&self, node: Index) -> bool {
        if let Some(parent) = self.0[node].parent {
            self.0[parent].first_child.unwrap() == self.0[node].next
        } else {
            true
        }
    }

    /// Borrows the data stored inside `node`.
    pub fn data(&self, node: Index) -> &D {
        &self.0[node].data
    }

    /// Mutably borrows the data stored inside `node`.
    pub fn data_mut(&mut self, node: Index) -> &mut D {
        &mut self.0[node].data
    }

    /// Delete the given root `node`, including all of its descendants.
    /// This invalidates all of their indices. Panics if `node` is not a root.
    pub fn delete_root(&mut self, node: Index) {
        if self.0[node].parent.is_some() {
            panic!("Forest - can only delete whole trees");
        }
        let root = self.root(node);
        // list of (nodes_first_sibling, node)
        let mut to_delete = vec![(root, root)];
        while let Some((first_sibling, node)) = to_delete.pop() {
            if self.next(node) != first_sibling {
                to_delete.push((first_sibling, self.next(node)));
            }
            if let Some(child) = self.first_child(node) {
                to_delete.push((child, child));
            }
            self.0.remove(node);
        }
    }

    /// True if the forest still contains this node (i.e. its tree hasn't been deleted).
    pub fn is_valid(&self, node: Index) -> bool {
        self.0.contains(node)
    }

    /// Remove `node` (and its descendants) from its place in the tree, making it the
    /// root of a new tree.
    pub fn detach(&mut self, node: Index) {
        let crack = Crack::new_remove(self, node);
        crack.close(self);
    }

    /// Swap the positions of two nodes (and their descendants). The two nodes may be part
    /// of the same tree or different trees, but they must not overlap - i.e. one node must
    /// not be an ancestor of the other.
    pub fn swap(&mut self, node_1: Index, node_2: Index) {
        if node_1 == node_2 {
            return;
        }
        let crack_1 = Crack::new_remove(self, node_1);
        let crack_2 = Crack::new_remove(self, node_2);
        crack_1.fill(self, node_2);
        crack_2.fill(self, node_1);
    }

    /// Insert the root node `node` before `at`, so that `node` becomes its previous sibling.
    /// Panics if `node` _is not_ a root, or if `at` _is_ a root.
    pub fn insert_before(&mut self, at: Index, node: Index) {
        if self.0[node].parent.is_some() {
            panic!("Forest - can only insert whole trees (before)");
        }
        let crack = Crack::new_before(self, at);
        crack.fill(self, node);
    }

    /// Insert the root node `node` after `at`, so that `node` becomes its next sibling.
    /// Panics if `node` _is not_ a root, or if `at` _is_ a root.
    pub fn insert_after(&mut self, at: Index, node: Index) {
        if self.0[node].parent.is_some() {
            panic!("Forest - can only insert whole trees (after)");
        }
        let crack = Crack::new_after(self, at);
        crack.fill(self, node);
    }

    /// Insert the root node `node` as the first child of `parent`.
    /// Panics if `node` _is not_ a root.
    pub fn insert_first_child(&mut self, parent: Index, node: Index) {
        if self.0[node].parent.is_some() {
            panic!("Forest - can only insert whole trees (first_child)");
        }
        let crack = Crack::new_first_child(self, parent);
        crack.fill(self, node);
    }

    /// Insert the root node `node` as the last child of `parent`.
    /// Panics if `node` _is not_ a root.
    pub fn insert_last_child(&mut self, parent: Index, node: Index) {
        if self.0[node].parent.is_some() {
            panic!("Forest - can only insert whole trees (last_child)");
        }
        let crack = Crack::new_last_child(self, parent);
        crack.fill(self, node);
    }

    fn link(&mut self, prev: Index, next: Index) {
        self.0[prev].next = next;
        self.0[next].prev = prev;
    }

    #[cfg(test)]
    fn is_linked(&self, prev: Index, next: Index) -> bool {
        self.0[prev].next == next && self.0[next].prev == prev
    }

    #[cfg(test)]
    fn all_roots(&self) -> Vec<Index> {
        self.0
            .iter()
            .filter(|(_, node)| node.parent.is_none())
            .map(|(i, _)| i)
            .collect()
    }

    #[cfg(test)]
    fn num_nodes(&self) -> usize {
        self.0.len()
    }
}

enum Crack {
    Root,
    WithoutSiblings {
        parent: Index,
    },
    WithSiblings {
        parent: Index,
        prev: Index,
        next: Index,
        is_first: bool,
    },
}

impl Crack {
    fn new_before<D>(f: &Forest<D>, node: Index) -> Crack {
        if let Some(parent) = f.0[node].parent {
            Crack::WithSiblings {
                parent,
                prev: f.prev(node),
                next: node,
                is_first: f.0[parent].first_child == Some(node),
            }
        } else {
            panic!("Forest - can't insert before a root");
        }
    }

    fn new_after<D>(f: &Forest<D>, node: Index) -> Crack {
        if let Some(parent) = f.0[node].parent {
            Crack::WithSiblings {
                parent,
                prev: node,
                next: f.next(node),
                is_first: false,
            }
        } else {
            panic!("Forest - can't insert after a root");
        }
    }

    fn new_first_child<D>(f: &Forest<D>, parent: Index) -> Crack {
        if let Some(old_first_child) = f.0[parent].first_child {
            Crack::WithSiblings {
                parent,
                prev: f.prev(old_first_child),
                next: old_first_child,
                is_first: true,
            }
        } else {
            Crack::WithoutSiblings { parent }
        }
    }

    fn new_last_child<D>(f: &Forest<D>, parent: Index) -> Crack {
        if let Some(old_first_child) = f.0[parent].first_child {
            Crack::WithSiblings {
                parent,
                prev: f.prev(old_first_child),
                next: old_first_child,
                is_first: false,
            }
        } else {
            Crack::WithoutSiblings { parent }
        }
    }

    fn new_remove<D>(f: &mut Forest<D>, node: Index) -> Crack {
        let crack = if let Some(parent) = f.0[node].parent {
            if (f.next(node) == node) {
                Crack::WithoutSiblings { parent }
            } else {
                Crack::WithSiblings {
                    parent,
                    prev: f.prev(node),
                    next: f.next(node),
                    is_first: f.0[parent].first_child == Some(node),
                }
            }
        } else {
            Crack::Root
        };

        f.0[node].parent = None;
        f.0[node].prev = node;
        f.0[node].next = node;

        crack
    }

    fn close<D>(self, f: &mut Forest<D>) {
        match self {
            Crack::Root => (),
            Crack::WithoutSiblings { parent } => {
                f.0[parent].first_child = None;
            }
            Crack::WithSiblings {
                parent,
                prev,
                next,
                is_first,
            } => {
                f.link(prev, next);
                if is_first {
                    f.0[parent].first_child = Some(next);
                }
            }
        }
    }

    fn fill<D>(self, f: &mut Forest<D>, node: Index) {
        match self {
            Crack::Root => {
                f.0[node].parent = None;
                f.link(node, node);
            }
            Crack::WithoutSiblings { parent } => {
                f.0[node].parent = Some(parent);
                f.0[parent].first_child = Some(node);
                f.link(node, node);
            }
            Crack::WithSiblings {
                parent,
                prev,
                next,
                is_first,
            } => {
                f.0[node].parent = Some(parent);
                if is_first {
                    f.0[parent].first_child = Some(node);
                }
                f.link(prev, node);
                f.link(node, next);
            }
        }
    }
}

#[cfg(test)]
mod forest_tests {
    use super::*;
    use std::fmt::{Debug, Display};

    /// Verify and print a forest. Panic if verification fails. Verification checks:
    ///
    /// - Every node is accounted for in a tree.
    /// - All next&prev links form cycles.
    /// - For every node P with a `first_child`, all siblings of that child (via next&prev)
    ///   have `parent = P`.
    struct Verifier<'a, D: Debug + Display> {
        node_count: usize,
        display: String,
        forest: &'a Forest<D>,
    }

    impl<'a, D: Debug + Display> Verifier<'a, D> {
        fn new(forest: &'a Forest<D>) -> Verifier<'a, D> {
            Verifier {
                node_count: 0,
                display: String::new(),
                forest,
            }
        }

        fn verify(mut self) -> String {
            // Walk each tree
            for root in self.forest.all_roots() {
                self.verify_tree(root, None, root);
            }
            // Check that every node has been accounted for
            assert_eq!(self.node_count, self.forest.num_nodes());
            self.display
        }

        fn verify_tree(
            &mut self,
            node: Index,
            expected_parent: Option<Index>,
            expected_root: Index,
        ) {
            assert!(self.forest.is_valid(node));
            assert_eq!(self.forest.parent(node), expected_parent);
            assert_eq!(self.forest.root(node), expected_root);
            assert!(self.forest.is_linked(self.forest.prev(node), node));
            assert!(self.forest.is_linked(node, self.forest.next(node)));

            self.display.push('(');
            self.display
                .push_str(&format!("{}", self.forest.data(node)));
            self.node_count += 1;
            if let Some(first_child) = self.forest.first_child(node) {
                let mut child = first_child;
                assert!(self.forest.is_first(child));
                assert!(self.forest.is_last(self.forest.prev(child)));
                loop {
                    self.display.push(' ');
                    self.verify_tree(child, Some(node), expected_root);
                    child = self.forest.next(child);
                    if child == first_child {
                        break;
                    }
                }
            }
            self.display.push(')');
        }
    }

    fn verify_and_print<D: Debug + Display>(forest: &Forest<D>) -> String {
        Verifier::new(forest).verify()
    }

    fn make_mirror(forest: &mut Forest<u32>, height: u32, id: u32) -> Index {
        if height == 0 {
            forest.new_node(id)
        } else {
            let parent = forest.new_node(id);
            for i in 0..height {
                let child = make_mirror(forest, i, id + 2_u32.pow(i));
                forest.insert_last_child(parent, child);
            }
            parent
        }
    }

    #[test]
    fn test_leaf() {
        let mut forest = Forest::new();
        forest.new_node("leaf");
        assert_eq!(verify_and_print(&forest), "(leaf)");
    }

    #[test]
    fn test_branch() {
        let mut forest = Forest::new();
        let parent = forest.new_node("parent");
        let elder_sister = forest.new_node("elderSister");
        let younger_sister = forest.new_node("youngerSister");
        forest.insert_first_child(parent, younger_sister);
        forest.insert_first_child(parent, elder_sister);
        assert_eq!(
            verify_and_print(&forest),
            "(parent (elderSister) (youngerSister))"
        );
    }

    #[test]
    fn test_mirror() {
        let mut f = Forest::new();
        make_mirror(&mut f, 3, 0);
        assert_eq!(verify_and_print(&f), "(0 (1) (2 (3)) (4 (5) (6 (7))))");
    }

    #[test]
    fn test_mutation() {
        fn nth_child<D>(f: &Forest<D>, n: usize, parent: Index) -> Index {
            let mut child = f.first_child(parent).unwrap();
            for _ in 0..n {
                child = f.next(child);
            }
            child
        }

        let mut f = Forest::new();
        let root = make_mirror(&mut f, 3, 0);
        *f.data_mut(root) = 100;
        *f.data_mut(nth_child(&f, 0, nth_child(&f, 1, root))) = 33;
        let last_child = nth_child(&f, 2, root);
        *f.data_mut(nth_child(&f, 0, last_child)) = 55;
        *f.data_mut(nth_child(&f, 0, nth_child(&f, 1, last_child))) = 77;
        assert_eq!(verify_and_print(&f), "(100 (1) (2 (33)) (4 (55) (6 (77))))");
    }

    #[test]
    fn test_modification() {
        let mut f = Forest::<&'static str>::new();

        let kid = f.new_node("kid");
        let mama = f.new_node("mama");
        f.insert_first_child(kid, mama);
        let papa = f.new_node("papa");
        f.insert_last_child(kid, papa);

        let gram = f.new_node("gram");
        f.insert_last_child(mama, gram);
        let gramp = f.new_node("gramp");
        f.insert_first_child(mama, gramp);

        let ogramp = f.new_node("ogramp");
        f.insert_last_child(papa, ogramp);
        let ogram = f.new_node("ogram");
        f.insert_last_child(papa, ogram);

        assert_eq!(
            verify_and_print(&f),
            "(kid (mama (gramp) (gram)) (papa (ogramp) (ogram)))"
        );

        f.detach(mama);
        f.detach(mama);
        assert_eq!(
            verify_and_print(&f),
            "(kid (papa (ogramp) (ogram)))(mama (gramp) (gram))"
        );

        f.detach(gramp);
        f.insert_first_child(kid, gramp);
        assert_eq!(
            verify_and_print(&f),
            "(kid (gramp) (papa (ogramp) (ogram)))(mama (gram))"
        );

        f.swap(kid, mama);
        f.swap(gramp, gram);
        assert_eq!(
            verify_and_print(&f),
            "(kid (gram) (papa (ogramp) (ogram)))(mama (gramp))"
        );

        f.detach(papa);
        f.delete_root(papa);
        assert!(!f.is_valid(papa));
        assert!(!f.is_valid(ogramp));
        assert!(!f.is_valid(ogram));
        assert_eq!(verify_and_print(&f), "(kid (gram))(mama (gramp))");
    }

    // Error Testing //

    #[test]
    #[should_panic(expected = "Forest - can only delete whole trees")]
    fn test_delete_child_panic() {
        let mut f = Forest::<()>::new();
        let parent = f.new_node(());
        let child = f.new_node(());
        f.insert_first_child(parent, child);
        f.delete_root(child);
    }

    #[test]
    #[should_panic(expected = "Forest - can't insert before a root")]
    fn test_insert_before_root_panic() {
        let mut f = Forest::<()>::new();
        let root = f.new_node(());
        let child = f.new_node(());
        f.insert_before(root, child);
    }

    #[test]
    #[should_panic(expected = "Forest - can't insert after a root")]
    fn test_insert_after_root_panic() {
        let mut f = Forest::<()>::new();
        let root = f.new_node(());
        let child = f.new_node(());
        f.insert_after(root, child);
    }

    #[test]
    #[should_panic(expected = "No element at index")]
    fn test_use_after_free_panic() {
        let mut f = Forest::<()>::new();
        let root = f.new_node(());
        f.delete_root(root);
        let root_2 = f.new_node(());
        f.data(root);
    }

    /*
    #[test]
    #[should_panic(expected = "Forest - attempt to create cycle using `insert_child` thwarted")]
    fn test_cycle() {
        let mut f = Forest::<u32, u32>::new();
        let tree = f.new_branch(0);
        tree.insert_child(&mut f, 0, tree);
    }

    #[test]
    #[should_panic(expected = "Forest - swap can only be called on non-overlapping nodes")]
    fn test_swap_cycle() {
        let mut f = Forest::<u32, u32>::new();
        let tree = f.new_branch(0);
        tree.swap(&mut f, tree);
    }
    */
}
