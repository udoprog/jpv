//! Types to perform romaji - hiragana - katakana conversions.

#[macro_use]
mod table;

#[macro_use]
mod chars;

#[cfg(test)]
mod tests;

use std::array::from_fn;

#[allow(unused)]
macro_rules! hira {
    () => {
        '\u{3041}'..='\u{3096}' | '\u{3099}'..='\u{309f}'
    }
}

#[allow(unused)]
macro_rules! kana {
    () => {
        '\u{30a0}'..='\u{30ff}'
    };
}

/// A transformation to perform.
pub enum Transform {
    /// Transform to hiragana.
    Hiragana,
    /// Transform to katakana.
    Katakana,
    /// Transform to romaji.
    Romaji,
}

/// Perform an analysis.
pub fn analyze(input: &str) -> Analysis<'_> {
    Analysis { input }
}

/// A string being analyzed.
pub struct Analysis<'a> {
    input: &'a str,
}

impl<'a> Iterator for Analysis<'a> {
    type Item = Segment<'a>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.input.is_empty() {
            return None;
        }

        let mut it = self.input.chars();
        let chars: [char; 4] = from_fn(|_| it.next().unwrap_or('\0'));

        macro_rules! pattern {
            ([$a:pat, $b:pat, $c:pat, $d:pat]) => {
                [$a, $b, $c, $d]
            };
            ([$a:pat, $b:pat, $c:pat]) => {
                [$a, $b, $c, _]
            };
            ([$a:pat, $b:pat]) => {
                [$a, $b, _, _]
            };
            ([$a:pat]) => {
                [$a, _, _, _]
            };
        }

        macro_rules! implement_match {
            (
                $((
                    $n:expr,
                    $hira:tt, $kata:tt,
                    $(w = $w:tt,)*
                ),)*
                $(
                    kana ($kana:tt, $(w = $kw_:expr,)*),
                )*
            ) => {
                match chars {
                    $(
                        chars!($hira, pattern) => $hira.len(),
                        chars!($kata, pattern) => $kata.len(),
                        $(chars!($w, pattern) => $w.len(),)*
                    )*
                    $(chars!($kana, pattern) => $kana.len(),)*
                    [a, _, _, _] => a.len_utf8(),
                }
            }
        }

        let n = romaji_table!(implement_match);
        let (string, tail) = self.input.split_at(n);
        self.input = tail;
        Some(Segment { string })
    }
}

/// A section that can be restructured.
#[derive(Debug, PartialEq, Eq)]
pub struct Segment<'a> {
    string: &'a str,
}

impl<'a> Segment<'a> {
    /// Convert the analyzed segment into hiragana.
    pub fn hiragana(&self) -> &'a str {
        macro_rules! implement_match {
            (
                $((
                    $n:expr,
                    $hira:tt, $kata:tt,
                    $(w = $w:expr,)*
                ),)*
                $(kana $kana:tt,)*
            ) => {
                match &self.string[..] {
                    $(
                        $kata => $hira,
                        $($w => $hira,)*
                    )*
                    string => string,
                }
            }
        }

        romaji_table!(implement_match)
    }

    /// Convert the analyzed segment into katakana.
    pub fn katakana(&self) -> &'a str {
        macro_rules! implement_match {
            (
                $((
                    $n:expr,
                    $hira:tt, $kata:tt,
                    $(w = $w:expr,)*
                ),)*
                $(kana $kana:tt,)*
            ) => {
                match &self.string[..] {
                    $(
                        $hira => $kata,
                        $($w => $kata,)*
                    )*
                    string => string,
                }
            }
        }

        romaji_table!(implement_match)
    }

    /// Romanize the analyzed segment.
    pub fn romanize(&self) -> &'a str {
        macro_rules! implement_match {
            (
                $((
                    $n:expr,
                    $hira:tt, $kata:tt,
                    w = $w:expr,
                    $(w = $w2:expr,)*
                ),)*
                $(
                    kana ($kana:expr, w = $kw:expr, $(w = $kw_:expr,)*),
                )*
            ) => {
                match &self.string[..] {
                    $(
                        $hira => $w,
                        $kata => $w,
                    )*
                    $($kana => $kw,)*
                    string => string,
                }
            }
        }

        romaji_table!(implement_match)
    }
}

impl PartialEq<str> for Segment<'_> {
    #[inline]
    fn eq(&self, other: &str) -> bool {
        self.string == other
    }
}

impl PartialEq<&str> for Segment<'_> {
    #[inline]
    fn eq(&self, other: &&str) -> bool {
        self.string == *other
    }
}
