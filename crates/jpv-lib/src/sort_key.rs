use std::cmp::Ordering;

use serde::{Deserialize, Serialize};

#[derive(Default, Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Weight {
    pub weight: f32,
    #[allow(unused)]
    pub query: f32,
    #[allow(unused)]
    pub priority: f32,
    #[allow(unused)]
    pub sense_count: f32,
    #[allow(unused)]
    pub conjugation: f32,
    #[allow(unused)]
    pub length: f32,
}

impl PartialEq for Weight {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.weight == other.weight
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
        match self.weight.partial_cmp(&other.weight) {
            None => Ordering::Equal,
            Some(ordering) => ordering.reverse(),
        }
    }
}

#[derive(Default, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct EntryKey {
    pub weight: Weight,
    pub sequence: u64,
}
