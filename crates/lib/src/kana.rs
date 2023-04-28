use core::fmt;

use crate::concat::Concat;
use crate::furigana::Furigana;

pub struct Word<'a> {
    /// Verb stem.
    pub text: &'a str,
    /// Furigana reading of verb stem.
    pub reading: &'a str,
}

impl<'a> Word<'a> {
    #[inline]
    pub const fn new(text: &'a str, reading: &'a str) -> Self {
        Self { text, reading }
    }

    /// Display the given combination as furigana.
    pub fn furigana(&self) -> Furigana<'a, 1, 0> {
        Furigana::new(self.text, self.reading)
    }
}

impl fmt::Display for Word<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.text != self.reading {
            write!(f, "{} ({})", self.text, self.reading)
        } else {
            write!(f, "{}", self.text)
        }
    }
}

/// A reading pair.
#[derive(Clone)]
pub struct Pair<'a, const N: usize, const S: usize> {
    kanji: Concat<'a, N>,
    reading: Concat<'a, N>,
    // Suffix always guaranteed to be kana.
    suffix: Concat<'a, S>,
}

impl<'a, const N: usize, const S: usize> Pair<'a, N, S> {
    /// Construct a kanji/reading pair with a common suffix.
    pub fn new<A, B, C>(kanji: A, reading: B, suffix: C) -> Self
    where
        A: IntoIterator<Item = &'a str>,
        B: IntoIterator<Item = &'a str>,
        C: IntoIterator<Item = &'a str>,
    {
        Pair {
            kanji: Concat::new(kanji),
            reading: Concat::new(reading),
            suffix: Concat::new(suffix),
        }
    }

    /// Access kanji prefix.
    pub(crate) fn kanji(&self) -> &Concat<'a, N> {
        &self.kanji
    }

    /// Access reading prefix.
    pub(crate) fn reading(&self) -> &Concat<'a, N> {
        &self.reading
    }

    /// Access shared suffix.
    pub(crate) fn suffix(&self) -> &Concat<'a, S> {
        &self.suffix
    }

    pub fn furigana(&self) -> Furigana<'a, N, S> {
        Furigana::inner(
            self.kanji.clone(),
            self.reading.clone(),
            self.suffix.clone(),
        )
    }

    /// Append suffixes to this pair.
    pub(crate) fn concat<I, T>(&self, strings: I) -> Self
    where
        I: IntoIterator<Item = &'a T>,
        T: 'a + ?Sized + AsRef<str>,
    {
        let mut suffix = self.suffix.clone();

        for string in strings {
            suffix.push_str(string.as_ref());
        }

        Self {
            kanji: self.kanji.clone(),
            reading: self.reading.clone(),
            suffix,
        }
    }
}

impl<const N: usize, const S: usize> fmt::Display for Pair<'_, N, S> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            kanji,
            reading,
            suffix,
        } = self;

        write!(f, "{kanji}{suffix} ({reading}{suffix})",)?;
        Ok(())
    }
}
