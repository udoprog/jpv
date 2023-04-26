use anyhow::{bail, Context, Result};

use crate::parser::{Output, Poll};

/// Parses a plain text element.
#[derive(Debug, Default)]
pub(super) struct Builder<'a> {
    text: Option<&'a str>,
}

impl<'a> Builder<'a> {
    #[inline]
    pub(super) fn wants_text(&self) -> bool {
        true
    }

    #[inline]
    pub(super) fn poll(&mut self, output: Output<'a>) -> Result<Poll<&'a str>> {
        match output {
            Output::Text(text) => {
                self.text = Some(text);
                Ok(Poll::Pending)
            }
            Output::Close => {
                let text = self.text.context("missing text")?;
                Ok(Poll::Ready(text))
            }
            _ => {
                bail!("unsupported {output:?}")
            }
        }
    }
}
