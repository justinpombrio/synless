use crate::infra::{bug, SynlessBug};
use generational_arena::Arena;

/// An index into a Forest, which represents a node in a tree.
pub type NodeIndex = generational_arena::Index;

/// A collection of trees. Every node in a tree has additional data D.
///
/// Deleting the ancestor of a node will delete the node.
///
/// **Methods on `Forest` will panic if they're given an Index to a node that was deleted.** The
/// one exception is the `is_valid()` method, which checks whether a node has been deleted.
///
/// This library solves these problems:
///
/// - Ensuring that parent/child links and prev/next sibling links always agree.
/// - Ensuring that every tree is accounted for.
/// - Preventing cycles at runtime.
///
/// It does NOT solve these problems:
///
/// - Preventing "use after free" (see the note on deletion above).
///   Along the same lines, preventing cycles at compile time.
/// - Removing the need to pass the `Forest` in to every method call.
pub struct Forest<D> {
    // TODO: Try making roots linked in a cycle internally
    arena: Arena<Node<D>>,
    /// Exists solely for the swap() method. Avoids messiness when swapping adjacent siblings.
    swap_dummy: NodeIndex,
}

/// A node in a doubly-linked-list representation of a tree that can store some
/// data `D`. All nodes store their parent (except root nodes which don't have
/// a parent).  Parents store only their first child. The siblings are linked to
/// each other in a circle.
#[derive(Debug)]
struct Node<D> {
    parent: Option<NodeIndex>,
    /// The first child, if any.
    child: Option<NodeIndex>,
    /// Previous sibling, going in a cycle.
    prev: NodeIndex,
    /// Next sibling, going in a cycle.
    next: NodeIndex,
    data: D,
}

impl<D> Forest<D> {
    /// Create a new empty forest. The `dummy_data` will never be used (don't worry about it).
    pub fn new(dummy_data: D) -> Forest<D> {
        let mut arena = Arena::new();
        let swap_dummy = arena.insert_with(|idx| Node {
            parent: None,
            child: None,
            prev: idx,
            next: idx,
            data: dummy_data,
        });
        Forest { arena, swap_dummy }
    }

    /// Create a new root node containing the given data.
    pub fn new_node(&mut self, data: D) -> NodeIndex {
        self.arena.insert_with(|idx| Node {
            parent: None,
            prev: idx,
            next: idx,
            child: None,
            data,
        })
    }

    /// Get the parent of `node`, or `None` if it's a root.
    pub fn parent(&self, node: NodeIndex) -> Option<NodeIndex> {
        self.arena[node].parent
    }

    /// Get the root of the tree that `node` belongs to (exactly the node you
    /// would get by calling `parent` repeatedly).
    pub fn root(&self, node: NodeIndex) -> NodeIndex {
        let mut ancestor = node;
        while let Some(parent) = self.arena[ancestor].parent {
            ancestor = parent;
        }
        ancestor
    }

    /// Get the first child of `node`, or `None` if it has no children.
    pub fn first_child(&self, node: NodeIndex) -> Option<NodeIndex> {
        self.arena[node].child
    }

    /// Get the `node`'s n'th child, if any.
    pub fn nth_child(&self, node: NodeIndex, n: usize) -> Option<NodeIndex> {
        if let Some(first_child) = self.arena[node].child {
            let mut child = first_child;
            for _ in 0..n {
                child = self.arena[child].next;
                if child == first_child {
                    return None;
                }
            }
            Some(child)
        } else {
            None
        }
    }

    /// Get the `node`'s previous sibling, if any.
    pub fn prev(&self, node: NodeIndex) -> Option<NodeIndex> {
        if self.is_first(node) {
            None
        } else {
            Some(self.arena[node].prev)
        }
    }

    /// Get the `node`'s next sibling, if any.
    pub fn next(&self, node: NodeIndex) -> Option<NodeIndex> {
        if self.is_last(node) {
            None
        } else {
            Some(self.arena[node].next)
        }
    }

    /// Get the `node`'s leftmost sibling (which could be itself).
    pub fn first_sibling(&self, node: NodeIndex) -> NodeIndex {
        if let Some(parent) = self.arena[node].parent {
            self.arena[parent].child.bug()
        } else {
            node
        }
    }

