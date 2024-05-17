//! An implementation of a map that stores its content directly on the persistent storage.
mod impls;

use crate::utils::{EntryState, StableMap};
use crate::CacheEntry;
use borsh::{BorshDeserialize, BorshSerialize};
use once_cell::unsync::OnceCell;
use std::borrow::Borrow;

const ERR_ELEMENT_DESERIALIZATION: &str = "Cannot deserialize element";
const ERR_ELEMENT_SERIALIZATION: &str = "Cannot serialize element";

/// An implementation of a map that stores its content directly on the persistent storage.
///
/// All operations are cached. The cache is flushed in the following cases:
///
/// * [`Self::flush`] method is called
/// * [`drop`] method is called
#[derive(BorshSerialize, BorshDeserialize)]
pub struct LookupMap<K, V>
where
    K: BorshSerialize + Ord,
    V: BorshSerialize + BorshDeserialize,
{
    prefix: Box<[u8]>,
    /// Cache for loads and intermediate changes to the underlying vector.
    /// The cached entries are wrapped in a [`Box`] to avoid existing pointers from being
    /// invalidated.
    #[borsh_skip]
    cache: StableMap<K, EntryAndHash<V>>,
}

struct EntryAndHash<V> {
    value: OnceCell<CacheEntry<V>>,
    hash: OnceCell<Vec<u8>>,
}

impl<V> Default for EntryAndHash<V> {
    fn default() -> Self {
        Self {
            value: Default::default(),
            hash: Default::default(),
        }
    }
}

fn to_key<Q: ?Sized>(prefix: &[u8], key: &Q, buffer: &mut Vec<u8>) -> Vec<u8>
where
    Q: BorshSerialize,
{
    // Prefix the serialized bytes and return a copy of this buffer.
    buffer.extend(prefix);
    key.serialize(buffer).unwrap_or_else(|_| crate::abort());

    buffer.clone()
}

impl<K, V> Drop for LookupMap<K, V>
where
    K: BorshSerialize + Ord,
    V: BorshSerialize + BorshDeserialize,
{
    fn drop(&mut self) {
        self.flush()
    }
}

