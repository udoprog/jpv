use musli::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::Weight;

#[borrowme::borrowme]
#[derive(Clone, Debug, Serialize, Deserialize, Encode, Decode)]
#[musli(mode = Binary, packed)]
pub struct Header<'a> {
    pub(super) file_version: &'a str,
    pub(super) database_version: &'a str,
    pub(super) date_of_creation: &'a str,
}

#[borrowme::borrowme]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Encode, Decode)]
#[musli(mode = Binary, packed)]
#[musli(mode = Text, name_all = "kebab-case")]
pub struct Character<'a> {
    pub literal: &'a str,
    #[borrowed_attr(serde(borrow))]
    pub code_point: Vec<CodePoint<'a>>,
    #[borrowed_attr(serde(borrow))]
    pub radical: Vec<Radical<'a>>,
    #[borrowed_attr(serde(borrow))]
    pub misc: Misc<'a>,
    #[borrowed_attr(serde(borrow))]
    pub dictionary_references: Vec<DictionaryReference<'a>>,
    #[borrowed_attr(serde(borrow))]
    pub query_codes: Vec<QueryCode<'a>>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[musli(mode = Text, default, skip_encoding_if = Vec::is_empty)]
    #[borrowed_attr(serde(borrow))]
    pub readings: Vec<Reading<'a>>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[musli(mode = Text, default, skip_encoding_if = Vec::is_empty)]
    #[borrowed_attr(serde(borrow))]
    pub meanings: Vec<Meaning<'a>>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[musli(mode = Text, default, skip_encoding_if = Vec::is_empty)]
    #[borrowed_attr(serde(borrow))]
    pub nanori: Vec<&'a str>,
}

impl Character<'_> {
    /// Entry weight.
    pub fn weight(&self, input: &str) -> Weight {
        // Boost based on exact query.
        let mut query = 1.0f32;
        // Calculate length boost.
        let length = (input.chars().count().min(10) as f32 / 10.0) * 1.2;

        if self.literal == input {
            query = query.max(3.0);
        }

        Weight::new(query * length)
    }
}

#[borrowme::borrowme]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Encode, Decode)]
#[musli(mode = Binary, packed)]
#[musli(mode = Text, name_all = "kebab-case")]
pub struct CodePoint<'a> {
    pub text: &'a str,
    #[serde(rename = "type")]
    #[musli(mode = Text, name = "type")]
    pub ty: &'a str,
}

#[borrowme::borrowme]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Encode, Decode)]
#[musli(mode = Binary, packed)]
#[musli(mode = Text, name_all = "kebab-case")]
pub struct Radical<'a> {
    pub text: &'a str,
    #[serde(rename = "type")]
    #[musli(mode = Text, name = "type")]
    pub ty: &'a str,
}

#[borrowme::borrowme]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Encode, Decode)]
#[musli(mode = Binary, packed)]
#[musli(mode = Text, name_all = "kebab-case")]
pub struct DictionaryReference<'a> {
    pub text: &'a str,
    #[serde(rename = "type")]
    #[musli(mode = Text, name = "type")]
    pub ty: &'a str,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[musli(mode = Text, default, skip_encoding_if = Option::is_none)]
    pub volume: Option<&'a str>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[musli(mode = Text, default, skip_encoding_if = Option::is_none)]
    pub page: Option<&'a str>,
}

#[borrowme::borrowme]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Encode, Decode)]
#[musli(mode = Binary, packed)]
#[musli(mode = Text, name_all = "kebab-case")]
pub struct QueryCode<'a> {
    pub text: &'a str,
    #[serde(rename = "type")]
    #[musli(mode = Text, name = "type")]
    pub ty: &'a str,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[musli(mode = Text, default, skip_encoding_if = Option::is_none)]
    pub skip_misclass: Option<&'a str>,
}

#[borrowme::borrowme]
#[derive(Default, Clone, Debug, PartialEq, Serialize, Deserialize, Encode, Decode)]
#[musli(mode = Binary, packed)]
#[musli(mode = Text, name_all = "kebab-case")]
pub struct Misc<'a> {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[musli(mode = Text, default, skip_encoding_if = Option::is_none)]
    pub grade: Option<u8>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[musli(mode = Text, default, skip_encoding_if = Vec::is_empty)]
    pub stroke_counts: Vec<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[musli(mode = Text, default, skip_encoding_if = Option::is_none)]
    #[borrowed_attr(serde(borrow))]
    pub variant: Option<Variant<'a>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[musli(mode = Text, default, skip_encoding_if = Option::is_none)]
    pub freq: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[musli(mode = Text, default, skip_encoding_if = Option::is_none)]
    pub jlpt: Option<u8>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[musli(mode = Text, default, skip_encoding_if = Vec::is_empty)]
    pub radical_names: Vec<&'a str>,
}

#[borrowme::borrowme]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Encode, Decode)]
#[musli(mode = Binary, packed)]
#[musli(mode = Text, name_all = "kebab-case")]
pub struct Variant<'a> {
    pub text: &'a str,
    #[serde(rename = "type")]
    #[musli(mode = Text, name = "type")]
    pub ty: &'a str,
}

#[borrowme::borrowme]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Encode, Decode)]
#[musli(mode = Binary, packed)]
#[musli(mode = Text, name_all = "kebab-case")]
pub struct Reading<'a> {
    pub text: &'a str,
    #[serde(rename = "type")]
    #[musli(mode = Text, name = "type")]
    pub ty: &'a str,
}

#[borrowme::borrowme]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Encode, Decode)]
#[musli(mode = Binary, packed)]
#[musli(mode = Text, name_all = "kebab-case")]
pub struct Meaning<'a> {
    pub text: &'a str,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[musli(mode = Text, default, skip_encoding_if = Option::is_none)]
    pub lang: Option<&'a str>,
}
