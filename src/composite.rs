use core::fmt;
use std::hash::{Hash, Hasher};

/// A composite string composed of multiple borrowed strings concatenated
/// together.
#[derive(Clone)]
pub struct Composite<'a, const N: usize> {
    storage: arrayvec::ArrayVec<&'a str, N>,
}

impl<'a, const N: usize> Composite<'a, N> {
    /// Concatenate the given strings together into a single composite string.
    pub fn new<I>(iter: I) -> Composite<'a, N>
    where
        I: IntoIterator<Item = &'a str>,
    {
        Composite {
            storage: iter.into_iter().collect(),
        }
    }

    /// Iterate over strings.
    pub fn strings(&self) -> impl Iterator<Item = &'a str> + '_ {
        self.storage.iter().copied()
    }

    /// Iterate over characters in the composite word.
    pub fn chars(&self) -> impl Iterator<Item = char> + '_ {
        self.storage.iter().flat_map(|s| s.chars())
    }
}

impl<'a, const A: usize, const B: usize> PartialEq<Composite<'a, A>> for Composite<'_, B> {
    fn eq(&self, other: &Composite<'a, A>) -> bool {
        let a = self.chars();
        let b = other.chars();
        a.eq(b)
    }
}

impl<const N: usize> Eq for Composite<'_, N> {}

impl<const N: usize> Hash for Composite<'_, N> {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        for c in self.chars() {
            c.hash(state);
        }
    }
}

impl<const N: usize> fmt::Display for Composite<'_, N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for string in &self.storage {
            string.fmt(f)?;
        }

        Ok(())
    }
}
