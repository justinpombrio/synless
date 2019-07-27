use std::fmt;
use std::ops::Add;
use std::ops::Sub;

/// (Used in `bound` module only, to simplify the implementation of
/// Bounds and Regions.)
/// A range of either rows or columns.
/// The start point is included, but not the end point, so:
///    Range(2,4) means rows/columns 2&3.
///    Range(2,2) means an empty range (at row/column 2)
///
/// INVARIANT: `Range(a,b)` implies b >= a
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct Range<N>(pub N, pub N);

impl<N> Range<N>
where
    N: Add<Output = N>,
    N: Sub<Output = N>,
    N: Ord,
    N: Copy,
    N: fmt::Debug,
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

    /// Return the number of elements in the range
    pub fn len(self) -> N {
        self.1 - self.0
    }

    /// Split the range into 2 ranges, the first of which has length `left_len`.
    /// Returns None if `left_len` is negative or larger than the length of
    /// `self`.
    pub fn split(self, left_len: N) -> Option<(Range<N>, Range<N>)> {
        let mid = self.0 + left_len;
        if mid > self.1 || mid < self.0 {
            None
        } else {
            Some((Range(self.0, mid), Range(mid, self.1)))
        }
    }

    pub fn splits<'a>(&self, lengths: &'a [N]) -> SplitRangeIter<'a, N> {
        SplitRangeIter {
            remaining: *self,
            lengths,
        }
    }
}

/// Iterator over sub-ranges of a Range that was split
pub struct SplitRangeIter<'a, N> {
    remaining: Range<N>,
    lengths: &'a [N],
}

impl<'a, N> Iterator for SplitRangeIter<'a, N>
where
    N: Add<Output = N>,
    N: Sub<Output = N>,
    N: Ord,
    N: Copy,
    N: fmt::Debug,
{
    type Item = Range<N>;

    /// Panics if the next length is greater than the length of the remaining Range.
    fn next(&mut self) -> Option<Self::Item> {
        match self.lengths {
            &[] => None, // Done! Returned all requested sub-ranges.
            &[len] if len == self.remaining.len() => {
                // This is the last requested length AND it's exactly the length
                // of the remaining Range, so we don't need to split.
                self.lengths = &[];
                Some(self.remaining)
            }
            &[len, ref rest..] => {
                // In all other cases, try to split.
                self.lengths = rest;
                let (left, right) = self.remaining.split(len).expect("Range: failed to split");
                self.remaining = right;
                Some(left)
            }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_range() {
        assert_eq!(Range(1, 5).split(0), Some((Range(1, 1), Range(1, 5))));
        assert_eq!(Range(1, 5).split(1), Some((Range(1, 2), Range(2, 5))));
        assert_eq!(Range(1, 5).split(2), Some((Range(1, 3), Range(3, 5))));
        assert_eq!(Range(1, 5).split(3), Some((Range(1, 4), Range(4, 5))));
        assert_eq!(Range(1, 5).split(4), Some((Range(1, 5), Range(5, 5))));
        assert_eq!(Range(1, 5).split(5), None);
        assert_eq!(Range(1, 5).split(-1), None);
        assert_eq!(Range(3, 3).split(1), None);
        assert_eq!(Range(3, 3).split(0), Some((Range(3, 3), Range(3, 3))));
    }
}
