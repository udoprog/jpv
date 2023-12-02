use core::fmt;
use std::collections::HashSet;

use fixed_map::Set;
use musli::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::entities::{Dialect, Field, KanjiInfo, Miscellaneous, PartOfSpeech, ReadingInfo};
use crate::priority::Priority;
use crate::Weight;

#[borrowme::borrowme]
#[derive(Clone, Debug, Serialize, Deserialize, Encode, Decode)]
#[musli(packed)]
pub struct Entry<'a> {
    pub sequence: u64,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[borrowed_attr(serde(borrow))]
    pub reading_elements: Vec<ReadingElement<'a>>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[borrowed_attr(serde(borrow))]
    pub kanji_elements: Vec<KanjiElement<'a>>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[borrowed_attr(serde(borrow))]
    pub senses: Vec<Sense<'a>>,
}

impl Entry<'_> {
    /// Entry weight.
    pub fn weight(&self, input: &str, conjugation: bool) -> Weight {
        // Boost based on exact query.
        let mut query = 1.0f32;
        // Store the priority which performs the maximum boost.
        let mut priority = 1.0f32;
        // Perform boost by number of senses, maximum boost at 10 senses.
        let sense_count = 1.0 + self.senses.len().min(10) as f32 / 10.0;
        // Conjugation boost.
        let conjugation = if conjugation { 1.2 } else { 1.0 };
        // Calculate length boost.
        let length = (input.chars().count().min(10) as f32 / 10.0) * 1.2;

        for element in &self.reading_elements {
            if element.text == input {
                if element.no_kanji || self.kanji_elements.iter().all(|k| k.is_rare()) {
                    query = query.max(3.0);
                } else {
                    query = query.max(2.0);
                }
            }

            for p in &element.priority {
                priority = priority.max(p.weight());
            }
        }

        for element in &self.kanji_elements {
            if element.text == input {
                query = query.max(3.0);
            }

            for p in &element.priority {
                priority = priority.max(p.weight());
            }
        }

        for sense in &self.senses {
            for gloss in &sense.gloss {
                if gloss.text == input {
                    query = query.max(1.5);
                }
            }
        }

        Weight::new(query * priority * sense_count * conjugation * length)
    }
}

#[borrowme::borrowme]
#[derive(Clone, Debug, Serialize, Deserialize, Encode, Decode)]
#[musli(packed)]
pub struct KanjiElement<'a> {
    pub text: &'a str,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub priority: Vec<Priority>,
    #[serde(default, skip_serializing_if = "Set::is_empty")]
    #[musli(with = crate::musli::set::<_>)]
    #[copy]
    pub info: Set<KanjiInfo>,
}

impl<'a> KanjiElement<'a> {
    /// Test if kanji is rare.
    pub fn is_rare(&self) -> bool {
        self.info.contains(KanjiInfo::RareKanji)
    }

    /// Test if kanji is search only.
    pub fn is_search_only(&self) -> bool {
        self.info.contains(KanjiInfo::SearchOnlyKanji)
    }

    /// Debug the kanji element, while avoiding formatting elements which are
    /// not defined.
    pub fn debug_sparse(&self) -> impl fmt::Debug + '_ {
        struct DebugSparse<'a>(&'a KanjiElement<'a>);

        impl fmt::Debug for DebugSparse<'_> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                let mut f = f.debug_struct("KanjiElement");

                f.field("text", &self.0.text);

                if !self.0.priority.is_empty() {
                    f.field("priority", &self.0.priority);
                }

                if !self.0.info.is_empty() {
                    f.field("info", &self.0.info);
                }

                f.finish_non_exhaustive()
            }
        }

        DebugSparse(self)
    }
}

