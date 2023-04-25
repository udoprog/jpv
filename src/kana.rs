use core::fmt;

use crate::{concat::Concat, furigana::Furigana};

pub struct Word<'a> {
    /// Verb stem.
    pub text: &'a str,
    /// Furigana reading of verb stem.
    pub reading: &'a str,
}

impl<'a> Word<'a> {
    pub fn new(text: &'a str, reading: &'a str) -> Self {
        Self { text, reading }
    }

    /// Display the given combination as furigana.
    pub fn furigana(&self) -> Furigana<'a, 1> {
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
pub struct Pair<'a, const N: usize> {
    kanji: Concat<'a, N>,
    reading: Concat<'a, N>,
    // Suffix always guaranteed to be kana.
    suffix: &'a str,
}

impl<'a, const N: usize> Pair<'a, N> {
    /// Construct a kanji/reading pair with a common suffix.
    pub fn new<A, B>(kanji: A, reading: B, suffix: &'a str) -> Self
    where
        A: IntoIterator<Item = &'a str>,
        B: IntoIterator<Item = &'a str>,
    {
        Pair {
            kanji: Concat::new(kanji),
            reading: Concat::new(reading),
            suffix,
        }
    }

    pub fn furigana(&self) -> Furigana<'a, N> {
        Furigana::inner(self.kanji.clone(), self.reading.clone(), self.suffix)
    }

    /// Coerce into an iterator.
    ///
    /// We use this instead of implementing [`IntoIterator`] because it allows
    /// the caller to control the size of the constructed composites.
    pub fn into_iter<const O: usize>(self) -> impl Iterator<Item = Concat<'a, O>> {
        let kanji = Concat::<O>::new(self.kanji.strings().chain([self.suffix]));
        let reading = Concat::<O>::new(self.reading.strings().chain([self.suffix]));
        [kanji, reading].into_iter()
    }
}

impl<const N: usize> fmt::Display for Pair<'_, N> {
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
