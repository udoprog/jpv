//! Database that can be used as a dictionary.

mod file;
mod index;
mod strings;

use std::collections::{hash_set, BTreeMap, BTreeSet, HashMap, HashSet};
use std::sync::Arc;

use anyhow::{anyhow, Context, Result};
use musli::mode::DefaultMode;
use musli::{Decode, Encode};
use musli_storage::int::{Fixed, FixedUsize, Variable};
use musli_storage::Encoding;
use serde::{Deserialize, Serialize};

use crate::adjective;
use crate::elements::{Entry, EntryKey};
use crate::inflection::Inflection;
use crate::parser::Parser;
use crate::verb;
use crate::PartOfSpeech;

/// Encoding used for storing database.
const ENCODING: Encoding<DefaultMode, Variable, Variable> = Encoding::new();

/// Encoding used for storing the header.
const HEADER_ENCODING: Encoding<DefaultMode, Fixed, FixedUsize<u32>> =
    Encoding::new().with_fixed_integers().with_fixed_lengths();

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntryResultKey {
    pub index: u32,
    #[serde(flatten)]
    pub key: EntryKey,
    pub sources: BTreeSet<IndexSource>,
}

/// Extra information about an index.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, Encode, Decode,
)]
#[non_exhaustive]
#[serde(tag = "type")]
pub enum IndexSource {
    /// No extra information on why the index was added.
    #[serde(rename = "base")]
    None,
    /// Index was added because of a verb inflection.
    #[serde(rename = "verb-c")]
    VerbInflection {
        reading: verb::Reading,
        inflection: Inflection,
    },
    /// Index was added because of an adjective inflection.
    #[serde(rename = "adj-c")]
    AdjectiveInflection { inflection: Inflection },
}

impl IndexSource {
    /// Test if extra indicates an inflection.
    pub fn is_inflection(&self) -> bool {
        match self {
            IndexSource::None => false,
            IndexSource::VerbInflection { .. } => true,
            IndexSource::AdjectiveInflection { .. } => true,
        }
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, Encode, Decode,
)]
pub struct Id {
    index: u32,
    extra: IndexSource,
}

impl Id {
    fn new(index: u32) -> Self {
        Self {
            index,
            extra: IndexSource::None,
        }
    }

    fn verb_inflection(index: u32, reading: verb::Reading, inflection: Inflection) -> Self {
        Self {
            index: index,
            extra: IndexSource::VerbInflection {
                reading,
                inflection,
            },
        }
    }

    fn adjective_inflection(index: u32, inflection: Inflection) -> Self {
        Self {
            index: index,
            extra: IndexSource::AdjectiveInflection { inflection },
        }
    }

    /// Get the unique index this id corresponds to.
    pub fn index(&self) -> u32 {
        self.index
    }

    /// Extra information on index.
    pub fn source(&self) -> IndexSource {
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

        let entry_offset = u32::try_from(output.len()).context("offset overflow")?;
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

        for (reading, c) in verb::conjugate(&entry) {
            for (inflection, pair) in c.iter() {
                let suffix_index = strings.insert(pair.suffix().to_string())?;

                for word in [pair.text(), pair.reading()] {
                    let word_index = strings.insert(word.to_string())?;

                    index
                        .lookup
                        .entry((word_index, suffix_index))
                        .or_default()
                        .push(Id::verb_inflection(entry_offset, reading, *inflection));
                }
            }
        }

        if let Some(c) = adjective::conjugate(&entry) {
            for (inflection, pair) in c.iter() {
                let suffix_index = strings.insert(pair.suffix().to_string())?;

                for word in [pair.text(), pair.reading()] {
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

#[derive(Clone)]
pub struct Database<'a> {
    #[allow(unused)]
    header: Arc<file::Header>,
    index: Arc<index::Index<'a>>,
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
                .get(prefix as usize)
                .with_context(|| anyhow!("Bad prefix string at lookup {i}"))?;

            let suffix = strings
                .get(suffix as usize)
                .with_context(|| anyhow!("Bad suffix string at lookup {i}"))?;

            index
                .lookup
                .entry(index::Pair::new(prefix, suffix))
                .or_default()
                .extend(value);
        }

        index.by_pos = index_data.by_pos;
        index.by_sequence = index_data.by_sequence;

        Ok(Self {
            header: Arc::new(header),
            index: Arc::new(index),
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
            .get((index as usize)..)
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
    pub fn lookup<'b>(&'b self, query: &'b str) -> impl Iterator<Item = Id> + 'b {
        let pair = index::Pair::new(query.as_bytes(), b"");
        self.index.lookup.get(&pair).into_iter().flatten().copied()
    }

    /// Test if db contains the given string.
    pub fn contains(&self, query: &str) -> bool {
        self.index
            .lookup
            .contains_key(&index::Pair::new(query.as_bytes(), b""))
    }

    /// Perform the given search.
    pub fn search(&self, input: &str) -> Result<Vec<(EntryResultKey, Entry<'a>)>> {
        let mut entries = Vec::new();
        let mut dedup = HashMap::new();

        for id in self.lookup(input) {
            let entry = self.get(id)?;

            let Some(&i) = dedup.get(&id.index()) else {
                dedup.insert(id.index(), entries.len());

                let data = EntryResultKey {
                    index: id.index(),
                    sources: [id.source()].into_iter().collect(),
                    key: EntryKey::default(),
                };

                entries.push((data, entry));
                continue;
            };

            let Some((data, _)) = entries.get_mut(i) else {
                continue;
            };

            data.sources.insert(id.source());
        }

        for (data, e) in &mut entries {
            let inflection = data.sources.iter().any(|index| index.is_inflection());
            data.key = e.sort_key(input, inflection);
        }

        Ok(entries)
    }

    /// Analyze the given string, looking it up in the database and returning
    /// all prefix matching entries and their texts.
    pub fn analyze(&self, q: &str, start: usize) -> BTreeMap<EntryKey, String> {
        let mut inputs = BTreeMap::new();

        let Some(suffix) = q.get(start..) else {
            return inputs;
        };

        let mut it = suffix.chars();

        while !it.as_str().is_empty() {
            let mut sort_key = None;

            for id in self.lookup(it.as_str()) {
                let Ok(e) = self.get(id) else {
                    continue;
                };

                let a = e.sort_key(it.as_str(), id.source().is_inflection());

                if let Some(b) = sort_key.take() {
                    sort_key = Some(a.min(b));
                } else {
                    sort_key = Some(a);
                }
            }

            if let Some(e) = sort_key.take() {
                inputs.insert(e, it.as_str().to_owned());
            }

            it.next_back();
        }

        inputs
    }
}

/// A collection of indexes.
pub struct Indexes<'a> {
    by_pos: Option<&'a HashSet<u32>>,
    iter: Option<hash_set::Iter<'a, u32>>,
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
