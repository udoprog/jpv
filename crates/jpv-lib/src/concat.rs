use core::fmt;
use std::hash::{Hash, Hasher};

/// A concatenation of multiple borrowed strings with fixed size storage.
#[derive(Clone, Copy)]
pub struct Concat<'a, const N: usize> {
    storage: [&'a str; N],
    len: usize,
}

impl<const N: usize> Default for Concat<'_, N> {
    #[inline]
    fn default() -> Self {
        Self::empty()
    }
}

impl<'a, const N: usize> Concat<'a, N> {
    /// Concatenate the given strings together into a single composite string.
    pub const fn new(string: &str) -> Concat<'_, N> {
        let mut this = Concat {
            storage: [""; N],
            len: 0,
        };

        if !string.is_empty() {
            this.storage[0] = string;
            this.len = 1;
        }

        this
    }

    /// Concatenate the given strings together into a single composite string.
    pub const fn empty() -> Concat<'a, N> {
        Concat {
            storage: [""; N],
            len: 0,
        }
    }

    /// Push the given string onto storage.
    pub fn push(&mut self, string: &'a str) {
        if !string.is_empty() {
            assert!(self.len < N, "Capacity overflow");
            self.storage[self.len] = string;
            self.len += 1;
        }
    }

    /// Iterate over strings.
    pub fn as_slice(&self) -> &[&'a str] {
        &self.storage[..self.len]
    }

    /// Iterate over characters in the composite word.
    pub fn chars(&self) -> impl Iterator<Item = char> + '_ {
        self.as_slice().iter().flat_map(|s| s.chars())
    }

    /// Test if concat is empty.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

/// Concatenate the given strings together into a single composite string.
impl<'a, const N: usize> FromIterator<&'a str> for Concat<'a, N> {
    fn from_iter<I: IntoIterator<Item = &'a str>>(iter: I) -> Self {
        let mut this = Self::empty();

        for string in iter {
            if !string.is_empty() {
                this.push(string);
            }
        }

        this
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

impl<const N: usize> fmt::Debug for Concat<'_, N> {
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
        IntoIter {
            iter: self.storage.into_iter().take(self.len),
        }
    }
}

/// Iterator over a concat string.
pub struct IntoIter<'a, const N: usize> {
    iter: std::iter::Take<std::array::IntoIter<&'a str, N>>,
}

impl<'a, const N: usize> Iterator for IntoIter<'a, N> {
    type Item = &'a str;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}
