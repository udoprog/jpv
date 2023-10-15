//! Database that can be used as a dictionary.

mod index;

use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet, HashMap};

use anyhow::Result;
use musli::mode::DefaultMode;
use musli::{Decode, Encode};
use musli_storage::int::Variable;
use musli_storage::Encoding;
use musli_zerocopy::pointer::{Ref, Slice};
use musli_zerocopy::{swiss, AlignedBuf, Buf, ZeroCopy};
use serde::{Deserialize, Serialize};

use crate::adjective;
use crate::elements::{Entry, EntryKey};
use crate::inflection::Inflection;
use crate::parser::Parser;
use crate::verb;
use crate::PartOfSpeech;

/// Encoding used for storing database.
const ENCODING: Encoding<DefaultMode, Variable, Variable> = Encoding::new();

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntryResultKey {
    pub index: usize,
    #[serde(flatten)]
    pub key: EntryKey,
    pub sources: BTreeSet<IndexSource>,
}

/// Extra information about an index.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    Serialize,
    Deserialize,
    Encode,
    Decode,
    ZeroCopy,
)]
#[non_exhaustive]
#[serde(tag = "type")]
#[repr(u8)]
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

#[test]
fn index_source_zerocopy() -> Result<()> {
    use musli_zerocopy::AlignedBuf;

    let mut buf = AlignedBuf::new();
    buf.extend_from_slice(&[0; 15]);
    let none = buf.store(&IndexSource::None);

    let expected2 = IndexSource::VerbInflection {
        reading: verb::Reading {
            kanji: verb::ReadingOption::None,
            reading: 0,
        },
        inflection: Inflection::all(),
    };

    let verb_inflection = buf.store(&expected2);

    let buf = buf.as_aligned();
    assert_eq!(buf.load(none)?, &IndexSource::None);
    assert_eq!(buf.load(verb_inflection)?, &expected2);
    Ok(())
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, ZeroCopy)]
#[repr(C)]
pub struct Id {
    index: Ref<Slice<u8>>,
    extra: IndexSource,
}

#[test]
fn test_id() -> Result<()> {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, ZeroCopy)]
    #[repr(C)]
    pub struct InnerEnum {
        index: Ref<Slice<u8>>,
        extra: IndexSource,
    }

    let mut buf = AlignedBuf::new();

    let index = buf.store_slice(&[1, 2, 3, 4]);
    let index = buf.store(&bytes);

    let id = InnerEnum {
        index,
        extra: IndexSource::None,
    };

    let id = buf.store(&id);

    let buf = buf.as_aligned();
    let id = buf.load(&id)?;
    dbg!(id);
    Ok(())
}

impl Id {
    fn new(index: Ref<Slice<u8>>) -> Self {
        Self {
            index,
            extra: IndexSource::None,
        }
    }

    fn verb_inflection(
        index: Ref<Slice<u8>>,
        reading: verb::Reading,
        inflection: Inflection,
    ) -> Self {
        Self {
            index,
            extra: IndexSource::VerbInflection {
                reading,
                inflection,
            },
        }
    }

    fn adjective_inflection(index: Ref<Slice<u8>>, inflection: Inflection) -> Self {
        Self {
            index,
            extra: IndexSource::AdjectiveInflection { inflection },
        }
    }

    /// Get the unique index this id corresponds to.
    pub fn index(&self) -> Ref<Slice<u8>> {
        self.index
    }

    /// Extra information on index.
    pub fn source(&self) -> IndexSource {
        self.extra
    }
}

