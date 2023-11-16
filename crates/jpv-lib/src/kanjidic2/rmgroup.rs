use core::mem::take;

use anyhow::Result;

use crate::kanjidic2::meaning::{self, Meaning};
use crate::kanjidic2::reading::{self, Reading};

#[derive(Debug, Default)]
enum State<'a> {
    #[default]
    Root,
    Reading(reading::Builder<'a>),
    Meaning(meaning::Builder<'a>),
}

#[derive(Debug, Default)]
pub(crate) struct Builder<'a> {
    state: State<'a>,
    readings: Vec<Reading<'a>>,
    meanings: Vec<Meaning<'a>>,
}

impl<'a> Builder<'a> {
    builder! {
        self => (Vec<Reading<'a>>, Vec<Meaning<'a>>) {
            "reading", Reading, value => {
                self.readings.push(value);
            }
            "meaning", Meaning, value => {
                self.meanings.push(value);
            }
        }
    }

    /// Build an [`DicRef`].
    fn build(&mut self) -> Result<(Vec<Reading<'a>>, Vec<Meaning<'a>>)> {
        let readings = take(&mut self.readings);
        let meanings = take(&mut self.meanings);
        Ok((readings, meanings))
    }
}
