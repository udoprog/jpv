use std::hash::{Hash, Hasher};

use twox_hash::XxHash64;

const SEED: u64 = 0x5EED_BEEF;

/// Calculate the hash for a value.
pub(crate) fn hash<T>(value: T) -> u64
where
    T: Hash,
{
    let mut hasher = XxHash64::with_seed(SEED);
    value.hash(&mut hasher);
    hasher.finish()
}
