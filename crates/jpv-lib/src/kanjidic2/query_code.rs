use anyhow::{bail, Context, Result};
use musli::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::kanjidic2::array::{Element, ElementBuilder};
use crate::kanjidic2::parser::{Output, Poll};

#[derive(Default, Debug)]
pub(crate) struct Builder<'a> {
    text: Option<&'a str>,
    ty: Option<&'a str>,
    skip_misclass: Option<&'a str>,
}

#[borrowme::borrowme]
#[derive(Clone, Debug, Serialize, Deserialize, Encode, Decode)]
#[musli(packed)]
pub struct QueryCode<'a> {
    text: &'a str,
    ty: &'a str,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    skip_misclass: Option<&'a str>,
}

impl<'a> Element<'a> for QueryCode<'a> {
    const NAME: &'static str = "q_code";

    type Builder = Builder<'a>;
}

impl<'a> ElementBuilder<'a> for Builder<'a> {
    type Value = QueryCode<'a>;

    fn wants_text(&self) -> bool {
        true
    }

    fn poll(&mut self, output: Output<'a>) -> Result<Poll<QueryCode<'a>>> {
        match output {
            Output::Text(text) if self.text.is_none() => {
                self.text = Some(text);
                Ok(Poll::Pending)
            }
            Output::Attribute("qc_type", value) if self.ty.is_none() => {
                self.ty = Some(value);
                Ok(Poll::Pending)
            }
            Output::Attribute("skip_misclass", value) if self.skip_misclass.is_none() => {
                self.skip_misclass = Some(value);
                Ok(Poll::Pending)
            }
            Output::Close => Ok(Poll::Ready(QueryCode {
                text: self.text.context("missing text")?,
                ty: self.ty.context("missing `cp_type`")?,
                skip_misclass: self.skip_misclass,
            })),
            _ => {
                bail!("Unsupported {output:?}")
            }
        }
    }
}
