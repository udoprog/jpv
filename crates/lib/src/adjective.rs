use std::collections::BTreeMap;

use musli::{Decode, Encode};

use crate::elements::Entry;
use crate::kana::{Pair, Word};
use crate::{Concat, PartOfSpeech};

/// Helper macro to build a kana pair.
macro_rules! pair {
    ($k:expr, $r:expr, special $special:expr) => {
        Pair::new([$special], [$special], "")
    };

    ($k:expr, $r:expr, prefix $a:expr, $b:expr, $suffix:expr) => {
        Pair::new([$k, $a], [$r, $b], $suffix)
    };

    ($k:expr, $r:expr, $last:expr) => {
        Pair::new([$k], [$r], $last)
    };

    ($k:expr, $r:expr, $a:expr, $last:expr) => {
        Pair::new([$k, $a], [$r, $a], $last)
    };
}

/// Setup a collection of conjugations.
macro_rules! conjugations {
    ($k:expr, $r:expr, $(
        $kind:ident ( $($tt:tt)* )
    ),* $(,)?) => {{
        let mut tree = BTreeMap::new();
        $(tree.insert(Conjugation::$kind, pair!($k, $r, $($tt)*));)*
        tree
    }};
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Encode, Decode)]
pub enum Conjugation {
    Present,
    PresentPolite,
    Past,
    PastPolite,
    Negative,
    NegativePolite,
    PastNegative,
    PastNegativePolite,
}

/// A collection of conjugations.
#[non_exhaustive]
pub struct Conjugations<'a> {
    pub dictionary: Word<'a>,
    conjugations: BTreeMap<Conjugation, Pair<'a, 2>>,
}

impl<'a> Conjugations<'a> {
    /// Get a conjugation.
    pub fn get(&self, conjugation: Conjugation) -> Option<&Pair<'a, 2>> {
        self.conjugations.get(&conjugation)
    }

    /// Iterate over all conjugations.
    pub fn iter(&self) -> impl Iterator<Item = (Conjugation, Concat<'a, 3>)> + '_ {
        self.conjugations
            .iter()
            .flat_map(|(k, p)| p.clone().into_iter().map(|p| (*k, p)))
    }
}

/// Try to conjugate the given entry as an adjective.
pub fn conjugate<'a>(entry: &Entry<'a>) -> Option<Conjugations<'a>> {
    let (Some(kind), [kanji, ..], [reading, ..]) = (as_adjective_kind(entry), &entry.kanji_elements[..], &entry.reading_elements[..]) else {
        return None;
    };

    match kind {
        AdjectiveKind::I => {
            let (Some(k), Some(r)) = (kanji.text.strip_suffix('い'), reading.text.strip_suffix('い')) else {
                return None;
            };

            let conjugations = conjugations! {
                k, r,
                Present("い"),
                PresentPolite("いです"),
                Past("かった"),
                PastPolite("かったです"),
                Negative("くない"),
                NegativePolite("くないです"),
                PastNegative("なかった"),
                PastNegativePolite("なかったです"),
            };

            Some(Conjugations {
                dictionary: Word {
                    text: kanji.text,
                    reading: reading.text,
                },
                conjugations,
            })
        }
        AdjectiveKind::Yoi => {
            let (Some(k), Some(r)) = (kanji.text.strip_suffix("いい"), reading.text.strip_suffix("いい")) else {
                return None;
            };

            let conjugations = conjugations! {
                k, r,
                Present("いい"),
                PresentPolite("いいです"),
                Past("よかった"),
                PastPolite("よかったです"),
                Negative("よくない"),
                NegativePolite("よくないです"),
                PastNegative("よなかった"),
                PastNegativePolite("よなかったです"),
            };

            Some(Conjugations {
                dictionary: Word::new(kanji.text, reading.text),
                conjugations,
            })
        }
        AdjectiveKind::Na => {
            let conjugations = conjugations! {
                kanji.text, reading.text,
                Present("だ"),
                PresentPolite("です"),
                Past("だった"),
                PastPolite("でした"),
                Negative("ではない"),
                NegativePolite("ではありません"),
                PastNegative("ではなかった"),
                PastNegativePolite("ではありませんでした"),
            };

            Some(Conjugations {
                dictionary: Word::new(kanji.text, reading.text),
                conjugations,
            })
        }
    }
}

enum AdjectiveKind {
    /// An i-adjective.
    I,
    /// Special yoi / ii class.
    Yoi,
    /// Na-adjective.
    Na,
}

