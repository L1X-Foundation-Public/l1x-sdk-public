use std::cell::RefCell;
use std::collections::BTreeMap;

pub(crate) struct StableMap<K, V> {
    map: RefCell<BTreeMap<K, Box<V>>>,
}

impl<K: Ord, V> Default for StableMap<K, V> {
    fn default() -> Self {
        Self {
            map: Default::default(),
        }
    }
}

impl<K, V> StableMap<K, V> {
    pub(crate) fn get(&self, k: K) -> &V
    where
        K: Ord,
        V: Default,
    {
        let mut map = self.map.borrow_mut();
        let v: &mut Box<V> = map.entry(k).or_default();
        let v: &V = &*v;
        unsafe { &*(v as *const V) }
    }

    pub(crate) fn get_mut(&mut self, k: K) -> &mut V
    where
        K: Ord,
        V: Default,
    {
        &mut *self.map.get_mut().entry(k).or_default()
    }

    pub(crate) fn inner(&mut self) -> &mut BTreeMap<K, Box<V>> {
        self.map.get_mut()
    }

    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.map.borrow().is_empty()
    }
}
