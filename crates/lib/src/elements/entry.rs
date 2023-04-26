use core::mem;

use anyhow::{Context, Result};

use crate::elements::{kanji_element, reading_element, sense, text};
use crate::elements::{KanjiElement, ReadingElement, Sense};

#[derive(Debug)]
pub struct Entry<'a> {
    pub sequence: u64,
    pub reading_elements: Vec<ReadingElement<'a>>,
    pub kanji_elements: Vec<KanjiElement<'a>>,
    pub senses: Vec<Sense<'a>>,
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
