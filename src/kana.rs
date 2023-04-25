#[cfg(test)]
mod tests;

use core::fmt;
use std::borrow::Cow;

use crate::composite::{comp, Composite};

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

    pub fn furigana(&self) -> Furigana<'a> {
        Furigana::borrowed(self.text, self.reading)
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
pub struct Pair<'a> {
    pub kanji: Composite<'a>,
    pub reading: Composite<'a>,
}

impl<'a> Pair<'a> {
    pub fn furigana(&self) -> Furigana<'a> {
        // TODO: get rid of buffering somehow.
        let mut a = String::new();
        let mut b = String::new();

        for string in self.kanji.strings() {
            a.push_str(string);
        }

        for string in self.reading.strings() {
            b.push_str(string);
        }

        Furigana {
            kanji: Cow::Owned(a),
            reading: Cow::Owned(b),
        }
    }
}

/// Construct a kanji/reading pair.
pub fn pair<'a, A, B>(kanji: A, reading: B) -> Pair<'a>
where
    A: IntoIterator<Item = &'a str>,
    B: IntoIterator<Item = &'a str>,
{
    Pair {
        kanji: comp(kanji),
        reading: comp(reading),
    }
}

fn is_kanji(c: char) -> bool {
    matches!(c, '\u{4e00}'..='\u{9faf}')
}

/// Formatter from [`Pair::furigana`].
pub struct Furigana<'a> {
    kanji: Cow<'a, str>,
    reading: Cow<'a, str>,
}

impl<'a> Furigana<'a> {
    pub fn borrowed(kanji: &'a str, reading: &'a str) -> Self {
        Self {
            kanji: Cow::Borrowed(kanji),
            reading: Cow::Borrowed(reading),
        }
    }

    /// Access underlying kanji.
    pub fn kanji(&self) -> &str {
        self.kanji.as_ref()
    }

    /// Access underlying reading.
    pub fn reading(&self) -> &str {
        self.reading.as_ref()
    }
}

impl fmt::Display for Furigana<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut kanji = &self.kanji[..];
        let mut reading = &self.reading[..];

        let Some(mut index) = kanji.find(is_kanji) else {
            return kanji.fmt(f);
        };

        while !kanji.is_empty() {
            let (kana, mut tail) = kanji.split_at(index);

            if !reading.starts_with(kana) {
                '['.fmt(f)?;
                let mut chars = reading.chars();

                while let Some(c) = chars.next() {
                    c.fmt(f)?;

                    if chars.as_str().starts_with(kana) {
                        break;
                    }
                }

                reading = chars.as_str();
                ']'.fmt(f)?;
            }

            kana.fmt(f)?;
            reading = reading.get(kana.len()..).unwrap_or_default();

            while let Some(c) = tail.chars().next().filter(|&c| is_kanji(c)) {
                c.fmt(f)?;
                tail = tail.get(c.len_utf8()..).unwrap_or_default();
            }

            kanji = tail;
            index = kanji.find(is_kanji).unwrap_or(kanji.len());
        }

        Ok(())
    }
}

impl<'a> IntoIterator for Pair<'a> {
    type Item = Composite<'a>;
    type IntoIter = std::array::IntoIter<Composite<'a>, 2>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        [self.kanji, self.reading].into_iter()
    }
}

impl fmt::Display for Pair<'_> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.kanji != self.reading {
            write!(f, "{} ({})", self.kanji, self.reading)?;
        } else {
            self.kanji.fmt(f)?;
        }

        Ok(())
    }
}
