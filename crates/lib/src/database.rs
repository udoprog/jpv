//! Database that can be used as a dictionary.

mod file;
mod index;
mod strings;

use std::collections::{hash_set, HashSet};

use anyhow::{anyhow, Context, Result};
use musli::mode::DefaultMode;
use musli::{Decode, Encode};
use musli_storage::int::{Fixed, FixedUsize, Variable};
use musli_storage::Encoding;

use crate::adjective;
use crate::elements::Entry;
use crate::inflection::Inflection;
use crate::parser::Parser;
use crate::verb;
use crate::PartOfSpeech;

/// Encoding used for storing database.
const ENCODING: Encoding<DefaultMode, Variable, Variable> = Encoding::new();

/// Encoding used for storing the header.
const HEADER_ENCODING: Encoding<DefaultMode, Fixed, FixedUsize<u32>> =
    Encoding::new().with_fixed_integers().with_fixed_lengths();

/// Extra information about an index.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Encode, Decode)]
#[non_exhaustive]
pub enum IndexExtra {
    /// No extra information on why the index was added.
    None,
    /// Index was added because of a verb inflection.
    VerbInflection(Inflection),
    /// Index was added because of an adjective inflection.
    AdjectiveInflection(Inflection),
}

impl IndexExtra {
    /// Test if extra indicates an inflection.
    pub fn is_inflection(&self) -> bool {
        match self {
            IndexExtra::None => false,
            IndexExtra::VerbInflection(_) => true,
            IndexExtra::AdjectiveInflection(_) => true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Encode, Decode)]
pub struct Id {
    index: u32,
    extra: IndexExtra,
}

impl Id {
    fn new(index: usize) -> Self {
        Self {
            index: index as u32,
            extra: IndexExtra::None,
        }
    }

    fn verb_inflection(index: usize, inflection: Inflection) -> Self {
        Self {
            index: index as u32,
            extra: IndexExtra::VerbInflection(inflection),
        }
    }

    fn adjective_inflection(index: usize, inflection: Inflection) -> Self {
        Self {
            index: index as u32,
            extra: IndexExtra::AdjectiveInflection(inflection),
        }
    }

    /// Get the unique index this id corresponds to.
    pub fn index(&self) -> usize {
        self.index as usize
    }

    /// Extra information on index.
    pub fn extra(&self) -> IndexExtra {
        self.extra
    }
}

/// Load the given dictionary and convert into the internal format.
pub fn load(dict: &str) -> Result<Vec<u8>> {
    let mut output = Vec::new();
    let mut strings = strings::Strings::default();

    let mut index = index::Data::default();
    let mut header = file::Header::default();

    HEADER_ENCODING.to_writer(&mut output, &header)?;
    header.entries = output.len();
    let empty = strings.insert("")?;

    let mut parser = Parser::new(dict);

    while let Some(entry) = parser.parse()? {
        tracing::trace!(?entry);

        let entry_offset = output.len();
        index.by_sequence.insert(entry.sequence, entry_offset);

        for sense in &entry.senses {
            for pos in &sense.pos {
                index.by_pos.entry(pos).or_default().insert(entry_offset);
            }

            for g in &sense.gloss {
                let prefix = strings.insert(g.text)?;

                index
                    .lookup
                    .entry((prefix, empty))
                    .or_default()
                    .push(Id::new(entry_offset));
            }
        }

        for el in &entry.reading_elements {
            let prefix = strings.insert(el.text)?;

            index
                .lookup
                .entry((prefix, empty))
                .or_default()
                .push(Id::new(entry_offset));
        }

        for el in &entry.kanji_elements {
            let prefix = strings.insert(el.text)?;

            index
                .lookup
                .entry((prefix, empty))
                .or_default()
                .push(Id::new(entry_offset));
        }

        if let Some(c) = verb::conjugate(&entry) {
            for (inflection, pair) in c.iter() {
                let suffix_index = strings.insert(pair.suffix().to_string())?;

                for word in [pair.kanji(), pair.reading()] {
                    let word_index = strings.insert(word.to_string())?;

                    index
                        .lookup
                        .entry((word_index, suffix_index))
                        .or_default()
                        .push(Id::verb_inflection(entry_offset, *inflection));
                }
            }
        }

        if let Some(c) = adjective::conjugate(&entry) {
            for (inflection, pair) in c.iter() {
                let suffix_index = strings.insert(pair.suffix().to_string())?;

                for word in [pair.kanji(), pair.reading()] {
                    let word_index = strings.insert(word.to_string())?;

                    index
                        .lookup
                        .entry((word_index, suffix_index))
                        .or_default()
                        .push(Id::adjective_inflection(entry_offset, *inflection));
                }
            }
        }

        ENCODING.to_writer(&mut output, &entry)?;
    }

    header.index = output.len();
    ENCODING.to_writer(&mut output, &index)?;
    header.strings = output.len();
    output.extend(strings.as_slice());
    // Write the real header.
    HEADER_ENCODING.to_writer(&mut output[..file::HEADER_SIZE], &header)?;
    tracing::trace!(?header, strings = ?strings.as_slice().len());
    Ok(output)
}

pub struct Database<'a> {
    #[allow(unused)]
    header: file::Header,
    index: index::Index,
    data: &'a [u8],
}

impl<'a> Database<'a> {
    /// Construct a new database wrapper.
    pub fn new(data: &'a [u8]) -> Result<Self> {
        let header: file::Header = HEADER_ENCODING
            .decode(data)
            .context("failed to decode header")?;

        tracing::trace!(?header);

        let index = data
            .get(header.index..header.strings)
            .context("Missing index")?;

        let strings =
            strings::StringsRef::new(data.get(header.strings..).context("Missing strings")?);

        let index_data: index::Data = ENCODING
            .decode(index)
            .context("failed to decode index data")?;

        let mut index = index::Index::default();

        for (i, ((prefix, suffix), value)) in index_data.lookup.into_iter().enumerate() {
            let prefix = strings
                .get(prefix)
                .with_context(|| anyhow!("Bad prefix string at lookup {i}"))?;

            let suffix = strings
                .get(suffix)
                .with_context(|| anyhow!("Bad suffix string at lookup {i}"))?;

            let mut string = prefix.to_vec();
            string.extend(suffix);
            index.lookup.insert(string, value);
        }

        index.by_pos = index_data.by_pos;
        index.by_sequence = index_data.by_sequence;

        Ok(Self {
            header,
            index,
            data,
        })
    }

    /// Get identifier by sequence.
    pub fn lookup_sequence(&self, sequence: u64) -> Option<Id> {
        let &index = self.index.by_sequence.get(&sequence)?;
        Some(Id::new(index))
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
    }

    /// Test if db contains the given string.
    pub fn contains(&self, query: &str) -> bool {
        self.index.lookup.contains_key(query.as_bytes())
    }
}

/// A collection of indexes.
pub struct Indexes<'a> {
    by_pos: Option<&'a HashSet<usize>>,
    iter: Option<hash_set::Iter<'a, usize>>,
}

impl Indexes<'_> {
    /// Test if the indexes collections contains the given index.
    pub fn contains(&self, id: &Id) -> bool {
        let Some(by_pos) = &self.by_pos else {
            return false;
        };

        by_pos.contains(&id.index())
    }
}

impl Iterator for Indexes<'_> {
    type Item = Id;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let &index = self.iter.as_mut()?.next()?;
        Some(Id::new(index))
    }
}
