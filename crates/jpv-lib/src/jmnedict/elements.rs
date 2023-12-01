use musli::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::entities::NameType;
use crate::Weight;

#[borrowme::borrowme]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Encode, Decode)]
pub struct Entry<'a> {
    pub sequence: u64,
    #[borrowed_attr(serde(borrow))]
    pub kanji: Vec<&'a str>,
    #[borrowed_attr(serde(borrow))]
    pub reading: Vec<Reading<'a>>,
    pub name_types: Vec<NameType>,
    #[borrowed_attr(serde(borrow))]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub translations: Vec<Translation<'a>>,
}

impl Entry<'_> {
    /// Entry weight.
    pub fn weight(&self, input: &str) -> Weight {
        // Boost based on exact query.
        let mut query = 1.0f32;
        // Calculate length boost.
        let length = (input.chars().count().min(10) as f32 / 10.0) * 1.2;

        for element in self.kanji.iter().copied() {
            if element == input {
                query = query.max(3.0);
            }
        }

        for reading in self.reading.iter() {
            if reading.text == input {
                query = query.max(3.0);
            }
        }

        Weight::new(query * length)
    }
}

#[borrowme::borrowme]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Encode, Decode)]
#[musli(packed)]
pub struct Translation<'a> {
    pub text: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lang: Option<&'a str>,
}

#[borrowme::borrowme]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Encode, Decode)]
#[musli(packed)]
pub struct Reading<'a> {
    pub text: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<&'a str>,
}
