use std::mem;

use anyhow::{bail, Context, Result};
use relative_path::RelativePathBuf;
use xmlparser::{ElementEnd, Token, Tokenizer};

use crate::jmdict::entry;
use crate::jmdict::Entry;

pub(crate) enum Poll<T> {
    Ready(T),
    Pending,
}

enum State<'a> {
    /// Initial parser state.
    Initial,
    /// Inside of the root node.
    Root,
    /// Building an entry.
    Entry(entry::Builder<'a>),
}

impl State<'_> {
    fn wants_text(&self) -> bool {
        match self {
            State::Initial => false,
            State::Root => false,
            State::Entry(entry) => entry.wants_text(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum Output<'a> {
    Text(&'a str),
    Open(&'a str),
    Attribute(&'a str, &'a str),
    Close,
    Eof,
}

pub struct Parser<'a> {
    state: State<'a>,
    closed: bool,
    path: RelativePathBuf,
    input: &'a str,
    tokenizer: Tokenizer<'a>,
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            state: State::Initial,
            closed: false,
            path: RelativePathBuf::new(),
            input,
            tokenizer: Tokenizer::from(input),
        }
    }

    /// Parse the next entry.
    pub fn parse(&mut self) -> Result<Option<Entry<'a>>> {
        loop {
            let output = self.parse_next()?;

            macro_rules! open {
                ($element:pat) => {
                    if !matches!(output, Output::Open($element)) {
                        bail!(
                            "expected {} element, but found {output:?}",
                            stringify!($element)
                        );
                    }
                };
            }

            match &mut self.state {
                State::Initial => {
                    open!("JMdict");
                    self.state = State::Root;
                }
                State::Root => {
                    match output {
                        Output::Open("entry") => {}
                        Output::Close => {
                            self.state = State::Initial;
                            return Ok(None);
                        }
                        output => {
                            bail!("expected `entry` element, but found {output:?}");
                        }
                    }

                    self.state = State::Entry(entry::Builder::default());
                }
                State::Entry(builder) => {
                    let span = tracing::info_span!("entry", path = ?self.path);
                    let _enter = span.enter();

                    if let Poll::Ready(entry) =
                        builder.poll(output).with_context(|| self.path.to_owned())?
                    {
                        self.state = State::Root;
                        return Ok(Some(entry));
                    }
                }
            }
        }
    }

    fn parse_next(&mut self) -> Result<Output<'a>> {
        loop {
            if mem::take(&mut self.closed) {
                self.path.pop();
            }

            let Some(token) = self.tokenizer.next() else {
                return Ok(Output::Eof);
            };

            let wants_text = self.state.wants_text();

            match token? {
                Token::Text { text } if wants_text => {
                    let text = &self.input[text.range()];
                    return Ok(Output::Text(text));
                }
                Token::Cdata { text, .. } => {
                    let text = &self.input[text.range()];
                    return Ok(Output::Text(text));
                }
                Token::ElementStart { local, .. } => {
                    let local = &self.input[local.range()];
                    self.path.push(local);
                    tracing::trace!(path = self.path.as_str(), "enter");
                    return Ok(Output::Open(local));
                }
                Token::ElementEnd { end, .. } => {
                    if let ElementEnd::Close { .. } | ElementEnd::Empty { .. } = end {
                        tracing::trace!(path = self.path.as_str(), "leave");
                        self.closed = true;
                        return Ok(Output::Close);
                    }
                }
                Token::Attribute { local, value, .. } => {
                    let local = &self.input[local.range()];
                    let value = &self.input[value.range()];
                    return Ok(Output::Attribute(local, value));
                }
                _ => {
                    // intentionally ignore unsupported data.
                }
            }
        }
    }
}
