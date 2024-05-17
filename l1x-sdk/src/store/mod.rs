//! Collections and types used when interacting with storage.
//!
//! These collections are more scalable versions of [`std::collections`] when used as contract
//! state because it allows values to be lazily loaded and stored based on what is actually
//! interacted with.
pub mod vec;
pub use self::vec::Vector;

pub mod lookup_set;
pub use self::lookup_set::LookupSet;

pub mod lookup_map;
pub use self::lookup_map::LookupMap;

mod index_map;
pub(crate) use self::index_map::IndexMap;
