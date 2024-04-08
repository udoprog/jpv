use std::cmp::Ordering;

use musli::{Decode, Encode};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, Encode, Decode)]
pub enum Key {
    Phrase(u64),
    Name(u64),
    Kanji(u32),
}

#[derive(Default, Clone, Copy, Debug, Serialize, Deserialize, Encode, Decode)]
pub struct Weight(f32);

impl Weight {
    /// Construct a new weight.
    pub fn new(weight: f32) -> Self {
        Self(weight)
    }

    /// Boost the weight with the given factor.
    pub fn boost(self, factor: f32) -> Self {
        Self(self.0 * factor)
    }
}

impl PartialEq for Weight {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for Weight {}

impl PartialOrd for Weight {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Weight {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        match self.0.partial_cmp(&other.0) {
            None => Ordering::Equal,
            Some(ordering) => ordering.reverse(),
        }
    }
}
