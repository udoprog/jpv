use core::fmt;
use std::hash::{Hash, Hasher};

/// Into iterator alias.
pub type IntoIter<'a, const N: usize> = arrayvec::IntoIter<&'a str, N>;

/// A concatenation of multiple borrowed strings with fixed size storage.
#[derive(Default, Clone)]
pub struct Concat<'a, const N: usize> {
    storage: arrayvec::ArrayVec<&'a str, N>,
}

impl<'a, const N: usize> Concat<'a, N> {
    /// Concatenate the given strings together into a single composite string.
    pub fn new<I>(iter: I) -> Concat<'a, N>
    where
        I: IntoIterator<Item = &'a str>,
    {
        Concat {
            storage: iter.into_iter().filter(|s| !s.is_empty()).collect(),
        }
    }

    /// Push the given string onto storage.
    pub fn push_str(&mut self, string: &'a str) {
        if !string.is_empty() {
            self.storage.push(string);
        }
    }

    /// Iterate over strings.
    pub fn as_slice(&self) -> &[&'a str] {
        self.storage.as_slice()
    }

    /// Iterate over characters in the composite word.
    pub fn chars(&self) -> impl Iterator<Item = char> + '_ {
        self.storage.iter().flat_map(|s| s.chars())
    }

    /// Test if concat is empty.
    pub fn is_empty(&self) -> bool {
        self.storage.iter().all(|s| s.is_empty())
    }
}

impl<'a, const A: usize, const B: usize> PartialEq<Concat<'a, A>> for Concat<'_, B> {
    fn eq(&self, other: &Concat<'a, A>) -> bool {
        let a = self.chars();
        let b = other.chars();
        a.eq(b)
    }
}

impl<const N: usize> Eq for Concat<'_, N> {}

impl<const N: usize> Hash for Concat<'_, N> {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        for c in self.chars() {
            c.hash(state);
        }
    }
}

impl<const N: usize> fmt::Display for Concat<'_, N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for string in &self.storage {
            string.fmt(f)?;
        }

        Ok(())
    }
}

impl<'a, const N: usize> IntoIterator for Concat<'a, N> {
    type Item = &'a str;
    type IntoIter = IntoIter<'a, N>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.storage.into_iter()
    }
}
