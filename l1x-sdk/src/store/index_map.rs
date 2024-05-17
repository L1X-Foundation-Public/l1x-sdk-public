use borsh::{BorshDeserialize, BorshSerialize};
use once_cell::unsync::OnceCell;

use crate::utils::StableMap;
use crate::{CacheEntry, EntryState};

const ERR_ELEMENT_DESERIALIZATION: &str = "Cannot deserialize element";
const ERR_ELEMENT_SERIALIZATION: &str = "Cannot serialize element";

#[derive(BorshSerialize, BorshDeserialize)]
pub(crate) struct IndexMap<T>
where
    T: BorshSerialize,
{
    pub(crate) prefix: Box<[u8]>,

    #[borsh_skip]
    pub(crate) cache: StableMap<u32, OnceCell<CacheEntry<T>>>,
}

impl<T> IndexMap<T>
where
    T: BorshSerialize,
{
    pub fn new(prefix: Vec<u8>) -> Self {
        Self {
            prefix: prefix.into_boxed_slice(),
            cache: Default::default(),
        }
    }

    fn index_to_lookup_key(prefix: &[u8], index: u32, buf: &mut Vec<u8>) {
        buf.extend_from_slice(prefix);
        buf.extend_from_slice(&index.to_le_bytes());
    }

    pub fn flush(&mut self) {
        let mut buf = Vec::new();
        let mut key_buf = Vec::with_capacity(self.prefix.len() + 4);
        for (k, v) in self.cache.inner().iter_mut() {
            if let Some(v) = v.get_mut() {
                if v.is_modified() {
                    key_buf.clear();
                    Self::index_to_lookup_key(&self.prefix, *k, &mut key_buf);
                    match v.value().as_ref() {
                        Some(modified) => {
                            buf.clear();
                            BorshSerialize::serialize(modified, &mut buf)
                                .unwrap_or_else(|_| crate::panic(ERR_ELEMENT_SERIALIZATION));
                            crate::storage_write(&key_buf, &buf);
                        }
                        None => {
                            crate::storage_remove(&key_buf);
                        }
                    }

                    v.replace_state(EntryState::Cached);
                }
            }
        }
    }

    pub fn set(&mut self, index: u32, value: Option<T>) {
        let entry = self.cache.get_mut(index);
        match entry.get_mut() {
            Some(entry) => *entry.value_mut() = value,
            None => {
                let _ = entry.set(CacheEntry::new_modified(value));
            }
        }
    }
}

impl<T> IndexMap<T>
where
    T: BorshSerialize + BorshDeserialize,
{
    fn deserialize_element(raw_element: &[u8]) -> T {
        T::try_from_slice(raw_element).unwrap_or_else(|_| crate::panic(ERR_ELEMENT_DESERIALIZATION))
    }

    pub fn get(&self, index: u32) -> Option<&T> {
        let entry = self.cache.get(index).get_or_init(|| {
            let mut buf = Vec::with_capacity(self.prefix.len() + 4);
            Self::index_to_lookup_key(&self.prefix, index, &mut buf);
            let storage_bytes = crate::storage_read(&buf);
            let value = storage_bytes.as_deref().map(Self::deserialize_element);
            CacheEntry::new_cached(value)
        });
        entry.value().as_ref()
    }

    pub(crate) fn get_mut_inner(&mut self, index: u32) -> &mut CacheEntry<T> {
        let prefix = &self.prefix;
        let entry = self.cache.get_mut(index);
        entry.get_or_init(|| {
            let mut key = Vec::with_capacity(prefix.len() + 4);
            Self::index_to_lookup_key(prefix, index, &mut key);
            let storage_bytes = crate::storage_read(&key);
            let value = storage_bytes.as_deref().map(Self::deserialize_element);
            CacheEntry::new_cached(value)
        });
        let entry = entry.get_mut().unwrap();
        entry
    }

    pub fn get_mut(&mut self, index: u32) -> Option<&mut T> {
        let entry = self.get_mut_inner(index);
        entry.value_mut().as_mut()
    }
}
