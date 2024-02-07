//! Database that can be used as a dictionary.

mod analyze_glossary;
mod string_indexer;

use std::borrow::Cow;
use std::collections::{hash_map, BTreeMap, BTreeSet, HashMap, HashSet};
use std::fmt;
use std::path::Path;
use std::sync::Arc;

use anyhow::{anyhow, ensure, Context, Result};
use fixed_map::Set;
use musli::mode::DefaultMode;
use musli::{Decode, Encode};
use musli_storage::int::Variable;
use musli_storage::Encoding;
use musli_zerocopy::endian::Little;
use musli_zerocopy::{slice, swiss, trie, OwnedBuf, Ref, ZeroCopy};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::config::Config;
use crate::data::Data;
use crate::inflection::{self, Inflection};
use crate::jmdict;
use crate::jmnedict;
use crate::kanjidic2;
use crate::reporter::Reporter;
use crate::romaji::{self, is_hiragana, is_katakana, Segment};
use crate::token::Token;
use crate::{PartOfSpeech, Weight};
use crate::{DICTIONARY_MAGIC, DICTIONARY_VERSION};

use self::string_indexer::StringIndexer;

/// Encoding used for storing database.
const ENCODING: Encoding<DefaultMode, Variable, Variable> = Encoding::new();

/// An error raised while interacting with the database.
#[derive(Debug, Error)]
pub enum IndexOpenError {
    #[error("Not valid due to magic mismatch")]
    MagicMismatch,
    #[error("Outdated")]
    Outdated,
    #[error("{0}")]
    Error(
        #[from]
        #[source]
        musli_zerocopy::Error,
    ),
}

/// Used for diagnostics to indicate where a dictionary was loaded from.
#[non_exhaustive]
pub enum Location {
    /// The dictionary was loaded from the given path.
    Path(Box<Path>),
    /// The dictionary was loaded from memory.
    #[allow(unused)]
    Memory(usize),
}

impl fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Location::Path(path) => path.display().fmt(f),
            Location::Memory(address) => write!(f, "<memory at {address:08x}>"),
        }
    }
}

pub struct CompactTrie;

impl trie::Flavor for CompactTrie {
    type String = slice::Packed<[u8], u32, u8>;
    type Values<T> = slice::Packed<[T], u32, u16> where T: ZeroCopy;
    type Children<T> = slice::Packed<[T], u32, u16> where T: ZeroCopy;
}

