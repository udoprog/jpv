use core::fmt;

use anyhow::{bail, Result};

use crate::parser::{Output, Poll};

#[derive(Debug)]
pub struct Gloss<'a> {
    pub text: Option<&'a str>,
    pub lang: Option<&'a str>,
}

impl<'a> Gloss<'a> {
    /// Debug the glossary element, while avoiding formatting elements which are
    /// not defined.
    pub fn debug_sparse(&self) -> impl fmt::Debug + '_ {
        DebugSparse(self)
    }
}

struct DebugSparse<'a>(&'a Gloss<'a>);

impl fmt::Debug for DebugSparse<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut f = f.debug_struct("Gloss");

        if let Some(text) = self.0.text {
            f.field("text", &text);
        }

        if let Some(text) = self.0.lang {
            f.field("lang", &text);
        }

        f.finish()
    }
}

#[derive(Debug, Default)]
pub(super) struct Builder<'a> {
    text: Option<&'a str>,
    ty: Option<&'a str>,
    lang: Option<&'a str>,
}

impl<'a> Builder<'a> {
    pub(super) fn wants_text(&self) -> bool {
        true
    }

    pub(super) fn poll(&mut self, output: Output<'a>) -> Result<Poll<Gloss<'a>>> {
        match output {
            Output::Text(text) => {
                self.text = Some(text);
                Ok(Poll::Pending)
            }
            Output::Attribute("g_type", value) => {
                self.ty = Some(value);
                Ok(Poll::Pending)
            }
            Output::Attribute("lang", value) => {
                self.lang = Some(value);
                Ok(Poll::Pending)
            }
            Output::Close => Ok(Poll::Ready(Gloss {
                text: self.text,
                lang: self.lang,
            })),
            _ => {
                bail!("Unsupported {output:?}")
            }
        }
    }
}
