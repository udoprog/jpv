use std::cmp::Ordering;
use std::mem;

use anyhow::{Context, Result};
use musli::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::elements::{kanji_element, reading_element, sense, text};
use crate::elements::{
    KanjiElement, OwnedKanjiElement, OwnedReadingElement, OwnedSense, ReadingElement, Sense,
};

#[derive(Default, Clone, Copy, Debug)]
struct Weight {
    weight: f32,
    #[allow(unused)]
    query: f32,
    #[allow(unused)]
    priority: f32,
    #[allow(unused)]
    sense_count: f32,
    #[allow(unused)]
    conjugation: f32,
    #[allow(unused)]
    length: f32,
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
        Some(self.weight.partial_cmp(&other.weight)?.reverse())
    }
}

impl Ord for Weight {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap_or(Ordering::Equal)
    }
}

#[derive(Default, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct EntryKey {
    weight: Weight,
    sequence: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, Encode, Decode)]
#[musli(packed)]
#[owned::to_owned]
pub struct Entry<'a> {
    pub sequence: u64,
    #[serde(borrow)]
    #[to_owned(ty = Vec<OwnedReadingElement>)]
    pub reading_elements: Vec<ReadingElement<'a>>,
    #[serde(borrow)]
    #[to_owned(ty = Vec<OwnedKanjiElement>)]
    pub kanji_elements: Vec<KanjiElement<'a>>,
    #[serde(borrow)]
    #[to_owned(ty = Vec<OwnedSense>)]
    pub senses: Vec<Sense<'a>>,
}

impl Entry<'_> {
    /// Entry weight.
    pub fn sort_key(&self, input: &str, conjugation: bool, len: usize) -> EntryKey {
        // Boost based on exact query.
        let mut query = 1.0f32;
        // Store the priority which performs the maximum boost.
        let mut priority = 1.0f32;
        // Perform boost by number of senses, maximum boost at 10 senses.
        let sense_count = 1.0 + self.senses.len().min(10) as f32 / 10.0;
        // Conjugation boost.
        let conjugation = conjugation.then_some(2.0).unwrap_or(1.0);
        // Calculate length boost.
        let length = (len.min(10) as f32 / 10.0) * 1.2;

        for element in &self.reading_elements {
            if element.text == input {
                query = query.max(2.0);
            }

            for p in &element.priority {
                priority = priority.max(p.weight());
            }
        }

        for element in &self.kanji_elements {
            if element.text == input {
                query = query.max(2.5);
            }

            for p in &element.priority {
                priority = priority.max(p.weight());
            }
        }

        for sense in &self.senses {
            for gloss in &sense.gloss {
                if gloss.text == input {
                    query = query.max(1.5);
                }
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

#[derive(Debug, Default)]
enum State<'a> {
    #[default]
    Root,
    EntrySequence(text::Builder<'a>),
    ReadingElement(reading_element::Builder<'a>),
    KanjiElement(kanji_element::Builder<'a>),
    Sense(sense::Builder<'a>),
}

#[derive(Default)]
pub(crate) struct Builder<'a> {
    state: State<'a>,
    sequence: Option<u64>,
    reading_elements: Vec<ReadingElement<'a>>,
    kanji_elements: Vec<KanjiElement<'a>>,
    senses: Vec<Sense<'a>>,
}

impl<'a> Builder<'a> {
    builder! {
        self => Entry<'a> {
            "ent_seq", EntrySequence, value => {
                self.sequence = Some(value.parse().context("Invalid sequence")?);
            }
            "r_ele", ReadingElement, value => {
                self.reading_elements.push(value);
            }
            "k_ele", KanjiElement, value => {
                self.kanji_elements.push(value);
            }
            "sense", Sense, value => {
                self.senses.push(value);
            }
        }
    }

    /// Build an [`Entry`].
    fn build(&mut self) -> Result<Entry<'a>> {
        let sequence = self.sequence.take().context("missing entry sequence")?;
        let reading_elements = mem::take(&mut self.reading_elements);
        let kanji_elements = mem::take(&mut self.kanji_elements);
        let senses = mem::take(&mut self.senses);

        Ok(Entry {
            sequence,
            reading_elements,
            kanji_elements,
            senses,
        })
    }
}
