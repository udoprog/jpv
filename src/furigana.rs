#[cfg(test)]
mod tests;

use core::fmt;

use crate::composite::Composite;

/// An iterator over furigana groups.
pub struct Furigana<'a, const N: usize> {
    kanji: Composite<'a, N>,
    reading: Composite<'a, N>,
    suffix: &'a str,
}

impl<'a, const N: usize> Furigana<'a, N> {
    /// Construct a new furigana wrapper based on an exact combo of kanji and
    /// reading.
    pub fn new(kanji: &'a str, reading: &'a str) -> Self {
        Self {
            kanji: Composite::new([kanji]),
            reading: Composite::new([reading]),
            suffix: "",
        }
    }

    pub(crate) fn inner(
        kanji: Composite<'a, N>,
        reading: Composite<'a, N>,
        suffix: &'a str,
    ) -> Self {
        Self {
            kanji,
            reading,
            suffix,
        }
    }

    /// Access underlying kanji.
    pub fn kanji(&self) -> Composite<'a, 3> {
        Composite::new(self.kanji.strings().chain([self.suffix]))
    }

    /// Access underlying reading.
    pub fn reading(&self) -> Composite<'a, 3> {
        Composite::new(self.reading.strings().chain([self.suffix]))
    }
}

impl<const N: usize> fmt::Display for Furigana<'_, N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (mut kanji, mut reading) in self.kanji.strings().zip(self.reading.strings()) {
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
        }

        self.suffix.fmt(f)?;
        Ok(())
    }
}

fn is_kanji(c: char) -> bool {
    matches!(c, '\u{4e00}'..='\u{9faf}')
}
