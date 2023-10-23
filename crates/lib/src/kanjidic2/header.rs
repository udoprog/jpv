use anyhow::{Context, Result};
use musli::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::kanjidic2::text;

#[derive(Debug, Default)]
enum State<'a> {
    #[default]
    Root,
    FileVersion(text::Builder<'a>),
    DatabaseVersion(text::Builder<'a>),
    DateOfCreation(text::Builder<'a>),
}

#[derive(Default)]
pub(crate) struct Builder<'a> {
    state: State<'a>,
    file_version: Option<&'a str>,
    database_version: Option<&'a str>,
    date_of_creation: Option<&'a str>,
}

#[borrowme::borrowme]
#[derive(Clone, Debug, Serialize, Deserialize, Encode, Decode)]
#[musli(packed)]
pub struct Header<'a> {
    file_version: &'a str,
    database_version: &'a str,
    date_of_creation: &'a str,
}

impl<'a> Builder<'a> {
    builder! {
        self => Header<'a> {
            "file_version", FileVersion, value => {
                self.file_version = Some(value);
            }
            "database_version", DatabaseVersion, value => {
                self.database_version = Some(value);
            }
            "date_of_creation", DateOfCreation, value => {
                self.date_of_creation = Some(value);
            }
        }
    }

    /// Build a [`Header`].
    fn build(&mut self) -> Result<Header<'a>> {
        Ok(Header {
            file_version: self.file_version.context("missing `file_version`")?,
            database_version: self
                .database_version
                .context("missing `database_version`")?,
            date_of_creation: self
                .date_of_creation
                .context("missing `date_of_creation`")?,
        })
    }
}
