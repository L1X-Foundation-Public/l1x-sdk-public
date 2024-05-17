//! An iterable implementation of vector that stores its content to the persitent storage.
mod impls;

use crate::abort;

use super::IndexMap;
use borsh::{BorshDeserialize, BorshSerialize};

const ERR_INDEX_OUT_OF_BOUNDS: &str = "Index out of bounds";

/// An iterable implementation of vector that stores its content to the persitent storage.
/// Uses the following map: index -> element.
///
/// All operations are cached. The cache is flushed in the following cases:
///
/// * [`Self::flush`] method is called
/// * [`drop`] method is called
pub struct Vector<T>
where
    T: BorshSerialize + BorshDeserialize,
{
    pub(crate) len: u32,
    pub(crate) values: IndexMap<T>,
}

impl<T> BorshSerialize for Vector<T>
where
    T: BorshSerialize + BorshDeserialize,
{
    fn serialize<W: borsh::maybestd::io::Write>(
        &self,
        writer: &mut W,
    ) -> Result<(), borsh::maybestd::io::Error> {
        BorshSerialize::serialize(&self.len, writer)?;
        BorshSerialize::serialize(&self.values, writer)?;
        Ok(())
    }
}

impl<T> BorshDeserialize for Vector<T>
where
    T: BorshSerialize + BorshDeserialize,
{
    fn deserialize(buf: &mut &[u8]) -> Result<Self, borsh::maybestd::io::Error> {
        Ok(Self {
            len: BorshDeserialize::deserialize(buf)?,
            values: BorshDeserialize::deserialize(buf)?,
        })
    }
}

impl<T> Vector<T>
where
    T: BorshSerialize + BorshDeserialize,
{
    /// Creates a new vector with zero length. Uses `prefix` as a unique prefix for indices.
    pub fn new(prefix: Vec<u8>) -> Self {
        Self {
            len: 0,
            values: IndexMap::new(prefix),
        }
    }

    /// Returns the number of elements in the vector, also referred to as its 'length'.
    pub fn len(&self) -> u32 {
        self.len
    }

    /// Returns `true` if the vector contains no elements.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Writes the cached operations to the persistent storage.
    ///
    /// # Panic
    ///
    /// Panics if serialization fails
    pub fn flush(&mut self) {
        self.values.flush();
    }

    /// Inserts an element at `index`.
    ///
    /// # Panic
    ///
    /// Panics if `index` is out of bounds.
    pub fn set(&mut self, index: u32, value: T) {
        if index >= self.len() {
            crate::panic(ERR_INDEX_OUT_OF_BOUNDS);
        }

        self.values.set(index, Some(value));
    }

    /// Appends an element to the back of a collection.
    ///
    /// # Panic
    ///
    /// Panics if the new length exceeds [`u32::MAX`].
    pub fn push(&mut self, element: T) {
        let last_idx = self.len();
        self.len = self
            .len
            .checked_add(1)
            .unwrap_or_else(|| crate::panic(ERR_INDEX_OUT_OF_BOUNDS));
        self.set(last_idx, element)
    }

    /// Returns a reference to an element.
    ///
    /// If given a position, returns a reference to the element at that position or `None` if out of bounds.
    pub fn get(&self, index: u32) -> Option<&T> {
        if index >= self.len() {
            return None;
        }
        self.values.get(index)
    }

    /// Returns a mutable reference to an element.
    ///
    /// If given a position, returns a reference to the element at that position or `None` if out of bounds.
    pub fn get_mut(&mut self, index: u32) -> Option<&mut T> {
        if index >= self.len {
            return None;
        }
        self.values.get_mut(index)
    }
}

impl<T> Vector<T>
where
    T: BorshSerialize + BorshDeserialize + Copy,
{
    /// Removes the last element from a vector and returns it, or `None` if it is empty.
    pub fn pop(&mut self) -> Option<T> {
        if self.len() == 0 {
            return None;
        }

        let last_idx = self.len() - 1;
        let last_value = self.values.get(last_idx).copied();

        self.values.set(last_idx, None);

        self.len -= 1;

        last_value
    }

    /// Removes an element from the vector and returns it.
    ///
    /// The removed element is replaced by the last element of the vector.
    ///
    /// This does not preserve ordering, but is O(1). If you need to preserve the element order, use remove instead.
    ///
    /// # Panics
    ///
    /// Panics if index is out of bounds.
    pub fn swap_remove(&mut self, index: u32) -> T {
        if index >= self.len() {
            crate::panic(ERR_INDEX_OUT_OF_BOUNDS);
        }

        let last_idx = self.len() - 1;
        if last_idx == index {
            self.pop().unwrap_or_else(|| abort())
        } else {
            let elem = self.values.get(index).copied().unwrap_or_else(|| abort());
            let last_elem = self.pop();

            self.values.set(index, last_elem);
            elem
        }
    }
}

//====================================================== TESTS =================================================================

#[cfg(test)]
mod tests {
    use super::super::super::tests::*;
    use super::*;
    use borsh::{BorshDeserialize, BorshSerialize};

