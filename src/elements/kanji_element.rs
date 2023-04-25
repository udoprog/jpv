use core::fmt;
use core::mem;

use crate::elements::text;
use crate::entities::KanjiInfo;
use crate::priority::Priority;

use anyhow::ensure;
use anyhow::{anyhow, Context, Result};
use fixed_map::Set;

#[derive(Debug)]
pub struct KanjiElement<'a> {
    pub text: &'a str,
    pub priority: Vec<Priority>,
    pub info: Set<KanjiInfo>,
}

impl<'a> KanjiElement<'a> {
    /// Debug the kanji element, while avoiding formatting elements which are
    /// not defined.
    pub fn debug_sparse(&self) -> impl fmt::Debug + '_ {
        DebugSparse(self)
    }
}

struct DebugSparse<'a>(&'a KanjiElement<'a>);

impl fmt::Debug for DebugSparse<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut f = f.debug_struct("KanjiElement");

        f.field("text", &self.0.text);

        if !self.0.priority.is_empty() {
            f.field("priority", &self.0.priority);
        }

        if !self.0.info.is_empty() {
            f.field("info", &self.0.info);
        }

        f.finish_non_exhaustive()
    }
}

#[derive(Debug, Default)]
enum State<'a> {
    #[default]
    Root,
    Text(text::Builder<'a>),
    Priority(text::Builder<'a>),
    Information(text::Builder<'a>),
}

#[derive(Debug, Default)]
pub(super) struct Builder<'a> {
    state: State<'a>,
    text: Option<&'a str>,
    priority: Vec<Priority>,
    info: Set<KanjiInfo>,
}

impl<'a> Builder<'a> {
    builder! {
        self => KanjiElement<'a> {
            "keb", Text, value => {
                ensure!(self.text.is_none(), "Only one keb element allowed");
                self.text = Some(value);
            }
            "ke_pri", Priority, value => {
                let priority = Priority::parse(value).with_context(|| anyhow!("Unsupported priority `{value}`"))?;
                self.priority.push(priority);
            }
            "ke_inf", Information, value => {
                let info = KanjiInfo::parse(value).with_context(|| anyhow!("Unsupported kanji info `{value}`"))?;
                self.info.insert(info);
            }
        }
    }

    fn build(&mut self) -> Result<KanjiElement<'a>> {
        let text = self.text.context("missing text")?;
        let priority = mem::take(&mut self.priority);

        Ok(KanjiElement {
            text,
            priority,
            info: self.info,
        })
    }
}