    /// Get the `node`'s rightmost sibling (which could be itself).
    pub fn last_sibling(&self, node: NodeIndex) -> NodeIndex {
        if let Some(parent) = self.arena[node].parent {
            self.arena[self.arena[parent].child.bug()].prev
        } else {
            node
        }
    }

    /// Whether this node is first among its siblings. (True if there's one sibling.)
    pub fn is_first(&self, node: NodeIndex) -> bool {
        if let Some(parent) = self.arena[node].parent {
            self.arena[parent].child.bug() == node
        } else {
            true
        }
    }

    /// Whether this node is last among its siblings. (True if there's one sibling.)
    pub fn is_last(&self, node: NodeIndex) -> bool {
        if let Some(parent) = self.arena[node].parent {
            self.arena[parent].child.bug() == self.arena[node].next
        } else {
            true
        }
    }

    pub fn num_children(&self, node: NodeIndex) -> usize {
        if let Some(child) = self.arena[node].child {
            let mut num_children = 1;
            let mut other_child = self.arena[child].next;
            while other_child != child {
                num_children += 1;
                other_child = self.arena[other_child].next;
            }
            num_children
        } else {
            0
        }
    }

    pub fn sibling_index(&self, node: NodeIndex) -> usize {
        let mut sibling = self.first_sibling(node);
        let mut sibling_index = 0;
        while sibling != node {
            sibling = self.arena[sibling].next;
            sibling_index += 1;
        }
        sibling_index
    }

    /// Borrows the data stored inside `node`.
    pub fn data(&self, node: NodeIndex) -> &D {
        &self.arena[node].data
    }

    /// Mutably borrows the data stored inside `node`.
    pub fn data_mut(&mut self, node: NodeIndex) -> &mut D {
        &mut self.arena[node].data
    }

    /// Delete the given `root` node, including all of its descendants.
    /// This invalidates all of their `NodeIndex`es. Panics if `node` is not a root.
    pub fn delete_root(&mut self, root: NodeIndex) {
        if self.arena[root].parent.is_some() {
            bug!("Forest - can only delete whole trees");
        }

        // Stack of (nodes_first_sibling, node)
        let mut to_delete = vec![(root, root)];
        while let Some((first_sibling, node)) = to_delete.pop() {
            if self.arena[node].next != first_sibling {
                to_delete.push((first_sibling, self.arena[node].next));
            }
            if let Some(child) = self.arena[node].child {
                to_delete.push((child, child));
            }
            self.arena.remove(node);
        }
    }

    /// True if the forest still contains this node (i.e. its tree hasn't been deleted).
    pub fn is_valid(&self, node: NodeIndex) -> bool {
        self.arena.contains(node)
    }

    /// Remove `node` (and its descendants) from its place in the tree, making it the
    /// root of a new tree.
    pub fn detach(&mut self, node: NodeIndex) {
        let crack = Crack::new_remove(self, node);
        crack.seal(self);
    }

    /// Swap the positions of two nodes (and their descendants). The two nodes may be part
    /// of the same tree or different trees, but they must not overlap - i.e. one node must
    /// not be an ancestor of the other. Unless they're the same node, which is a no-op.
    /// If the nodes do overlap, returns `false` and does nothing.
    pub fn swap(&mut self, node_1: NodeIndex, node_2: NodeIndex) -> bool {
        if node_1 == node_2 {
            return true;
        }
        if self.overlaps(node_1, node_2) {
            return false;
        }

        let crack_1 = Crack::new_remove(self, node_1);
        crack_1.fill(self, self.swap_dummy);

        let crack_2 = Crack::new_remove(self, node_2);
        crack_2.fill(self, node_1);

        let crack_1_again = Crack::new_remove(self, self.swap_dummy);
        crack_1_again.fill(self, node_2);

        true
    }

    /// Insert the root node `node` before `at`, so that `node` becomes its previous sibling.
    /// Panics if `node` _is not_ a root, or if `at` _is_ a root.
    pub fn insert_before(&mut self, at: NodeIndex, node: NodeIndex) {
        if self.arena[node].parent.is_some() {
            bug!("Forest - can only insert whole trees (before)");
        }
        let crack = Crack::new_before(self, at);
        crack.fill(self, node);
    }

