use std::borrow::Borrow;

use borsh::{BorshDeserialize, BorshSerialize};

use super::LookupMap;

impl<K, V> Extend<(K, V)> for LookupMap<K, V>
where
    K: BorshSerialize + Ord,
    V: BorshSerialize + BorshDeserialize,
{
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = (K, V)>,
    {
        for (key, value) in iter {
            self.set(key, Some(value))
        }
    }
}

impl<K, V, Q: ?Sized> core::ops::Index<&Q> for LookupMap<K, V>
where
    K: BorshSerialize + Ord + Borrow<Q>,
    V: BorshSerialize + BorshDeserialize,

    Q: BorshSerialize + ToOwned<Owned = K>,
{
    type Output = V;

    fn index(&self, index: &Q) -> &Self::Output {
        self.get(index)
            .unwrap_or_else(|| crate::panic("does not exist"))
    }
}
