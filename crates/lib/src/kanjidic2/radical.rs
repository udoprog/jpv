use anyhow::{bail, Context, Result};
use musli::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::kanjidic2::parser::{Output, Poll};

#[derive(Default, Debug)]
pub(crate) struct Builder<'a> {
    text: Option<&'a str>,
    ty: Option<&'a str>,
}

#[borrowme::borrowme]
#[derive(Clone, Debug, Serialize, Deserialize, Encode, Decode)]
#[musli(packed)]
pub struct Radical<'a> {
    pub text: &'a str,
    pub ty: &'a str,
}

impl<'a> Builder<'a> {
    pub(super) fn wants_text(&self) -> bool {
        true
    }

    pub(super) fn poll(&mut self, output: Output<'a>) -> Result<Poll<Radical<'a>>> {
        match output {
            Output::Text(text) if self.text.is_none() => {
                self.text = Some(text);
                Ok(Poll::Pending)
            }
            Output::Attribute("rad_type", value) if self.ty.is_none() => {
                self.ty = Some(value);
                Ok(Poll::Pending)
            }
            Output::Close => Ok(Poll::Ready(Radical {
                text: self.text.context("missing text")?,
                ty: self.ty.context("missing `rad_type`")?,
            })),
            _ => {
                bail!("Unsupported {output:?}")
            }
        }
    }
}