/// If the entry is an adjective, figure out the adjective kind.
fn as_adjective_kind(entry: &Entry<'_>) -> Option<AdjectiveKind> {
    for sense in &entry.senses {
        for pos in sense.pos.iter() {
            let kind = match pos {
                PartOfSpeech::AdjectiveF => continue,
                PartOfSpeech::AdjectiveI => AdjectiveKind::I,
                PartOfSpeech::AdjectiveIx => AdjectiveKind::Yoi,
                PartOfSpeech::AdjectiveKari => continue,
                PartOfSpeech::AdjectiveKu => continue,
                PartOfSpeech::AdjectiveNa => AdjectiveKind::Na,
                PartOfSpeech::AdjectiveNari => continue,
                PartOfSpeech::AdjectiveNo => continue,
                PartOfSpeech::AdjectivePn => continue,
                PartOfSpeech::AdjectiveShiku => continue,
                PartOfSpeech::AdjectiveT => continue,
                PartOfSpeech::Adverb => continue,
                PartOfSpeech::AdverbTo => continue,
                PartOfSpeech::Auxiliary => continue,
                PartOfSpeech::AuxiliaryAdjective => continue,
                PartOfSpeech::AuxiliaryVerb => continue,
                PartOfSpeech::Conjunction => continue,
                PartOfSpeech::Copular => continue,
                PartOfSpeech::Counter => continue,
                PartOfSpeech::Expression => continue,
                PartOfSpeech::Interjection => continue,
                PartOfSpeech::Noun => continue,
                PartOfSpeech::NounAdverbial => continue,
                PartOfSpeech::NounProper => continue,
                PartOfSpeech::NounPrefix => continue,
                PartOfSpeech::NounSuffix => continue,
                PartOfSpeech::NounTemporal => continue,
                PartOfSpeech::Numeric => continue,
                PartOfSpeech::Pronoun => continue,
                PartOfSpeech::Prefix => continue,
                PartOfSpeech::Particle => continue,
                PartOfSpeech::Suffix => continue,
                PartOfSpeech::Unclassified => continue,
                PartOfSpeech::VerbUnspecified => continue,
                PartOfSpeech::VerbIchidan => continue,
                PartOfSpeech::VerbIchidanS => continue,
                PartOfSpeech::VerbNidanAS => continue,
                PartOfSpeech::VerbNidanBK => continue,
                PartOfSpeech::VerbNidanBS => continue,
                PartOfSpeech::VerbNidanDK => continue,
                PartOfSpeech::VerbNidanDS => continue,
                PartOfSpeech::VerbNidanGK => continue,
                PartOfSpeech::VerbNidanGS => continue,
                PartOfSpeech::VerbNidanHK => continue,
                PartOfSpeech::VerbNidanHS => continue,
                PartOfSpeech::VerbNidanKK => continue,
                PartOfSpeech::VerbNidanKS => continue,
                PartOfSpeech::VerbNidanMK => continue,
                PartOfSpeech::VerbNidanMS => continue,
                PartOfSpeech::VerbNidanNS => continue,
                PartOfSpeech::VerbNidanRK => continue,
                PartOfSpeech::VerbNidanRS => continue,
                PartOfSpeech::VerbNidanSS => continue,
                PartOfSpeech::VerbNidanTK => continue,
                PartOfSpeech::VerbNidanTS => continue,
                PartOfSpeech::VerbNidanWS => continue,
                PartOfSpeech::VerbNidanYK => continue,
                PartOfSpeech::VerbNidanYS => continue,
                PartOfSpeech::VerbNidanZS => continue,
                PartOfSpeech::VerbYodanB => continue,
                PartOfSpeech::VerbYodanG => continue,
                PartOfSpeech::VerbYodanH => continue,
                PartOfSpeech::VerbYodanK => continue,
                PartOfSpeech::VerbYodanM => continue,
                PartOfSpeech::VerbYodanN => continue,
                PartOfSpeech::VerbYodanR => continue,
                PartOfSpeech::VerbYodanS => continue,
                PartOfSpeech::VerbYodanT => continue,
                PartOfSpeech::VerbGodanAru => continue,
                PartOfSpeech::VerbGodanB => continue,
                PartOfSpeech::VerbGodanG => continue,
                PartOfSpeech::VerbGodanK => continue,
                PartOfSpeech::VerbGodanKS => continue,
                PartOfSpeech::VerbGodanM => continue,
                PartOfSpeech::VerbGodanN => continue,
                PartOfSpeech::VerbGodanR => continue,
                PartOfSpeech::VerbGodanRI => continue,
                PartOfSpeech::VerbGodanS => continue,
                PartOfSpeech::VerbGodanT => continue,
                PartOfSpeech::VerbGodanU => continue,
                PartOfSpeech::VerbGodanUS => continue,
                PartOfSpeech::VerbGodanUru => continue,
                PartOfSpeech::VerbIntransitive => continue,
                PartOfSpeech::VerbKuru => continue,
                PartOfSpeech::VerbNu => continue,
                PartOfSpeech::VerbRu => continue,
                PartOfSpeech::VerbSuru => continue,
                PartOfSpeech::VerbSuC => continue,
                PartOfSpeech::VerbSuruIncluded => continue,
                PartOfSpeech::VerbSuruSpecial => continue,
                PartOfSpeech::VerbTransitive => continue,
                PartOfSpeech::VerbZuru => continue,
            };

            return Some(kind);
        }
    }

    None
}
