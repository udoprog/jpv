use anyhow::{bail, Context, Result};
use musli::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::kanjidic2::array::{Element, ElementBuilder};
use crate::kanjidic2::parser::{Output, Poll};

#[derive(Default, Debug)]
pub(crate) struct Builder<'a> {
    text: Option<&'a str>,
    ty: Option<&'a str>,
}

#[borrowme::borrowme]
#[derive(Clone, Debug, Serialize, Deserialize, Encode, Decode)]
#[musli(packed)]
pub struct CodePoint<'a> {
    text: &'a str,
    ty: &'a str,
}

impl<'a> Element<'a> for CodePoint<'a> {
    const NAME: &'static str = "cp_value";

    type Builder = Builder<'a>;
}

impl<'a> ElementBuilder<'a> for Builder<'a> {
    type Value = CodePoint<'a>;

    fn wants_text(&self) -> bool {
        true
    }

    fn poll(&mut self, output: Output<'a>) -> Result<Poll<CodePoint<'a>>> {
        match output {
            Output::Text(text) if self.text.is_none() => {
                self.text = Some(text);
                Ok(Poll::Pending)
            }
            Output::Attribute("cp_type", value) if self.ty.is_none() => {
                self.ty = Some(value);
                Ok(Poll::Pending)
            }
            Output::Close => Ok(Poll::Ready(CodePoint {
                text: self.text.context("missing text")?,
                ty: self.ty.context("missing `cp_type`")?,
            })),
            _ => {
                bail!("Unsupported {output:?}")
            }
        }
    }
}
