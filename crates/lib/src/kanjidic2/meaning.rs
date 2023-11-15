use anyhow::{bail, Context, Result};
use musli::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::kanjidic2::parser::{Output, Poll};

#[derive(Default, Debug)]
pub(crate) struct Builder<'a> {
    text: Option<&'a str>,
    lang: Option<&'a str>,
}

#[borrowme::borrowme]
#[derive(Clone, Debug, Serialize, Deserialize, Encode, Decode)]
#[musli(packed)]
pub struct Meaning<'a> {
    pub text: &'a str,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lang: Option<&'a str>,
}

impl<'a> Builder<'a> {
    pub(super) fn wants_text(&self) -> bool {
        true
    }

    pub(super) fn poll(&mut self, output: Output<'a>) -> Result<Poll<Meaning<'a>>> {
        match output {
            Output::Text(text) if self.text.is_none() => {
                self.text = Some(text);
                Ok(Poll::Pending)
            }
            Output::Attribute("m_lang", value) if self.lang.is_none() => {
                self.lang = Some(value);
                Ok(Poll::Pending)
            }
            Output::Close => Ok(Poll::Ready(Meaning {
                text: self.text.context("missing text")?,
                lang: self.lang,
            })),
            _ => {
                bail!("Unsupported {output:?}")
            }
        }
    }
}