#[borrowme::borrowme]
#[derive(Clone, Debug, Serialize, Deserialize, Encode, Decode)]
#[musli(packed)]
pub struct ReadingElement<'a> {
    pub text: &'a str,
    pub no_kanji: bool,
    #[serde(default, skip_serializing_if = "HashSet::is_empty")]
    pub reading_string: HashSet<&'a str>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub priority: Vec<Priority>,
    #[serde(default, skip_serializing_if = "Set::is_empty")]
    #[musli(with = crate::musli::set::<_>)]
    #[copy]
    pub info: Set<ReadingInfo>,
}

impl<'a> ReadingElement<'a> {
    /// Debug the reading element, while avoiding formatting elements which are
    /// not defined.
    pub fn debug_sparse(&self) -> impl fmt::Debug + '_ {
        struct DebugSparse<'a>(&'a ReadingElement<'a>);

        impl fmt::Debug for DebugSparse<'_> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                let mut f = f.debug_struct("ReadingElement");

                f.field("text", &self.0.text);

                if self.0.no_kanji {
                    f.field("no_kanji", &self.0.no_kanji);
                }

                if !self.0.reading_string.is_empty() {
                    f.field("reading_string", &self.0.reading_string);
                }

                if !self.0.priority.is_empty() {
                    f.field("priority", &self.0.priority);
                }

                if !self.0.info.is_empty() {
                    f.field("info", &self.0.info);
                }

                f.finish_non_exhaustive()
            }
        }

        DebugSparse(self)
    }

    /// Test if kana is search only.
    pub fn is_search_only(&self) -> bool {
        self.info.contains(ReadingInfo::SearchOnlyKana)
    }

    /// Test if this reading applies to the given string.
    pub fn applies_to(&self, text: &str) -> bool {
        if self.no_kanji || self.is_search_only() {
            return false;
        }

        if self.reading_string.is_empty() {
            return true;
        }

        self.reading_string.contains(text)
    }
}

impl OwnedReadingElement {
    /// If the reading element applies to nothing.
    pub fn applies_to_nothing(&self) -> bool {
        self.no_kanji || self.is_search_only()
    }

    /// Test if kana is search only.
    pub fn is_search_only(&self) -> bool {
        self.info.contains(ReadingInfo::SearchOnlyKana)
    }

    /// Test if this reading applies to the given string.
    pub fn applies_to(&self, text: &str) -> bool {
        if self.applies_to_nothing() {
            return false;
        }

        if self.reading_string.is_empty() {
            return true;
        }

        self.reading_string.contains(text)
    }
}

#[borrowme::borrowme]
#[derive(Clone, Debug, Serialize, Deserialize, Encode, Decode)]
#[musli(packed)]
pub struct Example<'a> {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[borrowed_attr(serde(borrow))]
    pub sentences: Vec<ExampleSentence<'a>>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[borrowed_attr(serde(borrow))]
    pub sources: Vec<ExampleSource<'a>>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[borrowed_attr(serde(borrow))]
    pub texts: Vec<&'a str>,
}

#[borrowme::borrowme]
#[derive(Clone, Debug, Serialize, Deserialize, Encode, Decode)]
#[musli(packed)]
pub struct ExampleSentence<'a> {
    pub text: &'a str,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lang: Option<&'a str>,
}

#[borrowme::borrowme]
#[derive(Clone, Debug, Serialize, Deserialize, Encode, Decode)]
#[musli(packed)]
pub struct ExampleSource<'a> {
    pub text: &'a str,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ty: Option<&'a str>,
}

#[borrowme::borrowme]
#[derive(Clone, Debug, Serialize, Deserialize, Encode, Decode)]
#[musli(packed)]
pub struct Glossary<'a> {
    pub text: &'a str,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ty: Option<&'a str>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lang: Option<&'a str>,
}

const DEFAULT_LANGUAGE: &str = "eng";

