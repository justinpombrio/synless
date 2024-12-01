use std::collections::HashMap;
use std::mem;
use std::ops::{Index, IndexMut};

/// A map from `String` to `T`, that also associates a `usize` with each element for faster
/// lookups.
#[derive(Debug, Clone)]
pub struct IndexedMap<T> {
    map: HashMap<String, usize>,
    values: Vec<T>,
}

impl<T> IndexedMap<T> {
    pub fn new() -> IndexedMap<T> {
        IndexedMap {
            map: HashMap::new(),
            values: Vec::new(),
        }
    }

    /// Inserts name->value into this map, replacing the binding if the name was already present.
    /// Returns `(new_id, Option<old_value>)`.
    pub fn insert(&mut self, name: String, value: T) -> (usize, Option<T>) {
        if let Some(old_id) = self.id(&name) {
            let old_value = mem::replace(&mut self.values[old_id], value);
            (old_id, Some(old_value))
        } else {
            let new_id = self.values.len();
            self.values.push(value);
            self.map.insert(name, new_id);
            (new_id, None)
        }
    }

    pub fn contains_name(&self, name: &str) -> bool {
        self.map.contains_key(name)
    }

    pub fn get(&self, id: usize) -> Option<&T> {
        self.values.get(id)
    }

    pub fn get_mut(&mut self, id: usize) -> Option<&mut T> {
        self.values.get_mut(id)
    }

    pub fn get_by_name(&self, name: &str) -> Option<&T> {
        Some(&self.values[*self.map.get(name)?])
    }

    pub fn get_by_name_mut(&mut self, name: &str) -> Option<&mut T> {
        Some(&mut self.values[*self.map.get(name)?])
    }

    pub fn id(&self, name: &str) -> Option<usize> {
        self.map.get(name).copied()
    }

    pub fn names(&self) -> impl ExactSizeIterator<Item = &str> {
        self.map.keys().map(|name| name.as_ref())
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }
}

impl<T> Default for IndexedMap<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Index<&str> for IndexedMap<T> {
    type Output = T;

    fn index(&self, name: &str) -> &T {
        &self.values[self.map[name]]
    }
}

impl<T> IndexMut<&str> for IndexedMap<T> {
    fn index_mut(&mut self, name: &str) -> &mut T {
        &mut self.values[self.map[name]]
    }
}

impl<T> Index<usize> for IndexedMap<T> {
    type Output = T;

    fn index(&self, id: usize) -> &T {
        &self.values[id]
    }
}

impl<T> IndexMut<usize> for IndexedMap<T> {
    fn index_mut(&mut self, id: usize) -> &mut T {
        &mut self.values[id]
    }
}

impl<T> IntoIterator for &IndexedMap<T> {
    type Item = usize;
    type IntoIter = std::ops::Range<usize>;

    fn into_iter(self) -> Self::IntoIter {
        0..self.values.len()
    }
}
