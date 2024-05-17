//! An implementation of a set that stores its content directly on the persistent storage.
mod impls;

use borsh::BorshSerialize;
use std::borrow::Borrow;

use crate::store::LookupMap;

/// An implementation of a set that stores its content directly on the persistent storage.
/// LookupSet is essentially a LookupMap where the key is the element
/// and the value is a constant to signify its presence.
pub struct LookupSet<K>
where
    K: BorshSerialize + Ord,
{
    // We can use any type for V, such as a single byte, because we only care about the key.
    map: LookupMap<K, ()>,
}

impl<K> LookupSet<K>
where
    K: BorshSerialize + Ord,
{
    /// Creates a new set. Uses `prefix` as a unique prefix for keys.
    pub fn new(prefix: Vec<u8>) -> Self {
        Self {
            map: LookupMap::new(prefix),
        }
    }

    #[cfg(test)]
    pub fn to_key_test<Q>(&self, prefix: &[u8], key: &Q, buffer: &mut Vec<u8>) -> Vec<u8>
    where
        Q: ?Sized + BorshSerialize,
    {
        LookupMap::<K, ()>::to_key_test(prefix, key, buffer)
    }

    /// Returns the unique byte prefix used for key generation in the `LookupSet`.
    pub fn get_prefix(&self) -> &Box<[u8]> {
        self.map.get_prefix()
    }

    /// Adds a value to the set.
    ///
    /// Returns whether the value was newly inserted. That is:
    ///
    /// * If the set did not previously contain this value, true is returned.
    /// * If the set already contained this value, false is returned.
    pub fn insert(&mut self, k: K) -> bool
    where
        K: Clone,
    {
        self.map.insert(k, ()).is_none()
    }

    /// Removes a value from the set. Returns whether the value was present in the set.
    pub fn remove(&mut self, k: K) -> bool
    where
        K: Clone,
    {
        self.map.remove(k).is_some()
    }

    /// Returns true if the set contains a value.
    pub fn contains<Q: ?Sized>(&self, k: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: BorshSerialize + ToOwned<Owned = K>,
    {
        self.map.contains_key(k)
    }

    /// Flushes the set's cache.
    pub fn flush(&mut self) {
        self.map.flush();
    }
}

impl<K> Drop for LookupSet<K>
where
    K: BorshSerialize + Ord,
{
    fn drop(&mut self) {
        self.flush()
    }
}

//======================================================= TESTS =======================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::LookupMap;

    use borsh::{BorshDeserialize, BorshSerialize};

    #[derive(BorshSerialize, BorshDeserialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
    struct TestValue(i32);

    #[test]
    fn test_new() {
        let set: LookupSet<TestValue> = LookupSet::new(b"test".to_vec());
        assert_eq!(set.get_prefix().as_ref(), b"test");
    }

    #[test]
    fn test_insert() {
        let mut set: LookupSet<TestValue> = LookupSet::new(b"test".to_vec());

        // Insert value
        assert!(set.insert(TestValue(10)));

        // Inserting the same value again should return false
        assert!(!set.insert(TestValue(10)));
    }

    #[test]
    fn test_contains() {
        let mut set: LookupSet<TestValue> = LookupSet::new(b"test".to_vec());

        // Insert value
        set.insert(TestValue(10));

        // Check for inserted value
        assert!(set.contains(&TestValue(10)));

        // Check for non-inserted value
        assert!(!set.contains(&TestValue(20)));
    }

    #[test]
    fn test_insert_duplicate_values() {
        let mut set: LookupSet<TestValue> = LookupSet::new(b"test".to_vec());

        // Insert a value
        assert!(set.insert(TestValue(1)));

        // Try to insert the same value again. This time it should return false because it is a duplicate.
        assert!(!set.insert(TestValue(1)));
    }

    #[test]
    fn test_contains_non_existent_value() {
        let set: LookupSet<TestValue> = LookupSet::new(b"test".to_vec());

        // Check for a value that hasn't been inserted
        assert!(!set.contains(&TestValue(15)));
    }

    #[test]
    fn test_insert_persistence() {
        let mut set: LookupSet<TestValue> = LookupSet::new(b"test".to_vec());

        // Insert value
        assert!(set.insert(TestValue(10)));

        // Flush the changes to ensure they're written to storage
        set.flush();

        // Check storage for value
        let lookup_key = LookupMap::<TestValue, ()>::to_key_test(
            &set.get_prefix(),
            &TestValue(10),
            &mut Vec::new(),
        );
        let stored_value = crate::storage_read(lookup_key.as_ref());

        assert!(
            stored_value.is_some(),
            "Expected the value to be set in storage"
        );
    }

    #[test]
    fn test_remove() {
        let mut lookup_set: LookupSet<u32> = LookupSet::new(vec![0, 1, 2]);

        // Insert values
        lookup_set.insert(10);
        lookup_set.insert(20);
        lookup_set.insert(30);

        // Check if values exist
        assert_eq!(lookup_set.contains(&10), true);
        assert_eq!(lookup_set.contains(&20), true);
        assert_eq!(lookup_set.contains(&30), true);

        // Remove values
        lookup_set.remove(10);
        lookup_set.remove(20);

        // Check if values exist after removing
        assert_eq!(lookup_set.contains(&10), false);
        assert_eq!(lookup_set.contains(&20), false);
        assert_eq!(lookup_set.contains(&30), true);

        // Try to remove a non-existent element and verify that it returns false
        assert_eq!(lookup_set.remove(40), false);
        assert_eq!(lookup_set.contains(&40), false);
    }
}
