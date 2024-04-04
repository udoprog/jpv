#[cfg(test)]
mod tests;

use core::fmt;
use core::slice;

use crate::concat::{self, Concat};
use crate::kana::{is_hiragana, is_katakana};

/// An iterator over furigana groups.
#[derive(Clone, Copy)]
pub struct Furigana<'a, const N: usize, const S: usize> {
    kanji: Concat<'a, N>,
    reading: Concat<'a, N>,
    suffix: Concat<'a, S>,
}

impl<'a> Furigana<'a, 1, 1> {
    /// Construct a new furigana wrapper based on an exact combo of kanji and
    /// reading.
    pub const fn new(kanji: &'a str, reading: &'a str, suffix: &'a str) -> Self {
        Self {
            kanji: Concat::new(kanji),
            reading: Concat::new(reading),
            suffix: Concat::new(suffix),
        }
    }
}

impl<'a, const N: usize, const S: usize> Furigana<'a, N, S> {
    pub(crate) const fn inner(
        kanji: Concat<'a, N>,
        reading: Concat<'a, N>,
        suffix: Concat<'a, S>,
    ) -> Self {
        Self {
            kanji,
            reading,
            suffix,
        }
    }

    /// Construct an iterator over furigana groups.
    pub fn iter(&self) -> Iter<'_, 'a, N, S> {
        Iter::new(self.kanji.as_slice(), self.reading.as_slice(), self.suffix)
    }

    /// Access underlying kanji.
    pub fn kanji(&self) -> Concat<'a, 6> {
        Concat::from_iter(
            self.kanji
                .as_slice()
                .iter()
                .copied()
                .chain(self.suffix.as_slice().iter().copied()),
        )
    }

    /// Access underlying reading.
    pub fn reading(&self) -> Concat<'a, 6> {
        Concat::from_iter(
            self.reading
                .as_slice()
                .iter()
                .copied()
                .chain(self.suffix.as_slice().iter().copied()),
        )
    }
}

impl<const N: usize, const S: usize> fmt::Display for Furigana<'_, N, S> {
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

fn is_kana(c: char) -> bool {
    is_hiragana(c) || is_katakana(c)
}

fn is_not_kana(c: char) -> bool {
    !is_kana(c)
}

/// A single furigana group.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FuriganaGroup<'a> {
    /// Kanji with associated kana, such as `私[わたし]`.
    Kanji(&'a str, &'a str),
    /// Literal kana, such as 'する`.
    Kana(&'a str),
}

pub struct Iter<'this, 'a, const N: usize, const S: usize> {
    kanji: slice::Iter<'this, &'a str>,
    reading: slice::Iter<'this, &'a str>,
    current: Option<(&'a str, &'a str)>,
    kana: Option<&'a str>,
    suffix: concat::IntoIter<'a, S>,
}

impl<'this, 'a, const N: usize, const S: usize> Iterator for Iter<'this, 'a, N, S> {
    type Item = FuriganaGroup<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(kana) = self.kana.take() {
            return Some(FuriganaGroup::Kana(kana));
        }

        if let Some(group) = self.group() {
            return Some(group);
        }

        let kana = self.suffix.next()?;
        Some(FuriganaGroup::Kana(kana))
    }
}

impl<'this, 'a, const N: usize, const S: usize> Iter<'this, 'a, N, S> {
    fn new(kanji: &'this [&'a str], reading: &'this [&'a str], suffix: Concat<'a, S>) -> Self {
        let mut this = Self {
            kanji: kanji.iter(),
            reading: reading.iter(),
            current: None,
            kana: None,
            suffix: suffix.into_iter(),
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

        match kanji.find(is_not_kana) {
            Some(0) => {
                // Kanji found in the leading position, so we process the
                // remaining string to test if it's all kanji or not.
                let Some((n, _)) = kanji.char_indices().find(|(_, c)| is_kana(*c)) else {
                    // Remainder is all kanji, output it as a furigana group.
                    let group = FuriganaGroup::Kanji(kanji, reading);
                    self.current = self.advance();
                    return Some(group);
                };

                // Kana found, so we extract that as a group to look for in the
                // reading group. After it has been found we emit it as a Kanji
                // group.
                let (group_kanji, trailing) = kanji.split_at(n);

                let Some(suffix) = trailing.find(is_not_kana) else {
                    // Trailing is *all* kana, so simply find its offset in the
                    // reading group and extract it.
                    let group_kana =
                        reading.get(..reading.rfind(trailing).unwrap_or(reading.len()))?;

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

/// Analyze the given inputs as furigana.
///
/// This is more accurate than [`Furigana`], but allocates and requires that the
/// inputs are contiguous strings.
pub fn furigana2<'a>(
    kanji: &'a str,
    reading: &'a str,
    mut suffix: &'a str,
) -> impl Iterator<Item = FuriganaGroup<'a>> {
    use core::mem;

    use FuriganaGroup::*;

    let mut it = kanji.chars().rev();
    let mut s = None;
    let mut i = kanji.len();

    let groups = core::iter::from_fn(|| loop {
        let Some(c) = it.next() else {
            return s.take().map(|s| 0..s);
        };

        let old = i;
        i = i - c.len_utf8();

        if is_kana(c) {
            if s.is_none() {
                s = Some(old);
            }

            continue;
        }

        if let Some(s) = s.take() {
            return Some(old..s);
        }
    });

    let mut positions = Vec::new();
    let mut p = reading.len();

    for g in groups {
        let kana = &kanji[g.clone()];

        let Some(at) = memchr::memmem::rfind(reading[..p].as_bytes(), kana.as_bytes()) else {
            positions.clear();
            break;
        };

        positions.push((kana, at, g));
        p = at;
    }

    let mut last = (0, 0);
    let mut it = positions.into_iter().rev();
    let mut deferred = None;

    core::iter::from_fn(move || {
        if let Some(group) = deferred.take() {
            return Some(group);
        }

        let (k, r) = last;

        let Some((kana, at, g)) = it.next() else {
            if !kanji[k..].is_empty() {
                last = (kanji.len(), reading.len());
                return Some(Kanji(&kanji[k..], &reading[r..]));
            }

            if !suffix.is_empty() {
                return Some(Kana(mem::take(&mut suffix)));
            }

            return None;
        };

        let ret = if kanji[k..].starts_with(kana) {
            Some(Kana(kana))
        } else {
            deferred = Some(Kana(kana));
            Some(Kanji(&kanji[k..g.start], &reading[r..at]))
        };

        last = (g.end, at + kana.len());
        ret
    })
}
