mod classify;
#[doc(inline)]
pub use self::classify::{
    is_hiragana, is_hiragana_lower, is_hiragana_upper, is_kanji, is_katakana,
};

use core::fmt;

use crate::concat::Concat;
use crate::furigana::{Furigana, OwnedFurigana};

/// A kana pair made up of complete text fragments.
#[borrowme::borrowme]
pub struct Full<'a> {
    /// Verb stem.
    pub text: &'a str,
    /// Furigana reading of verb stem.
    pub reading: &'a str,
    /// Common suffix.
    pub suffix: &'a str,
}

impl<'a> Full<'a> {
    #[inline]
    pub const fn new(text: &'a str, reading: &'a str, suffix: &'a str) -> Self {
        Self {
            text,
            reading,
            suffix,
        }
    }

    /// Display the given combination as furigana.
    pub fn furigana(&self) -> Furigana<'a> {
        Furigana::new(self.text, self.reading, self.suffix)
    }
}

impl OwnedFull {
    /// Display the given combination as furigana.
    pub fn furigana(&self) -> Furigana<'_> {
        Furigana::new(
            self.text.as_str(),
            self.reading.as_str(),
            self.suffix.as_str(),
        )
    }
}

impl fmt::Display for Full<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.text != self.reading {
            write!(
                f,
                "{}{suffix} ({}{suffix})",
                self.text,
                self.reading,
                suffix = self.suffix
            )
        } else {
            write!(f, "{}{}", self.text, self.suffix)
        }
    }
}

/// A kana pair made up of many text fragments.
#[derive(Debug, Default, Clone)]
pub struct Fragments<'a> {
    // Text prefix.
    text: Concat<'a, 3>,
    // Reading prefix.
    reading: Concat<'a, 3>,
    // Suffix always guaranteed to be kana.
    suffix: Concat<'a, 4>,
}

impl<'a> Fragments<'a> {
    /// Construct a kanji/reading pair with a common suffix.
    pub fn new<A, B, C>(text: A, reading: B, suffix: C) -> Self
    where
        A: IntoIterator<Item = &'a str>,
        B: IntoIterator<Item = &'a str>,
        C: IntoIterator<Item = &'a str>,
    {
        Fragments {
            text: Concat::from_iter(text),
            reading: Concat::from_iter(reading),
            suffix: Concat::from_iter(suffix),
        }
    }

    /// Test if fragments is empty.
    pub fn is_empty(&self) -> bool {
        self.text.is_empty() && self.suffix.is_empty()
    }

    /// Access text prefix.
    pub(crate) fn text(&self) -> &Concat<'a, 3> {
        &self.text
    }

    /// Access reading prefix.
    pub(crate) fn reading(&self) -> &Concat<'a, 3> {
        &self.reading
    }

    /// Access shared suffix.
    pub(crate) fn suffix(&self) -> &Concat<'a, 4> {
        &self.suffix
    }

    pub fn furigana(&self) -> OwnedFurigana {
        OwnedFurigana::new(self.text, self.reading, self.suffix)
    }

    /// Append suffixes to this pair.
    pub(crate) fn concat<I, T>(&self, strings: I) -> Self
    where
        I: IntoIterator<Item = &'a T>,
        T: 'a + ?Sized + AsRef<str>,
    {
        let mut suffix = self.suffix;

        for string in strings {
            suffix.push(string.as_ref());
        }

        Self {
            text: self.text,
            reading: self.reading,
            suffix,
        }
    }
}

impl fmt::Display for Fragments<'_> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            text: kanji,
            reading,
            suffix,
        } = self;

        write!(f, "{kanji}{suffix} [{reading}{suffix}]",)?;
        Ok(())
    }
}
