//! Database that can be used as a dictionary.

mod analyze_glossary;

use std::borrow::Cow;
use std::cmp::Reverse;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use anyhow::{anyhow, Result};
use musli::mode::DefaultMode;
use musli::{Decode, Encode};
use musli_storage::int::Variable;
use musli_storage::Encoding;
use musli_zerocopy::{swiss, Buf, OwnedBuf, Ref, ZeroCopy};
use serde::{Deserialize, Serialize};

use crate::inflection::Inflection;
use crate::jmdict::{self, EntryKey};
use crate::kanjidic2;
use crate::romaji::{is_hiragana, is_katakana, Segment};
use crate::PartOfSpeech;
use crate::{inflection, romaji};

/// A deserialized database entry.
pub enum Entry<'a> {
    Kanji(kanjidic2::Character<'a>),
    Dict(jmdict::Entry<'a>),
}

#[derive(ZeroCopy)]
#[repr(C)]
pub(super) struct Index {
    pub(super) lookup: swiss::MapRef<Ref<str>, Ref<[Id]>>,
    pub(super) by_pos: swiss::MapRef<PartOfSpeech, Ref<[u32]>>,
    pub(super) by_sequence: swiss::MapRef<u32, u32>,
}

/// Encoding used for storing database.
const ENCODING: Encoding<DefaultMode, Variable, Variable> = Encoding::new();

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntryResultKey {
    pub index: u32,
    #[serde(flatten)]
    pub key: EntryKey,
    pub sources: BTreeSet<IndexSource>,
}

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
pub enum KanjiReading {
    /// The literal reading.
    Literal,
    Kunyomi,
    KunyomiFull,
    KunyomiRomanize,
    KunyomiKatakana,
    /// This includes the contextual kunyomi part as well.
    KunyomiFullRomanize,
    KunyomiFullKatakana,
    Onyomi,
    OnyomiRomanize,
    OnyomiHiragana,
    Meaning,
    Other,
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
    Word,
    #[serde(rename = "kanji")]
    Kanji { reading: KanjiReading },
    /// Index was added because of a verb inflection.
    #[serde(rename = "verb-c")]
    VerbInflection {
        reading: inflection::Reading,
        inflection: Inflection,
    },
    /// Index was added because of an adjective inflection.
    #[serde(rename = "adj-c")]
    AdjectiveInflection {
        reading: inflection::Reading,
        inflection: Inflection,
    },
}

impl IndexSource {
    /// Test if extra indicates an inflection.
    pub fn is_inflection(&self) -> bool {
        match self {
            IndexSource::VerbInflection { .. } => true,
            IndexSource::AdjectiveInflection { .. } => true,
            _ => false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, ZeroCopy)]
#[repr(C)]
pub struct Id {
    index: u32,
    source: IndexSource,
}

impl Id {
    fn new(index: u32) -> Self {
        Self {
            index,
            source: IndexSource::Word,
        }
    }

    fn kanji_reading(index: u32, reading: KanjiReading) -> Self {
        Self {
            index,
            source: IndexSource::Kanji { reading },
        }
    }

    fn verb_inflection(index: u32, reading: inflection::Reading, inflection: Inflection) -> Self {
        Self {
            index,
            source: IndexSource::VerbInflection {
                reading,
                inflection,
            },
        }
    }

    fn adjective_inflection(
        index: u32,
        reading: inflection::Reading,
        inflection: Inflection,
    ) -> Self {
        Self {
            index,
            source: IndexSource::AdjectiveInflection {
                reading,
                inflection,
            },
        }
    }

    /// Get the unique index this id corresponds to.
    pub fn index(&self) -> u32 {
        self.index
    }

    /// Extra information on index.
    pub fn source(&self) -> IndexSource {
        self.source
    }
}

/// A search result.
pub struct Search<'a> {
    pub entries: Vec<(EntryResultKey, jmdict::Entry<'a>)>,
    pub characters: Vec<kanjidic2::Character<'a>>,
}

/// Load the given dictionary and convert into the internal format.
pub fn load(jmdict: &str, kanjidic2: &str) -> Result<OwnedBuf> {
    let mut buf = OwnedBuf::new();

    let index = buf.store_uninit::<Index>();
    let mut output = Vec::new();

    let mut kanjidic2 = kanjidic2::Parser::new(kanjidic2);
    let mut readings = Vec::new();

    tracing::info!("Parsing kanjidic");

    while let Some(c) = kanjidic2.parse()? {
        output.clear();
        ENCODING.to_writer(&mut output, &c)?;

        let kanji_ref = buf.store_slice(&output).offset() as u32;

        readings.push((
            Cow::Borrowed(c.literal),
            Id::kanji_reading(kanji_ref, KanjiReading::Literal),
        ));

        for reading in &c.reading_meaning.readings {
            match reading.ty {
                "ja_kun" => {
                    if let Some((prefix, _)) = reading.text.split_once('.') {
                        let a = Id::kanji_reading(kanji_ref, KanjiReading::KunyomiRomanize);
                        let b = Id::kanji_reading(kanji_ref, KanjiReading::KunyomiKatakana);
                        other_readings(&mut readings, prefix, a, b, |s| s.katakana());
                        let id = Id::kanji_reading(kanji_ref, KanjiReading::Kunyomi);
                        readings.push((Cow::Borrowed(prefix), id));
                    }

                    let a = Id::kanji_reading(kanji_ref, KanjiReading::KunyomiFullRomanize);
                    let b = Id::kanji_reading(kanji_ref, KanjiReading::KunyomiFullKatakana);
                    other_readings(&mut readings, reading.text, a, b, |s| s.katakana());

                    let id = Id::kanji_reading(kanji_ref, KanjiReading::KunyomiFull);
                    readings.push((Cow::Borrowed(reading.text), id));
                }
                "ja_on" => {
                    let a = Id::kanji_reading(kanji_ref, KanjiReading::OnyomiRomanize);
                    let b = Id::kanji_reading(kanji_ref, KanjiReading::OnyomiHiragana);
                    other_readings(&mut readings, reading.text, a, b, |s| s.hiragana());
                    let id = Id::kanji_reading(kanji_ref, KanjiReading::Onyomi);
                    readings.push((Cow::Borrowed(reading.text), id));
                }
                _ => {
                    let id = Id::kanji_reading(kanji_ref, KanjiReading::Other);
                    readings.push((Cow::Borrowed(reading.text), id));
                }
            };
        }

        for meaning in &c.reading_meaning.meanings {
            let id = Id::kanji_reading(kanji_ref, KanjiReading::Meaning);
            populate_analyzed(meaning.text, &mut readings, id);
        }
    }

    tracing::info!("Parsing JMdict");

    let mut jmdict = jmdict::Parser::new(jmdict);

    let mut by_sequence = HashMap::new();
    let mut by_pos = HashMap::<_, HashSet<_>>::new();

    while let Some(entry) = jmdict.parse()? {
        output.clear();
        ENCODING.to_writer(&mut output, &entry)?;

        let entry_ref = buf.store_slice(&output).offset() as u32;

        by_sequence.insert(entry.sequence as u32, entry_ref);

        for sense in &entry.senses {
            for pos in &sense.pos {
                by_pos.entry(pos).or_default().insert(entry_ref);
            }

            let id = Id::new(entry_ref);

            for g in &sense.gloss {
                if g.ty == Some("expl") {
                    continue;
                }

                populate_analyzed(g.text, &mut readings, id);
            }
        }

        for el in &entry.reading_elements {
            readings.push((Cow::Borrowed(el.text), Id::new(entry_ref)));
        }

        for el in &entry.kanji_elements {
            readings.push((Cow::Borrowed(el.text), Id::new(entry_ref)));
        }

        for (reading, c, kind) in inflection::conjugate(&entry) {
            for (inflection, pair) in c.iter() {
                for word in [pair.text(), pair.reading()] {
                    let key = Cow::Owned(format!("{}{}", word, pair.suffix()));

                    let id = match kind {
                        inflection::Kind::Verb => {
                            Id::verb_inflection(entry_ref, reading, *inflection)
                        }
                        inflection::Kind::Adjective => {
                            Id::adjective_inflection(entry_ref, reading, *inflection)
                        }
                    };

                    readings.push((key, id));
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
                (Ref::with_metadata(*existing, key.len()), true)
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

    buf.load_uninit_mut(index).write(&Index {
        lookup,
        by_pos,
        by_sequence,
    });

    Ok(buf)
}

fn populate_analyzed<'a>(text: &'a str, readings: &mut Vec<(Cow<'a, str>, Id)>, id: Id) {
    fn is_common(phrase: &str) -> bool {
        match phrase {
            "to" | "a" | "in" | "of" | "for" | "so" | "if" | "by" | "but" | "not" | "any"
            | "way" | "into" => true,
            string if string.starts_with("e.g.") => {
                if let Some(rest) = string.strip_prefix("e.g. ") {
                    is_common(rest)
                } else {
                    string == "e.g."
                }
            }
            _ => false,
        }
    }

    for phrase in analyze_glossary::analyze(text) {
        if phrase.chars().count() > 32 {
            continue;
        }

        // Skip overly common prefixes which would mostly just be
        // unhelpful to index.
        if !is_common(phrase) {
            readings.push((Cow::Borrowed(phrase), id));
        }
    }
}

fn other_readings(
    output: &mut Vec<(Cow<'_, str>, Id)>,
    text: &str,
    a: Id,
    b: Id,
    f: for<'a> fn(&'a Segment<'_>) -> &'a str,
) {
    let mut romanized = String::new();
    let mut other = String::new();

    for part in romaji::analyze(text) {
        romanized += part.romanize();
        other += f(&part);
    }

    output.push((Cow::Owned(romanized), a));
    output.push((Cow::Owned(other), b));
}

#[derive(Clone)]
pub struct Database<'a> {
    index: &'a Index,
    data: &'a Buf,
}

impl<'a> Database<'a> {
    /// Construct a new database wrapper.
    pub fn new(data: &'a [u8]) -> Result<Self> {
        let data = Buf::new(data);
        let index = data.load(Ref::<Index>::zero())?;

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
        let Some(bytes) = self.data.get(id.index() as usize..) else {
            return Err(anyhow!("Missing entry at {}", id.index()));
        };

        Ok(match id.source {
            IndexSource::Kanji { .. } => Entry::Kanji(ENCODING.from_slice(bytes)?),
            _ => Entry::Dict(ENCODING.from_slice(bytes)?),
        })
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
    pub fn search(&self, input: &str) -> Result<Search<'a>> {
        let mut entries = Vec::new();
        let mut characters = Vec::new();
        let mut dedup = HashMap::new();
        let mut seen = HashSet::new();

        self.populate_kanji(input, &mut seen, &mut characters)?;

        for id in self.lookup(input)? {
            let entry = match self.get(id)? {
                Entry::Kanji(kanji) => {
                    if seen.insert(kanji.literal) {
                        characters.push(kanji);
                    }

                    continue;
                }
                Entry::Dict(entry) => entry,
            };

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

        entries.sort_by(|a, b| a.0.key.cmp(&b.0.key));

        for (_, entry) in &entries {
            for kanji in &entry.kanji_elements {
                self.populate_kanji(kanji.text, &mut seen, &mut characters)?;
            }
        }

        Ok(Search {
            entries,
            characters,
        })
    }

    fn populate_kanji(
        &self,
        input: &str,
        seen: &mut HashSet<&'a str>,
        out: &mut Vec<kanjidic2::Character<'a>>,
    ) -> Result<(), anyhow::Error> {
        for c in input.chars() {
            if is_katakana(c) || is_hiragana(c) || c.is_ascii_alphabetic() {
                continue;
            }

            for id in self.lookup(c.encode_utf8(&mut [0; 4]))? {
                if !matches!(
                    id.source,
                    IndexSource::Kanji {
                        reading: KanjiReading::Literal
                    }
                ) {
                    continue;
                }

                if let Entry::Kanji(kanji) = self.get(id)? {
                    if seen.insert(kanji.literal) {
                        out.push(kanji);
                    }
                }
            }
        }

        Ok(())
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
                let Ok(Entry::Dict(e)) = self.get(id) else {
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