    /// Insert the root node `node` after `at`, so that `node` becomes its next sibling.
    /// Panics if `node` _is not_ a root, or if `at` _is_ a root.
    pub fn insert_after(&mut self, at: NodeIndex, node: NodeIndex) {
        if self.arena[node].parent.is_some() {
            bug!("Forest - can only insert whole trees (after)");
        }
        let crack = Crack::new_after(self, at);
        crack.fill(self, node);
    }

    /// Insert the root node `node` as the first child of `parent`.
    /// Panics if `node` _is not_ a root.
    pub fn insert_first_child(&mut self, parent: NodeIndex, node: NodeIndex) {
        if self.arena[node].parent.is_some() {
            bug!("Forest - can only insert whole trees (first_child)");
        }
        if self.root(parent) == node {
            bug!("Forest - attempt to create cycle using `insert_child` thwarted");
        }
        let crack = Crack::new_first_child(self, parent);
        crack.fill(self, node);
    }

    /// Insert the root node `node` as the last child of `parent`.
    /// Panics if `node` _is not_ a root.
    pub fn insert_last_child(&mut self, parent: NodeIndex, node: NodeIndex) {
        if self.arena[node].parent.is_some() {
            bug!("Forest - can only insert whole trees (last_child)");
        }
        let crack = Crack::new_last_child(self, parent);
        crack.fill(self, node);
    }

    fn overlaps(&self, node_1: NodeIndex, node_2: NodeIndex) -> bool {
        self.is_ancestor_of(node_1, node_2) || self.is_ancestor_of(node_2, node_1)
    }

    fn is_ancestor_of(&self, potential_ancestor: NodeIndex, node: NodeIndex) -> bool {
        if node == potential_ancestor {
            return true;
        }

        let mut ancestor = node;
        while let Some(parent) = self.arena[ancestor].parent {
            if parent == potential_ancestor {
                return true;
            }
            ancestor = parent;
        }
        false
    }

    fn link(&mut self, prev: NodeIndex, next: NodeIndex) {
        self.arena[prev].next = next;
        self.arena[next].prev = prev;
    }

    #[cfg(test)]
    fn is_linked(&self, prev: NodeIndex, next: NodeIndex) -> bool {
        self.arena[prev].next == next && self.arena[next].prev == prev
    }

    #[cfg(test)]
    fn all_roots(&self) -> impl Iterator<Item = NodeIndex> + '_ {
        self.arena
            .iter()
            .filter(|(_, node)| node.parent.is_none())
            .map(|(i, _)| i)
    }

    #[cfg(test)]
    fn num_nodes(&self) -> usize {
        self.arena.len()
    }
}

// NOTE: Never create two adjacent cracks. It'll be like that episode of The Good Place.
enum Crack {
    Root,
    WithoutSiblings {
        parent: NodeIndex,
    },
    WithSiblings {
        parent: NodeIndex,
        prev: NodeIndex,
        next: NodeIndex,
        is_first: bool,
    },
}

impl Crack {
    fn new_before<D>(f: &Forest<D>, node: NodeIndex) -> Crack {
        if let Some(parent) = f.arena[node].parent {
            Crack::WithSiblings {
                parent,
                prev: f.arena[node].prev,
                next: node,
                is_first: f.arena[parent].child == Some(node),
            }
        } else {
            bug!("Forest - can't insert before a root");
        }
    }

    fn new_after<D>(f: &Forest<D>, node: NodeIndex) -> Crack {
        if let Some(parent) = f.arena[node].parent {
            Crack::WithSiblings {
                parent,
                prev: node,
                next: f.arena[node].next,
                is_first: false,
            }
        } else {
            bug!("Forest - can't insert after a root");
        }
    }

    fn new_first_child<D>(f: &Forest<D>, parent: NodeIndex) -> Crack {
        if let Some(old_first_child) = f.arena[parent].child {
            Crack::WithSiblings {
                parent,
                prev: f.arena[old_first_child].prev,
                next: old_first_child,
                is_first: true,
            }
        } else {
            Crack::WithoutSiblings { parent }
        }
    }

