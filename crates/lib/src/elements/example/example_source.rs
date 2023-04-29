use anyhow::{bail, Context, Result};
use musli::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::parser::{Output, Poll};

#[derive(Clone, Debug, Serialize, Deserialize, Encode, Decode)]
#[musli(packed)]
#[owned::to_owned]
pub struct ExampleSource<'a> {
    #[to_owned(ty = String)]
    pub text: &'a str,
    #[to_owned(ty = Option<String>)]
    pub ty: Option<&'a str>,
}

#[derive(Debug, Default)]
pub(super) struct Builder<'a> {
    text: Option<&'a str>,
    ty: Option<&'a str>,
}

impl<'a> Builder<'a> {
    pub(super) fn wants_text(&self) -> bool {
        true
    }

    pub(super) fn poll(&mut self, output: Output<'a>) -> Result<Poll<ExampleSource<'a>>> {
        match output {
            Output::Text(text) if self.text.is_none() => {
                self.text = Some(text);
                Ok(Poll::Pending)
            }
            Output::Attribute("exsrc_type", value) if self.ty.is_none() => {
                self.ty = Some(value);
                Ok(Poll::Pending)
            }
            Output::Close => Ok(Poll::Ready(ExampleSource {
                text: self.text.context("missing text")?,
                ty: self.ty,
            })),
            _ => {
                bail!("Unsupported {output:?}")
            }
        }
    }
}
