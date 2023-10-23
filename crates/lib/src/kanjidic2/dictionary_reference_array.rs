use core::mem::take;

use anyhow::Result;

use crate::kanjidic2::dictionary_reference::{self, DictionaryReference};

#[derive(Debug, Default)]
enum State<'a> {
    #[default]
    Root,
    DicRef(dictionary_reference::Builder<'a>),
}

#[derive(Debug, Default)]
pub(crate) struct Builder<'a> {
    state: State<'a>,
    values: Vec<DictionaryReference<'a>>,
}

impl<'a> Builder<'a> {
    builder! {
        self => Vec<DictionaryReference<'a>> {
            "dic_ref", DicRef, value => {
                self.values.push(value);
            }
        }
    }

    /// Build an [`DicRef`].
    fn build(&mut self) -> Result<Vec<DictionaryReference<'a>>> {
        Ok(take(&mut self.values))
    }
}
