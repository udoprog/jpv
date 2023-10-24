use core::fmt::Debug;
use core::mem::take;

use anyhow::Result;

use crate::kanjidic2::parser::{Output, Poll};

pub(crate) trait ElementBuilder<'a>: Debug + Default {
    type Value;

    fn wants_text(&self) -> bool;

    fn poll(&mut self, output: Output<'a>) -> Result<Poll<Self::Value>>;
}

pub(crate) trait Element<'a>: Sized + Debug {
    const NAME: &'static str;

    type Builder: ElementBuilder<'a, Value = Self>;
}

#[derive(Debug, Default)]
enum State<B> {
    #[default]
    Root,
    Value(B),
}

pub(crate) struct Builder<'a, E>
where
    E: Element<'a>,
{
    state: State<E::Builder>,
    values: Vec<E>,
}

impl<'a, E> Builder<'a, E>
where
    E: Element<'a>,
{
    builder! {
        self => Vec<E> {
            E::NAME, Value, value => {
                self.values.push(value);
            }
        }
    }

    /// Build an [`CodePoint`].
    fn build(&mut self) -> Result<Vec<E>> {
        Ok(take(&mut self.values))
    }
}

impl<'a, E> Debug for Builder<'a, E>
where
    E: Debug + Element<'a>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Builder")
            .field("state", &self.state)
            .field("values", &self.values)
            .finish()
    }
}

impl<'a, E> Default for Builder<'a, E>
where
    E: Element<'a>,
{
    fn default() -> Self {
        Self {
            state: State::default(),
            values: Vec::default(),
        }
    }
}