    #[derive(BorshSerialize, BorshDeserialize, PartialEq, Clone, Copy, Debug)]
    struct TestValue(i32);

    #[test]
    fn test_vector_new_and_len() {
        let vector: Vector<TestValue> = Vector::new(b"test".to_vec());
        assert_eq!(vector.len(), 0);
        assert!(vector.is_empty());
    }

    #[test]
    fn test_vector_push_and_get() {
        let mut vector: Vector<TestValue> = Vector::new(b"test".to_vec());
        vector.push(TestValue(10));
        assert_eq!(vector.len(), 1);
        assert!(!vector.is_empty());
        assert_eq!(vector.get(0), Some(&TestValue(10)));
    }

    #[test]
    fn test_vector_set_and_get_mut() {
        let mut vector: Vector<TestValue> = Vector::new(b"test".to_vec());
        vector.push(TestValue(10));
        vector.set(0, TestValue(20));
        assert_eq!(vector.get(0), Some(&TestValue(20)));
        if let Some(value) = vector.get_mut(0) {
            *value = TestValue(30);
        }
        assert_eq!(vector.get(0), Some(&TestValue(30)));
    }

    #[test]
    fn test_vector_out_of_bounds() {
        let vector: Vector<TestValue> = Vector::new(b"test".to_vec());
        assert_eq!(vector.get(0), None);
    }

    #[test]
    fn test_vector_non_empty_prefix() {
        let vector: Vector<TestValue> = Vector::new(b"non_empty_prefix".to_vec());
        assert_eq!(vector.len(), 0);
    }

    #[test]
    fn test_vector_push_multiple() {
        let mut vector: Vector<TestValue> = Vector::new(b"test".to_vec());
        vector.push(TestValue(10));
        vector.push(TestValue(20));
        vector.push(TestValue(30));
        assert_eq!(vector.len(), 3);
        assert_eq!(vector.get(0), Some(&TestValue(10)));
        assert_eq!(vector.get(1), Some(&TestValue(20)));
        assert_eq!(vector.get(2), Some(&TestValue(30)));
    }

    #[test]
    fn test_vector_pop() {
        let mut vector: Vector<TestValue> = Vector::new(b"test".to_vec());
        assert_eq!(vector.pop(), None);

        vector.push(TestValue(10));
        vector.push(TestValue(20));
        vector.push(TestValue(30));

        assert_eq!(vector.len(), 3);
        assert_eq!(vector.pop(), Some(TestValue(30)));
        assert_eq!(vector.len(), 2);
        assert_eq!(vector.pop(), Some(TestValue(20)));
        assert_eq!(vector.len(), 1);
        assert_eq!(vector.pop(), Some(TestValue(10)));
        assert_eq!(vector.len(), 0);
        assert_eq!(vector.pop(), None);
    }

    #[test]
    fn test_vector_swap_remove() {
        let mut vector: Vector<TestValue> = Vector::new(b"test".to_vec());

        vector.push(TestValue(10));

        assert_eq!(vector.swap_remove(0), TestValue(10));
        assert_eq!(vector.len(), 0);

        vector.push(TestValue(10));
        vector.push(TestValue(20));
        vector.push(TestValue(30));

        assert_eq!(vector.swap_remove(2), TestValue(30));
        assert_eq!(vector.len(), 2);
        assert_eq!(vector.get(2), None);

        vector.push(TestValue(50));

        assert_eq!(vector.swap_remove(1), TestValue(20));
        assert_eq!(vector.len(), 2);
        assert_eq!(vector.get(1), Some(&TestValue(50)));
    }

    #[test]
    #[should_panic]
    fn test_vector_swap_remove_panic() {
        let mut vector: Vector<TestValue> = Vector::new(b"test".to_vec());

        vector.push(TestValue(10));
        vector.swap_remove(1);
    }

    #[test]
    fn test_push_persistence() {
        let mut vector: Vector<TestValue> = Vector::new(b"test".to_vec());

        vector.push(TestValue(10));
        vector.flush();

        // Construct the expected storage key
        let mut expected_key = b"test".to_vec();
        expected_key.extend_from_slice(&0u32.to_le_bytes());

        // Check that the value has been written in the underlying storage
        let written_value =
            TestValue::try_from_slice(&mut &*storage_read(&expected_key).unwrap()).unwrap();
        assert_eq!(written_value, TestValue(10));
    }

    #[test]
    fn test_set_persistence() {
        let mut vector: Vector<TestValue> = Vector::new(b"test".to_vec());

        vector.push(TestValue(10));
        vector.flush();
        vector.set(0, TestValue(20));
        vector.flush();

        // Construct the expected storage key
        let mut expected_key = b"test".to_vec();
        expected_key.extend_from_slice(&0u32.to_le_bytes());

        // Check that the value has been updated in the underlying storage
        let written_value =
            TestValue::try_from_slice(&mut &*storage_read(&expected_key).unwrap()).unwrap();
        assert_eq!(written_value, TestValue(20));
    }
}
