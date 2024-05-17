use borsh::{BorshDeserialize, BorshSerialize};

use super::{Vector, ERR_INDEX_OUT_OF_BOUNDS};

impl<T> Drop for Vector<T>
where
    T: BorshSerialize + BorshDeserialize,
{
    fn drop(&mut self) {
        self.flush()
    }
}

impl<T> Extend<T> for Vector<T>
where
    T: BorshSerialize + BorshDeserialize,
{
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = T>,
    {
        for item in iter {
            self.push(item)
        }
    }
}

impl<T> core::ops::Index<u32> for Vector<T>
where
    T: BorshSerialize + BorshDeserialize,
{
    type Output = T;

    fn index(&self, index: u32) -> &Self::Output {
        self.get(index)
            .unwrap_or_else(|| crate::panic(ERR_INDEX_OUT_OF_BOUNDS))
    }
}
