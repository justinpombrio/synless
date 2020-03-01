use std::fmt::Debug;
use std::iter;

/// A set of elements of type `T`. The elements have two `usize` dimensions `x`
/// and `y`, as defined by the `Stair` trait. If one element has smaller `x`
/// and smaller `y` than another, we say it _dominates_ it. This set only stores
/// elements that are not dominated by any other element in the set. As a
/// result, it will never need to store more than `max(X, Y)` elements, where
/// `X` and `Y` are the number of possible values of `x` and `y` respectively.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Staircase<T>
where
    T: Stair,
{
    stairs: Vec<T>,
}

pub trait Stair: Clone + Debug + PartialEq + Eq {
    fn x(&self) -> usize;
    fn y(&self) -> usize;
}

impl<T: Stair> Staircase<T> {
    /// Construct an empty staircase.
    pub fn new() -> Staircase<T> {
        Staircase { stairs: Vec::new() }
    }

    pub fn is_empty(&self) -> bool {
        self.stairs.is_empty()
    }

    pub fn len(&self) -> usize {
        self.stairs.len()
    }

    pub fn insert(&mut self, stair: T) {
        let (skip_left, skip_right, delete_left, delete_right) = self.indices(stair.x(), stair.y());
        // If the new stair is already covered, skip it.
        if skip_left < skip_right {
            return;
        }
        // If the new stair covers existing stairs, delete them.
        if delete_left < delete_right {
            self.stairs.drain(delete_left..delete_right);
        }
        // Insert the new stair.
        self.stairs.insert(delete_left, stair);
    }

    pub fn unchecked_insert(&mut self, stair: T) {
        let (skip_left, _, _, _) = self.indices(stair.x(), stair.y());
        self.stairs.insert(skip_left, stair);
    }

    pub fn iter<'a>(&'a self) -> impl Iterator<Item = &'a T> {
        self.into_iter()
    }

    pub fn drain<'a>(&'a mut self) -> impl Iterator<Item = T> + 'a {
        self.stairs.drain(..)
    }

    pub fn min_by_x(&self) -> Option<&T> {
        self.stairs.iter().last()
    }

    pub fn min_by_y(&self) -> Option<&T> {
        self.stairs.iter().next()
    }

    fn indices(&self, x: usize, y: usize) -> (usize, usize, usize, usize) {
        // TODO: linear search for efficiency
        // (Basic testing shows that it's more efficient for sets of size <100 or so.)
        let x_index = self
            .stairs
            .binary_search_by_key(&-(x as isize), |stair| -(stair.x() as isize));
        let y_index = self.stairs.binary_search_by_key(&y, |stair| stair.y());
        let (skip_left, delete_right) = match x_index {
            Ok(i) => (i, i + 1),
            Err(i) => (i, i),
        };
        let (delete_left, skip_right) = match y_index {
            Ok(i) => (i, i + 1),
            Err(i) => (i, i),
        };
        (skip_left, skip_right, delete_left, delete_right)
    }
}

impl<T: Stair> iter::IntoIterator for Staircase<T> {
    type Item = T;
    type IntoIter = <Vec<T> as iter::IntoIterator>::IntoIter;
    fn into_iter(self) -> Self::IntoIter {
        self.stairs.into_iter()
    }
}

impl<'a, T: Stair> iter::IntoIterator for &'a Staircase<T> {
    type Item = &'a T;
    type IntoIter = <&'a [T] as iter::IntoIterator>::IntoIter;
    fn into_iter(self) -> Self::IntoIter {
        self.stairs.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct CharStair {
        x: usize,
        y: usize,
        c: char,
    }

    impl CharStair {
        fn new(x: usize, y: usize, c: char) -> Self {
            Self { x, y, c }
        }
    }

    impl Stair for CharStair {
        fn x(&self) -> usize {
            self.x
        }
        fn y(&self) -> usize {
            self.y
        }
    }

    fn basic_stairs() -> Staircase<CharStair> {
        let mut stairs = Staircase::new();
        stairs.insert(CharStair::new(6, 2, 'a'));
        stairs.insert(CharStair::new(2, 6, 'a'));
        stairs.insert(CharStair::new(4, 4, 'a'));
        stairs
    }

    #[test]
    fn test_empty_staircase() {
        let stairs: Staircase<CharStair> = Staircase::new();
        assert_eq!(stairs.stairs, vec![]);
        assert_eq!(stairs.min_by_x(), None);
        assert_eq!(stairs.min_by_y(), None);
    }

    #[test]
    fn test_staircase() {
        let mut stairs = basic_stairs();
        stairs.insert(CharStair::new(5, 5, 'x'));
        stairs.insert(CharStair::new(4, 4, 'b'));
        stairs.insert(CharStair::new(4, 5, 'x'));
        stairs.insert(CharStair::new(3, 5, 'b'));
        stairs.insert(CharStair::new(8, 5, 'x'));
        stairs.insert(CharStair::new(5, 3, 'b'));
        stairs.insert(CharStair::new(6, 3, 'x'));
        assert_eq!(
            stairs.stairs,
            vec![
                CharStair::new(6, 2, 'a'),
                CharStair::new(5, 3, 'b'),
                CharStair::new(4, 4, 'a'),
                CharStair::new(3, 5, 'b'),
                CharStair::new(2, 6, 'a')
            ]
        );
        assert_eq!(stairs.min_by_x(), Some(&CharStair::new(2, 6, 'a')));
        assert_eq!(stairs.min_by_y(), Some(&CharStair::new(6, 2, 'a')));
    }
}
