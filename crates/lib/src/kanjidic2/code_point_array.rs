use core::mem::take;

use anyhow::Result;

use crate::kanjidic2::code_point::{self, CodePoint};

#[derive(Debug, Default)]
enum State<'a> {
    #[default]
    Root,
    CodePointValue(code_point::Builder<'a>),
}

#[derive(Debug, Default)]
pub(crate) struct Builder<'a> {
    state: State<'a>,
    values: Vec<CodePoint<'a>>,
}

impl<'a> Builder<'a> {
    builder! {
        self => Vec<CodePoint<'a>> {
            "cp_value", CodePointValue, value => {
                self.values.push(value);
            }
        }
    }

    /// Build an [`CodePoint`].
    fn build(&mut self) -> Result<Vec<CodePoint<'a>>> {
        Ok(take(&mut self.values))
    }
}
