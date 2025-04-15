#[cfg(feature = "alloc")]
pub type HashMap<K, V> = alloc::collections::BTreeMap<K, V>;
