//! Database that can be used as a dictionary.

mod analyze_glossary;
mod string_indexer;

use std::borrow::Cow;
use std::collections::{hash_map, BTreeMap, BTreeSet, HashMap, HashSet};

use anyhow::{anyhow, Result};
use musli::mode::DefaultMode;
use musli::{Decode, Encode};
use musli_storage::int::Variable;
use musli_storage::Encoding;
use musli_zerocopy::endian::Little;
use musli_zerocopy::{slice, swiss, trie, Buf, OwnedBuf, Ref, ZeroCopy};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::inflection::Inflection;
use crate::jmdict::{self, EntryKey};
use crate::romaji;
use crate::romaji::{is_hiragana, is_katakana, Segment};
use crate::PartOfSpeech;
use crate::{inflection, DICTIONARY_VERSION};
use crate::{kanjidic2, DICTIONARY_MAGIC};

use self::string_indexer::StringIndexer;

pub(super) struct CompactTrie;

impl trie::Flavor for CompactTrie {
    type String = slice::Packed<[u8], u32, u8>;
    type Values<T> = slice::Packed<[T], u32, u16> where T: ZeroCopy;
    type Children<T> = slice::Packed<[T], u32, u16> where T: ZeroCopy;
}

/// A deserialized database entry.
pub enum Entry<'a> {
    Kanji(kanjidic2::Character<'a>),
    Dict(jmdict::Entry<'a>),
}

#[derive(ZeroCopy)]
#[repr(C)]
struct Header {
    magic: u32,
    version: u32,
    index: Ref<Index, Little>,
}

#[derive(ZeroCopy)]
#[repr(C)]
pub(super) struct Index {
    pub(super) lookup: trie::TrieRef<Id, CompactTrie>,
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

/// Build a dictionary from the given jmdict and kanjidic sources.
pub fn build(jmdict: &str, kanjidic2: &str) -> Result<OwnedBuf> {
    let mut buf = OwnedBuf::new();

    let header = buf.store_uninit::<Header>();
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
            if let Some(s) = full_to_half_string(el.text) {
                readings.push((Cow::Owned(s), Id::new(entry_ref)));
            }

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

    readings.sort_by(|(a, _), (b, _)| b.as_ref().cmp(a.as_ref()));
    tracing::info!("Inserting {} readings", readings.len());

    let mut max = 0usize;

    let mut indexer = StringIndexer::new();

    let mut readings2 = Vec::with_capacity(readings.len());

    for (key, id) in &readings {
        if indexer.total() % 100000 == 0 {
            tracing::info!("Building strings: {}: {key:?}", indexer.total());
        }

        max = max.max(key.as_ref().len());
        let s = indexer.store(&mut buf, key.as_ref())?;
        readings2.push((s, *id));
    }

    let mut lookup = trie::Builder::with_flavor();

    for (key, id) in readings2.into_iter().rev() {
        lookup.insert(&buf, key, id)?;
    }

    tracing::info!(
        "Reused {} string(s) (out of {})",
        indexer.reuse(),
        indexer.total()
    );

    tracing::info!("Serializing to zerocopy structure");

    let lookup = lookup.build(&mut buf)?;

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

    buf.load_uninit_mut(header).write(&Header {
        magic: DICTIONARY_MAGIC,
        version: DICTIONARY_VERSION,
        index: index.assume_init(),
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
        if phrase.chars().count() > 24 {
            continue;
        }

        // Skip overly common prefixes which would mostly just be
        // unhelpful to index.
        if !is_common(phrase) {
            readings.push((Cow::Borrowed(phrase), id));
        }
    }
}

fn full_to_half_char(c: char) -> Option<char> {
    let c = match c {
        '\u{FF01}' => '!',
        '\u{FF02}' => '"',
        '\u{FF03}' => '#',
        '\u{FF04}' => '$',
        '\u{FF05}' => '%',
        '\u{FF06}' => '&',
        '\u{FF07}' => '\'',
        '\u{FF08}' => '(',
        '\u{FF09}' => ')',
        '\u{FF0A}' => '*',
        '\u{FF0B}' => '+',
        '\u{FF0C}' => ',',
        '\u{FF0D}' => '-',
        '\u{FF0E}' => '.',
        '\u{FF0F}' => '/',
        '\u{FF10}'..='\u{FF19}' => ((c as u32 - 0xFF10) as u8 + b'0') as char,
        '\u{FF1A}' => ':',
        '\u{FF1B}' => ';',
        '\u{FF1C}' => '<',
        '\u{FF1D}' => '=',
        '\u{FF1E}' => '>',
        '\u{FF1F}' => '?',
        '\u{FF20}' => '@',
        '\u{FF21}'..='\u{FF3A}' => ((c as u32 - 0xFF21) as u8 + b'A') as char,
        '\u{FF3B}' => '[',
        '\u{FF3C}' => '\\',
        '\u{FF3D}' => ']',
        '\u{FF3E}' => '^',
        '\u{FF3F}' => '_',
        '\u{FF40}' => '`',
        '\u{FF41}'..='\u{FF5A}' => ((c as u32 - 0xFF41) as u8 + b'a') as char,
        '\u{FF5B}' => '{',
        '\u{FF5C}' => '|',
        '\u{FF5D}' => '}',
        '\u{FF5E}' => '~',
        '\u{FF5F}' => '\u{2985}',
        '\u{FF60}' => '\u{2986}',
        '\u{FFE0}' => '\u{00A2}',
        '\u{FFE1}' => '\u{00A3}',
        '\u{FFE2}' => '\u{00AC}',
        '\u{FFE3}' => '\u{00AF}',
        '\u{FFE4}' => '\u{00A6}',
        '\u{FFE5}' => '\u{00A5}',
        '\u{FFE6}' => '\u{20A9}',
        _ => {
            return None;
        }
    };

    Some(c)
}

fn full_to_half_string(input: &str) -> Option<String> {
    let mut output = String::new();

    let mut it = input.char_indices();

    'escape: {
        for (index, c) in it.by_ref() {
            let Some(c) = full_to_half_char(c) else {
                continue;
            };

            output.push_str(&input[..index]);
            output.push(c);
            break 'escape;
        }

        return None;
    }

    for (_, c) in it {
        output.push(full_to_half_char(c).unwrap_or(c));
    }

    Some(output)
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

/// An error raised while interacting with the database.
#[derive(Debug, Error)]
pub enum DatabaseOpenError {
    #[error("Not valid due to magic mismatch")]
    MagicMismatch,
    #[error("Outdated")]
    Outdated,
    #[error("Error: {0}")]
    Error(
        #[from]
        #[source]
        musli_zerocopy::Error,
    ),
}

impl<'a> Database<'a> {
    /// Construct a new database wrapper.
    ///
    /// This returns `Ok(None)` if the database is incompatible with the current
    /// version.
    pub fn open(data: &'a [u8]) -> Result<Self, DatabaseOpenError> {
        let data = Buf::new(data);
        let header = data.load(Ref::<Header>::zero())?;

        if header.magic != DICTIONARY_MAGIC {
            return Err(DatabaseOpenError::MagicMismatch);
        }

        if header.version != DICTIONARY_VERSION {
            return Err(DatabaseOpenError::Outdated);
        }

        let index = data.load(header.index)?;
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

            for id in self.data.load(*by_pos)? {
                output.push(Id::new(*id));
            }
        }

        tracing::trace!(output = output.len());
        Ok(output)
    }

    /// Lookup all entries matching the given prefix.
    #[tracing::instrument(skip_all)]
    pub fn prefix(&self, prefix: &str) -> Result<Vec<Id>> {
        let mut output = Vec::new();

        for id in self.index.lookup.values_in(self.data, prefix) {
            output.push(*id?);
        }

        Ok(output)
    }

    /// Perform a free text lookup.
    #[tracing::instrument(skip_all)]
    pub fn lookup(&self, query: &str) -> Result<Vec<Id>> {
        let mut output = Vec::new();

        match query.split_once(|c: char| matches!(c, '*' | '＊')) {
            Some((prefix, suffix)) if !prefix.is_empty() => {
                if suffix.is_empty() {
                    for id in self.index.lookup.values_in(self.data, prefix) {
                        output.push(*id?);
                    }
                } else {
                    for id in self.index.lookup.iter_in(self.data, prefix) {
                        let (string, id) = id?;

                        let Some(s) = string.strip_prefix(prefix.as_bytes()) else {
                            continue;
                        };

                        if !s.ends_with(suffix.as_bytes()) {
                            continue;
                        }

                        output.push(*id);
                    }
                }
            }
            _ => {
                if let Some(lookup) = self.index.lookup.get(self.data, query)? {
                    for id in lookup {
                        output.push(*id);
                    }
                }
            }
        }

        tracing::trace!(output = output.len());
        Ok(output)
    }

    /// Test if db contains the given string.
    pub fn contains(&self, query: &str) -> Result<bool> {
        Ok(self.index.lookup.get(self.data, query)?.is_some())
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
    ) -> Result<()> {
        for c in input.chars() {
            if is_katakana(c) || is_hiragana(c) || c.is_ascii_alphabetic() {
                continue;
            }

            let Some(lookup) = self
                .index
                .lookup
                .get(self.data, c.encode_utf8(&mut [0; 4]))?
            else {
                continue;
            };

            for id in lookup {
                if !matches!(
                    id.source,
                    IndexSource::Kanji {
                        reading: KanjiReading::Literal
                    }
                ) {
                    continue;
                }

                let Entry::Kanji(kanji) = self.get(*id)? else {
                    continue;
                };

                if seen.insert(kanji.literal) {
                    out.push(kanji);
                }
            }
        }

        Ok(())
    }

    /// Analyze the given string, looking it up in the database and returning
    /// all prefix matching entries and their texts.
    pub fn analyze(&self, q: &str, start: usize) -> Result<BTreeMap<EntryKey, String>> {
        let Some(suffix) = q.get(start..) else {
            return Ok(BTreeMap::new());
        };

        let mut results = HashMap::<_, EntryKey>::new();

        let mut it = suffix.chars();

        while !it.as_str().is_empty() {
            if let Some(values) = self.index.lookup.get(self.data, it.as_str())? {
                for id in values {
                    let Entry::Dict(e) = self.get(*id)? else {
                        continue;
                    };

                    let key = e.sort_key(it.as_str(), id.source().is_inflection());

                    match results.entry(it.as_str()) {
                        hash_map::Entry::Occupied(mut e) => {
                            e.insert((*e.get()).max(key));
                        }
                        hash_map::Entry::Vacant(e) => {
                            e.insert(key);
                        }
                    }
                }
            }

            it.next_back();
        }

        let mut inputs = BTreeMap::new();

        for (string, key) in results {
            inputs.insert(key, string.to_owned());
        }

        Ok(inputs)
    }
}
