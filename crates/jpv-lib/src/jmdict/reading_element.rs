use core::fmt;
use core::mem;
use std::collections::HashSet;

use anyhow::ensure;
use anyhow::{anyhow, Context, Result};
use fixed_map::Set;
use musli::{Decode, Encode};
use serde::Deserialize;
use serde::Serialize;

use crate::entities::ReadingInfo;
use crate::jmdict::empty;
use crate::jmdict::text;

use crate::priority::Priority;

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

#[derive(Debug, Default)]
enum State<'a> {
    #[default]
    Root,
    Text(text::Builder<'a>),
    NoKanji(empty::Builder),
    ReadingString(text::Builder<'a>),
    Priority(text::Builder<'a>),
    Information(text::Builder<'a>),
}

#[derive(Debug, Default)]
pub(super) struct Builder<'a> {
    state: State<'a>,
    text: Option<&'a str>,
    no_kanji: bool,
    reading_string: HashSet<&'a str>,
    priority: Vec<Priority>,
    info: Set<ReadingInfo>,
}

impl<'a> Builder<'a> {
    builder! {
        self => ReadingElement<'a> {
            "reb", Text, value => {
                ensure!(self.text.is_none(), "Only one reb element allowed");
                self.text = Some(value);
            }
            "re_nokanji", NoKanji, () => {
                self.no_kanji = true;
            }
            "re_restr", ReadingString, value => {
                self.reading_string.insert(value);
            }
            "re_pri", Priority, value => {
                let priority = Priority::parse(value).with_context(|| anyhow!("Unsupported priority `{value}`"))?;
                self.priority.push(priority);
            }
            "re_inf", Information, value => {
                let info = ReadingInfo::parse(value).with_context(|| anyhow!("Unsupported info `{value}`"))?;
                self.info.insert(info);
            }
        }
    }

    fn build(&mut self) -> Result<ReadingElement<'a>> {
        let text = self.text.context("missing text")?;
        let reading_string = mem::take(&mut self.reading_string);
        let priority = mem::take(&mut self.priority);
        let info = mem::take(&mut self.info);

        Ok(ReadingElement {
            text,
            no_kanji: self.no_kanji,
            reading_string,
            priority,
            info,
        })
    }
}
