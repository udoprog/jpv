use std::collections::{hash_set, HashMap, HashSet};

use anyhow::{anyhow, Context, Result};

use crate::elements::Entry;
use crate::parser::Parser;
use crate::verb;
use crate::PartOfSpeech;

/// Extra information about an index.
#[non_exhaustive]
pub enum IndexExtra {
    /// No extra information on why the index was added.
    None,
    /// Index was added because of a conjugation.
    Conjugation(verb::Conjugation),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
enum IndexKind {
    /// An exact dictionary index.
    Exact(usize),
    /// A lookup based on a conjugation.
    VerbConjugation(usize, verb::Conjugation),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Index {
    kind: IndexKind,
}

impl Index {
    /// Extra information on index.
    pub fn extra(&self) -> IndexExtra {
        match &self.kind {
            IndexKind::Exact(_) => IndexExtra::None,
            &IndexKind::VerbConjugation(_, conjugation) => IndexExtra::Conjugation(conjugation),
        }
    }
}

pub struct Database<'a> {
    database: Vec<Entry<'a>>,
    lookup: HashMap<String, Vec<IndexKind>>,
    by_pos: HashMap<PartOfSpeech, HashSet<usize>>,
}

impl<'a> Database<'a> {
    /// Get an entry from the database.
    pub fn get(&self, index: Index) -> Result<&Entry<'a>> {
        let index = match index.kind {
            IndexKind::Exact(index) => index,
            IndexKind::VerbConjugation(index, ..) => index,
        };

        self.database
            .get(index)
            .with_context(|| anyhow!("Missing index `{index}`"))
    }

    /// Load the given dictionary.
    pub fn load(dict: &'a str) -> Result<Self> {
        let mut database = Vec::new();
        let mut lookup = HashMap::<_, Vec<IndexKind>>::new();
        let mut by_pos = HashMap::<_, HashSet<usize>>::new();

        let mut parser = Parser::new(&dict);

        while let Some(entry) = parser.parse()? {
            tracing::trace!(?entry);

            let index = database.len();

            for sense in &entry.senses {
                for pos in &sense.pos {
                    by_pos.entry(pos).or_default().insert(index);
                }

                for g in &sense.gloss {
                    for part in g.text.split_whitespace() {
                        let part = part.trim();

                        lookup
                            .entry(part.to_string())
                            .or_default()
                            .push(IndexKind::Exact(index));
                    }
                }
            }

            for el in &entry.reading_elements {
                lookup
                    .entry(el.text.to_string())
                    .or_default()
                    .push(IndexKind::Exact(index));
            }

            for el in &entry.kanji_elements {
                lookup
                    .entry(el.text.to_string())
                    .or_default()
                    .push(IndexKind::Exact(index));
            }

            if let Some(c) = verb::conjugate(&entry) {
                for (conjugation, phrase) in c.iter() {
                    lookup
                        .entry(phrase.to_string())
                        .or_default()
                        .push(IndexKind::VerbConjugation(index, conjugation));
                }
            }

            database.push(entry);
        }

        Ok(Self {
            database,
            lookup,
            by_pos,
        })
    }

    /// Get indexes by part of speech.
    pub fn by_pos(&self, pos: PartOfSpeech) -> Indexes<'_> {
        let by_pos = self.by_pos.get(&pos);
        let iter = by_pos.map(|set| set.iter());

        Indexes { by_pos, iter }
    }

    /// Perform a free text lookup.
    pub fn lookup(&self, query: &str) -> impl Iterator<Item = Index> + '_ {
        self.lookup
            .get(query)
            .into_iter()
            .flatten()
            .copied()
            .map(|kind| Index { kind })
    }
}

/// A collection of indexes.
pub struct Indexes<'a> {
    by_pos: Option<&'a HashSet<usize>>,
    iter: Option<hash_set::Iter<'a, usize>>,
}

impl Indexes<'_> {
    /// Test if the indexes collections contains the given index.
    pub fn contains(&self, index: &Index) -> bool {
        let Some(by_pos) = &self.by_pos else {
            return false;
        };

        let index = match &index.kind {
            IndexKind::Exact(index) => index,
            IndexKind::VerbConjugation(index, ..) => index,
        };

        by_pos.contains(index)
    }
}

impl Iterator for Indexes<'_> {
    type Item = Index;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let &index = self.iter.as_mut()?.next()?;

        Some(Index {
            kind: IndexKind::Exact(index),
        })
    }
}
