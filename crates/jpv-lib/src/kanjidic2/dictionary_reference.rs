use anyhow::{bail, Context, Result};
use musli::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::kanjidic2::array::{Element, ElementBuilder};
use crate::kanjidic2::parser::{Output, Poll};

#[derive(Default, Debug)]
pub(crate) struct Builder<'a> {
    text: Option<&'a str>,
    ty: Option<&'a str>,
    volume: Option<&'a str>,
    page: Option<&'a str>,
}

#[borrowme::borrowme]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Encode, Decode)]
#[musli(packed)]
pub struct DictionaryReference<'a> {
    text: &'a str,
    ty: &'a str,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    volume: Option<&'a str>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    page: Option<&'a str>,
}

impl<'a> Element<'a> for DictionaryReference<'a> {
    const NAME: &'static str = "dic_ref";

    type Builder = Builder<'a>;
}

impl<'a> ElementBuilder<'a> for Builder<'a> {
    type Value = DictionaryReference<'a>;

    fn wants_text(&self) -> bool {
        true
    }

    fn poll(&mut self, output: Output<'a>) -> Result<Poll<DictionaryReference<'a>>> {
        match output {
            Output::Text(text) if self.text.is_none() => {
                self.text = Some(text);
                Ok(Poll::Pending)
            }
            Output::Attribute("dr_type", value) if self.ty.is_none() => {
                self.ty = Some(value);
                Ok(Poll::Pending)
            }
            Output::Attribute("m_vol", value) if self.volume.is_none() => {
                self.volume = Some(value);
                Ok(Poll::Pending)
            }
            Output::Attribute("m_page", value) if self.page.is_none() => {
                self.page = Some(value);
                Ok(Poll::Pending)
            }
            Output::Close => Ok(Poll::Ready(DictionaryReference {
                text: self.text.context("missing text")?,
                ty: self.ty.context("missing `dr_type`")?,
                volume: self.volume,
                page: self.page,
            })),
            _ => {
                bail!("Unsupported {output:?}")
            }
        }
    }
}
