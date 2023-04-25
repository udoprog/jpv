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
        Furigana {
            kanji: Cow::Borrowed(self.text),
            reading: Cow::Borrowed(self.reading),
        }
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

fn is_kana(c: char) -> bool {
    matches!(c, '\u{3041}'..='\u{3096}' | '\u{3099}'..='\u{309f}' | '\u{30a0}'..='\u{30ff}')
}

/// Formatter from [`Pair::furigana`].
pub struct Furigana<'a> {
    kanji: Cow<'a, str>,
    reading: Cow<'a, str>,
}

impl Furigana<'_> {
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

        while !kanji.is_empty() {
            let index = kanji.find(|c| !is_kana(c)).unwrap_or(kanji.len());
            let (head, mut tail) = kanji.split_at(index);

            if reading.starts_with(head) {
                head.fmt(f)?;
            } else {
                '['.fmt(f)?;

                while !reading.starts_with(head) {
                    let Some(c) = reading.chars().next() else {
                        break;
                    };

                    c.fmt(f)?;
                    reading = &reading[c.len_utf8()..];
                }

                ']'.fmt(f)?;
                head.fmt(f)?;
            }

            while let Some(c) = tail.chars().next() {
                if is_kana(c) {
                    break;
                }

                c.fmt(f)?;
                tail = &tail[c.len_utf8()..];
            }

            kanji = tail;
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
