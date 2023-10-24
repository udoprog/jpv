use core::mem::take;

use anyhow::Result;
use musli::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::kanjidic2::meaning::Meaning;
use crate::kanjidic2::reading::Reading;
use crate::kanjidic2::rmgroup;
use crate::kanjidic2::text;

#[derive(Debug, Default)]
enum State<'a> {
    #[default]
    Root,
    RmGroup(rmgroup::Builder<'a>),
    Nanori(text::Builder<'a>),
}

#[derive(Debug, Default)]
pub(crate) struct Builder<'a> {
    state: State<'a>,
    readings: Vec<Reading<'a>>,
    meanings: Vec<Meaning<'a>>,
    nanori: Vec<&'a str>,
}

#[borrowme::borrowme]
#[derive(Default, Debug, Clone, Serialize, Deserialize, Encode, Decode)]
#[musli(packed)]
pub struct ReadingMeaning<'a> {
    #[serde(borrow)]
    pub readings: Vec<Reading<'a>>,
    #[serde(borrow)]
    pub meanings: Vec<Meaning<'a>>,
    #[serde(borrow)]
    pub nanori: Vec<&'a str>,
}

impl<'a> Builder<'a> {
    builder! {
        self => ReadingMeaning<'a> {
            "rmgroup", RmGroup, (readings, meanings) => {
                self.readings.extend(readings);
                self.meanings.extend(meanings);
            }
            "nanori", Nanori, value => {
                self.nanori.push(value);
            }
        }
    }

    /// Build an [`ReadingMeaning`].
    fn build(&mut self) -> Result<ReadingMeaning<'a>> {
        Ok(ReadingMeaning {
            readings: take(&mut self.readings),
            meanings: take(&mut self.meanings),
            nanori: take(&mut self.nanori),
        })
    }
}