#[borrowme::borrowme]
#[derive(Default, Clone, Debug, Serialize, Deserialize, Encode, Decode)]
#[musli(packed)]
pub struct Sense<'a> {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub xref: Vec<&'a str>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub gloss: Vec<Glossary<'a>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub info: Option<&'a str>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub stagk: Vec<&'a str>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub stagr: Vec<&'a str>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub source_language: Vec<SourceLanguage<'a>>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub antonym: Vec<&'a str>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub examples: Vec<Example<'a>>,
    #[musli(with = crate::musli::set::<_>)]
    #[serde(default, skip_serializing_if = "Set::is_empty")]
    #[copy]
    pub pos: Set<PartOfSpeech>,
    #[musli(with = crate::musli::set::<_>)]
    #[serde(default, skip_serializing_if = "Set::is_empty")]
    #[copy]
    pub misc: Set<Miscellaneous>,
    #[musli(with = crate::musli::set::<_>)]
    #[serde(default, skip_serializing_if = "Set::is_empty")]
    #[copy]
    pub dialect: Set<Dialect>,
    #[musli(with = crate::musli::set::<_>)]
    #[serde(default, skip_serializing_if = "Set::is_empty")]
    #[copy]
    pub field: Set<Field>,
}

impl<'a> Sense<'a> {
    /// Debug the sense element, while avoiding formatting elements which are
    /// not defined.
    pub fn debug_sparse(&self) -> impl fmt::Debug + '_ {
        struct DebugSparse<'a>(&'a Sense<'a>);

        impl fmt::Debug for DebugSparse<'_> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                struct SparseDebug<'a>(&'a [SourceLanguage<'a>]);

                impl<'a> fmt::Debug for SparseDebug<'a> {
                    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                        let mut f = f.debug_list();

                        for item in self.0 {
                            f.entry(&item.debug_sparse());
                        }

                        f.finish()
                    }
                }

                let mut f = f.debug_struct("Sense");

                if !self.0.pos.is_empty() {
                    f.field("pos", &self.0.pos);
                }

                if !self.0.xref.is_empty() {
                    f.field("xref", &self.0.xref);
                }

                if let Some(field) = self.0.info {
                    f.field("info", &field);
                }

                if !self.0.misc.is_empty() {
                    f.field("misc", &self.0.misc);
                }

                if !self.0.dialect.is_empty() {
                    f.field("dialect", &self.0.dialect);
                }

                if !self.0.stagk.is_empty() {
                    f.field("stagk", &self.0.stagk);
                }

                if !self.0.stagr.is_empty() {
                    f.field("stagr", &self.0.stagr);
                }

                if !self.0.field.is_empty() {
                    f.field("fields", &self.0.field);
                }

                if !self.0.source_language.is_empty() {
                    f.field("source_language", &SparseDebug(&self.0.source_language));
                }

                if !self.0.antonym.is_empty() {
                    f.field("antonym", &self.0.antonym);
                }

                f.finish_non_exhaustive()
            }
        }

        DebugSparse(self)
    }

    pub fn is_lang(&self, arg: &str) -> bool {
        for g in &self.gloss {
            if g.lang.unwrap_or(DEFAULT_LANGUAGE) == arg {
                return true;
            }
        }

        false
    }
}

#[borrowme::borrowme]
#[derive(Clone, Debug, Serialize, Deserialize, Encode, Decode)]
#[musli(packed)]
pub struct SourceLanguage<'a> {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<&'a str>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lang: Option<&'a str>,
    pub waseigo: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ty: Option<&'a str>,
}

impl<'a> SourceLanguage<'a> {
    /// Debug the source language  element, while avoiding formatting elements
    /// which are not defined.
    pub fn debug_sparse(&self) -> impl fmt::Debug + '_ {
        struct DebugSparse<'a>(&'a SourceLanguage<'a>);

        impl fmt::Debug for DebugSparse<'_> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                let mut f = f.debug_struct("SourceLanguage");

                if let Some(field) = self.0.text {
                    f.field("text", &field);
                }

                if let Some(field) = self.0.lang {
                    f.field("lang", &field);
                }

                f.field("lang", &self.0.waseigo);

                if let Some(field) = self.0.ty {
                    f.field("ty", &field);
                }

                f.finish_non_exhaustive()
            }
        }

        DebugSparse(self)
    }
}
