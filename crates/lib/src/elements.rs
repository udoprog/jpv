macro_rules! ready {
    ($expr:expr) => {
        match $expr? {
            crate::parser::Poll::Ready(ready) => ready,
            crate::parser::Poll::Pending => return Ok(crate::parser::Poll::Pending),
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

        pub(crate) fn poll(&mut $self, output: crate::parser::Output<'a>) -> Result<crate::parser::Poll<$return>> {
            tracing::trace!(state = ?$self.state, ?output);

            match &mut $self.state {
                State::Root => match output {
                    $(crate::parser::Output::Open($name) => {
                        $self.state = State::$variant(Default::default());
                        return Ok(crate::parser::Poll::Pending);
                    })*
                    crate::parser::Output::Close => {
                        return Ok(crate::parser::Poll::Ready($self.build()?));
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
                    return Ok(crate::parser::Poll::Pending);
                })*
            }
        }
    }
}

pub(crate) mod empty;
pub(crate) mod entry;
pub(crate) mod example;
pub(crate) mod gloss;
pub(crate) mod kanji_element;
pub(crate) mod reading_element;
pub(crate) mod sense;
pub(crate) mod source_language;
pub(crate) mod text;

pub use self::entry::{Entry, EntryKey};
pub use self::example::{
    Example, ExampleSent, ExampleSource, OwnedExample, OwnedExampleSent, OwnedExampleSource,
};
pub use self::gloss::{Glossary, OwnedGlossary};
pub use self::kanji_element::{KanjiElement, OwnedKanjiElement};
pub use self::reading_element::{OwnedReadingElement, ReadingElement};
pub use self::sense::{OwnedSense, Sense};
pub use self::source_language::{OwnedSourceLanguage, SourceLanguage};
