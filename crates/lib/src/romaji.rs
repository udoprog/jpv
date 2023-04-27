//! Types to perform romaji - hiragana - katakana conversions.

#[macro_use]
mod table;

#[macro_use]
mod chars;

#[cfg(test)]
mod tests;

#[derive(PartialEq, Eq)]
#[repr(u8)]
enum Class {
    U,
    L,
    P,
    X,
}

use std::array::from_fn;

use Class::*;

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
                    $(nih = $nih:tt,)?
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
                        $(chars!($nih, pattern) => $nih.len(),)*
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
                    $(nih = $nih:expr,)?
                    $(w = $w:expr,)*
                ),)*
                $(kana $kana:tt,)*
            ) => {
                match &self.string[..] {
                    $(
                        $kata => $hira,
                        $($nih => $hira,)*
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
                    $(nih = $nih:expr,)?
                    $(w = $w:expr,)*
                ),)*
                $(kana $kana:tt,)*
            ) => {
                match &self.string[..] {
                    $(
                        $hira => $kata,
                        $($nih => $kata,)*
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
        macro_rules! out {
            (nih = $expr:expr, $($tt:tt)*) => {
                $expr
            };

            (w = $expr:expr, $($tt:tt)*) => {
                $expr
            };
        }

        macro_rules! implement_match {
            (
                $((
                    $n:expr,
                    $hira:tt, $kata:tt,
                    $(nih = $nih:expr,)?
                    $(w = $w:expr,)*
                ),)*
                $(
                    kana ($kana:expr, w = $kw:expr, $(w = $kw_:expr,)*),
                )*
            ) => {
                match &self.string[..] {
                    $(
                        $hira => out!($(nih = $nih,)* $(w = $w,)*),
                        $kata => out!($(nih = $nih,)* $(w = $w,)*),
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

const HIRA_B: usize = 0x3040;

#[rustfmt::skip]
const HIRA_T: [Class; 0x60] = [
    /*U+304x*/
    /*　*/ X, /*ぁ*/ U, /*あ*/ U, /*ぃ*/ U, /*い*/ U, /*ぅ*/ U, /*う*/ U, /*ぇ*/ U,
    /*え*/ U, /*ぉ*/ U, /*お*/ U, /*か*/ U, /*が*/ U, /*き*/ U, /*ぎ*/ U, /*く*/ U,
    /*U+305x*/
    /*ぐ*/ U, /*け*/ U, /*げ*/ U, /*こ*/ U, /*ご*/ U, /*さ*/ U, /*ざ*/ U, /*し*/ U,
    /*じ*/ U, /*す*/ U, /*ず*/ U, /*せ*/ U, /*ぜ*/ U, /*そ*/ U, /*ぞ*/ U, /*た*/ U,
    /*U+306x*/
    /*だ*/ U, /*ち*/ U, /*ぢ*/ U, /*っ*/ U, /*つ*/ U, /*づ*/ U, /*て*/ U, /*で*/ U,
    /*と*/ U, /*ど*/ U, /*な*/ U, /*に*/ U, /*ぬ*/ U, /*ね*/ U, /*の*/ U, /*は*/ U,
    /*U+307x*/
    /*ば*/ U, /*ぱ*/ U, /*ひ*/ U, /*び*/ U, /*ぴ*/ U, /*ふ*/ U, /*ぶ*/ U, /*ぷ*/ U,
    /*へ*/ U, /*べ*/ U, /*ぺ*/ U, /*ほ*/ U, /*ぼ*/ U, /*ぽ*/ U, /*ま*/ U, /*み*/ U,
    /*U+308x*/
    /*む*/ U, /*め*/ U, /*も*/ U, /*ゃ*/ L, /*や*/ U, /*ゅ*/ L, /*ゆ*/ U, /*ょ*/ L,
    /*よ*/ U, /*ら*/ U, /*り*/ U, /*る*/ U, /*れ*/ U, /*ろ*/ U, /*ゎ*/ U, /*わ*/ U,
    /*U+309x*/
    /*ゐ*/ U, /*ゑ*/ U, /*を*/ U, /*ん*/ U, /*ゔ*/ U, /*ゕ*/ U, /*ゖ*/ U, /*　*/ X,
    /*　*/ X, /*　*/ P, /*　*/ P, /*　*/ P, /*　*/ P, /*ゝ*/ P, /*ゞ*/ P, /*ゟ*/ P,
];

const KATA_B: usize = 0x30a0;

#[rustfmt::skip]
const KATA_T: [Class; 0x60] = [
    /*U+30Ax */
    /*゠*/ P, /*ァ*/ L, /*ア*/ U, /*ィ*/ L, /*イ*/ U, /*ゥ*/ L, /*ウ*/ U, /*ェ*/ L,
    /*エ*/ U, /*ォ*/ L, /*オ*/ U, /*カ*/ U, /*ガ*/ U, /*キ*/ U, /*ギ*/ U, /*ク*/ U,
    /*U+30Bx */
    /*グ*/ U, /*ケ*/ U, /*ゲ*/ U, /*コ*/ U, /*ゴ*/ U, /*サ*/ U, /*ザ*/ U, /*シ*/ U,
    /*ジ*/ U, /*ス*/ U, /*ズ*/ U, /*セ*/ U, /*ゼ*/ U, /*ソ*/ U, /*ゾ*/ U, /*タ*/ U,
    /*U+30Cx */
    /*ダ*/ U, /*チ*/ U, /*ヂ*/ U, /*ッ*/ L, /*ツ*/ U, /*ヅ*/ U, /*テ*/ U, /*デ*/ U,
    /*ト*/ U, /*ド*/ U, /*ナ*/ U, /*ニ*/ U, /*ヌ*/ U, /*ネ*/ U, /*ノ*/ U, /*ハ*/ U,
    /*U+30Dx */
    /*バ*/ U, /*パ*/ U, /*ヒ*/ U, /*ビ*/ U, /*ピ*/ U, /*フ*/ U, /*ブ*/ U, /*プ*/ U,
    /*ヘ*/ U, /*ベ*/ U, /*ペ*/ U, /*ホ*/ U, /*ボ*/ U, /*ポ*/ U, /*マ*/ U, /*ミ*/ U,
    /*U+30Ex */
    /*ム*/ U, /*メ*/ U, /*モ*/ U, /*ャ*/ L, /*ヤ*/ U, /*ュ*/ L, /*ユ*/ U, /*ョ*/ L,
    /*ヨ*/ U, /*ラ*/ U, /*リ*/ U, /*ル*/ U, /*レ*/ U, /*ロ*/ U, /*ヮ*/ L, /*ワ*/ U,
    /*U+30Fx */
    /*ヰ*/ U, /*ヱ*/ U, /*ヲ*/ U, /*ン*/ U, /*ヴ*/ U, /*ヵ*/ U, /*ヶ*/ U, /*ヷ*/ U,
    /*ヸ*/ U, /*ヹ*/ U, /*ヺ*/ U, /*・*/ P, /*ー*/ P, /*ヽ*/ P, /*ヾ*/ P, /*ヿ*/ P,
];

#[inline]
#[allow(unused)]
fn is_kana(c: char, class: Class) -> bool {
    let Ok(c) = usize::try_from(c as u32) else {
        return false;
    };

    let Some(c) = c.checked_sub(KATA_B) else {
        return false;
    };

    let Some(c) = KATA_T.get(c) else {
        return false;
    };

    *c == class
}

#[inline]
#[allow(unused)]
fn is_hira(c: char, class: Class) -> bool {
    let Ok(c) = usize::try_from(c as u32) else {
        return false;
    };

    let Some(c) = c.checked_sub(HIRA_B) else {
        return false;
    };

    let Some(c) = HIRA_T.get(c) else {
        return false;
    };

    *c == class
}
