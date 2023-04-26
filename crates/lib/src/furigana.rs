#[cfg(test)]
mod tests;

use core::fmt;
use std::slice;

use crate::concat::Concat;

/// An iterator over furigana groups.
pub struct Furigana<'a, const N: usize> {
    kanji: Concat<'a, N>,
    reading: Concat<'a, N>,
    suffix: &'a str,
}

impl<'a> Furigana<'a, 1> {
    /// Construct a new furigana wrapper based on an exact combo of kanji and
    /// reading.
    pub fn new(kanji: &'a str, reading: &'a str) -> Self {
        Self {
            kanji: Concat::new([kanji]),
            reading: Concat::new([reading]),
            suffix: "",
        }
    }
}

impl<'a, const N: usize> Furigana<'a, N> {
    pub(crate) fn inner(kanji: Concat<'a, N>, reading: Concat<'a, N>, suffix: &'a str) -> Self {
        Self {
            kanji,
            reading,
            suffix,
        }
    }

    /// Construct an iterator over furigana groups.
    pub fn iter(&self) -> Iter<'_, 'a, N> {
        Iter::new(self.kanji.as_slice(), self.reading.as_slice(), self.suffix)
    }

    /// Access underlying kanji.
    pub fn kanji(&self) -> Concat<'a, 3> {
        Concat::new(self.kanji.as_slice().iter().copied().chain([self.suffix]))
    }

    /// Access underlying reading.
    pub fn reading(&self) -> Concat<'a, 3> {
        Concat::new(self.reading.as_slice().iter().copied().chain([self.suffix]))
    }
}

impl<const N: usize> fmt::Display for Furigana<'_, N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for group in self.iter() {
            match group {
                FuriganaGroup::Kanji(kanji, kana) => {
                    write!(f, "{kanji}[{kana}]")?;
                }
                FuriganaGroup::Kana(kana) => {
                    write!(f, "{kana}")?;
                }
            }
        }

        Ok(())
    }
}

fn is_kanji(c: char) -> bool {
    matches!(c, '\u{4e00}'..='\u{9faf}')
}

/// A single furigana group.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FuriganaGroup<'a> {
    /// Kanji with associated kana, such as `私[わたし]`.
    Kanji(&'a str, &'a str),
    /// Literal kana, such as 'する`.
    Kana(&'a str),
}

pub struct Iter<'this, 'a, const N: usize> {
    kanji: slice::Iter<'this, &'a str>,
    reading: slice::Iter<'this, &'a str>,
    current: Option<(&'a str, &'a str)>,
    kana: Option<&'a str>,
    suffix: Option<&'a str>,
}

impl<'this, 'a, const N: usize> Iterator for Iter<'this, 'a, N> {
    type Item = FuriganaGroup<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(kana) = self.kana.take() {
            return Some(FuriganaGroup::Kana(kana));
        }

        if let Some(group) = self.group() {
            return Some(group);
        }

        let kana = self.suffix.take()?;
        Some(FuriganaGroup::Kana(kana))
    }
}

impl<'this, 'a, const N: usize> Iter<'this, 'a, N> {
    fn new(kanji: &'this [&'a str], reading: &'this [&'a str], suffix: &'a str) -> Self {
        let mut this = Self {
            kanji: kanji.iter(),
            reading: reading.iter(),
            current: None,
            kana: None,
            suffix: (!suffix.is_empty()).then_some(suffix),
        };

        this.current = this.advance();
        this
    }

    fn advance(&mut self) -> Option<(&'a str, &'a str)> {
        Some((self.kanji.next()?, self.reading.next()?))
    }

    fn group(&mut self) -> Option<FuriganaGroup<'a>> {
        // NB: We only use `split_at` for instances where the index originates
        // from the string being split. Kanji and reading strings are not
        // strictly guaranteed to be the same and might not overlap in necessary
        // lengths.

        let (kanji, reading) = self.current?;

        match kanji.find(is_kanji) {
            Some(0) => {
                // Kanji found in the first position, so we process the
                // remaining string to test if it's all kanji or not.
                let Some((n, _)) = kanji.char_indices().find(|(_, c)| !is_kanji(*c)) else {
                    // Remainder is all kanji, output it as a furigana group.
                    let group = FuriganaGroup::Kanji(kanji, reading);
                    self.current = self.advance();
                    return Some(group);
                };

                // Kana found, so we extract that as a group to look for in the
                // reading group. After it has been found we emit it as a Kanji
                // group.
                let (group_kanji, trailing) = kanji.split_at(n);

                let Some(suffix) = trailing.find(is_kanji) else {
                    // Trailing is *all* kanji, so simply find its offset in the
                    // reading group and extract it.
                    let group_kana =
                        reading.get(..reading.find(trailing).unwrap_or(reading.len()))?;

                    self.kana = Some(trailing);
                    self.current = self.advance();
                    return Some(FuriganaGroup::Kanji(group_kanji, group_kana));
                };

                let (kana_suffix, remaining_kanji) = trailing.split_at(suffix);

                let (group_kana, remaining_kana) =
                    reading.split_at(reading.find(kana_suffix).unwrap_or(reading.len()));

                // Store the immediate kana suffix to avoid having to do that
                // work again, this will be emitted in the next iteration.
                self.kana = Some(kana_suffix);
                self.current = Some((remaining_kanji, remaining_kana.get(kana_suffix.len()..)?));
                Some(FuriganaGroup::Kanji(group_kanji, group_kana))
            }
            Some(n) => {
                // Kanji found, but it's prefixed by Kana which we need to emit
                // right away.
                let kana = reading.get(..n)?;
                self.current = Some((kanji.get(n..)?, reading.get(n..)?));
                Some(FuriganaGroup::Kana(kana))
            }
            None => {
                // Kanji not found, so the remaining output must be kana. Emit
                // all of it and advance iteration.
                self.current = self.advance();
                Some(FuriganaGroup::Kana(reading))
            }
        }
    }
}
