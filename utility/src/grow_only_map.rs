use std::borrow::Borrow;
use std::cell::UnsafeCell;
use std::sync::Mutex;

// TODO: Use an existing solution, like https://docs.rs/typed-arena/1.5.0/typed_arena/struct.Arena.html
/// Like a HashMap, but it can only get bigger.
/// It is (hopefully) safe to extend

#[derive(Default)]
pub struct GrowOnlyMap<K, V> {
    // The Mutex is to make sure that two different threads aren't
    // mucking around with the UnsafeCell at once. (I want to ensure
    // that `insert` and `get` are never being invoked
    // simultaneously.)

    // The `UnsafeCell` is necessary because the *values* in the
    // vector are immutable, while the vector *itself* is mutable,
    // and an UnsafeCell is the only way to allow immutable references
    // to within this mutable vector.

    // I'm using a vector of pairs instead of a HashMap because
    // HashMaps might do tricksy things that I don't know about.

    // The values are Boxed because as the vector grows, it might
    // re-allocate. The extra level of indirection will ensure that
    // even though the Box moves, the value V inside it doesn't, thus
    // keeping references to it valid.
    #[allow(clippy::type_complexity)] // whatever, this struct is getting deleted soon
    mutex: Mutex<UnsafeCell<Vec<(K, Box<V>)>>>,
}

impl<K: Eq, V> GrowOnlyMap<K, V> {
    pub fn new() -> GrowOnlyMap<K, V> {
        GrowOnlyMap {
            mutex: Mutex::new(UnsafeCell::new(vec![])),
        }
    }

    /// Insert a key-value pair into the map.
    ///
    /// If the key is already in the map, this will have no effect.
    pub fn insert(&self, key: K, value: V) {
        let cell = expect!(self.mutex.lock(), "GrowOnlyMap: mutex error");
        let vec = unsafe { &mut *cell.get() };
        for (k, _) in &*vec {
            if k == &key {
                return;
            }
        }
        vec.push((key, Box::new(value)));
    }

    /// Get a reference to the value corresponding to the key.
    pub fn get<Q: ?Sized>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Eq,
    {
        let cell = expect!(self.mutex.lock(), "GrowOnlyMap: mutex error");
        let vec = unsafe { &*cell.get() };
        for (k, value) in vec {
            if key == k.borrow() {
                return Some(value);
            }
        }
        None
    }
}

#[test]
fn test_grow_only_map() {
    let mut junk = vec![];
    let map = GrowOnlyMap::new();

    map.insert(
        "hello".to_string(),
        ("hello".to_string(), "hello".to_string()),
    );
    let h1 = &map.get("hello").unwrap().0[1..];
    assert_eq!(h1, "ello");

    map.insert("hello".to_string(), ("hey".to_string(), "hey".to_string()));
    let h2 = &map.get("hello").unwrap().0[1..];
    assert_eq!(h1, "ello");
    assert_eq!(h2, "ello");

    map.insert(
        "world".to_string(),
        ("there".to_string(), "world".to_string()),
    );
    let t = &map.get("world").unwrap().0;
    let w = &map.get("world").unwrap().1[2..];
    let pair = &map.get("world").unwrap();
    assert_eq!(h1, "ello");
    assert_eq!(h2, "ello");
    assert_eq!(t, "there");
    assert_eq!(w, "rld");
    assert_eq!(&pair.0, "there");
    assert_eq!(&pair.1, "world");

    for _ in 0..100_000 {
        map.insert("stuff".to_string(), ("s".to_string(), "s".to_string()));
        junk.push("junk".to_string());
    }
    let s = &map.get("stuff").unwrap().0;
    assert_eq!(h1, "ello");
    assert_eq!(h2, "ello");
    assert_eq!(t, "there");
    assert_eq!(w, "rld");
    assert_eq!(s, "s");
    assert_eq!(&pair.0, "there");
    assert_eq!(&pair.1, "world");
}
