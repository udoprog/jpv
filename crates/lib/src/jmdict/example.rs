use std::mem;

use anyhow::Result;
use musli::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::jmdict::{example_sentence, example_source, text};
use crate::jmdict::{ExampleSentence, ExampleSource};

#[borrowme::borrowme]
#[derive(Clone, Debug, Serialize, Deserialize, Encode, Decode)]
#[musli(packed)]
pub struct Example<'a> {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[borrowed_attr(serde(borrow))]
    pub sentences: Vec<ExampleSentence<'a>>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[borrowed_attr(serde(borrow))]
    pub sources: Vec<ExampleSource<'a>>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[borrowed_attr(serde(borrow))]
    pub texts: Vec<&'a str>,
}

#[derive(Debug, Default)]
enum State<'a> {
    #[default]
    Root,
    Sent(example_sentence::Builder<'a>),
    Source(example_source::Builder<'a>),
    Text(text::Builder<'a>),
}

#[derive(Debug, Default)]
pub(super) struct Builder<'a> {
    state: State<'a>,
    sentences: Vec<ExampleSentence<'a>>,
    sources: Vec<ExampleSource<'a>>,
    texts: Vec<&'a str>,
}

impl<'a> Builder<'a> {
    builder! {
        self => Example<'a> {
            "ex_sent", Sent, value => {
                self.sentences.push(value);
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
            sentences: mem::take(&mut self.sentences),
            sources: mem::take(&mut self.sources),
            texts: mem::take(&mut self.texts),
        })
    }
}