    fn new_last_child<D>(f: &Forest<D>, parent: NodeIndex) -> Crack {
        if let Some(old_first_child) = f.arena[parent].child {
            Crack::WithSiblings {
                parent,
                prev: f.arena[old_first_child].prev,
                next: old_first_child,
                is_first: false,
            }
        } else {
            Crack::WithoutSiblings { parent }
        }
    }

    fn new_remove<D>(f: &mut Forest<D>, node: NodeIndex) -> Crack {
        if let Some(parent) = f.arena[node].parent {
            let crack = if f.arena[node].next == node {
                Crack::WithoutSiblings { parent }
            } else {
                Crack::WithSiblings {
                    parent,
                    prev: f.arena[node].prev,
                    next: f.arena[node].next,
                    is_first: f.arena[parent].child == Some(node),
                }
            };

            f.arena[node].parent = None;
            f.link(node, node);

            crack
        } else {
            Crack::Root
        }
    }

    fn seal<D>(self, f: &mut Forest<D>) {
        match self {
            Crack::Root => (),
            Crack::WithoutSiblings { parent } => {
                f.arena[parent].child = None;
            }
            Crack::WithSiblings {
                parent,
                prev,
                next,
                is_first,
            } => {
                f.link(prev, next);
                if is_first {
                    f.arena[parent].child = Some(next);
                }
            }
        }
    }

