use core::mem::take;

use anyhow::Result;

use crate::kanjidic2::query_code::{self, QueryCode};

#[derive(Debug, Default)]
enum State<'a> {
    #[default]
    Root,
    QueryCode(query_code::Builder<'a>),
}

#[derive(Debug, Default)]
pub(crate) struct Builder<'a> {
    state: State<'a>,
    values: Vec<QueryCode<'a>>,
}

impl<'a> Builder<'a> {
    builder! {
        self => Vec<QueryCode<'a>> {
            "q_code", QueryCode, value => {
                self.values.push(value);
            }
        }
    }

    /// Build an array of [`QueryCode`].
    fn build(&mut self) -> Result<Vec<QueryCode<'a>>> {
        Ok(take(&mut self.values))
    }
}
