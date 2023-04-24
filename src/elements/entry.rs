use std::mem;

use anyhow::{Context, Result};

use crate::elements::{kanji_element, reading_element, sense, text};
use crate::elements::{KanjiElement, ReadingElement, Sense};
use crate::entities::PartOfSpeech;
use crate::parser::{Output, Poll};

#[derive(Debug, Clone, Copy)]
pub enum VerbKind {
    /// Ichidan verb.
    Ichidan,
    /// Godan verb.
    Godan,
}

#[derive(Debug)]
pub struct Entry<'a> {
    pub sequence: &'a str,
    pub reading_elements: Vec<ReadingElement<'a>>,
    pub kanji_elements: Vec<KanjiElement<'a>>,
    pub senses: Vec<Sense<'a>>,
}

impl Entry<'_> {
    /// If the entry is a verb, figure out the verb kind.
    pub(crate) fn as_verb_kind(&self) -> Option<VerbKind> {
        for sense in &self.senses {
            for pos in sense.part_of_speech.iter() {
                let kind = match pos {
                    PartOfSpeech::AdjectiveF => continue,
                    PartOfSpeech::AdjectiveI => continue,
                    PartOfSpeech::AdjectiveIx => continue,
                    PartOfSpeech::AdjectiveKari => continue,
                    PartOfSpeech::AdjectiveKu => continue,
                    PartOfSpeech::AdjectiveNa => continue,
                    PartOfSpeech::AdjectiveNari => continue,
                    PartOfSpeech::AdjectiveNo => continue,
                    PartOfSpeech::AdjectivePn => continue,
                    PartOfSpeech::AdjectiveShiku => continue,
                    PartOfSpeech::AdjectiveT => continue,
                    PartOfSpeech::Adverb => continue,
                    PartOfSpeech::AdverbTo => continue,
                    PartOfSpeech::Auxiliary => continue,
                    PartOfSpeech::AuxiliaryAdjective => continue,
                    PartOfSpeech::AuxiliaryVerb => continue,
                    PartOfSpeech::Conjunction => continue,
                    PartOfSpeech::Copular => continue,
                    PartOfSpeech::Counter => continue,
                    PartOfSpeech::Expression => continue,
                    PartOfSpeech::Interjection => continue,
                    PartOfSpeech::Noun => continue,
                    PartOfSpeech::NounAdverbial => continue,
                    PartOfSpeech::NounProper => continue,
                    PartOfSpeech::NounPrefix => continue,
                    PartOfSpeech::NounSuffix => continue,
                    PartOfSpeech::NounTemporal => continue,
                    PartOfSpeech::Numeric => continue,
                    PartOfSpeech::Pronoun => continue,
                    PartOfSpeech::Prefix => continue,
                    PartOfSpeech::Particle => continue,
                    PartOfSpeech::Suffix => continue,
                    PartOfSpeech::Unclassified => continue,
                    PartOfSpeech::VerbUnspecified => continue,
                    PartOfSpeech::VerbIchidan => VerbKind::Ichidan,
                    PartOfSpeech::VerbIchidanS => VerbKind::Ichidan,
                    PartOfSpeech::VerbNidanAS => continue,
                    PartOfSpeech::VerbNidanBK => continue,
                    PartOfSpeech::VerbNidanBS => continue,
                    PartOfSpeech::VerbNidanDK => continue,
                    PartOfSpeech::VerbNidanDS => continue,
                    PartOfSpeech::VerbNidanGK => continue,
                    PartOfSpeech::VerbNidanGS => continue,
                    PartOfSpeech::VerbNidanHK => continue,
                    PartOfSpeech::VerbNidanHS => continue,
                    PartOfSpeech::VerbNidanKK => continue,
                    PartOfSpeech::VerbNidanKS => continue,
                    PartOfSpeech::VerbNidanMK => continue,
                    PartOfSpeech::VerbNidanMS => continue,
                    PartOfSpeech::VerbNidanNS => continue,
                    PartOfSpeech::VerbNidanRK => continue,
                    PartOfSpeech::VerbNidanRS => continue,
                    PartOfSpeech::VerbNidanSS => continue,
                    PartOfSpeech::VerbNidanTK => continue,
                    PartOfSpeech::VerbNidanTS => continue,
                    PartOfSpeech::VerbNidanWS => continue,
                    PartOfSpeech::VerbNidanYK => continue,
                    PartOfSpeech::VerbNidanYS => continue,
                    PartOfSpeech::VerbNidanZS => continue,
                    PartOfSpeech::VerbYodanB => continue,
                    PartOfSpeech::VerbYodanG => continue,
                    PartOfSpeech::VerbYodanH => continue,
                    PartOfSpeech::VerbYodanK => continue,
                    PartOfSpeech::VerbYodanM => continue,
                    PartOfSpeech::VerbYodanN => continue,
                    PartOfSpeech::VerbYodanR => continue,
                    PartOfSpeech::VerbYodanS => continue,
                    PartOfSpeech::VerbYodanT => continue,
                    PartOfSpeech::VerbGodanAru => VerbKind::Godan,
                    PartOfSpeech::VerbGodanB => VerbKind::Godan,
                    PartOfSpeech::VerbGodanG => VerbKind::Godan,
                    PartOfSpeech::VerbGodanK => VerbKind::Godan,
                    PartOfSpeech::VerbGodanKS => VerbKind::Godan,
                    PartOfSpeech::VerbGodanM => VerbKind::Godan,
                    PartOfSpeech::VerbGodanN => VerbKind::Godan,
                    PartOfSpeech::VerbGodanR => VerbKind::Godan,
                    PartOfSpeech::VerbGodanRI => VerbKind::Godan,
                    PartOfSpeech::VerbGodanS => VerbKind::Godan,
                    PartOfSpeech::VerbGodanT => VerbKind::Godan,
                    PartOfSpeech::VerbGodanU => VerbKind::Godan,
                    PartOfSpeech::VerbGodanUS => VerbKind::Godan,
                    PartOfSpeech::VerbGodanUru => VerbKind::Godan,
                    PartOfSpeech::VerbIntransitive => continue,
                    PartOfSpeech::VerbKuru => continue,
                    PartOfSpeech::VerbNu => continue,
                    PartOfSpeech::VerbRu => continue,
                    PartOfSpeech::VerbSuru => continue,
                    PartOfSpeech::VerbSuC => continue,
                    PartOfSpeech::VerbSuruIncluded => continue,
                    PartOfSpeech::VerbSuruSpecial => continue,
                    PartOfSpeech::VerbTransitive => continue,
                    PartOfSpeech::VerbZuru => continue,
                };

                return Some(kind);
            }
        }

        None
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
    sequence: Option<&'a str>,
    reading_elements: Vec<ReadingElement<'a>>,
    kanji_elements: Vec<KanjiElement<'a>>,
    senses: Vec<Sense<'a>>,
}

impl<'a> Builder<'a> {
    builder! {
        self => Entry<'a> {
            "ent_seq", EntrySequence, value => {
                self.sequence = Some(value);
            },
            "r_ele", ReadingElement, value => {
                self.reading_elements.push(value);
            },
            "k_ele", KanjiElement, value => {
                self.kanji_elements.push(value);
            },
            "sense", Sense, value => {
                self.senses.push(value);
            },
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