/// A deserialized database entry.
#[derive(Serialize)]
#[serde(tag = "type")]
pub enum Entry<'a> {
    #[serde(rename = "phrase")]
    Phrase(jmdict::Entry<'a>),
    #[serde(rename = "kanji")]
    Kanji(kanjidic2::Character<'a>),
    #[serde(rename = "name")]
    Name(jmnedict::Entry<'a>),
}

#[derive(ZeroCopy)]
#[repr(C)]
struct Header {
    magic: u32,
    version: u32,
    index: Ref<IndexHeader, Little>,
}

#[derive(Clone, Copy, ZeroCopy)]
#[repr(C)]
pub struct IndexHeader {
    pub name: Ref<str>,
    pub lookup: trie::TrieRef<Id, CompactTrie>,
    pub by_pos: swiss::MapRef<PartOfSpeech, Ref<[u32]>>,
    pub by_kanji_literal: swiss::MapRef<Ref<str>, u32>,
    pub by_sequence: swiss::MapRef<u32, u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntryResultKey {
    pub index: usize,
    pub offset: u32,
    pub sources: BTreeSet<IndexSource>,
    pub weight: Weight,
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
    #[serde(rename = "kanji")]
    Kanji { reading: KanjiReading },
    /// No extra information on why the index was added.
    #[serde(rename = "phrase")]
    Phrase,
    /// Index was added because of an inflection.
    #[serde(rename = "inflection")]
    Inflection {
        reading: inflection::Reading,
        inflection: Inflection,
    },
    #[serde(rename = "kanji")]
    Name { reading: KanjiReading },
}

impl IndexSource {
    /// Test if extra indicates an inflection.
    pub fn is_inflection(&self) -> bool {
        match self {
            IndexSource::Inflection { .. } => true,
            _ => false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, ZeroCopy)]
#[repr(C)]
pub struct Id {
    offset: u32,
    source: IndexSource,
}

impl Id {
    fn phrase(offset: u32) -> Self {
        Self {
            offset,
            source: IndexSource::Phrase,
        }
    }

    fn name(offset: u32, reading: KanjiReading) -> Self {
        Self {
            offset,
            source: IndexSource::Name { reading },
        }
    }

    fn kanji(offset: u32, reading: KanjiReading) -> Self {
        Self {
            offset,
            source: IndexSource::Kanji { reading },
        }
    }

    fn inflection(offset: u32, reading: inflection::Reading, inflection: Inflection) -> Self {
        Self {
            offset,
            source: IndexSource::Inflection {
                reading,
                inflection,
            },
        }
    }

    /// Get the unique index this id corresponds to.
    pub fn offset(&self) -> u32 {
        self.offset
    }

    /// Extra information on index.
    pub fn source(&self) -> IndexSource {
        self.source
    }
}

/// Input to build the database.
pub enum Input<'a> {
    Jmdict(&'a str),
    Kanjidic2(&'a str),
    Jmnedict(&'a str),
}

impl Input<'_> {
    fn name(&self) -> &'static str {
        match self {
            Input::Jmdict(..) => "JMdict",
            Input::Kanjidic2(..) => "Kanjidic2",
            Input::Jmnedict(..) => "JMnedict",
        }
    }
}

/// A search result.
pub struct Search<'a> {
    pub phrases: Vec<(EntryResultKey, jmdict::Entry<'a>)>,
    pub names: Vec<(EntryResultKey, jmnedict::Entry<'a>)>,
    pub characters: Vec<kanjidic2::Character<'a>>,
}

/// Build a dictionary from the given jmdict and kanjidic sources.
pub fn build(
    reporter: &dyn Reporter,
    shutdown: &Token,
    name: &str,
    input: Input<'_>,
) -> Result<OwnedBuf> {
    let mut buf = OwnedBuf::new();

    let header = buf.store_uninit::<Header>();
    let index = buf.store_uninit::<IndexHeader>();

    let name = buf.store_unsized(name);

    let mut output = Vec::new();
    let mut lookup = Vec::new();

    let mut by_sequence = HashMap::new();
    let mut by_pos = HashMap::<_, HashSet<_>>::new();
    let mut kanji_literals = HashMap::new();

    reporter.instrument_start(
        module_path!(),
        &format_args!("Processing dictionary `{}`", input.name()),
        None,
    );

    let mut count = 0;

    match input {
        Input::Jmdict(input) => {
            let mut jmdict = jmdict::Parser::new(input);

            while let Some(entry) = jmdict.parse()? {
                ensure!(!shutdown.is_set(), "Task shut down");

                if count % 1000 == 0 {
                    reporter.instrument_progress(1000);
                }

                count += 1;

                output.clear();
                ENCODING.to_writer(&mut output, &entry)?;

                let entry_ref = buf.store_slice(&output).offset() as u32;
                by_sequence.insert(entry.sequence as u32, entry_ref);

                for sense in &entry.senses {
                    for pos in &sense.pos {
                        by_pos.entry(pos).or_default().insert(entry_ref);
                    }

                    let id = Id::phrase(entry_ref);

                    for g in &sense.gloss {
                        if g.ty == Some("expl") {
                            continue;
                        }

                        populate_analyzed(g.text, &mut lookup, id);
                    }
                }

                for el in &entry.reading_elements {
                    lookup.push((Cow::Borrowed(el.text), Id::phrase(entry_ref)));
                }

                for el in &entry.kanji_elements {
                    if let Some(s) = full_to_half_string(el.text) {
                        lookup.push((Cow::Owned(s), Id::phrase(entry_ref)));
                    }

                    lookup.push((Cow::Borrowed(el.text), Id::phrase(entry_ref)));
                }

                for (reading, c, kind) in inflection::conjugate(&entry) {
                    for (inflection, pair) in c.iter() {
                        for word in [pair.text(), pair.reading()] {
                            let key = Cow::Owned(format!("{}{}", word, pair.suffix()));

                            let id = match kind {
                                inflection::Kind::Verb => {
                                    Id::inflection(entry_ref, reading, *inflection)
                                }
                                inflection::Kind::Adjective => {
                                    Id::inflection(entry_ref, reading, *inflection)
                                }
                            };

                            lookup.push((key, id));
                        }
                    }
                }
            }
        }
        Input::Kanjidic2(input) => {
            let mut kanjidic2 = kanjidic2::Parser::new(input);

            while let Some(c) = kanjidic2.parse()? {
                ensure!(!shutdown.is_set(), "Task shut down");

                if count % 1000 == 0 {
                    reporter.instrument_progress(1000);
                }

                count += 1;

                output.clear();
                ENCODING.to_writer(&mut output, &c)?;

                let kanji_ref = buf.store_slice(&output).offset() as u32;

                kanji_literals.insert(c.literal, kanji_ref);

                lookup.push((
                    Cow::Borrowed(c.literal),
                    Id::kanji(kanji_ref, KanjiReading::Literal),
                ));

                for reading in &c.reading_meaning.readings {
                    match reading.ty {
                        "ja_kun" => {
                            if let Some((prefix, _)) = reading.text.split_once('.') {
                                let a = Id::kanji(kanji_ref, KanjiReading::KunyomiRomanize);
                                let b = Id::kanji(kanji_ref, KanjiReading::KunyomiKatakana);
                                other_readings(&mut lookup, prefix, a, b, |s| s.katakana());
                                let id = Id::kanji(kanji_ref, KanjiReading::Kunyomi);
                                lookup.push((Cow::Borrowed(prefix), id));
                            }

                            let a = Id::kanji(kanji_ref, KanjiReading::KunyomiFullRomanize);
                            let b = Id::kanji(kanji_ref, KanjiReading::KunyomiFullKatakana);
                            other_readings(&mut lookup, reading.text, a, b, |s| s.katakana());

                            let id = Id::kanji(kanji_ref, KanjiReading::KunyomiFull);
                            lookup.push((Cow::Borrowed(reading.text), id));
                        }
                        "ja_on" => {
                            let a = Id::kanji(kanji_ref, KanjiReading::OnyomiRomanize);
                            let b = Id::kanji(kanji_ref, KanjiReading::OnyomiHiragana);
                            other_readings(&mut lookup, reading.text, a, b, |s| s.hiragana());
                            let id = Id::kanji(kanji_ref, KanjiReading::Onyomi);
                            lookup.push((Cow::Borrowed(reading.text), id));
                        }
                        _ => {
                            let id = Id::kanji(kanji_ref, KanjiReading::Other);
                            lookup.push((Cow::Borrowed(reading.text), id));
                        }
                    };
                }

                for meaning in &c.reading_meaning.meanings {
                    let id = Id::kanji(kanji_ref, KanjiReading::Meaning);
                    populate_analyzed(meaning.text, &mut lookup, id);
                }
            }
        }
        Input::Jmnedict(input) => {
            let mut jmnedict = jmnedict::Parser::new(input);

            while let Some(entry) = jmnedict.next()? {
                ensure!(!shutdown.is_set(), "Task shut down");

                if count % 1000 == 0 {
                    reporter.instrument_progress(1000);
                }

                count += 1;

                output.clear();
                ENCODING.to_writer(&mut output, &entry)?;

                let name_ref = buf.store_slice(&output).offset() as u32;

                for kanji in entry.kanji.iter().copied() {
                    lookup.push((
                        Cow::Borrowed(kanji),
                        Id::name(name_ref, KanjiReading::Literal),
                    ));
                }

                for reading in entry.reading {
                    lookup.push((
                        Cow::Borrowed(reading.text),
                        Id::name(name_ref, KanjiReading::KunyomiFull),
                    ));
                    let a = Id::name(name_ref, KanjiReading::KunyomiFullRomanize);
                    let b = Id::name(name_ref, KanjiReading::KunyomiFullKatakana);
                    other_readings(&mut lookup, reading.text, a, b, |s| s.katakana());
                }
            }
        }
    }

    reporter.instrument_end(count);

    lookup.sort_by(|(a, _), (b, _)| b.as_ref().cmp(a.as_ref()));
    report_info!(reporter, "Inserting {} readings", lookup.len());

    let mut readings2 = Vec::with_capacity(lookup.len());
    let by_kanji_literal;

    {
        let mut indexer = StringIndexer::new();

        reporter.instrument_start(module_path!(), &"Inserting strings", Some(lookup.len()));

        for (index, (key, id)) in lookup.iter().enumerate() {
            ensure!(!shutdown.is_set(), "Task shut down");

            if index % 100_000 == 0 {
                reporter.instrument_progress(100_000);
            }

            let s = indexer.store(&mut buf, key.as_ref())?;
            readings2.push((s, *id));
        }

        reporter.instrument_end(lookup.len());

        by_kanji_literal = {
            let mut output = HashMap::new();

            for (key, value) in kanji_literals {
                let s = indexer.store(&mut buf, key.as_ref())?;
                output.insert(s, value);
            }

            output
        };

        report_info!(
            reporter,
            "Reused {} string(s) (out of {})",
            indexer.reuse(),
            indexer.total()
        );
    }

    drop(lookup);

    let step_len = readings2.len();

    reporter.instrument_start(module_path!(), &"Building lookup table", Some(step_len));

    let mut lookup = trie::Builder::with_flavor();

    for (index, (key, id)) in readings2.into_iter().rev().enumerate() {
        if index % 100000 == 0 {
            reporter.instrument_progress(100000);
        }

        ensure!(!shutdown.is_set(), "Task shut down");
        lookup.insert(&buf, key, id)?;
    }

    reporter.instrument_end(step_len);

    reporter.instrument_start(module_path!(), &"Saving index", None);

    let lookup = lookup.build(&mut buf)?;

    let by_pos = {
        let mut entries = Vec::new();

        for (key, set) in by_pos.into_iter() {
            ensure!(!shutdown.is_set(), "Task shut down");

            let mut values = Vec::new();

            for v in set {
                values.push(v);
            }

            values.sort();
            let set = buf.store_slice(&values);
            entries.push((key, set));
        }

        report_info!(reporter, "Storing by_pos: {}...", entries.len());
        swiss::store_map(&mut buf, entries)?
    };

    let by_kanji_literal = {
        report_info!(
            reporter,
            "Storing by_kanji_literal: {}...",
            by_kanji_literal.len()
        );
        swiss::store_map(&mut buf, by_kanji_literal)?
    };

    let by_sequence = {
        report_info!(reporter, "Storing by_sequence: {}...", by_sequence.len());
        swiss::store_map(&mut buf, by_sequence)?
    };

    buf.load_uninit_mut(index).write(&IndexHeader {
        name,
        lookup,
        by_pos,
        by_kanji_literal,
        by_sequence,
    });

    buf.load_uninit_mut(header).write(&Header {
        magic: DICTIONARY_MAGIC,
        version: DICTIONARY_VERSION,
        index: index.assume_init(),
    });

    reporter.instrument_end(0);
    Ok(buf)
}

fn populate_analyzed<'a>(text: &'a str, lookup: &mut Vec<(Cow<'a, str>, Id)>, id: Id) {
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
            lookup.push((Cow::Borrowed(phrase), id));
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

// A loaded index.
pub struct Index {
    header: IndexHeader,
    data: Data,
}

impl Index {
    /// Construct a new database wrapper.
    ///
    /// This returns `Ok(None)` if the database is incompatible with the current
    /// version.
    pub fn open(data: Data) -> Result<Self, IndexOpenError> {
        let buf = data.as_buf();
        let header = buf.load(Ref::<Header>::zero())?;

        if header.magic != DICTIONARY_MAGIC {
            return Err(IndexOpenError::MagicMismatch);
        }

        if header.version != DICTIONARY_VERSION {
            return Err(IndexOpenError::Outdated);
        }

        let header = *buf.load(header.index)?;
        Ok(Self { header, data })
    }

    /// Load the name of the index.
    pub fn name(&self) -> Result<&str> {
        Ok(self.data.as_buf().load(self.header.name)?)
    }

    /// Get an entry from the database.
    pub fn entry_at(&self, id: Id) -> Result<Entry<'_>> {
        let Some(bytes) = self.data.as_buf().get(id.offset as usize..) else {
            return Err(anyhow!("Missing entry at {}", id.offset));
        };

        Ok(match id.source {
            IndexSource::Kanji { .. } => Entry::Kanji(ENCODING.from_slice(bytes)?),
            IndexSource::Name { .. } => Entry::Name(ENCODING.from_slice(bytes)?),
            _ => Entry::Phrase(ENCODING.from_slice(bytes)?),
        })
    }
}

#[derive(Clone)]
pub struct Database {
    indexes: Arc<[Index]>,
    disabled: Arc<[String]>,
}

impl Database {
    /// Open a sequence of indexes.
    pub fn open<I>(iter: I, config: &Config) -> Result<Self>
    where
        I: IntoIterator<Item = (Data, Location)>,
    {
        let mut indexes = Vec::new();
        let mut disabled = Vec::new();

        for (data, location) in iter {
            let index = match Index::open(data) {
                Ok(index) => index,
                Err(error) => {
                    log::error!("Failed to load index from {location}");
                    log::error!("Caused by: {}", error);
                    continue;
                }
            };

            if !config.is_enabled(index.name()?) {
                disabled.push(index.name()?.to_owned());
                continue;
            }

            indexes.push(index);
        }

        Ok(Self {
            indexes: indexes.into(),
            disabled: disabled.into(),
        })
    }

    /// Get the identifiers of all installed indexes.
    pub fn installed(&self) -> Result<HashSet<String>> {
        let mut output = HashSet::with_capacity(self.indexes.len());

        for index in self.indexes.iter() {
            output.insert(index.data.as_buf().load(index.header.name)?.to_owned());
        }

        output.extend(self.disabled.iter().cloned());
        Ok(output)
    }

    /// Convert a sequence to Id.
    pub fn sequence_to_id(&self, sequence: u32) -> Result<Vec<(usize, Id)>> {
        let mut output = Vec::new();

        for (index, d) in self.indexes.iter().enumerate() {
            let Some(offset) = d.header.by_sequence.get(d.data.as_buf(), &sequence)? else {
                continue;
            };

            output.push((index, Id::phrase(*offset)));
        }

        Ok(output)
    }

    /// Get all entries matching the given id.
    pub fn entry_at(&self, index: usize, id: Id) -> Result<Entry<'_>> {
        let i = self.indexes.get(index).context("missing index")?;
        i.entry_at(id)
    }

    /// Get kanji by character.
    pub fn literal_to_kanji(&self, literal: &str) -> Result<Option<kanjidic2::Character<'_>>> {
        for d in self.indexes.iter() {
            let Some(index) = d.header.by_kanji_literal.get(d.data.as_buf(), literal)? else {
                continue;
            };

            let Some(bytes) = d.data.as_buf().get(*index as usize..) else {
                return Err(anyhow!("Missing entry at {}", *index));
            };

            return Ok(Some(ENCODING.from_slice(bytes)?));
        }

        Ok(None)
    }

    /// Get identifier by sequence.
    pub fn sequence_to_entry(&self, sequence: u32) -> Result<Option<jmdict::Entry<'_>>> {
        for d in self.indexes.iter() {
            let Some(index) = d.header.by_sequence.get(d.data.as_buf(), &sequence)? else {
                continue;
            };

            let Some(bytes) = d.data.as_buf().get(*index as usize..) else {
                return Err(anyhow!("Missing entry at {}", *index));
            };

            return Ok(Some(ENCODING.from_slice(bytes)?));
        }

        Ok(None)
    }

    /// Get indexes by part of speech.
    #[tracing::instrument(skip_all)]
    pub fn by_pos(&self, pos: Set<PartOfSpeech>) -> Result<Vec<(usize, Id)>> {
        let mut unique = BTreeSet::new();
        let mut output = Vec::new();

        for (index, d) in self.indexes.iter().enumerate() {
            let mut first = true;
            unique.clear();

            for pos in pos.iter() {
                if let Some(by_pos) = d.header.by_pos.get(d.data.as_buf(), &pos)? {
                    if first {
                        first = false;
                        unique.extend(by_pos.iter());
                    } else {
                        let new_set = by_pos.iter().collect::<HashSet<_>>();
                        unique.retain(|n| new_set.contains(n));
                    }
                }
            }

            for &by_pos in unique.iter() {
                tracing::trace!(?by_pos);
                let id = d.data.as_buf().load(by_pos)?;
                output.push((index, Id::phrase(*id)));
            }
        }

        tracing::trace!(output = output.len());
        Ok(output)
    }

    /// Lookup all entries matching the given prefix.
    #[tracing::instrument(skip_all)]
    pub fn prefix(&self, prefix: &str) -> Result<Vec<Id>> {
        let mut output = Vec::new();

        for d in self.indexes.iter() {
            for id in d.header.lookup.values_in(d.data.as_buf(), prefix) {
                output.push(*id?);
            }
        }

        Ok(output)
    }

    /// Perform a free text lookup.
    #[tracing::instrument(skip_all)]
    pub fn lookup(&self, query: &str) -> Result<Vec<(usize, Id)>> {
        let mut output = Vec::new();

        match query.split_once(|c: char| matches!(c, '*' | '＊')) {
            Some((prefix, suffix)) if !prefix.is_empty() => {
                if suffix.is_empty() {
                    for (n, i) in self.indexes.iter().enumerate() {
                        for id in i.header.lookup.values_in(i.data.as_buf(), prefix) {
                            output.push((n, *id?));
                        }
                    }
                } else {
                    for (n, i) in self.indexes.iter().enumerate() {
                        for id in i.header.lookup.iter_in(i.data.as_buf(), prefix) {
                            let (string, id) = id?;

                            let Some(s) = string.strip_prefix(prefix.as_bytes()) else {
                                continue;
                            };

                            if !s.ends_with(suffix.as_bytes()) {
                                continue;
                            }

                            output.push((n, *id));
                        }
                    }
                }
            }
            _ => {
                for (n, i) in self.indexes.iter().enumerate() {
                    if let Some(lookup) = i.header.lookup.get(i.data.as_buf(), query)? {
                        for id in lookup {
                            output.push((n, *id));
                        }
                    }
                }
            }
        }

        tracing::trace!(output = output.len());
        Ok(output)
    }

    /// Perform the given search.
    pub fn search(&self, mut input: &str) -> Result<Search<'_>> {
        let mut phrases = Vec::new();
        let mut names = Vec::new();
        let mut characters = Vec::new();
        let mut dedup_phrases = HashMap::new();
        let mut dedup_names = HashMap::new();
        let mut seen = HashSet::new();

        let mut tags = fixed_map::Set::new();

        input = input.trim();

        while let Some(n) = input.rfind('#') {
            let (prefix, suffix) = input.split_at(n);

            input = prefix.trim();

            if let Some(suffix) = suffix.strip_prefix('#') {
                if let Some(pos) = PartOfSpeech::parse_keyword(suffix) {
                    tags.insert(pos);
                }
            }
        }

        let results = if input.is_empty() || input.chars().all(|c| matches!(c, '*' | '＊')) {
            self.by_pos(tags)?
        } else {
            self.populate_kanji(input, &mut seen, &mut characters)?;
            self.lookup(input)?
        };

        for (index, id) in results {
            let entry = self.entry_at(index, id)?;

            match entry {
                Entry::Kanji(kanji) => {
                    if seen.insert(kanji.literal) {
                        characters.push(kanji);
                    }

                    continue;
                }
                Entry::Phrase(entry) => {
                    if !tags.is_empty() {
                        let mut matched = tags;

                        entry.senses.iter().for_each(|sense| {
                            for p in sense.pos.iter() {
                                matched.remove(p);
                            }
                        });

                        if !matched.is_empty() {
                            continue;
                        }
                    }

                    let Some(&i) = dedup_phrases.get(&(index, id.offset())) else {
                        dedup_phrases.insert((index, id.offset()), phrases.len());

                        let data = EntryResultKey {
                            index,
                            offset: id.offset(),
                            sources: [id.source()].into_iter().collect(),
                            weight: Weight::default(),
                        };

                        phrases.push((data, entry));
                        continue;
                    };

                    let Some((data, _)) = phrases.get_mut(i) else {
                        continue;
                    };

                    data.sources.insert(id.source());
                }
                Entry::Name(entry) => {
                    let Some(&i) = dedup_names.get(&(index, id.offset())) else {
                        dedup_names.insert((index, id.offset()), names.len());

                        let data = EntryResultKey {
                            index,
                            offset: id.offset(),
                            sources: [id.source()].into_iter().collect(),
                            weight: Weight::default(),
                        };

                        names.push((data, entry));
                        continue;
                    };

                    let Some((data, _)) = names.get_mut(i) else {
                        continue;
                    };

                    data.sources.insert(id.source());
                }
            };
        }

        for (data, e) in &mut phrases {
            let inflection = data.sources.iter().any(|index| index.is_inflection());
            data.weight = e.weight(input, inflection);
        }

        names.sort_by(|a, b| a.0.weight.cmp(&b.0.weight));
        phrases.sort_by(|a, b| a.0.weight.cmp(&b.0.weight));

        for (_, entry) in &phrases {
            for kanji in &entry.kanji_elements {
                self.populate_kanji(kanji.text, &mut seen, &mut characters)?;
            }
        }

        for (_, entry) in &names {
            for kanji in &entry.kanji {
                self.populate_kanji(kanji, &mut seen, &mut characters)?;
            }
        }

        Ok(Search {
            phrases,
            names,
            characters,
        })
    }

    fn populate_kanji<'this>(
        &'this self,
        input: &str,
        seen: &mut HashSet<&'this str>,
        out: &mut Vec<kanjidic2::Character<'this>>,
    ) -> Result<()> {
        for c in input.chars() {
            if is_katakana(c) || is_hiragana(c) || c.is_ascii_alphabetic() {
                continue;
            }

            for d in self.indexes.iter() {
                let Some(lookup) = d
                    .header
                    .lookup
                    .get(d.data.as_buf(), c.encode_utf8(&mut [0; 4]))?
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

                    let Entry::Kanji(kanji) = d.entry_at(*id)? else {
                        continue;
                    };

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
    pub fn analyze<'q>(&self, q: &'q str, start: usize) -> Result<BTreeMap<Weight, &'q str>> {
        let Some(suffix) = q.get(start..) else {
            return Ok(BTreeMap::new());
        };

        let mut results = HashMap::<_, Weight>::new();

        let mut it = suffix.chars();

        while !it.as_str().is_empty() {
            for d in self.indexes.iter() {
                let Some(values) = d.header.lookup.get(d.data.as_buf(), it.as_str())? else {
                    continue;
                };

                for id in values {
                    let key = match d.entry_at(*id)? {
                        Entry::Phrase(e) => e.weight(it.as_str(), id.source().is_inflection()),
                        Entry::Name(e) => e.weight(it.as_str()).boost(0.5),
                        Entry::Kanji(e) => e.weight(it.as_str()).boost(0.5),
                    };

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
            inputs.insert(key, string);
        }

        Ok(inputs)
    }
}
