macro_rules! ready {
    ($expr:expr) => {
        match $expr? {
            crate::kanjidic2::parser::Poll::Ready(ready) => ready,
            crate::kanjidic2::parser::Poll::Pending => {
                return Ok(crate::kanjidic2::parser::Poll::Pending)
            }
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

        pub(crate) fn poll(&mut $self, output: crate::kanjidic2::parser::Output<'a>) -> Result<crate::kanjidic2::parser::Poll<$return>> {
            tracing::trace!(state = ?$self.state, ?output);

            match &mut $self.state {
                State::Root => match output {
                    $(crate::kanjidic2::parser::Output::Open($name) => {
                        $self.state = State::$variant(Default::default());
                        return Ok(crate::kanjidic2::parser::Poll::Pending);
                    })*
                    crate::kanjidic2::parser::Output::Close => {
                        return Ok(crate::kanjidic2::parser::Poll::Ready($self.build()?));
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
                    return Ok(crate::kanjidic2::parser::Poll::Pending);
                })*
            }
        }
    }
}

pub use self::parser::Parser;
mod parser;

pub use self::character::Character;
mod character;

pub use self::header::Header;
mod header;

mod code_point_array;
mod radical_array;

pub use self::code_point::CodePoint;
mod code_point;

pub use self::radical::Radical;
mod radical;

pub use self::misc::Misc;
mod misc;

pub use self::variant::Variant;
mod variant;

mod dictionary_reference_array;

pub use self::dictionary_reference::DictionaryReference;
mod dictionary_reference;

mod query_code_array;

pub use self::query_code::QueryCode;
mod query_code;

pub use self::reading_meaning::ReadingMeaning;
mod reading_meaning;

mod rmgroup;

pub use self::reading::Reading;
mod reading;

pub use self::meaning::Meaning;
mod meaning;

mod text;
