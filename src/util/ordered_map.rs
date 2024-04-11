use crate::util::SynlessBug;
use std::borrow::Borrow;
use std::mem;
use std::ops::{Index, IndexMut};

/// A map that preserves insertion order.
// TODO: OrderedMap is currently quadratic time. Could be made more efficient as a HashMap+Vec.
#[derive(Debug, Clone)]
pub struct OrderedMap<K: Eq, V>(Vec<(K, V)>);

impl<K: Eq, V> OrderedMap<K, V> {
    pub fn new() -> OrderedMap<K, V> {
        OrderedMap(Vec::new())
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        if let Some(index) = self.index(&key) {
            Some(mem::replace(&mut self.0[index].1, value))
        } else {
            self.0.push((key, value));
            None
        }
    }

    pub fn contains_key<Q>(&self, key: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Eq + ?Sized,
    {
        self.0.iter().any(|(k, _)| k.borrow() == key)
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Eq + ?Sized,
    {
        if let Some(index) = self.index(key) {
            Some(&self.0[index].1)
        } else {
            None
        }
    }

    pub fn get_mut<Q>(&mut self, key: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
        Q: Eq + ?Sized,
    {
        if let Some(index) = self.index(key) {
            Some(&mut self.0[index].1)
        } else {
            None
        }
    }

    /// Iterate over the keys in insertion order.
    pub fn keys(&self) -> impl Iterator<Item = &K> {
        self.0.iter().map(|(key, _)| key)
    }

    pub fn iter(&self) -> impl Iterator<Item = &(K, V)> {
        self.into_iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut (K, V)> {
        self.into_iter()
    }

    /// Add the bindings from `other` to `self`, overriding existing bindings.
    pub fn append(&mut self, other: OrderedMap<K, V>) {
        for (key, val) in other {
            self.insert(key, val);
        }
    }

    fn index<Q>(&self, key: &Q) -> Option<usize>
    where
        K: Borrow<Q>,
        Q: Eq + ?Sized,
    {
        for (i, (existing_key, _)) in self.0.iter().enumerate() {
            if existing_key.borrow() == key {
                return Some(i);
            }
        }
        None
    }
}

impl<K: Eq, V> Default for OrderedMap<K, V> {
    fn default() -> OrderedMap<K, V> {
        OrderedMap::new()
    }
}

impl<K, Q, V> Index<&Q> for OrderedMap<K, V>
where
    K: Eq + Borrow<Q>,
    Q: Eq + ?Sized,
{
    type Output = V;

    fn index(&self, key: &Q) -> &V {
        let index = self.index(key).bug_msg("OrderedMap: key not found");
        &self.0[index].1
    }
}

impl<K, Q, V> IndexMut<&Q> for OrderedMap<K, V>
where
    K: Eq + Borrow<Q>,
    Q: Eq + ?Sized,
{
    fn index_mut(&mut self, key: &Q) -> &mut V {
        let index = self.index(key).bug_msg("OrderedMap: key not found");
        &mut self.0[index].1
    }
}

impl<K: Eq, V> IntoIterator for OrderedMap<K, V> {
    type Item = (K, V);
    type IntoIter = std::vec::IntoIter<(K, V)>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a, K: Eq, V> IntoIterator for &'a OrderedMap<K, V> {
    type Item = &'a (K, V);
    type IntoIter = std::slice::Iter<'a, (K, V)>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<'a, K: Eq, V> IntoIterator for &'a mut OrderedMap<K, V> {
    type Item = &'a mut (K, V);
    type IntoIter = std::slice::IterMut<'a, (K, V)>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter_mut()
    }
}
