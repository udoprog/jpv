use core::mem::take;

use anyhow::Result;
use musli::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::kanjidic2::text;
use crate::kanjidic2::variant::{self, Variant};

#[derive(Debug, Default)]
enum State<'a> {
    #[default]
    Root,
    Grade(text::Builder<'a>),
    StrokeCount(text::Builder<'a>),
    Variant(variant::Builder<'a>),
    Freq(text::Builder<'a>),
    Jlpt(text::Builder<'a>),
    RadicalName(text::Builder<'a>),
}

#[derive(Debug, Default)]
pub(crate) struct Builder<'a> {
    state: State<'a>,
    grade: Option<u8>,
    stroke_count: Option<u8>,
    variant: Option<Variant<'a>>,
    freq: Option<u32>,
    jlpt: Option<u8>,
    radical_names: Vec<&'a str>,
}

#[borrowme::borrowme]
#[derive(Clone, Debug, Serialize, Deserialize, Encode, Decode)]
#[musli(packed)]
pub struct Misc<'a> {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    grade: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    stroke_count: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[borrowed_attr(serde(borrow))]
    variant: Option<Variant<'a>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    freq: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    jlpt: Option<u8>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    radical_names: Vec<&'a str>,
}

impl<'a> Builder<'a> {
    builder! {
        self => Misc<'a> {
            "grade", Grade, value => {
                self.grade = Some(value.parse()?);
            }
            "stroke_count", StrokeCount, value => {
                self.stroke_count = Some(value.parse()?);
            }
            "variant", Variant, value => {
                self.variant = Some(value);
            }
            "freq", Freq, value => {
                self.freq = Some(value.parse()?);
            }
            "jlpt", Jlpt, value => {
                self.jlpt = Some(value.parse()?);
            }
            "rad_name", RadicalName, value => {
                self.radical_names.push(value);
            }
        }
    }

    /// Build a [`Misc`].
    fn build(&mut self) -> Result<Misc<'a>> {
        Ok(Misc {
            grade: self.grade,
            stroke_count: self.stroke_count,
            variant: self.variant.take(),
            freq: self.freq,
            jlpt: self.jlpt,
            radical_names: take(&mut self.radical_names),
        })
    }
}