    fn fill<D>(self, f: &mut Forest<D>, node: NodeIndex) {
        if f.parent(node).is_some() {
            bug!("Forest - can only fill with a root");
        }

        match self {
            Crack::Root => (),
            Crack::WithoutSiblings { parent } => {
                f.arena[node].parent = Some(parent);
                f.arena[parent].child = Some(node);
                f.link(node, node);
            }
            Crack::WithSiblings {
                parent,
                prev,
                next,
                is_first,
            } => {
                f.arena[node].parent = Some(parent);
                if is_first {
                    f.arena[parent].child = Some(node);
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
    /// - For every node P with a `child`, all siblings of that child (via next&prev)
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
                if root == self.forest.swap_dummy {
                    self.node_count += 1;
                } else {
                    self.verify_tree(root, None, root);
                }
            }
            // Check that every node has been accounted for.
            assert_eq!(self.node_count, self.forest.num_nodes());
            self.display
        }

        fn verify_tree(
            &mut self,
            node: NodeIndex,
            expected_parent: Option<NodeIndex>,
            expected_root: NodeIndex,
        ) {
            assert!(self.forest.is_valid(node));
            assert_eq!(self.forest.parent(node), expected_parent);
            assert_eq!(self.forest.root(node), expected_root);
            assert!(self.forest.is_linked(self.forest.arena[node].prev, node));
            assert!(self.forest.is_linked(node, self.forest.arena[node].next));

            self.display.push('(');
            self.display
                .push_str(&format!("{}", self.forest.data(node)));
            self.node_count += 1;
            let mut num_children = 0;
            if let Some(first_child) = self.forest.first_child(node) {
                let mut child = first_child;
                assert!(self.forest.is_first(child));
                assert!(self.forest.prev(child).is_none());
                assert!(self.forest.is_last(self.forest.arena[child].prev));
                assert!(self.forest.next(self.forest.arena[child].prev).is_none());
                loop {
                    self.display.push(' ');
                    self.verify_tree(child, Some(node), expected_root);
                    num_children += 1;
                    match self.forest.next(child) {
                        None => break,
                        Some(c) => child = c,
                    }
                }
            }
            assert_eq!(self.forest.num_children(node), num_children);
            self.display.push(')');
        }
    }

    fn verify_and_print<D: Debug + Display>(forest: &Forest<D>) -> String {
        Verifier::new(forest).verify()
    }

    fn make_mirror(forest: &mut Forest<u32>, height: u32, id: u32) -> NodeIndex {
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

    fn make_sisters(forest: &mut Forest<&'static str>) -> NodeIndex {
        let parent = forest.new_node("parent");
        let elder_sister = forest.new_node("elderSister");
        let younger_sister = forest.new_node("youngerSister");
        forest.insert_first_child(parent, younger_sister);
        forest.insert_first_child(parent, elder_sister);
        parent
    }

    #[test]
    fn test_leaf() {
        let mut forest = Forest::new("");
        forest.new_node("leaf");
        assert_eq!(verify_and_print(&forest), "(leaf)");
    }

    #[test]
    fn test_branch() {
        let mut forest = Forest::new("");
        make_sisters(&mut forest);
        assert_eq!(
            verify_and_print(&forest),
            "(parent (elderSister) (youngerSister))"
        );
    }

    #[test]
    fn test_siblings() {
        let mut forest = Forest::new("");
        let parent = make_sisters(&mut forest);
        let elder = forest.first_child(parent).unwrap();
        let younger = forest.next(elder).unwrap();

        assert_eq!(forest.first_sibling(younger), elder);
        assert_eq!(forest.first_sibling(elder), elder);
        assert_eq!(forest.last_sibling(younger), younger);
        assert_eq!(forest.last_sibling(elder), younger);
    }

    #[test]
    fn test_swap() {
        let mut forest = Forest::new("");
        let parent = make_sisters(&mut forest);
        let elder = forest.first_child(parent).unwrap();
        let younger = forest.next(elder).unwrap();

        forest.swap(elder, elder);
        assert_eq!(
            verify_and_print(&forest),
            "(parent (elderSister) (youngerSister))"
        );

        forest.swap(elder, younger);
        assert_eq!(
            verify_and_print(&forest),
            "(parent (youngerSister) (elderSister))"
        );

        forest.swap(elder, younger);
        assert_eq!(
            verify_and_print(&forest),
            "(parent (elderSister) (youngerSister))"
        );

        let middle = forest.new_node("middleSister");
        forest.insert_after(elder, middle);
        forest.swap(elder, middle);
        assert_eq!(
            verify_and_print(&forest),
            "(parent (middleSister) (elderSister) (youngerSister))"
        );

        forest.swap(middle, younger);
        assert_eq!(
            verify_and_print(&forest),
            "(parent (youngerSister) (elderSister) (middleSister))"
        );
    }

    #[test]
    fn test_mirror() {
        let mut f = Forest::new(0);
        make_mirror(&mut f, 3, 0);
        assert_eq!(verify_and_print(&f), "(0 (1) (2 (3)) (4 (5) (6 (7))))");
    }

    #[test]
    fn test_mutation() {
        fn nth_child<D>(f: &Forest<D>, n: usize, parent: NodeIndex) -> NodeIndex {
            let mut child = f.first_child(parent).unwrap();
            for _ in 0..n {
                child = f.next(child).unwrap();
            }
            child
        }

        let mut f = Forest::new(0);
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
        let mut f = Forest::<&'static str>::new("");

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
        let mut f = Forest::<()>::new(());
        let parent = f.new_node(());
        let child = f.new_node(());
        f.insert_first_child(parent, child);
        f.delete_root(child);
    }

    #[test]
    #[should_panic(expected = "Forest - can't insert before a root")]
    fn test_insert_before_root_panic() {
        let mut f = Forest::<()>::new(());
        let root = f.new_node(());
        let child = f.new_node(());
        f.insert_before(root, child);
    }

    #[test]
    #[should_panic(expected = "Forest - can't insert after a root")]
    fn test_insert_after_root_panic() {
        let mut f = Forest::<()>::new(());
        let root = f.new_node(());
        let child = f.new_node(());
        f.insert_after(root, child);
    }

    #[test]
    #[should_panic(expected = "No element at index")]
    fn test_use_after_free_panic() {
        let mut f = Forest::<()>::new(());
        let root = f.new_node(());
        f.delete_root(root);
        f.new_node(());
        f.data(root);
    }

    #[test]
    #[should_panic(expected = "Forest - attempt to create cycle using `insert_child` thwarted")]
    fn test_cycle() {
        let mut f = Forest::<u32>::new(0);
        let tree = f.new_node(0);
        f.insert_first_child(tree, tree);
    }

    #[test]
    fn test_swap_cycle() {
        let mut f = Forest::<u32>::new(0);
        let parent = f.new_node(0);
        let child = f.new_node(0);
        f.insert_first_child(parent, child);
        assert!(!f.swap(parent, child));
    }
}
