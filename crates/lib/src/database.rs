//! Database that can be used as a dictionary.

mod index;

use std::borrow::Cow;
use std::cmp::Reverse;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use anyhow::{anyhow, Result};
use musli::mode::DefaultMode;
use musli::{Decode, Encode};
use musli_storage::int::Variable;
use musli_storage::Encoding;
use musli_zerocopy::pointer::{Ref, Unsized};
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
    pub index: u32,
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
    index: u32,
    extra: IndexSource,
}

impl Id {
    fn new(index: u32) -> Self {
        Self {
            index: index,
            extra: IndexSource::None,
        }
    }

    fn verb_inflection(index: u32, reading: verb::Reading, inflection: Inflection) -> Self {
        Self {
            index,
            extra: IndexSource::VerbInflection {
                reading,
                inflection,
            },
        }
    }

    fn adjective_inflection(index: u32, inflection: Inflection) -> Self {
        Self {
            index,
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
pub fn load(dict: &str) -> Result<AlignedBuf> {
    let mut buf = AlignedBuf::new();

    let index = buf.store_uninit::<index::Index>();

    let mut parser = Parser::new(dict);
    let mut output = Vec::new();
    let mut readings = Vec::new();

    let mut by_sequence = HashMap::new();
    let mut by_pos = HashMap::<_, HashSet<_>>::new();

    while let Some(entry) = parser.parse()? {
        output.clear();
        ENCODING.to_writer(&mut output, &entry)?;

        let entry_ref = buf.store_slice(&output).offset() as u32;

        by_sequence.insert(entry.sequence as u32, entry_ref);

        for sense in &entry.senses {
            for pos in &sense.pos {
                by_pos.entry(pos).or_default().insert(entry_ref);
            }

            for g in &sense.gloss {
                for word in g.text.split_whitespace() {
                    let word = word.trim_matches(|c| !char::is_alphabetic(c));

                    if word.is_empty() {
                        continue;
                    }

                    readings.push((Cow::Borrowed(word), Id::new(entry_ref)));
                }
            }
        }

        for el in &entry.reading_elements {
            readings.push((Cow::Borrowed(el.text), Id::new(entry_ref)));
        }

        for el in &entry.kanji_elements {
            readings.push((Cow::Borrowed(el.text), Id::new(entry_ref)));
        }

        for (reading, c) in verb::conjugate(&entry) {
            for (inflection, pair) in c.iter() {
                for word in [pair.text(), pair.reading()] {
                    let key = Cow::Owned(format!("{}{}", word, pair.suffix()));
                    readings.push((key, Id::verb_inflection(entry_ref, reading, *inflection)));
                }
            }
        }

        if let Some(c) = adjective::conjugate(&entry) {
            for (inflection, pair) in c.iter() {
                for word in [pair.text(), pair.reading()] {
                    let key = Cow::Owned(format!("{}{}", word, pair.suffix()));
                    readings.push((key, Id::adjective_inflection(entry_ref, *inflection)));
                }
            }
        }
    }

    tracing::info!("Sorting readings");
    readings.sort_by_cached_key(|a| (Reverse(a.0.chars().count()), a.0.clone()));

    let mut lookup = HashMap::<_, Vec<_>>::new();

    tracing::info!("Inserting {} readings", readings.len());

    {
        let mut existing = HashMap::<_, usize>::new();
        let mut reuse = 0usize;
        let mut total = 0usize;

        for (index, (key, id)) in readings.into_iter().enumerate() {
            if index % 100000 == 0 {
                tracing::info!("Building strings: {}: {key}", index);
            }

            total += 1;

            let (unsize, substring) = if let Some(existing) = existing.get(key.as_ref()) {
                reuse += 1;
                (Unsized::new(*existing, key.len()), true)
            } else {
                let unsize = buf.store_unsized(key.as_ref());
                (unsize, false)
            };

            lookup.entry(unsize).or_default().push(id);

            if !substring {
                for (n, _) in key.char_indices() {
                    let mut s = String::new();

                    for c in key[n..].chars() {
                        s.push(c);

                        if !existing.contains_key(&s) {
                            existing.insert(s.clone(), unsize.offset() + n);
                        }
                    }
                }

                existing.insert(key.into_owned(), unsize.offset());
            }
        }

        tracing::info!("Reused {} string(s) (out of {})", reuse, total);
    }

    tracing::info!("Serializing to zerocopy structure");

    let lookup = {
        let mut entries = Vec::new();

        for (index, (key, set)) in lookup.into_iter().enumerate() {
            if index % 100000 == 0 {
                tracing::info!("Building lookup: {}", index);
            }

            let slice = buf.store_slice(&set);

            entries.push((key, slice));
        }

        tracing::info!("Storing lookup {}:...", entries.len());
        swiss::store_map(&mut buf, entries)?
    };

    let by_pos = {
        let mut entries = Vec::new();

        for (index, (key, set)) in by_pos.into_iter().enumerate() {
            if index % 10000 == 0 {
                tracing::info!("{}", index);
            }

            let mut values = Vec::new();

            for v in set {
                values.push(v);
            }

            values.sort();
            let set = buf.store_slice(&values);
            entries.push((key, set));
        }

        tracing::info!("Storing by_pos: {}...", entries.len());
        swiss::store_map(&mut buf, entries)?
    };

    let by_sequence = {
        tracing::info!("Storing by_sequence: {}...", by_sequence.len());
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
    pub fn lookup_sequence(&self, sequence: u32) -> Result<Option<Id>> {
        let Some(index) = self.index.by_sequence.get(self.data, &sequence)? else {
            return Ok(None);
        };

        Ok(Some(Id::new(*index)))
    }

    /// Get an entry from the database.
    pub fn get(&self, id: Id) -> Result<Entry<'a>> {
        let Some(bytes) = self.data.as_slice().get(id.index() as usize..) else {
            return Err(anyhow!("Missing entry at {}", id.index()));
        };

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
