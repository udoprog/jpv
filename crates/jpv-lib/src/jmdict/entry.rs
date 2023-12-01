use std::mem;

use anyhow::{Context, Result};
use musli::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::jmdict::{kanji_element, reading_element, sense, text};
use crate::jmdict::{KanjiElement, ReadingElement, Sense};
use crate::{EntryKey, Weight};

#[borrowme::borrowme]
#[derive(Clone, Debug, Serialize, Deserialize, Encode, Decode)]
#[musli(packed)]
pub struct Entry<'a> {
    pub sequence: u64,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[borrowed_attr(serde(borrow))]
    pub reading_elements: Vec<ReadingElement<'a>>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[borrowed_attr(serde(borrow))]
    pub kanji_elements: Vec<KanjiElement<'a>>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[borrowed_attr(serde(borrow))]
    pub senses: Vec<Sense<'a>>,
}

impl Entry<'_> {
    /// Entry weight.
    pub fn sort_key(&self, input: &str, conjugation: bool) -> EntryKey {
        // Boost based on exact query.
        let mut query = 1.0f32;
        // Store the priority which performs the maximum boost.
        let mut priority = 1.0f32;
        // Perform boost by number of senses, maximum boost at 10 senses.
        let sense_count = 1.0 + self.senses.len().min(10) as f32 / 10.0;
        // Conjugation boost.
        let conjugation = if conjugation { 1.2 } else { 1.0 };
        // Calculate length boost.
        let length = (input.chars().count().min(10) as f32 / 10.0) * 1.2;

        for element in &self.reading_elements {
            if element.text == input {
                if element.no_kanji || self.kanji_elements.iter().all(|k| k.is_rare()) {
                    query = query.max(3.0);
                } else {
                    query = query.max(2.0);
                }
            }

            for p in &element.priority {
                priority = priority.max(p.weight());
            }
        }

        for element in &self.kanji_elements {
            if element.text == input {
                query = query.max(3.0);
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
