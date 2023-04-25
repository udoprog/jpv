//! Module which performs verb conjugation, based on a words class.

use std::collections::BTreeMap;

use crate::elements::Entry;
use crate::entities::KanjiInfo;
use crate::kana::{Pair, Word};
use crate::{Concat, PartOfSpeech};

/// Helper macro to build a kana pair.
macro_rules! pair {
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

/// Try to conjugate the given entry as a verb.
pub fn conjugate<'a>(entry: &Entry<'a>) -> Option<VerbConjugations<'a>> {
    let (Some(kind), [kanji, ..], [reading, ..]) = (as_verb_kind(entry), &entry.kanji_elements[..], &entry.reading_elements[..]) else {
        return None;
    };

    let kanji_text = if kanji.info.contains(KanjiInfo::RareKanji) {
        reading.text
    } else {
        kanji.text
    };

    match kind {
        VerbKind::Ichidan => {
            let mut k = kanji_text.chars();
            let mut r = reading.text.chars();

            let (Some('る'), Some('る')) = (k.next_back(), r.next_back()) else {
                return None;
            };

            let k = k.as_str();
            let r = r.as_str();

            let conjugations = conjugations! {
                k, r,
                Causative("させる"),
                Command("ろ"),
                CommandAlt("よ"),
                Conditional("たら"),
                Hypothetical("ば"),
                Negative("ない"),
                Passive("られる"),
                Past("た"),
                PastNegative("なかった"),
                Potential("られる"),
                PotentialAlt("れる"),
                Tai("たい"),
                Te("て"),
                Volitional("よう"),
                PoliteIndicative("ます"),
                PoliteNegative("ません"),
                PolitePast("ました"),
                PolitePastNegative("ませんでした"),
            };

            Some(VerbConjugations {
                dictionary: Word {
                    text: kanji_text,
                    reading: reading.text,
                },
                conjugations,
            })
        }
        VerbKind::Godan | VerbKind::GodanSpecial => {
            let mut k = kanji_text.chars();
            let mut r = reading.text.chars();

            let ([a, i, e, o], te, past) = match k.next_back() {
                Some('う') => (["わ", "い", "え", "お"], "って", "った"),
                Some('つ') => (["た", "ち", "て", "と"], "って", "った"),
                Some('る') => (["ら", "り", "れ", "ろ"], "って", "った"),
                Some('く') => (["か", "き", "け", "こ"], "いて", "いた"),
                Some('ぐ') => (["が", "ぎ", "げ", "ご"], "いで", "いだ"),
                Some('む') => (["ま", "み", "め", "も"], "んで", "んだ"),
                Some('ぶ') => (["ば", "び", "べ", "ぼ"], "んで", "んだ"),
                Some('ぬ') => (["な", "に", "ね", "の"], "んで", "んだ"),
                Some('す') => (["さ", "し", "せ", "そ"], "して", "した"),
                _ => return None,
            };

            // Special te-conjugation.
            let (te, past) = match kind {
                VerbKind::GodanSpecial => ("って", "った"),
                _ => (te, past),
            };

            r.next_back();

            let k = k.as_str();
            let r = r.as_str();

            let conjugations = conjugations! {
                k, r,
                Causative(a, "せる"),
                Command(e),
                Conditional(past, "ら"),
                Hypothetical(e, "ば"),
                Negative(a, "ない"),
                Passive(a, "れる"),
                Past(past),
                PastNegative(a, "なかった"),
                Potential(e, "る"),
                Tai(i, "たい"),
                Te(te),
                Volitional(o, "う"),
                PoliteIndicative(i, "ます"),
                PoliteNegative(i, "ません"),
                PolitePast(i, "ました"),
                PolitePastNegative(i, "ませんでした"),
            };

            Some(VerbConjugations {
                dictionary: Word {
                    text: kanji_text,
                    reading: reading.text,
                },
                conjugations,
            })
        }
        VerbKind::Suru => {
            let (Some(k), Some(r)) = (kanji_text.strip_suffix("する"), reading.text.strip_suffix("する")) else {
                return None;
            };

            let conjugations = conjugations! {
                k, r,
                Causative("させる"),
                Command("しろ"),
                Conditional("したら"),
                Hypothetical("すれば"),
                Negative("しない"),
                Passive("される"),
                Past("した"),
                PastNegative("しなかった"),
                Potential("できる"),
                Tai("したい"),
                Te("して"),
                Volitional("しよう"),
                PoliteIndicative("します"),
                PoliteNegative("しません"),
                PolitePast("しました"),
                PolitePastNegative("しませんでした"),
            };

            Some(VerbConjugations {
                dictionary: Word {
                    text: kanji_text,
                    reading: reading.text,
                },
                conjugations,
            })
        }
        VerbKind::Kuru => {
            let (Some(k), Some(r)) = (kanji_text.strip_suffix("来る"), reading.text.strip_suffix("くる")) else {
                return None;
            };

            let conjugations = conjugations! {
                k, r,
                Causative(prefix "来", "こ", "させる"),
                Command(prefix "来", "こ", "い"),
                Conditional(prefix "来", "き", "たら"),
                Hypothetical(prefix "来", "く", "れば"),
                Negative(prefix "来", "こ", "ない"),
                Passive(prefix "来", "こ", "られる"),
                Past(prefix "来", "き", "た"),
                PastNegative(prefix "来", "こ", "なかった"),
                Potential(prefix "来", "こ", "られる"),
                Tai(prefix "来", "き", "たい"),
                Te(prefix "来", "き", "て"),
                Volitional(prefix "来", "こ", "よう"),
                PoliteIndicative(prefix "来", "き", "ます"),
                PoliteNegative(prefix "来", "き", "ません"),
                PolitePast(prefix "来", "き", "ました"),
                PolitePastNegative(prefix "来", "き", "ませんでした"),
            };

            Some(VerbConjugations {
                dictionary: Word::new(kanji_text, reading.text),
                conjugations,
            })
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Conjugation {
    Causative,
    Command,
    CommandAlt,
    Conditional,
    Hypothetical,
    Indicative,
    Negative,
    Passive,
    Past,
    PastNegative,
    Potential,
    PotentialAlt,
    Tai,
    Te,
    Volitional,
    PoliteIndicative,
    PoliteNegative,
    PolitePast,
    PolitePastNegative,
}

/// A collection of conjugations.
#[non_exhaustive]
pub struct VerbConjugations<'a> {
    pub dictionary: Word<'a>,
    conjugations: BTreeMap<Conjugation, Pair<'a, 2>>,
}

impl<'a> VerbConjugations<'a> {
    /// Test if any polite conjugations exist.
    pub fn has_polite(&self) -> bool {
        for polite in [
            Conjugation::PoliteIndicative,
            Conjugation::PoliteNegative,
            Conjugation::PolitePast,
            Conjugation::PolitePastNegative,
        ] {
            if self.conjugations.contains_key(&polite) {
                return true;
            }
        }

        false
    }

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

#[derive(Debug, Clone, Copy)]
pub enum VerbKind {
    /// Ichidan verb.
    Ichidan,
    /// Godan verb.
    Godan,
    /// Special godan verb.
    GodanSpecial,
    /// Suru irregular suru verb.
    Suru,
    /// Special irregular kuru verb.
    Kuru,
}

/// If the entry is a verb, figure out the verb kind.
pub(crate) fn as_verb_kind(entry: &Entry<'_>) -> Option<VerbKind> {
    for sense in &entry.senses {
        for pos in sense.pos.iter() {
            let kind = match pos {
                PartOfSpeech::AdjectiveF => continue,
                PartOfSpeech::AdjectiveI => continue,
                PartOfSpeech::AdjectiveIx => continue,
                PartOfSpeech::AdjectiveKari => continue,
                PartOfSpeech::AdjectiveKu => continue,
                PartOfSpeech::AdjectiveNa => continue,
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
                PartOfSpeech::VerbIchidan => VerbKind::Ichidan,
                PartOfSpeech::VerbIchidanS => VerbKind::Ichidan,
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
                PartOfSpeech::VerbGodanAru => VerbKind::Godan,
                PartOfSpeech::VerbGodanB => VerbKind::Godan,
                PartOfSpeech::VerbGodanG => VerbKind::Godan,
                PartOfSpeech::VerbGodanK => VerbKind::Godan,
                PartOfSpeech::VerbGodanKS => VerbKind::GodanSpecial,
                PartOfSpeech::VerbGodanM => VerbKind::Godan,
                PartOfSpeech::VerbGodanN => VerbKind::Godan,
                PartOfSpeech::VerbGodanR => VerbKind::Godan,
                PartOfSpeech::VerbGodanRI => VerbKind::Godan,
                PartOfSpeech::VerbGodanS => VerbKind::Godan,
                PartOfSpeech::VerbGodanT => VerbKind::Godan,
                PartOfSpeech::VerbGodanU => VerbKind::Godan,
                PartOfSpeech::VerbGodanUS => VerbKind::Godan,
                PartOfSpeech::VerbGodanUru => VerbKind::Godan,
                PartOfSpeech::VerbIntransitive => continue,
                PartOfSpeech::VerbKuru => VerbKind::Kuru,
                PartOfSpeech::VerbNu => continue,
                PartOfSpeech::VerbRu => continue,
                PartOfSpeech::VerbSuru => VerbKind::Suru,
                PartOfSpeech::VerbSuC => continue,
                PartOfSpeech::VerbSuruIncluded => VerbKind::Suru,
                PartOfSpeech::VerbSuruSpecial => VerbKind::Suru,
                PartOfSpeech::VerbTransitive => continue,
                PartOfSpeech::VerbZuru => continue,
            };

            return Some(kind);
        }
    }

    None
}
