use core::fmt;

use anyhow::{bail, Result};

use crate::parser::{Output, Poll};

#[derive(Debug)]
pub struct SourceLanguage<'a> {
    pub text: Option<&'a str>,
    pub lang: Option<&'a str>,
    pub waseigo: bool,
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
            Output::Text(text) => {
                self.text = Some(text);
                Ok(Poll::Pending)
            }
            Output::Attribute("lang", value) => {
                self.lang = Some(value);
                Ok(Poll::Pending)
            }
            Output::Attribute("ls_wasei", "y") => {
                self.waseigo = true;
                Ok(Poll::Pending)
            }
            Output::Attribute("ls_type", value) => {
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
