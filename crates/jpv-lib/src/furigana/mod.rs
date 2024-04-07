#[cfg(test)]
mod tests;

use core::fmt;
use core::ops::Range;

use crate::kana::{is_hiragana, is_katakana};
use crate::morae;

/// A string pair.
#[derive(Debug)]
pub struct Pair<'a> {
    prefix: &'a str,
    suffix: &'a str,
}

impl<'a> Pair<'a> {
    const fn new(prefix: &'a str, suffix: &'a str) -> Self {
        Self { prefix, suffix }
    }
}

impl fmt::Display for Pair<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.prefix, self.suffix)
    }
}

impl PartialEq for Pair<'_> {
    fn eq(&self, other: &Self) -> bool {
        let mn = self.prefix.len().min(other.prefix.len());

        let (a_prefix, a) = self.prefix.split_at(mn);
        let (b_prefix, b) = other.prefix.split_at(mn);

        let (a, b, prefix) = if a.is_empty() {
            (self.suffix, other.suffix, b)
        } else {
            (other.suffix, self.suffix, a)
        };

        a_prefix == b_prefix && a.strip_prefix(prefix).map_or(false, |rest| rest == b)
    }
}

impl Eq for Pair<'_> {}

impl<'a> IntoIterator for Pair<'a> {
    type Item = &'a str;
    type IntoIter = std::array::IntoIter<&'a str, 2>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        [self.prefix, self.suffix].into_iter()
    }
}

/// An iterator over furigana groups.
#[derive(Clone, Copy)]
pub struct Furigana<'a> {
    kanji: &'a str,
    reading: &'a str,
    suffix: &'a str,
}

impl<'a> Furigana<'a> {
    /// Construct a new furigana wrapper based on an exact combo of kanji and
    /// reading.
    pub const fn new(kanji: &'a str, reading: &'a str, suffix: &'a str) -> Self {
        Self {
            kanji,
            reading,
            suffix,
        }
    }

    /// Construct an iterator over furigana groups.
    pub fn iter(&self) -> impl Iterator<Item = FuriganaGroup<'a>> {
        furigana(self.kanji, self.reading, self.suffix)
    }

    /// Access underlying kanji.
    pub fn kanji(&self) -> Pair<'a> {
        Pair::new(self.kanji, self.suffix)
    }

    /// Access underlying reading.
    pub fn reading(&self) -> Pair<'a> {
        Pair::new(self.reading, self.suffix)
    }
}

pub struct OwnedFurigana {
    kanji: String,
    reading: String,
    suffix: String,
}

impl OwnedFurigana {
    pub(crate) fn new<K, R, S>(kanji: K, reading: R, suffix: S) -> Self
    where
        K: IntoIterator,
        K::Item: AsRef<str>,
        R: IntoIterator,
        R::Item: AsRef<str>,
        S: IntoIterator,
        S::Item: AsRef<str>,
    {
        fn concat<I>(iter: I) -> String
        where
            I: IntoIterator,
            I::Item: AsRef<str>,
        {
            let mut s = String::new();

            for item in iter {
                s.push_str(item.as_ref());
            }

            s
        }

        Self {
            kanji: concat(kanji),
            reading: concat(reading),
            suffix: concat(suffix),
        }
    }

    /// Construct an iterator over furigana groups.
    pub fn iter(&self) -> impl Iterator<Item = FuriganaGroup<'_>> {
        furigana(&self.kanji, &self.reading, &self.suffix)
    }

    /// Access underlying kanji.
    pub fn kanji(&self) -> Pair<'_> {
        Pair::new(&self.kanji, &self.suffix)
    }

    /// Access underlying reading.
    pub fn reading(&self) -> Pair<'_> {
        Pair::new(&self.reading, &self.suffix)
    }

    /// Borrow owned furigana.
    pub fn borrow(&self) -> Furigana<'_> {
        Furigana::new(&self.kanji, &self.reading, &self.suffix)
    }
}

impl fmt::Display for Furigana<'_> {
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

/// A single furigana group.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FuriganaGroup<'a> {
    /// Kanji with associated kana, such as `私[わたし]`.
    Kanji(&'a str, &'a str),
    /// Literal kana, such as 'する`.
    Kana(&'a str),
}

/// Partition a string by kanji and kana.
///
/// The produced iterator will yield the ranges for each kana group inside of a
/// string.
fn groups(kanji: &str) -> impl Iterator<Item = Range<usize>> + '_ {
    let mut it = kanji.chars();
    let mut e = None;
    let mut len = kanji.len();

    core::iter::from_fn(move || loop {
        let Some(c) = it.next_back() else {
            return e.take().map(|s| 0..s);
        };

        let s = len;
        len -= c.len_utf8();

        if is_kana(c) {
            if e.is_none() {
                e = Some(s);
            }
        } else if let Some(e) = e.take() {
            return Some(s..e);
        }
    })
}

fn reverse_find<'a>(
    kanji: &'a str,
    reading: &'a str,
) -> impl Iterator<Item = (&'a str, usize, Range<usize>)> {
    use memchr::memmem::rfind;

    let mut reading_len = reading.len();
    let mut kanji_len = kanji.len();

    groups(kanji).flat_map(move |g| {
        let kana = &kanji[g.start..g.end];
        let kanji_count = kanji[g.end..kanji_len].chars().count();
        let mut morae_count = 0;

        reading_len = 'out: {
            let mut current = reading_len;
            let mut last = None;

            while let Some(next) = rfind(reading[..current].as_bytes(), kana.as_bytes()) {
                let c = reading[next..].chars().next()?;

                // Special case: we're matching exact kana.
                if kanji_count == 0 {
                    break 'out next;
                }

                let reading_kana = &reading[next + c.len_utf8()..reading_len];
                morae_count += morae::iter(reading_kana).count();

                // Find the largest group that doesn't violate the heuristic that
                // each kanji has two morae.
                if morae_count > kanji_count * 2 {
                    break 'out last.unwrap_or(next);
                }

                last = Some(next);
                current = next;
            }

            last?
        };

        kanji_len = g.start;
        Some((kana, reading_len, g))
    })
}

/// Analyze the given inputs as furigana.
///
/// This is more accurate than [`Furigana`], but allocates and requires that the
/// inputs are contiguous strings.
fn furigana<'a>(
    kanji: &'a str,
    reading: &'a str,
    mut suffix: &'a str,
) -> impl Iterator<Item = FuriganaGroup<'a>> {
    use core::mem;
    use FuriganaGroup::*;

    let positions = reverse_find(kanji, reading).collect::<Vec<_>>();

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
