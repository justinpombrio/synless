use std::fmt::Debug;
use std::iter;

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

    /// Insert a new stair into a staircase.
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

    fn indices(&self, x: usize, y: usize) -> (usize, usize, usize, usize) {
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
    }
}
