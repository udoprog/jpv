mod example_sent;
mod example_source;

use std::mem;

use anyhow::Result;
use musli::{Decode, Encode};
use serde::{Deserialize, Serialize};

pub use self::example_sent::{ExampleSent, OwnedExampleSent};
pub use self::example_source::{ExampleSource, OwnedExampleSource};
use crate::elements::text;

#[derive(Clone, Debug, Serialize, Deserialize, Encode, Decode)]
#[musli(packed)]
#[owned::to_owned]
pub struct Example<'a> {
    #[serde(borrow)]
    #[to_owned(ty = Vec<OwnedExampleSent>)]
    pub sent: Vec<ExampleSent<'a>>,
    #[serde(borrow)]
    #[to_owned(ty = Vec<OwnedExampleSource>)]
    pub sources: Vec<ExampleSource<'a>>,
    #[serde(borrow)]
    #[to_owned(ty = Vec<String>)]
    pub texts: Vec<&'a str>,
}

#[derive(Debug, Default)]
enum State<'a> {
    #[default]
    Root,
    Sent(example_sent::Builder<'a>),
    Source(example_source::Builder<'a>),
    Text(text::Builder<'a>),
}

#[derive(Debug, Default)]
pub(super) struct Builder<'a> {
    state: State<'a>,
    sent: Vec<ExampleSent<'a>>,
    sources: Vec<ExampleSource<'a>>,
    texts: Vec<&'a str>,
}

impl<'a> Builder<'a> {
    builder! {
        self => Example<'a> {
            "ex_sent", Sent, value => {
                self.sent.push(value);
            }
            "ex_srce", Source, value => {
                self.sources.push(value);
            }
            "ex_text", Text, value => {
                self.texts.push(value);
            }
        }
    }

    fn build(&mut self) -> Result<Example<'a>> {
        Ok(Example {
            sent: mem::take(&mut self.sent),
            sources: mem::take(&mut self.sources),
            texts: mem::take(&mut self.texts),
        })
    }
}
