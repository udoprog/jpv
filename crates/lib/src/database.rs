//! Database that can be used as a dictionary.

use std::collections::{hash_set, HashMap, HashSet};

use anyhow::{anyhow, Context, Result};
use musli::mode::DefaultMode;
use musli::Decode;
use musli::Encode;
use musli_storage::int::Variable;
use musli_storage::Encoding;

use crate::adjective;
use crate::elements::Entry;
use crate::inflection::Inflection;
use crate::parser::Parser;
use crate::verb;
use crate::PartOfSpeech;

/// Encoding used for storing database.
const ENCODING: Encoding<DefaultMode, Variable, Variable> = Encoding::new();

/// Extra information about an index.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[non_exhaustive]
pub enum IndexExtra {
    /// No extra information on why the index was added.
    None,
    /// Index was added because of a verb inflection.
    VerbInflection(Inflection),
    /// Index was added because of an adjective inflection.
    AdjectiveInflection(Inflection),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Encode, Decode)]
enum IdKind {
    /// An exact dictionary index.
    Exact(usize),
    /// A lookup based on a inflection.
    VerbInflection(usize, Inflection),
    /// A lookup based on an adjective inflection.
    AdjectiveInflection(usize, Inflection),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Encode, Decode)]
pub struct Id {
    kind: IdKind,
}

impl Id {
    /// Get the unique index this id corresponds to.
    pub fn index(&self) -> usize {
        match self.kind {
            IdKind::Exact(index) => index,
            IdKind::VerbInflection(index, _) => index,
            IdKind::AdjectiveInflection(index, _) => index,
        }
    }

    /// Extra information on index.
    pub fn extra(&self) -> IndexExtra {
        match self.kind {
            IdKind::Exact(_) => IndexExtra::None,
            IdKind::VerbInflection(_, inflection) => IndexExtra::VerbInflection(inflection),
            IdKind::AdjectiveInflection(_, inflection) => {
                IndexExtra::AdjectiveInflection(inflection)
            }
        }
    }
}

#[derive(Encode)]
pub struct Index {
    lookup: HashMap<String, Vec<IdKind>>,
    by_pos: HashMap<PartOfSpeech, HashSet<usize>>,
    by_sequence: HashMap<u64, usize>,
}

impl Index {
    /// Convert index into bytes.
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        Ok(ENCODING.to_vec(self)?)
    }
}

#[derive(Decode)]
pub struct IndexRef<'a> {
    lookup: HashMap<&'a [u8], Vec<IdKind>>,
    by_pos: HashMap<PartOfSpeech, HashSet<usize>>,
    by_sequence: HashMap<u64, usize>,
}

impl<'a> IndexRef<'a> {
    /// Build index from bytes.
    pub fn from_bytes(bytes: &'a [u8]) -> Result<Self> {
        Ok(ENCODING.from_slice(bytes)?)
    }
}

/// Load the given dictionary and convert into the internal format.
pub fn load(dict: &str) -> Result<(Vec<u8>, Index)> {
    let mut data = Vec::new();
    let mut lookup = HashMap::<_, Vec<IdKind>>::new();
    let mut by_pos = HashMap::<_, HashSet<usize>>::new();
    let mut by_sequence = HashMap::new();

    let mut parser = Parser::new(dict);

    while let Some(entry) = parser.parse()? {
        tracing::trace!(?entry);

        let index = data.len();

        by_sequence.insert(entry.sequence, index);

        for sense in &entry.senses {
            for pos in &sense.pos {
                by_pos.entry(pos).or_default().insert(index);
            }

            for g in &sense.gloss {
                lookup
                    .entry(g.text.to_string())
                    .or_default()
                    .push(IdKind::Exact(index));
            }
        }

        for el in &entry.reading_elements {
            lookup
                .entry(el.text.to_string())
                .or_default()
                .push(IdKind::Exact(index));
        }

        for el in &entry.kanji_elements {
            lookup
                .entry(el.text.to_string())
                .or_default()
                .push(IdKind::Exact(index));
        }

        if let Some(c) = verb::conjugate(&entry) {
            for (inflection, phrase) in c.iter() {
                lookup
                    .entry(phrase.to_string())
                    .or_default()
                    .push(IdKind::VerbInflection(index, inflection));
            }
        }

        if let Some(c) = adjective::conjugate(&entry) {
            for (inflection, phrase) in c.iter() {
                lookup
                    .entry(phrase.to_string())
                    .or_default()
                    .push(IdKind::AdjectiveInflection(index, inflection));
            }
        }

        ENCODING.to_writer(&mut data, &entry)?;
    }

    let index = Index {
        lookup,
        by_pos,
        by_sequence,
    };
    Ok((data, index))
}

pub struct Database<'a> {
    data: &'a [u8],
    index: IndexRef<'a>,
}

impl<'a> Database<'a> {
    /// Construct a new database wrapper.
    pub fn new(data: &'a [u8], index: IndexRef<'a>) -> Self {
        Self { data, index }
    }

    /// Get identifier by sequence.
    pub fn lookup_sequence(&self, sequence: u64) -> Option<Id> {
        let &index = self.index.by_sequence.get(&sequence)?;

        Some(Id {
            kind: IdKind::Exact(index),
        })
    }

    /// Get an entry from the database.
    pub fn get(&self, id: Id) -> Result<Entry<'a>> {
        let index = id.index();

        let slice = self
            .data
            .get(index..)
            .with_context(|| anyhow!("Missing index `{index}`"))?;

        Ok(ENCODING.from_slice(slice)?)
    }

    /// Get indexes by part of speech.
    pub fn by_pos(&self, pos: PartOfSpeech) -> Indexes<'_> {
        let by_pos = self.index.by_pos.get(&pos);
        let iter = by_pos.map(|set| set.iter());
        Indexes { by_pos, iter }
    }

    /// Perform a free text lookup.
    pub fn lookup(&self, query: &str) -> impl Iterator<Item = Id> + '_ {
        self.index
            .lookup
            .get(query.as_bytes())
            .into_iter()
            .flatten()
            .copied()
            .map(|kind| Id { kind })
    }
}

/// A collection of indexes.
pub struct Indexes<'a> {
    by_pos: Option<&'a HashSet<usize>>,
    iter: Option<hash_set::Iter<'a, usize>>,
}

impl Indexes<'_> {
    /// Test if the indexes collections contains the given index.
    pub fn contains(&self, index: &Id) -> bool {
        let Some(by_pos) = &self.by_pos else {
            return false;
        };

        let index = match &index.kind {
            IdKind::Exact(index) => index,
            IdKind::VerbInflection(index, ..) => index,
            IdKind::AdjectiveInflection(index, ..) => index,
        };

        by_pos.contains(index)
    }
}

impl Iterator for Indexes<'_> {
    type Item = Id;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let &index = self.iter.as_mut()?.next()?;

        Some(Id {
            kind: IdKind::Exact(index),
        })
    }
}