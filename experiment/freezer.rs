use std::borrow::Borrow;
use std::collections::HashMap;
use std::hash::Hash;
use typed_arena::Arena;

struct FreezerStorage<V>(Arena<V>);

impl<V> FreezerStorage<V> {
    fn new() -> FreezerStorage<V> {
        FreezerStorage(Arena::new())
    }
}

struct Freezer<'a, K, V> {
    storage: &'a FreezerStorage<V>,
    map: HashMap<K, &'a V>,
}

impl<'a, K, V> Freezer<'a, K, V>
where
    K: Hash + Eq,
{
    fn new(storage: &'a FreezerStorage<V>) -> Freezer<'a, K, V> {
        Freezer {
            storage,
            map: HashMap::new(),
        }
    }

    pub fn insert(&mut self, key: K, value: V) -> Result<&'a V, V> {
        if self.map.contains_key(&key) {
            return Err(value);
        }
        let refn = self.storage.0.alloc(value);
        self.map.insert(key, refn);
        Ok(refn)
    }

    pub fn get<Q: ?Sized>(&self, key: &Q) -> Option<&'a V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        self.map.get(key).map(|r| *r)
    }
}
