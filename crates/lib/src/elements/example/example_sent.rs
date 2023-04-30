use anyhow::{bail, Context, Result};
use musli::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::parser::{Output, Poll};

#[owned::owned]
#[derive(Clone, Debug, Serialize, Deserialize, Encode, Decode)]
#[musli(packed)]
pub struct ExampleSent<'a> {
    #[owned(ty = String)]
    pub text: &'a str,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[owned(ty = Option<String>)]
    pub lang: Option<&'a str>,
}

#[derive(Debug, Default)]
pub(super) struct Builder<'a> {
    text: Option<&'a str>,
    lang: Option<&'a str>,
}

impl<'a> Builder<'a> {
    pub(super) fn wants_text(&self) -> bool {
        true
    }

    pub(super) fn poll(&mut self, output: Output<'a>) -> Result<Poll<ExampleSent<'a>>> {
        match output {
            Output::Text(text) if self.text.is_none() => {
                self.text = Some(text);
                Ok(Poll::Pending)
            }
            Output::Attribute("lang", value) if self.lang.is_none() => {
                self.lang = Some(value);
                Ok(Poll::Pending)
            }
            Output::Close => Ok(Poll::Ready(ExampleSent {
                text: self.text.context("missing text")?,
                lang: self.lang,
            })),
            _ => {
                bail!("Unsupported {output:?}")
            }
        }
    }
}