impl<K, V> LookupMap<K, V>
where
    K: BorshSerialize + Ord,
    V: BorshSerialize + BorshDeserialize,
{
    /// Creates a new map. Uses `prefix` as a unique prefix for keys.
    pub fn new(prefix: Vec<u8>) -> Self {
        Self {
            prefix: prefix.into_boxed_slice(),
            cache: Default::default(),
        }
    }

    #[cfg(test)]
    pub fn to_key_test<Q>(prefix: &[u8], key: &Q, buffer: &mut Vec<u8>) -> Vec<u8>
    where
        Q: ?Sized + BorshSerialize,
    {
        to_key(prefix, key, buffer)
    }

    /// Returns the unique byte prefix used for key generation in the `LookupSet`.
    pub fn get_prefix(&self) -> &Box<[u8]> {
        &self.prefix
    }

    /// Inserts or removes a key-value to the map.
    ///
    /// * If `value` is `None` then the specified key is removed.
    /// * If `value` is `Some(v)` then `v` is inserted by the specified key
    pub fn set(&mut self, key: K, value: Option<V>) {
        let entry = self.cache.get_mut(key);
        match entry.value.get_mut() {
            Some(entry) => *entry.value_mut() = value,
            None => {
                let _ = entry.value.set(CacheEntry::new_modified(value));
            }
        }
    }

    fn deserialize_element(bytes: &[u8]) -> V {
        V::try_from_slice(bytes).unwrap_or_else(|_| crate::panic(ERR_ELEMENT_DESERIALIZATION))
    }

    fn load_element<Q: ?Sized>(prefix: &[u8], key: &Q) -> (Vec<u8>, Option<V>)
    where
        Q: BorshSerialize,
        K: Borrow<Q>,
    {
        let key = to_key(prefix, key, &mut Vec::new());
        let storage_bytes = crate::storage_read(key.as_ref());
        (key, storage_bytes.as_deref().map(Self::deserialize_element))
    }

    /// Returns a reference to the value corresponding to the key.
    ///
    /// If the map doesn't have the key present, returns `None`
    pub fn get<Q: ?Sized>(&self, k: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: BorshSerialize + ToOwned<Owned = K>,
    {
        let cached = self.cache.get(k.to_owned());
        let entry = cached.value.get_or_init(|| {
            let (key, element) = Self::load_element(&self.prefix, k);
            let _ = cached.hash.set(key);
            CacheEntry::new_cached(element)
        });
        entry.value().as_ref()
    }

    /// Returns a mutable reference to the value corresponding to the key.
    ///
    /// If the map doesn't have the key present, returns `None`
    pub fn get_mut<Q: ?Sized>(&mut self, k: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
        Q: BorshSerialize + ToOwned<Owned = K>,
    {
        let cached = self.cache.get_mut(k.to_owned());
        cached.value.get_or_init(|| {
            let (key, value) = Self::load_element(&self.prefix, k);
            let _ = cached.hash.set(key);
            CacheEntry::new_cached(value)
        });

        let entry = cached.value.get_mut().unwrap_or_else(|| crate::abort());
        match entry.value() {
            Some(_) => Some(entry.value_mut().as_mut().unwrap_or_else(|| crate::abort())),
            None => None,
        }
    }

    pub(crate) fn get_mut_inner<Q: ?Sized>(&mut self, k: &Q) -> &mut CacheEntry<V>
    where
        K: Borrow<Q>,
        Q: BorshSerialize + ToOwned<Owned = K>,
    {
        let prefix = &self.prefix;
        let entry = self.cache.get_mut(k.to_owned());
        entry.value.get_or_init(|| {
            let (key, value) = Self::load_element(prefix, k);
            let _ = entry.hash.set(key);
            CacheEntry::new_cached(value)
        });
        let entry = entry.value.get_mut().unwrap_or_else(|| crate::abort());
        entry
    }

    /// Inserts a key-value pair into the map.
    ///
    /// If the map did not have this key present, None is returned.
    ///
    /// If the map did have this key present, the value is updated, and the old value is returned.
    pub fn insert(&mut self, k: K, v: V) -> Option<V>
    where
        K: Clone,
    {
        self.get_mut_inner(&k).replace(Some(v))
    }

    /// Removes a key from the map, returning the value at the key if the key was previously in the map.
    pub fn remove(&mut self, k: K) -> Option<V>
    where
        K: Clone,
    {
        self.get_mut_inner(&k).replace(None)
    }

    /// Returns true if the map contains a value for the specified key.
    pub fn contains_key<Q: ?Sized>(&self, k: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: BorshSerialize + ToOwned<Owned = K>,
    {
        self.get(k).is_some()
    }

    /// Writes the cached operations to the persistent storage.
    ///
    /// # Panic
    ///
    /// Panics if serialization fails
    pub fn flush(&mut self) {
        let mut buf = Vec::new();
        for (k, v) in self.cache.inner().iter_mut() {
            if let Some(val) = v.value.get_mut() {
                if val.is_modified() {
                    let prefix = &self.prefix;
                    let key = v.hash.get_or_init(|| {
                        buf.clear();
                        to_key(prefix, k, &mut buf)
                    });
                    match val.value().as_ref() {
                        Some(modified) => {
                            buf.clear();
                            BorshSerialize::serialize(modified, &mut buf)
                                .unwrap_or_else(|_| crate::panic(ERR_ELEMENT_SERIALIZATION));
                            crate::storage_write(key.as_ref(), &buf);
                        }
                        None => {
                            crate::storage_remove(key.as_ref());
                        }
                    }

                    // Update state of flushed state as cached, to avoid duplicate writes/removes
                    // while also keeping the cached values in memory.
                    val.replace_state(EntryState::Cached);
                }
            }
        }
    }
}

//====================================================== TESTS =================================================================

#[cfg(test)]
mod tests {
    use super::super::super::tests::*;
    use super::*;
    use borsh::{BorshDeserialize, BorshSerialize};

    #[derive(BorshSerialize, BorshDeserialize, Ord, PartialOrd, Eq, PartialEq, Clone, Debug)]
    struct TestKey(i32);

    #[derive(BorshSerialize, BorshDeserialize, PartialEq, Clone, Debug)]
    struct TestValue(i32);

    #[test]
    fn test_new() {
        let map: LookupMap<TestKey, TestValue> = LookupMap::new(b"test".to_vec());
        assert_eq!(&*map.prefix, b"test");
        assert!(map.cache.is_empty());
    }

    #[test]
    fn test_set_and_get() {
        let mut map: LookupMap<TestKey, TestValue> = LookupMap::new(b"test".to_vec());

        // Set key-value pair
        map.set(TestKey(1), Some(TestValue(10)));

        // Get value for key
        let value = map.get(&TestKey(1));
        assert_eq!(value, Some(&TestValue(10)));
    }

    #[test]
    fn test_insert_and_get() {
        let mut map: LookupMap<TestKey, TestValue> = LookupMap::new(b"test".to_vec());

        // Insert key-value pair
        map.insert(TestKey(1), TestValue(10));

        // Get value for key
        let value = map.get(&TestKey(1));
        assert_eq!(value, Some(&TestValue(10)));
    }

    #[test]
    fn test_remove() {
        let mut map: LookupMap<TestKey, TestValue> = LookupMap::new(b"test".to_vec());

        // Insert key-value pair
        map.insert(TestKey(1), TestValue(10));

        // Remove key-value pair
        map.set(TestKey(1), None);

        // Get value for key
        let value = map.get(&TestKey(1));
        assert_eq!(value, None);
    }

    #[test]
    fn test_flush() {
        let mut map: LookupMap<TestKey, TestValue> = LookupMap::new(b"test".to_vec());

        // Insert key-value pair
        map.insert(TestKey(1), TestValue(10));

        // Flush the map
        map.flush();

        // Check storage for key-value pair
        let stored_value = storage_read(&to_key(b"test", &TestKey(1), &mut Vec::new()));

        assert_eq!(
            TestValue::try_from_slice(stored_value.unwrap().as_slice())
                .unwrap_or_else(|_| panic!("Failed to deserialize")),
            TestValue(10)
        );
    }

    #[test]
    fn test_insert_persistence() {
        let mut map: LookupMap<TestKey, TestValue> = LookupMap::new(b"test".to_vec());

        map.insert(TestKey(1), TestValue(10));
        map.flush();

        let key_with_prefix = to_key(b"test", &TestKey(1), &mut Vec::new());
        let stored_value = storage_read(&key_with_prefix);

        let stored_value = TestValue::try_from_slice(stored_value.unwrap().as_slice())
            .unwrap_or_else(|_| panic!("Failed to deserialize"));

        assert_eq!(stored_value, TestValue(10));
    }

    #[test]
    fn test_set_persistence() {
        let mut map: LookupMap<TestKey, TestValue> = LookupMap::new(b"test".to_vec());

        // Set a key-value pair and flush to storage
        map.set(TestKey(1), Some(TestValue(10)));
        map.flush();

        // Check storage for the key
        let stored_value_bytes = storage_read(&to_key(b"test", &TestKey(1), &mut Vec::new()));

        assert!(
            stored_value_bytes.is_some(),
            "Expected the key to be set in storage"
        );

        // Decode stored value
        let stored_value: TestValue =
            borsh::BorshDeserialize::deserialize(&mut stored_value_bytes.unwrap().as_slice())
                .unwrap();

        // Assert the stored value matches what was set
        assert_eq!(stored_value, TestValue(10));
    }

    #[test]
    fn test_update_persistence() {
        let mut map: LookupMap<TestKey, TestValue> = LookupMap::new(b"test".to_vec());

        // Insert a key-value pair and flush to storage
        map.insert(TestKey(1), TestValue(10));
        map.flush();

        // Update the value for the key and flush to storage
        map.insert(TestKey(1), TestValue(20));
        map.flush();

        // Check storage for key-value pair
        let stored_value_bytes = storage_read(&to_key(b"test", &TestKey(1), &mut Vec::new()));

        let stored_value = TestValue::try_from_slice(stored_value_bytes.unwrap().as_slice())
            .unwrap_or_else(|_| panic!("Failed to deserialize"));

        assert_eq!(
            stored_value,
            TestValue(20),
            "Expected the value to be updated in storage"
        );
    }

    #[test]
    fn test_remove_persistence() {
        let mut map: LookupMap<TestKey, TestValue> = LookupMap::new(b"test".to_vec());

        // Insert a key-value pair and flush to storage
        map.insert(TestKey(1), TestValue(10));
        map.flush();

        // Remove the key-value pair and flush to storage
        map.set(TestKey(1), None);
        map.flush();

        // Check storage for the key
        let stored_value_bytes = storage_read(&to_key(b"test", &TestKey(1), &mut Vec::new()));

        assert!(
            stored_value_bytes.is_none(),
            "Expected the key to be removed from storage"
        );
    }

    #[test]
    fn test_remove_function() {
        let mut map: LookupMap<TestKey, TestValue> = LookupMap::new(b"test".to_vec());

        // Insert key-value pair
        map.insert(TestKey(1), TestValue(10));

        // Remove key-value pair
        let removed = map.remove(TestKey(1));

        // Assert that the removed value is correct
        assert_eq!(removed, Some(TestValue(10)));

        // Get value for key
        let value = map.get(&TestKey(1));
        assert_eq!(value, None);
    }

    #[test]
    fn test_contains_key() {
        let mut map = LookupMap::new(b"mymap".to_vec());

        // The map is initially empty, so it doesn't contain the key.
        assert!(!map.contains_key(&1));

        // After inserting a value, the map should contain the key.
        map.insert(1, "one".to_string());
        assert!(map.contains_key(&1));

        // After removing a value, the map should no longer contain the key.
        map.remove(1);
        assert!(!map.contains_key(&1));
    }
}
