macro_rules! ready {
    ($expr:expr) => {
        match $expr? {
            crate::jmdict::parser::Poll::Ready(ready) => ready,
            crate::jmdict::parser::Poll::Pending => return Ok(crate::jmdict::parser::Poll::Pending),
        }
    };
}

macro_rules! builder {
    ($self:ident => $return:ty { $($name:expr, $variant:ident, $var:pat => $action:block)* }) => {
        pub(crate) fn wants_text(&self) -> bool {
            match &self.state {
                State::Root => false,
                $(State::$variant(builder) => builder.wants_text(),)*
            }
        }

        pub(crate) fn poll(&mut $self, output: crate::jmdict::parser::Output<'a>) -> Result<crate::jmdict::parser::Poll<$return>> {
            tracing::trace!(state = ?$self.state, ?output);

            match &mut $self.state {
                State::Root => match output {
                    $(crate::jmdict::parser::Output::Open($name) => {
                        $self.state = State::$variant(Default::default());
                        return Ok(crate::jmdict::parser::Poll::Pending);
                    })*
                    crate::jmdict::parser::Output::Close => {
                        return Ok(crate::jmdict::parser::Poll::Ready($self.build()?));
                    }
                    output => {
                        ::anyhow::bail!("Unsupported {output:?}")
                    }
                }
                $(State::$variant(builder) => {
                    let span = ::tracing::info_span!($name);
                    let _enter = span.enter();
                    #[allow(clippy::let_unit_value)]
                    let $var = ready!(builder.poll(output));
                    $action;
                    $self.state = State::Root;
                    return Ok(crate::jmdict::parser::Poll::Pending);
                })*
            }
        }
    }
}

pub use self::parser::Parser;
mod parser;

pub(crate) mod empty;

pub use self::entry::{Entry, OwnedEntry};
pub(crate) mod entry;

pub use self::example::{Example, OwnedExample};
pub(crate) mod example;

pub use self::example_sentence::{ExampleSentence, OwnedExampleSentence};
pub(crate) mod example_sentence;

pub use self::example_source::{ExampleSource, OwnedExampleSource};
pub(crate) mod example_source;

pub use self::gloss::{Glossary, OwnedGlossary};
pub(crate) mod gloss;

pub use self::kanji_element::{KanjiElement, OwnedKanjiElement};
pub(crate) mod kanji_element;

pub use self::reading_element::{OwnedReadingElement, ReadingElement};
pub(crate) mod reading_element;

pub use self::sense::{OwnedSense, Sense};
pub(crate) mod sense;

pub use self::source_language::{OwnedSourceLanguage, SourceLanguage};
pub(crate) mod source_language;

pub(crate) mod text;
