use core::fmt;
use std::hash::{Hash, Hasher};

/// Concatenate the given strings together into a single composite string.
pub fn comp<'a, I>(iter: I) -> Composite<'a>
where
    I: IntoIterator<Item = &'a str>,
{
    Composite {
        storage: iter.into_iter().collect(),
    }
}

/// A composite string composed of multiple borrowed strings concatenated togeter.
#[derive(Clone)]
pub struct Composite<'a> {
    storage: arrayvec::ArrayVec<&'a str, 4>,
}

impl<'a> Composite<'a> {
    /// Iterate over characters in the composite word.
    pub fn chars(&self) -> impl Iterator<Item = char> + '_ {
        self.storage.iter().flat_map(|s| s.chars())
    }
}

impl<'a> PartialEq for Composite<'a> {
    fn eq(&self, other: &Self) -> bool {
        let a = self.chars();
        let b = other.chars();
        a.eq(b)
    }
}

impl<'a> Eq for Composite<'a> {}

impl Hash for Composite<'_> {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        for c in self.chars() {
            c.hash(state);
        }
    }
}

impl fmt::Display for Composite<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for string in &self.storage {
            string.fmt(f)?;
        }

        Ok(())
    }
}
