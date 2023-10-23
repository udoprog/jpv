use core::mem::take;

use anyhow::Result;

use crate::kanjidic2::radical::{self, Radical};

#[derive(Debug, Default)]
enum State<'a> {
    #[default]
    Root,
    Radical(radical::Builder<'a>),
}

#[derive(Debug, Default)]
pub(crate) struct Builder<'a> {
    state: State<'a>,
    values: Vec<Radical<'a>>,
}

impl<'a> Builder<'a> {
    builder! {
        self => Vec<Radical<'a>> {
            "rad_value", Radical, value => {
                self.values.push(value);
            }
        }
    }

    /// Build an [`Radical`].
    fn build(&mut self) -> Result<Vec<Radical<'a>>> {
        Ok(take(&mut self.values))
    }
}
