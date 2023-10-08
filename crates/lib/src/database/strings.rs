use std::collections::HashMap;

use anyhow::{anyhow, Context, Result};
use musli::mode::DefaultMode;
use musli_storage::int::Variable;
use musli_storage::Encoding;

/// Encoding used for storing database.
const ENCODING: Encoding<DefaultMode, Variable, Variable> = Encoding::new();

#[derive(Default)]
pub(super) struct Strings {
    data: Vec<u8>,
    lookup: HashMap<Vec<u8>, u32>,
}

impl Strings {
    /// Insert a string.
    pub fn insert<S>(&mut self, string: S) -> Result<u32>
    where
        S: AsRef<[u8]>,
    {
        let string = string.as_ref();

        if let Some(&index) = self.lookup.get(string) {
            return Ok(index);
        }

        let index = u32::try_from(self.data.len()).context("string offset overflow")?;
        self.lookup.insert(string.to_owned(), index);
        ENCODING.to_writer(&mut self.data, &string)?;
        Ok(index)
    }

    pub(super) fn as_slice(&self) -> &[u8] {
        self.data.as_slice()
    }
}

pub(super) struct StringsRef<'a> {
    data: &'a [u8],
}

impl<'a> StringsRef<'a> {
    pub(super) fn new(data: &'a [u8]) -> Self {
        tracing::info!(strings = data.len());
        Self { data }
    }

    /// Get a string at the given index.
    pub(super) fn get(&self, index: usize) -> Result<&'a [u8]> {
        let data = self
            .data
            .get(index..)
            .with_context(|| anyhow!("Missing string at {index}"))?;

        Ok(ENCODING
            .decode(data)
            .with_context(|| anyhow!("Failed to decode string at {index}"))?)
    }
}
