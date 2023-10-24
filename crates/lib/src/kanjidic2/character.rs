use core::mem::take;

use anyhow::{Context, Result};
use musli::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::kanjidic2::code_point::CodePoint;
use crate::kanjidic2::code_point_array;
use crate::kanjidic2::dictionary_reference::DictionaryReference;
use crate::kanjidic2::dictionary_reference_array;
use crate::kanjidic2::misc::{self, Misc};
use crate::kanjidic2::query_code::QueryCode;
use crate::kanjidic2::query_code_array;
use crate::kanjidic2::radical::Radical;
use crate::kanjidic2::radical_array;
use crate::kanjidic2::reading_meaning::{self, ReadingMeaning};
use crate::kanjidic2::text;

#[derive(Debug, Default)]
enum State<'a> {
    #[default]
    Root,
    Literal(text::Builder<'a>),
    CodePoint(code_point_array::Builder<'a>),
    Radical(radical_array::Builder<'a>),
    Misc(misc::Builder<'a>),
    DicNumber(dictionary_reference_array::Builder<'a>),
    QueryCode(query_code_array::Builder<'a>),
    ReadingMeaning(reading_meaning::Builder<'a>),
}

#[derive(Default)]
pub(crate) struct Builder<'a> {
    state: State<'a>,
    literal: Option<&'a str>,
    code_point: Vec<CodePoint<'a>>,
    radical: Vec<Radical<'a>>,
    misc: Option<Misc<'a>>,
    dictionary_references: Vec<DictionaryReference<'a>>,
    query_codes: Vec<QueryCode<'a>>,
    reading_meaning: Option<ReadingMeaning<'a>>,
}

#[borrowme::borrowme]
#[derive(Clone, Debug, Serialize, Deserialize, Encode, Decode)]
#[musli(packed)]
pub struct Character<'a> {
    pub literal: &'a str,
    #[serde(borrow)]
    pub code_point: Vec<CodePoint<'a>>,
    #[serde(borrow)]
    pub radical: Vec<Radical<'a>>,
    #[serde(borrow)]
    pub misc: Misc<'a>,
    #[serde(borrow)]
    pub dictionary_references: Vec<DictionaryReference<'a>>,
    #[serde(borrow)]
    pub query_codes: Vec<QueryCode<'a>>,
    #[serde(borrow)]
    pub reading_meaning: ReadingMeaning<'a>,
}

impl<'a> Builder<'a> {
    builder! {
        self => Character<'a> {
            "literal", Literal, value => {
                self.literal = Some(value);
            }
            "codepoint", CodePoint, value => {
                self.code_point.extend(value);
            }
            "radical", Radical, value => {
                self.radical.extend(value);
            }
            "misc", Misc, value => {
                self.misc = Some(value);
            }
            "dic_number", DicNumber, value => {
                self.dictionary_references.extend(value);
            }
            "query_code", QueryCode, value => {
                self.query_codes.extend(value);
            }
            "reading_meaning", ReadingMeaning, value => {
                self.reading_meaning = Some(value);
            }
        }
    }

    /// Build an [`Character`].
    fn build(&mut self) -> Result<Character<'a>> {
        Ok(Character {
            literal: self.literal.context("missing `literal`")?,
            code_point: take(&mut self.code_point),
            radical: take(&mut self.radical),
            misc: self.misc.take().context("missing `misc`")?,
            dictionary_references: take(&mut self.dictionary_references),
            query_codes: take(&mut self.query_codes),
            reading_meaning: self.reading_meaning.take().unwrap_or_default(),
        })
    }
}
