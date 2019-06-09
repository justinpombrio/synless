use std::fmt;
use std::ops::Add;
use std::ops::Sub;

/// (Used in `bound` module only, to simplify the implementation of
/// Bounds and Regions.)
/// A range of either rows or columns.
/// The start point is included, but not the end point, so
/// Range(2,4) means rows/columns 2&3.
///
/// INVARIANT: `Range(a,b)` implies `b-a>=1`.
#[derive(PartialEq, Eq, Clone, Copy)]
pub struct Range<N>(pub N, pub N);

impl<N> Range<N>
where
    N: Add<Output = N>,
    N: Sub<Output = N>,
    N: Ord,
    N: Copy,
{
    pub fn overlaps(self, other: Range<N>) -> bool {
        !self.is_left_of(other) && !other.is_left_of(self)
    }

    pub fn contains(self, n: N) -> bool {
        self.0 <= n && n < self.1
    }

    pub fn covers(self, other: Range<N>) -> bool {
        self.0 <= other.0 && other.1 <= self.1
    }

    pub fn is_left_of(self, other: Range<N>) -> bool {
        self.1 <= other.0
    }

    pub fn transform(self, n: N) -> Option<N> {
        if self.contains(n) {
            Some(n - self.0)
        } else {
            None
        }
    }
}

impl<N> Add<N> for Range<N>
where
    N: Add<N, Output = N>,
    N: Copy,
{
    type Output = Range<N>;
    fn add(self, n: N) -> Range<N> {
        Range(self.0 + n, self.1 + n)
    }
}

impl<N> fmt::Display for Range<N>
where
    N: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}-{}", self.0, self.1)
    }
}

/// Allow Range to be used in a for-loop by converting it to the std lib's range
/// type, which is an iterator.
impl<N> IntoIterator for Range<N>
where
    N: ::std::iter::Step,
{
    type Item = N;
    type IntoIter = ::std::ops::Range<N>;

    fn into_iter(self) -> Self::IntoIter {
        self.0..self.1
    }
}
