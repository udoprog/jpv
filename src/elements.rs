macro_rules! ready {
    ($expr:expr) => {
        match $expr? {
            crate::parser::Poll::Ready(ready) => ready,
            crate::parser::Poll::Pending => return Ok(crate::parser::Poll::Pending),
        }
    };
}

macro_rules! builder {
    ($self:ident => $return:ty { $($name:expr, $variant:ident, $var:pat => $action:expr),* $(,)? }) => {
        pub(crate) fn wants_text(&self) -> bool {
            match &self.state {
                State::Root => false,
                $(State::$variant(builder) => builder.wants_text(),)*
            }
        }

        pub(crate) fn poll(&mut $self, output: Output<'a>) -> Result<Poll<$return>> {
            tracing::trace!(state = ?$self.state, ?output);

            match &mut $self.state {
                State::Root => match output {
                    $(Output::Open($name) => {
                        $self.state = State::$variant(Default::default());
                        return Ok(Poll::Pending);
                    })*
                    Output::Close => {
                        return Ok(Poll::Ready($self.build()?));
                    }
                    output => {
                        ::anyhow::bail!("Unsupported {output:?}")
                    }
                }
                $(State::$variant(builder) => {
                    let span = ::tracing::info_span!($name);
                    let _enter = span.enter();
                    let $var = ready!(builder.poll(output));
                    $action;
                    $self.state = State::Root;
                    return Ok(Poll::Pending);
                })*
            }
        }
    }
}

pub(crate) mod empty;
pub(crate) mod entry;
pub(crate) mod gloss;
pub(crate) mod kanji_element;
pub(crate) mod reading_element;
pub(crate) mod sense;
pub(crate) mod source_language;
pub(crate) mod text;

pub use self::entry::Entry;
pub use self::gloss::Gloss;
pub use self::kanji_element::KanjiElement;
pub use self::reading_element::ReadingElement;
pub use self::sense::Sense;
pub use self::source_language::SourceLanguage;