/// Load the given dictionary and convert into the internal format.
pub fn load(dict: &str) -> Result<AlignedBuf> {
    let mut buf = AlignedBuf::new();

    let index = buf.store_uninit::<index::Index>();

    let mut data = index::Data::default();

    let mut parser = Parser::new(dict);

    let mut output = Vec::new();

    while let Some(entry) = parser.parse()? {
        output.clear();
        ENCODING.to_writer(&mut output, &entry)?;

        let slice = buf.store_slice(&output);
        let entry_ref = buf.store_uninit::<Slice<u8>>();
        buf.load_uninit_mut(entry_ref).write(&slice);
        let entry_ref = entry_ref.assume_init();

        data.by_sequence.insert(entry.sequence, entry_ref);

        for sense in &entry.senses {
            for pos in &sense.pos {
                data.by_pos.entry(pos).or_default().insert(entry_ref);
            }

            for g in &sense.gloss {
                data.lookup
                    .entry(Cow::Borrowed(g.text))
                    .or_default()
                    .push(Id::new(entry_ref));
            }
        }

        for el in &entry.reading_elements {
            data.lookup
                .entry(Cow::Borrowed(el.text))
                .or_default()
                .push(Id::new(entry_ref));
        }

        for el in &entry.kanji_elements {
            data.lookup
                .entry(Cow::Borrowed(el.text))
                .or_default()
                .push(Id::new(entry_ref));
        }

        for (reading, c) in verb::conjugate(&entry) {
            for (inflection, pair) in c.iter() {
                for word in [pair.text(), pair.reading()] {
                    let key = Cow::Owned(format!("{}{}", word, pair.suffix()));

                    data.lookup
                        .entry(key)
                        .or_default()
                        .push(Id::verb_inflection(entry_ref, reading, *inflection));
                }
            }
        }

        if let Some(c) = adjective::conjugate(&entry) {
            for (inflection, pair) in c.iter() {
                for word in [pair.text(), pair.reading()] {
                    let key = Cow::Owned(format!("{}{}", word, pair.suffix()));

                    data.lookup
                        .entry(key)
                        .or_default()
                        .push(Id::adjective_inflection(entry_ref, *inflection));
                }
            }
        }
    }

    tracing::info!("Serializing to zerocopy structure");

    tracing::info!("lookup: {}", data.lookup.len());

    let lookup = {
        let mut lookup = Vec::new();

        for (index, (key, set)) in data.lookup.into_iter().enumerate() {
            if index % 10000 == 0 {
                tracing::info!("{}", index);
            }

            let mut values = Vec::new();

            for v in set {
                values.push(v);
            }

            let key = buf.store_unsized(key.as_ref());
            let set = buf.store_slice(&values);
            lookup.push((key, set));
        }

        tracing::info!("storing map...");
        swiss::store_map(&mut buf, lookup)?
    };

    tracing::info!("by_pos: {}", data.by_pos.len());

    let by_pos = {
        let mut by_pos = Vec::new();

        for (index, (key, set)) in data.by_pos.into_iter().enumerate() {
            if index % 1000 == 0 {
                tracing::info!("{}", index);
            }

            let mut values = Vec::new();

            for v in set {
                values.push(v);
            }

            values.sort();
            let set = buf.store_slice(&values);
            by_pos.push((key, set));
        }

        tracing::info!("storing map...");
        swiss::store_map(&mut buf, by_pos)?
    };

    tracing::info!("by_sequence: {}", data.by_sequence.len());

    let by_sequence = {
        let mut by_sequence = Vec::new();

        for (key, value) in data.by_sequence {
            by_sequence.push((key, value));
        }

        tracing::info!("storing map...");
        swiss::store_map(&mut buf, by_sequence)?
    };

    buf.load_uninit_mut(index).write(&index::Index {
        lookup,
        by_pos,
        by_sequence,
    });

    Ok(buf)
}

#[derive(Clone)]
pub struct Database<'a> {
    index: &'a index::Index,
    data: &'a Buf,
}

impl<'a> Database<'a> {
    /// Construct a new database wrapper.
    pub fn new(data: &'a [u8]) -> Result<Self> {
        let data = Buf::new(data);
        let index = data.load(Ref::<index::Index>::zero())?;

        Ok(Self { index, data })
    }

    /// Get identifier by sequence.
    pub fn lookup_sequence(&self, sequence: u64) -> Result<Option<Id>> {
        let Some(index) = self.index.by_sequence.get(self.data, &sequence)? else {
            return Ok(None);
        };

        Ok(Some(Id::new(*index)))
    }

    /// Get an entry from the database.
    pub fn get(&self, id: Id) -> Result<Entry<'a>> {
        let index = id.index();
        let slice = *self.data.load(index)?;
        let bytes = self.data.load(slice)?;
        Ok(ENCODING.from_slice(bytes)?)
    }

    /// Get indexes by part of speech.
    #[tracing::instrument(skip_all)]
    pub fn by_pos(&self, pos: PartOfSpeech) -> Result<Vec<Id>> {
        let mut output = Vec::new();

        if let Some(by_pos) = self.index.by_pos.get(self.data, &pos)? {
            tracing::trace!(?by_pos);

            for id in self.data.load(by_pos)? {
                output.push(Id::new(*id));
            }
        }

        tracing::trace!(output = output.len());
        Ok(output)
    }

    /// Perform a free text lookup.
    #[tracing::instrument(skip_all)]
    pub fn lookup(&self, query: &str) -> Result<Vec<Id>> {
        let mut output = Vec::new();

        if let Some(lookup) = self.index.lookup.get(self.data, query)? {
            tracing::trace!(?lookup);

            for id in self.data.load(lookup)? {
                output.push(*id);
            }
        }

        tracing::trace!(output = output.len());
        Ok(output)
    }

    /// Test if db contains the given string.
    pub fn contains(&self, query: &str) -> Result<bool> {
        Ok(self.index.lookup.contains_key(self.data, query)?)
    }

    /// Perform the given search.
    pub fn search(&self, input: &str) -> Result<Vec<(EntryResultKey, Entry<'a>)>> {
        let mut entries = Vec::new();
        let mut dedup = HashMap::new();

        for id in self.lookup(input)? {
            let entry = self.get(id)?;

            let Some(&i) = dedup.get(&id.index()) else {
                dedup.insert(id.index(), entries.len());

                let data = EntryResultKey {
                    index: id.index().offset(),
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

            let lookup = match self.lookup(it.as_str()) {
                Ok(lookup) => lookup,
                Err(error) => {
                    log::error!("Lookup failed: {error}");
                    continue;
                }
            };

            for id in lookup {
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
