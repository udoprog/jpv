use anyhow::{bail, Result};

use crate::jmdict::parser::{Output, Poll};

/// Parses an empty element.
#[derive(Debug, Default)]
pub(crate) struct Builder;

impl Builder {
    #[inline]
    pub(crate) fn wants_text(&self) -> bool {
        true
    }

    #[inline]
    pub(crate) fn poll(&mut self, output: Output<'_>) -> Result<Poll<()>> {
        match output {
            Output::Close => Ok(Poll::Ready(())),
            _ => {
                bail!("Unsupported {output:?}")
            }
        }
    }
}
