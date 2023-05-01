use core::fmt;

use anyhow::{bail, Result};
use musli::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::parser::{Output, Poll};

#[borrowme::borrowme]
#[derive(Clone, Debug, Serialize, Deserialize, Encode, Decode)]
#[musli(packed)]
pub struct SourceLanguage<'a> {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[owned(Option<String>)]
    pub text: Option<&'a str>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[owned(Option<String>)]
    pub lang: Option<&'a str>,
    #[copy]
    pub waseigo: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[owned(Option<String>)]
    pub ty: Option<&'a str>,
}

impl<'a> SourceLanguage<'a> {
    /// Debug the source language  element, while avoiding formatting elements
    /// which are not defined.
    pub fn debug_sparse(&self) -> impl fmt::Debug + '_ {
        DebugSparse(self)
    }
}

struct DebugSparse<'a>(&'a SourceLanguage<'a>);

impl fmt::Debug for DebugSparse<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut f = f.debug_struct("SourceLanguage");

        if let Some(field) = self.0.text {
            f.field("text", &field);
        }

        if let Some(field) = self.0.lang {
            f.field("lang", &field);
        }

        f.field("lang", &self.0.waseigo);

        if let Some(field) = self.0.ty {
            f.field("ty", &field);
        }

        f.finish_non_exhaustive()
    }
}

#[derive(Debug, Default)]
pub(super) struct Builder<'a> {
    text: Option<&'a str>,
    lang: Option<&'a str>,
    waseigo: bool,
    ty: Option<&'a str>,
}

impl<'a> Builder<'a> {
    pub(super) fn wants_text(&self) -> bool {
        true
    }

    pub(super) fn poll(&mut self, output: Output<'a>) -> Result<Poll<SourceLanguage<'a>>> {
        match output {
            Output::Text(text) if self.text.is_none() => {
                self.text = Some(text);
                Ok(Poll::Pending)
            }
            Output::Attribute("lang", value) if self.lang.is_none() => {
                self.lang = Some(value);
                Ok(Poll::Pending)
            }
            Output::Attribute("ls_wasei", "y") if !self.waseigo => {
                self.waseigo = true;
                Ok(Poll::Pending)
            }
            Output::Attribute("ls_type", value) if self.ty.is_none() => {
                self.ty = Some(value);
                Ok(Poll::Pending)
            }
            Output::Close => Ok(Poll::Ready(SourceLanguage {
                text: self.text,
                lang: self.lang,
                waseigo: self.waseigo,
                ty: self.ty,
            })),
            _ => {
                bail!("Unsupported {output:?}")
            }
        }
    }
}
