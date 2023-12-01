use musli::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::entities::NameType;
use crate::{EntryKey, Weight};

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
    pub fn sort_key(&self, input: &str) -> EntryKey {
        // Boost based on exact query.
        let mut query = 1.0f32;
        // Store the priority which performs the maximum boost.
        let priority = 1.0;
        // Perform boost by number of senses, maximum boost at 10 senses.
        let sense_count = 1.0;
        // Conjugation boost.
        let conjugation = 1.0;
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

        EntryKey {
            weight: Weight {
                weight: query * priority * sense_count * conjugation * length,
                query,
                priority,
                sense_count,
                conjugation,
                length,
            },
            sequence: self.sequence,
        }
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
