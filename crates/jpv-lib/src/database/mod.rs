//! Database that can be used as a dictionary.

mod analyze_glossary;
mod stored;
mod string_indexer;
#[cfg(test)]
mod tests;

use std::borrow::Cow;
use std::collections::{hash_map, BTreeMap, BTreeSet, HashMap, HashSet};
use std::fmt;
use std::path::Path;
use std::sync::Arc;

use anyhow::{anyhow, ensure, Context, Result};
use fixed_map::Set;
use musli::storage::Encoding;
use musli::{Decode, Encode};
use musli_zerocopy::{swiss, trie, OwnedBuf, Ref, ZeroCopy};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::config::Config;
use crate::data::Data;
use crate::inflection::{self, Inflection};
use crate::jmdict;
use crate::jmnedict;
use crate::kana;
use crate::kanjidic2;
use crate::kradfile;
use crate::reporter::Reporter;
use crate::romaji::{self, Segment};
use crate::token::Token;
use crate::{PartOfSpeech, Weight};
use crate::{DATABASE_MAGIC, DATABASE_VERSION};

use self::string_indexer::StringIndexer;

/// Encoding used for storing database.
const ENCODING: Encoding = Encoding::new();

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

/// A deserialized database entry.
#[derive(Serialize)]
#[serde(tag = "type")]
#[non_exhaustive]
pub enum Entry<'a> {
    #[serde(rename = "phrase")]
    Phrase(jmdict::Entry<'a>),
    #[serde(rename = "kanji")]
    Kanji(kanjidic2::Character<'a>),
    #[serde(rename = "name")]
    Name(jmnedict::Entry<'a>),
}

#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
#[musli(mode = Text, name_all = "kebab-case")]
pub struct EntryResultKey {
    pub key: Key,
    pub sources: BTreeSet<Source>,
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
pub enum KanjiIndex {
    /// Indexed by entry.
    Entry,
    /// The literal reading.
    Literal,
    /// Kunyomi readings only.
    KunyomiHiragana,
    KunyomiRomanize,
    KunyomiKatakana,
    /// Kunyomi readings including contextual part.
    KunyomiFullHiragana,
    KunyomiFullRomanized,
    KunyomiFullKatakana,
    /// Onyomi readings.
    OnyomiKatakana,
    OnyomiRomanized,
    OnyomiHiragana,
    /// Meaning (or sense).
    Meaning,
    /// Other language.
    Other,
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
pub enum NameIndex {
    /// The literal reading.
    Literal,
    /// Hiragana reading.
    Hiragana,
    /// Katakana reading.
    Katakana,
    /// Romanized.
    Romanized,
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
pub enum PhraseIndex {
    /// Indexed by entry.
    Entry,
    /// Indexed by kanji reading.
    Kanji,
    /// Indexed by half-kanji reading.
    KanjiHalf,
    /// Indexed by hiragana reading.
    Hiragana,
    /// Indexed by katakana reading.
    Katakana,
    /// Indexed by romanized reading.
    Romanized,
    /// Indexed by meaning.
    Meaning,
}

/// Data stored for a given inflection.
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
#[repr(C)]
pub struct InflectionData {
    pub reading: inflection::Reading,
    pub inflection: Inflection,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, Encode, Decode,
)]
#[non_exhaustive]
#[serde(tag = "type", rename_all = "kebab-case")]
#[musli(mode = Text, tag = "type", name_all = "kebab-case")]
pub enum Source {
    /// Indexed due to a kanji.
    Kanji { index: KanjiIndex },
    /// Indexed due to a phrase.
    Phrase { index: PhraseIndex },
    /// Matched via inflection stemming.
    Inflection {
        #[serde(flatten)]
        data: InflectionData,
    },
    /// Indexed to to a name.
    Name { index: NameIndex },
}

impl Source {
    /// Test if extra indicates an inflection.
    pub fn is_inflection(&self) -> bool {
        match self {
            Source::Inflection { .. } => true,
            _ => false,
        }
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, Encode, Decode,
)]
pub struct Key {
    index: u32,
    offset: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Id {
    /// The index the element was fetched from.
    index: u32,
    /// Raw offset inside of the identifier of the entry.
    offset: u32,
    /// What the identifier refers to.
    source: Source,
}

impl Id {
    /// Get a unique key that identifiers the entry being pointed to.
    pub fn key(&self) -> Key {
        Key {
            index: self.index,
            offset: self.offset,
        }
    }

    /// Stored source.
    pub fn source(&self) -> &Source {
        &self.source
    }
}

/// Input to build the database.
pub enum Input<'a> {
    /// A JMdict file, optionally with a multilingual JMdict whose non-English
    /// glosses are merged into the matching entries (by sequence).
    Jmdict {
        jmdict: &'a str,
        glosses: Option<&'a str>,
    },
    Kanjidic2(&'a str),
    Jmnedict(&'a str),
    Kradfile(&'a [u8]),
}

impl Input<'_> {
    fn name(&self) -> &'static str {
        match self {
            Input::Jmdict { .. } => "JMdict",
            Input::Kanjidic2(..) => "Kanjidic2",
            Input::Jmnedict(..) => "JMnedict",
            Input::Kradfile(..) => "Kradfile",
        }
    }
}

/// Collect the senses with non-English glosses from a multilingual JMdict,
/// keyed by entry sequence. English glosses are skipped, since they duplicate
/// the ones in the primary JMdict.
///
/// The multilingual file puts each language in its own senses, which don't
/// line up with the English ones, so they are kept as separate glosses-only
/// senses to append.
fn collect_extra_senses<'a>(
    shutdown: &Token,
    input: &'a str,
) -> Result<HashMap<u64, Vec<jmdict::Sense<'a>>>> {
    let mut extra = HashMap::<_, Vec<_>>::new();
    let mut parser = jmdict::Parser::new(input);

    while let Some(entry) = parser.parse()? {
        ensure!(!shutdown.is_set(), "Task shut down");

        for sense in entry.senses {
            let gloss = sense
                .gloss
                .into_iter()
                .filter(|g| !g.is_default_lang())
                .collect::<Vec<_>>();

            if gloss.is_empty() {
                continue;
            }

            extra
                .entry(entry.sequence)
                .or_default()
                .push(jmdict::Sense {
                    gloss,
                    ..jmdict::Sense::default()
                });
        }
    }

    Ok(extra)
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

    let header = buf.store_uninit::<stored::GlobalHeader>()?;
    let index = buf.store_uninit::<stored::IndexHeader>()?;

    let name = buf.store_unsized(name)?;

    let mut output = Vec::new();
    let mut lookup = Vec::new();

    let mut by_sequence = HashMap::new();
    let mut by_pos = HashMap::<_, HashSet<_>>::new();
    let mut kanji_literals = HashMap::new();
    let mut input_radicals = HashMap::new();
    let mut input_radicals_to_kanji = HashMap::<_, Vec<_>>::new();
    let mut phrases = Vec::new();
    let mut kanji = Vec::new();

    // Non-English senses to merge into JMdict entries, by sequence.
    let mut extra_senses = HashMap::new();

    if let Input::Jmdict {
        glosses: Some(glosses),
        ..
    } = input
    {
        reporter.instrument_start(
            module_path!(),
            &"Collecting glosses from multilingual JMdict",
            None,
        );

        extra_senses = collect_extra_senses(shutdown, glosses)?;
        reporter.instrument_end(extra_senses.len());
    }

    reporter.instrument_start(
        module_path!(),
        &format_args!("Processing dictionary `{}`", input.name()),
        None,
    );

    let mut count = 0;

    match input {
        Input::Jmdict { jmdict, .. } => {
            let mut jmdict = jmdict::Parser::new(jmdict);

            while let Some(mut entry) = jmdict.parse()? {
                ensure!(!shutdown.is_set(), "Task shut down");

                // Entries only in the multilingual file are dropped, since
                // the source with examples has every entry with an English
                // gloss.
                if let Some(senses) = extra_senses.remove(&entry.sequence) {
                    entry.senses.extend(senses);
                }

                if count % 1000 == 0 {
                    reporter.instrument_progress(1000);
                }

                count += 1;

                output.clear();
                ENCODING.to_writer(&mut output, &entry)?;

                let entry_ref = buf.store_slice(&output)?.offset() as u32;
                phrases.push(entry_ref);

                by_sequence.insert(
                    entry.sequence as u32,
                    stored::PhrasePos {
                        offset: entry_ref,
                        reading: PhraseIndex::Entry,
                    },
                );

                for sense in &entry.senses {
                    for pos in &sense.pos {
                        by_pos.entry(pos).or_default().insert(stored::PhrasePos {
                            offset: entry_ref,
                            reading: PhraseIndex::Meaning,
                        });
                    }

                    let id = stored::Id::phrase(entry_ref, PhraseIndex::Meaning);

                    for g in &sense.gloss {
                        if g.ty == Some("expl") {
                            continue;
                        }

                        // Glossary analysis is for English text.
                        if !g.is_default_lang() {
                            continue;
                        }

                        populate_analyzed(g.text, &mut lookup, id);
                    }
                }

                for el in &entry.reading_elements {
                    lookup.push((
                        Cow::Borrowed(el.text),
                        stored::Id::phrase(entry_ref, PhraseIndex::Hiragana),
                    ));

                    let a = stored::Id::phrase(entry_ref, PhraseIndex::Romanized);
                    let b = stored::Id::phrase(entry_ref, PhraseIndex::Katakana);
                    other_readings(&mut lookup, el.text, a, b, |s| s.katakana());
                }

                for el in &entry.kanji_elements {
                    if let Some(s) = full_to_half_string(el.text) {
                        lookup.push((
                            Cow::Owned(s),
                            stored::Id::phrase(entry_ref, PhraseIndex::KanjiHalf),
                        ));
                    }

                    lookup.push((
                        Cow::Borrowed(el.text),
                        stored::Id::phrase(entry_ref, PhraseIndex::Kanji),
                    ));
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

                let kanji_ref = buf.store_slice(&output)?.offset() as u32;
                kanji.push(kanji_ref);

                kanji_literals.insert(c.literal, kanji_ref);

                lookup.push((
                    Cow::Borrowed(c.literal),
                    stored::Id::kanji(kanji_ref, KanjiIndex::Literal),
                ));

                for reading in &c.readings {
                    match reading.ty {
                        "ja_kun" => {
                            if let Some((prefix, _)) = reading.text.split_once('.') {
                                let a = stored::Id::kanji(kanji_ref, KanjiIndex::KunyomiRomanize);
                                let b = stored::Id::kanji(kanji_ref, KanjiIndex::KunyomiKatakana);
                                other_readings(&mut lookup, prefix, a, b, |s| s.katakana());
                                let id = stored::Id::kanji(kanji_ref, KanjiIndex::KunyomiHiragana);
                                lookup.push((Cow::Borrowed(prefix), id));
                            }

                            let a = stored::Id::kanji(kanji_ref, KanjiIndex::KunyomiFullRomanized);
                            let b = stored::Id::kanji(kanji_ref, KanjiIndex::KunyomiFullKatakana);
                            other_readings(&mut lookup, reading.text, a, b, |s| s.katakana());

                            let id = stored::Id::kanji(kanji_ref, KanjiIndex::KunyomiFullHiragana);
                            lookup.push((Cow::Borrowed(reading.text), id));
                        }
                        "ja_on" => {
                            let a = stored::Id::kanji(kanji_ref, KanjiIndex::OnyomiRomanized);
                            let b = stored::Id::kanji(kanji_ref, KanjiIndex::OnyomiHiragana);
                            other_readings(&mut lookup, reading.text, a, b, |s| s.hiragana());
                            let id = stored::Id::kanji(kanji_ref, KanjiIndex::OnyomiKatakana);
                            lookup.push((Cow::Borrowed(reading.text), id));
                        }
                        _ => {
                            let id = stored::Id::kanji(kanji_ref, KanjiIndex::Other);
                            lookup.push((Cow::Borrowed(reading.text), id));
                        }
                    };
                }

                for meaning in &c.meanings {
                    let id = stored::Id::kanji(kanji_ref, KanjiIndex::Meaning);
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

                let name_ref = buf.store_slice(&output)?.offset() as u32;

                for kanji in entry.kanji.iter().copied() {
                    lookup.push((
                        Cow::Borrowed(kanji),
                        stored::Id::name(name_ref, NameIndex::Literal),
                    ));
                }

                for reading in entry.reading {
                    lookup.push((
                        Cow::Borrowed(reading.text),
                        stored::Id::name(name_ref, NameIndex::Hiragana),
                    ));
                    let a = stored::Id::name(name_ref, NameIndex::Romanized);
                    let b = stored::Id::name(name_ref, NameIndex::Katakana);
                    other_readings(&mut lookup, reading.text, a, b, |s| s.katakana());
                }
            }
        }
        Input::Kradfile(data) => {
            let mut parser = kradfile::Parser::new(data);

            while let Some(entry) = parser.parse() {
                ensure!(!shutdown.is_set(), "Task shut down");

                if count % 1000 == 0 {
                    reporter.instrument_progress(1000);
                }

                count += 1;

                output.clear();
                ENCODING.to_writer(&mut output, &entry)?;

                let radicals_ref = buf.store_slice(&output)?.offset() as u32;
                input_radicals.insert(entry.kanji, radicals_ref);

                for radical in entry.radicals {
                    input_radicals_to_kanji
                        .entry(radical)
                        .or_default()
                        .push(radicals_ref);
                }
            }
        }
    }

    let phrases = buf.store_slice(&phrases)?;
    let kanji = buf.store_slice(&kanji)?;

    reporter.instrument_end(count);

    lookup.sort_by(|(a, _), (b, _)| b.as_ref().cmp(a.as_ref()));
    tracing::info!("Inserting {} readings", lookup.len());

    let mut readings2 = Vec::with_capacity(lookup.len());
    let by_kanji_literal;
    let radicals;
    let radicals_to_kanji;

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

        radicals = {
            let mut output = HashMap::new();

            for (key, value) in &input_radicals {
                let s = indexer.store(&mut buf, key)?;
                output.insert(s, *value);
            }

            output
        };

        radicals_to_kanji = {
            let mut output = HashMap::new();

            for (key, values) in &input_radicals_to_kanji {
                let s = indexer.store(&mut buf, key)?;
                output.insert(s, values);
            }

            output
        };

        tracing::info!(
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
            let set = buf.store_slice(&values)?;
            entries.push((key, set));
        }

        tracing::info!("Storing by_pos: {}...", entries.len());
        swiss::store_map(&mut buf, entries)?
    };

    let by_kanji_literal = {
        tracing::info!("Storing by_kanji_literal: {}...", by_kanji_literal.len());
        swiss::store_map(&mut buf, by_kanji_literal)?
    };

    let radicals = {
        tracing::info!("Storing radicals: {}...", radicals.len());
        swiss::store_map(&mut buf, radicals)?
    };

    let radicals_to_kanji = {
        tracing::info!("Storing radicals_to_kanji: {}...", radicals_to_kanji.len());

        let mut intermediate = Vec::new();

        for (key, values) in radicals_to_kanji {
            let values = buf.store_slice(values)?;
            intermediate.push((key, values));
        }

        swiss::store_map(&mut buf, intermediate)?
    };

    let by_sequence = {
        tracing::info!("Storing by_sequence: {}...", by_sequence.len());
        swiss::store_map(&mut buf, by_sequence)?
    };

    buf.load_uninit_mut(index)?.write(&stored::IndexHeader {
        name,
        lookup,
        by_pos,
        by_kanji_literal,
        radicals,
        radicals_to_kanji,
        by_sequence,
        phrases,
        kanji,
    });

    buf.load_uninit_mut(header)?.write(&stored::GlobalHeader {
        magic: DATABASE_MAGIC,
        version: DATABASE_VERSION,
        index: index.assume_init(),
    });

    reporter.instrument_end(0);
    Ok(buf)
}

fn populate_analyzed<'a>(
    text: &'a str,
    lookup: &mut Vec<(Cow<'a, str>, stored::Id)>,
    id: stored::Id,
) {
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
        if is_common(phrase) {
            continue;
        }

        let lowercase = phrase.to_lowercase();

        let key = if phrase == lowercase {
            Cow::Borrowed(phrase)
        } else {
            Cow::Owned(lowercase)
        };

        lookup.push((key, id));
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
    output: &mut Vec<(Cow<'_, str>, stored::Id)>,
    text: &str,
    a: stored::Id,
    b: stored::Id,
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

fn normalize_to_hiragana(input: &str) -> String {
    let lowered;
    let input = if input.bytes().any(|b| b.is_ascii_uppercase()) {
        lowered = input.to_ascii_lowercase();
        lowered.as_str()
    } else {
        input
    };

    let mut output = String::with_capacity(input.len());

    for part in romaji::analyze(input) {
        output.push_str(part.hiragana());
    }

    output
}

// A loaded index.
pub struct Index {
    header: stored::IndexHeader,
    data: Data,
}

impl Index {
    /// Construct a new database wrapper.
    ///
    /// This returns `Ok(None)` if the database is incompatible with the current
    /// version.
    pub fn open(data: Data) -> Result<Self, IndexOpenError> {
        let buf = data.as_buf();
        let header = buf.load(Ref::<stored::GlobalHeader>::zero())?;

        if header.magic != DATABASE_MAGIC {
            return Err(IndexOpenError::MagicMismatch);
        }

        if header.version != DATABASE_VERSION {
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
    fn entry_at(&self, id: Id) -> Result<Entry<'_>> {
        let Some(bytes) = self.data.as_buf().get(id.offset as usize..) else {
            return Err(anyhow!("Missing entry at {}", id.offset));
        };

        Ok(match id.source {
            Source::Kanji { .. } => Entry::Kanji(ENCODING.decode(bytes)?),
            Source::Name { .. } => Entry::Name(ENCODING.decode(bytes)?),
            Source::Phrase { .. } | Source::Inflection { .. } => {
                Entry::Phrase(ENCODING.decode(bytes)?)
            }
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
    pub fn sequence_to_id(&self, sequence: u32) -> Result<Vec<Id>> {
        let mut output = Vec::new();

        for (index, d) in self.indexes.iter().enumerate() {
            let Some(pos) = d.header.by_sequence.get(d.data.as_buf(), &sequence)? else {
                continue;
            };

            output.push(self.convert_id(index, stored::Id::phrase(pos.offset, pos.reading))?);
        }

        Ok(output)
    }

    /// Get all entries matching the given id.
    pub fn entry_at(&self, id: Id) -> Result<Entry<'_>> {
        let i = self
            .indexes
            .get(id.index as usize)
            .context("missing index")?;
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

            return Ok(Some(ENCODING.decode(bytes)?));
        }

        Ok(None)
    }

    /// Get radicals by character.
    pub fn literal_to_radicals(&self, literal: &str) -> Result<Option<kradfile::Entry<'_>>> {
        for d in self.indexes.iter() {
            let Some(index) = d.header.radicals.get(d.data.as_buf(), literal)? else {
                continue;
            };

            let Some(bytes) = d.data.as_buf().get(*index as usize..) else {
                return Err(anyhow!("Missing entry at {}", *index));
            };

            return Ok(Some(ENCODING.decode(bytes)?));
        }

        Ok(None)
    }

    /// Get identifier by sequence.
    pub fn sequence_to_entry(&self, sequence: u32) -> Result<Option<jmdict::Entry<'_>>> {
        for d in self.indexes.iter() {
            let Some(pos) = d.header.by_sequence.get(d.data.as_buf(), &sequence)? else {
                continue;
            };

            let Some(bytes) = d.data.as_buf().get(pos.offset as usize..) else {
                return Err(anyhow!("Missing entry at {}", pos.offset));
            };

            return Ok(Some(ENCODING.decode(bytes)?));
        }

        Ok(None)
    }

    /// Get indexes by part of speech.
    #[tracing::instrument(skip_all)]
    pub fn by_pos(&self, pos: Set<PartOfSpeech>) -> Result<Vec<Id>> {
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
                let pos = d.data.as_buf().load(by_pos)?;
                output.push(self.convert_id(index, stored::Id::phrase(pos.offset, pos.reading))?);
            }
        }

        tracing::trace!(output = output.len());
        Ok(output)
    }

    /// Lookup any entries matching a custom filter.
    #[tracing::instrument(skip_all)]
    pub fn all(&self) -> Result<Vec<Id>> {
        let mut output = Vec::new();

        for (index, d) in self.indexes.iter().enumerate() {
            for result in d.header.phrases.iter() {
                let id = *d.data.as_buf().load(result)?;
                let id = stored::Id::phrase(id, PhraseIndex::Entry);
                output.push(self.convert_id(index, id)?);
            }

            for result in d.header.kanji.iter() {
                let id = *d.data.as_buf().load(result)?;
                let id = stored::Id::kanji(id, KanjiIndex::Entry);
                output.push(self.convert_id(index, id)?);
            }
        }

        Ok(output)
    }

    /// Perform a free text lookup.
    #[tracing::instrument(skip_all)]
    pub fn lookup(&self, query: &str) -> Result<Vec<Id>> {
        let mut output = Vec::new();

        if query.chars().all(|c| matches!(c, '*' | '＊')) {
            for (index, d) in self.indexes.iter().enumerate() {
                for result in d.header.phrases.iter() {
                    let id = *d.data.as_buf().load(result)?;
                    let id = stored::Id::phrase(id, PhraseIndex::Entry);
                    output.push(self.convert_id(index, id)?);
                }
            }

            return Ok(output);
        }

        let Some((prefix, suffix)) = query.split_once(['*', '＊']) else {
            for (n, d) in self.indexes.iter().enumerate() {
                if let Some(lookup) = d.header.lookup.get(d.data.as_buf(), query)? {
                    for id in lookup {
                        output.push(self.convert_id(n, *id)?);
                    }
                }
            }

            self.visit_inflections(query, |index, offset, _entry, reading, inflection| {
                output.push(Id {
                    index: index as u32,
                    offset,
                    source: Source::Inflection {
                        data: InflectionData {
                            reading,
                            inflection,
                        },
                    },
                });
            })?;

            return Ok(output);
        };

        let parts = suffix
            .split(['*', '＊'])
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>();

        for (n, d) in self.indexes.iter().enumerate() {
            'outer: for id in d.header.lookup.iter_in(d.data.as_buf(), prefix) {
                let (string, id) = id?;

                let Some(mut rest) = string.strip_prefix(prefix.as_bytes()) else {
                    continue;
                };

                if let [head @ .., tail] = &parts[..] {
                    for &part in head {
                        let Some(next) = memchr::memmem::find(rest, part.as_bytes()) else {
                            continue 'outer;
                        };

                        rest = &rest[next + part.len()..];
                    }

                    if !rest.ends_with(tail.as_bytes()) {
                        continue;
                    }
                }

                output.push(self.convert_id(n, *id)?);
            }
        }

        Ok(output)
    }

    #[tracing::instrument(skip_all)]
    fn convert_id(&self, index: usize, id: stored::Id) -> Result<Id> {
        Ok(Id {
            index: index as u32,
            offset: id.offset,
            source: self.convert_source(index, id.source)?,
        })
    }

    #[tracing::instrument(skip_all)]
    fn convert_source(&self, _index: usize, source: stored::Source) -> Result<Source> {
        Ok(match source {
            stored::Source::Kanji { index } => Source::Kanji { index },
            stored::Source::Phrase { index } => Source::Phrase { index },
            stored::Source::Name { index } => Source::Name { index },
        })
    }

    fn visit_inflections<F>(&self, query: &str, mut f: F) -> Result<()>
    where
        F: FnMut(usize, u32, &jmdict::Entry<'_>, inflection::Reading, Inflection),
    {
        if query.is_empty() {
            return Ok(());
        }

        let normalized = normalize_to_hiragana(query);

        if normalized.is_empty() || normalized.chars().any(|c| c.is_ascii_alphabetic()) {
            return Ok(());
        }

        let mut stems = inflection::stem(&normalized);
        stems.retain(|stem| !(stem.inflection.is_empty() && stem.dictionary == normalized));

        if normalized != query && !query.chars().any(|c| c.is_ascii_alphabetic()) {
            for stem in inflection::stem(query) {
                if stem.inflection.is_empty() && stem.dictionary == query {
                    continue;
                }

                stems.push(stem);
            }
        }

        if stems.is_empty() {
            return Ok(());
        }

        let mut by_dictionary = BTreeMap::<&str, Vec<&inflection::Stem>>::new();

        for stem in &stems {
            by_dictionary
                .entry(stem.dictionary.as_str())
                .or_default()
                .push(stem);
        }

        let mut candidates = BTreeMap::<(usize, u32), Vec<&inflection::Stem>>::new();

        for (dict, dict_stems) in by_dictionary {
            for (index, d) in self.indexes.iter().enumerate() {
                let Some(lookup) = d.header.lookup.get(d.data.as_buf(), dict)? else {
                    continue;
                };

                for stored_id in lookup {
                    if !matches!(
                        stored_id.source,
                        stored::Source::Phrase {
                            index: PhraseIndex::Kanji | PhraseIndex::Hiragana
                        }
                    ) {
                        continue;
                    }

                    candidates
                        .entry((index, stored_id.offset))
                        .or_default()
                        .extend_from_slice(&dict_stems);
                }
            }
        }

        for ((index, offset), entry_stems) in candidates {
            let d = &self.indexes[index];
            let Some(bytes) = d.data.as_buf().get(offset as usize..) else {
                return Err(anyhow!("Missing entry at {offset}"));
            };
            let entry: jmdict::Entry<'_> = ENCODING.decode(bytes)?;

            for (reading, inflection) in inflection::match_stems(entry_stems, &entry) {
                f(index, offset, &entry, reading, inflection);
            }
        }

        Ok(())
    }

    /// Perform the given search.
    #[tracing::instrument(skip_all)]
    pub fn search(&self, input: &str) -> Result<Search<'_>> {
        let mut phrases = Vec::new();
        let mut names = Vec::new();
        let mut characters = Vec::new();
        let mut dedup_phrases = HashMap::new();
        let mut dedup_names = HashMap::new();
        let mut seen = HashSet::new();

        let query = crate::search::parse(input);

        let mut inputs = query.phrases.into_iter();

        let Some(first) = inputs.next() else {
            return Ok(Search {
                phrases,
                names,
                characters,
            });
        };

        self.populate_kanji(first, &mut seen, &mut characters)?;
        let mut ids = self.lookup(first)?;

        for remainder in inputs {
            self.populate_kanji(remainder, &mut seen, &mut characters)?;
            let current = self.lookup(remainder)?;

            let current = current
                .into_iter()
                .map(|i| (i.index, i.offset))
                .collect::<HashSet<_>>();

            ids.retain(|id| current.contains(&(id.index, id.offset)));
        }

        let mut current = HashSet::new();
        let mut buf = String::new();

        for id in ids {
            match self.entry_at(id)? {
                Entry::Kanji(kanji) => {
                    if seen.insert(kanji.literal) {
                        characters.push(kanji);
                    }

                    continue;
                }
                Entry::Phrase(entry) => {
                    if !query.entities.is_empty() {
                        current.clear();
                        current.extend(query.entities.iter().copied());

                        entry.visit_entities(&mut buf, |entity| {
                            current.remove(entity);
                        });

                        if !current.is_empty() {
                            continue;
                        }
                    }

                    let Some(&i) = dedup_phrases.get(&id.key()) else {
                        dedup_phrases.insert(id.key(), phrases.len());

                        let data = EntryResultKey {
                            key: id.key(),
                            sources: [id.source].into_iter().collect(),
                            weight: Weight::default(),
                        };

                        phrases.push((data, entry));
                        continue;
                    };

                    let Some((data, _)) = phrases.get_mut(i) else {
                        continue;
                    };

                    data.sources.insert(id.source);
                }
                Entry::Name(entry) => {
                    if !query.entities.is_empty() {
                        current.clear();
                        current.extend(query.entities.iter().copied());

                        entry.visit_entities(|entity| {
                            current.remove(entity);
                        });

                        if !current.is_empty() {
                            continue;
                        }
                    }

                    let Some(&i) = dedup_names.get(&id.key()) else {
                        dedup_names.insert(id.key(), names.len());

                        let data = EntryResultKey {
                            key: id.key(),
                            sources: [id.source].into_iter().collect(),
                            weight: Weight::default(),
                        };

                        names.push((data, entry));
                        continue;
                    };

                    let Some((data, _)) = names.get_mut(i) else {
                        continue;
                    };

                    data.sources.insert(id.source);
                }
            }
        }

        for (data, e) in &mut phrases {
            let inflection = data.sources.iter().any(|source| source.is_inflection());
            data.weight = e.weight(input, inflection);
        }

        names.sort_by_key(|a| a.0.weight);
        phrases.sort_by_key(|a| a.0.weight);

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
            if kana::is_katakana(c) || kana::is_hiragana(c) || c.is_ascii_alphabetic() {
                continue;
            }

            for (index, d) in self.indexes.iter().enumerate() {
                let Some(lookup) = d
                    .header
                    .lookup
                    .get(d.data.as_buf(), c.encode_utf8(&mut [0; 4]))?
                else {
                    continue;
                };

                for id in lookup {
                    let id = self.convert_id(index, *id)?;

                    if let Entry::Kanji(kanji) = d.entry_at(id)? {
                        if seen.insert(kanji.literal) {
                            out.push(kanji);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Analyze the given string, looking it up in the database and returning
    /// all prefix matching entries and their texts.
    pub fn analyze<'q>(&self, q: &'q str, start: usize) -> Result<Vec<(Weight, &'q str)>> {
        let Some(suffix) = q.get(start..) else {
            return Ok(Vec::new());
        };

        let mut results = HashMap::<_, Weight>::new();

        let mut it = suffix.chars();

        while !it.as_str().is_empty() {
            let prefix = it.as_str();

            for (index, d) in self.indexes.iter().enumerate() {
                let Some(values) = d.header.lookup.get(d.data.as_buf(), prefix)? else {
                    continue;
                };

                for stored_id in values {
                    let id = self.convert_id(index, *stored_id)?;

                    let key = match d.entry_at(id)? {
                        Entry::Phrase(e) => e.weight(prefix, id.source.is_inflection()),
                        Entry::Name(e) => e.weight(prefix).boost(0.5),
                        Entry::Kanji(e) => e.weight(prefix).boost(0.5),
                    };

                    match results.entry(prefix) {
                        hash_map::Entry::Occupied(mut e) => {
                            e.insert((*e.get()).max(key));
                        }
                        hash_map::Entry::Vacant(e) => {
                            e.insert(key);
                        }
                    }
                }
            }

            self.visit_inflections(prefix, |_index, _offset, entry, _reading, _inflection| {
                let key = entry.weight(prefix, true);

                match results.entry(prefix) {
                    hash_map::Entry::Occupied(mut e) => {
                        e.insert((*e.get()).max(key));
                    }
                    hash_map::Entry::Vacant(e) => {
                        e.insert(key);
                    }
                }
            })?;

            it.next_back();
        }

        // Weights can collide between different prefixes, so collect into a
        // vector instead of keying by weight.
        let mut inputs = results
            .into_iter()
            .map(|(string, key)| (key, string))
            .collect::<Vec<_>>();

        inputs.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| b.1.len().cmp(&a.1.len())));
        Ok(inputs)
    }
}
