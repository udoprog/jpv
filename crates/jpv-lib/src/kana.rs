use core::fmt;

use crate::concat::Concat;
use crate::furigana::Furigana;

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Class {
    // Upper hiragana.
    U,
    // Lower hiragana used for composite kana.
    L,
    // Not hiragana (don't remember if there's another meaning).
    P,
    // Not hiragana.
    X,
}

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
    pub fn furigana(&self) -> Furigana<'a, 1, 1> {
        Furigana::new(self.text, self.reading, self.suffix)
    }
}

impl OwnedFull {
    /// Display the given combination as furigana.
    pub fn furigana(&self) -> Furigana<'_, 1, 1> {
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

    pub fn furigana(&self) -> Furigana<'a, 3, 4> {
        Furigana::inner(self.text, self.reading, self.suffix)
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

mod tables {
    use super::Class::*;

    pub(super) static HIRA_B: usize = 0x3040;

    #[rustfmt::skip]
    pub(super) static HIRA_T: [super::Class; 0x60] = [
        /*U+304x*/
        /*　*/ X, /*ぁ*/ U, /*あ*/ U, /*ぃ*/ L, /*い*/ U, /*ぅ*/ L, /*う*/ U, /*ぇ*/ L,
        /*え*/ U, /*ぉ*/ L, /*お*/ U, /*か*/ U, /*が*/ U, /*き*/ U, /*ぎ*/ U, /*く*/ U,
        /*U+305x*/
        /*ぐ*/ U, /*け*/ U, /*げ*/ U, /*こ*/ U, /*ご*/ U, /*さ*/ U, /*ざ*/ U, /*し*/ U,
        /*じ*/ U, /*す*/ U, /*ず*/ U, /*せ*/ U, /*ぜ*/ U, /*そ*/ U, /*ぞ*/ U, /*た*/ U,
        /*U+306x*/
        /*だ*/ U, /*ち*/ U, /*ぢ*/ U, /*っ*/ L, /*つ*/ U, /*づ*/ U, /*て*/ U, /*で*/ U,
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

    pub(super) static KATA_B: usize = 0x30a0;

    #[rustfmt::skip]
    pub(super) static KATA_T: [super::Class; 0x60] = [
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
}

/// Test if a character is neither katakana nor hiragana assuming it is
/// otherwise Japanese script.
pub fn is_kanji(c: char) -> bool {
    !is_hiragana(c) && !is_katakana(c)
}

/// Test if something is katakana.
pub fn is_katakana(c: char) -> bool {
    matches!(get_katakana(c), Some(c) if matches!(c, Class::U | Class::L))
}

/// Test if a character is hiragana.
pub fn is_hiragana(c: char) -> bool {
    matches!(get_hiragana(c), Some(c) if matches!(c, Class::U | Class::L))
}

/// Test if something belongs to the upper hirgana class.
#[inline]
pub fn is_hiragana_upper(c: char) -> bool {
    matches!(get_hiragana(c), Some(c) if matches!(c, Class::U))
}

/// Test if something belongs to the lower hirgana class.
#[inline]
pub fn is_hiragana_lower(c: char) -> bool {
    matches!(get_hiragana(c), Some(c) if matches!(c, Class::L))
}

fn get_katakana(c: char) -> Option<Class> {
    let c = usize::try_from(c as u32).ok()?;
    let c = c.checked_sub(tables::KATA_B)?;
    Some(*tables::KATA_T.get(c)?)
}

fn get_hiragana(c: char) -> Option<Class> {
    let c = usize::try_from(c as u32).ok()?;
    let c = c.checked_sub(tables::HIRA_B)?;
    Some(*tables::HIRA_T.get(c)?)
}
