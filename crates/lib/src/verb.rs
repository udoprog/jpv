//! Module which performs verb conjugation, based on a words class.

use crate::conjugation::Conjugations;
use crate::elements::Entry;
use crate::entities::KanjiInfo;
use crate::kana::Word;
use crate::PartOfSpeech;

/// Try to conjugate the given entry as a verb.
pub fn conjugate<'a>(entry: &Entry<'a>) -> Option<Conjugations<'a>> {
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
            let (Some(k), Some(r)) = (kanji_text.strip_suffix('る'), reading.text.strip_suffix('る')) else {
                return None;
            };

            let conjugations = conjugations! {
                k, r,
                Present("させる"),
                Present + ?Polite("ます"),
                Command("ろ"),
                Command + ?Alternate("よ"),
                Conditional("たら"),
                Hypothetical("ば"),
                Negative("ない"),
                Passive("られる"),
                Past("た"),
                Past + ?Polite("ました"),
                Past + Negative("なかった"),
                Past + Negative + ?Polite("ませんでした"),
                Potential("られる"),
                Potential + ?Alternate("れる"),
                Tai("たい"),
                Te("て"),
                Volitional("よう"),
                Negative + ?Polite("ません"),
            };

            Some(Conjugations {
                dictionary: Word::new(kanji_text, reading.text),
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
                Present(a, "せる"),
                Present + ?Polite(i, "ます"),
                Negative(a, "ない"),
                Negative + ?Polite(i, "ません"),
                Past(past),
                Past + ?Polite(i, "ました"),
                Past + Negative(a, "なかった"),
                Past + Negative + ?Polite(i, "ませんでした"),
                Command(e),
                Conditional(past, "ら"),
                Hypothetical(e, "ば"),
                Passive(a, "れる"),
                Potential(e, "る"),
                Tai(i, "たい"),
                Te(te),
                Volitional(o, "う"),
            };

            Some(Conjugations {
                dictionary: Word::new(kanji_text, reading.text),
                conjugations,
            })
        }
        VerbKind::Suru => {
            let (Some(k), Some(r)) = (kanji_text.strip_suffix("する"), reading.text.strip_suffix("する")) else {
                return None;
            };

            let conjugations = conjugations! {
                k, r,
                Present("させる"),
                Present + ?Polite("します"),
                Negative("しない"),
                Negative + ?Polite("しません"),
                Past("した"),
                Past + ?Polite("しました"),
                Past + Negative("しなかった"),
                Past + Negative + ?Polite("しませんでした"),
                Command("しろ"),
                Conditional("したら"),
                Hypothetical("すれば"),
                Passive("される"),
                Potential("できる"),
                Tai("したい"),
                Te("して"),
                Volitional("しよう"),
            };

            Some(Conjugations {
                dictionary: Word::new(kanji_text, reading.text),
                conjugations,
            })
        }
        VerbKind::Kuru => {
            let (Some(k), Some(r)) = (kanji_text.strip_suffix("来る"), reading.text.strip_suffix("くる")) else {
                return None;
            };

            let conjugations = conjugations! {
                k, r,
                Present("来", "こ", "させる"),
                Present + ?Polite("来", "き", "ます"),
                Negative("来", "こ", "ない"),
                Negative + ?Polite("来", "き", "ません"),
                Past("来", "き", "た"),
                Past + ?Polite("来", "き", "ました"),
                Past + Negative ("来", "こ", "なかった"),
                Past + Negative + ?Polite("来", "き", "ませんでした"),
                Command("来", "こ", "い"),
                Conditional("来", "き", "たら"),
                Hypothetical("来", "く", "れば"),
                Passive("来", "こ", "られる"),
                Potential("来", "こ", "られる"),
                Tai("来", "き", "たい"),
                Te("来", "き", "て"),
                Volitional("来", "こ", "よう"),
            };

            Some(Conjugations {
                dictionary: Word::new(kanji_text, reading.text),
                conjugations,
            })
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum VerbKind {
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
fn as_verb_kind(entry: &Entry<'_>) -> Option<VerbKind> {
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
