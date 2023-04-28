//! Module which performs verb inflection, based on a words class.

mod godan;

use crate::elements::Entry;
use crate::entities::KanjiInfo;
use crate::inflection::Inflections;
use crate::kana::Word;
use crate::{Inflection, PartOfSpeech};

// Construct ichidan conjugations.
#[rustfmt::skip]
macro_rules! ichidan {
    ($target:expr, $i:expr, $pair:expr, $base:literal) => {{
        $target.insert($i | inflect!(Present + *Polite), $pair.with_suffix([concat!($base, "ます")]));
        $target.insert($i | inflect!(Negative), $pair.with_suffix([concat!($base, "ない")]));
        $target.insert($i | inflect!(Negative + *Polite), $pair.with_suffix([concat!($base, "ません")]));
        $target.insert($i | inflect!(Past), $pair.with_suffix([concat!($base, "た")]));
        $target.insert($i | inflect!(Past + *Polite), $pair.with_suffix([concat!($base, "ました")]));
        $target.insert($i | inflect!(Past + Negative), $pair.with_suffix([concat!($base, "なかった")]));
        $target.insert($i | inflect!(Past + Negative + *Polite), $pair.with_suffix([concat!($base, "ませんでした")]));
        $target.insert($i | inflect!(Command), $pair.with_suffix([concat!($base, "ろ")]));
        $target.insert($i | inflect!(Command + *Polite), $pair.with_suffix([concat!($base, "なさい")]));
        $target.insert($i | inflect!(Command + *Alternate), $pair.with_suffix([concat!($base, "よ")]));
        $target.insert($i | inflect!(Command + *Alternate + *Polite), $pair.with_suffix([concat!($base, "てください")]));
        $target.insert($i | inflect!(Command + Negative), $pair.with_suffix([concat!($base, "るな")]));
        $target.insert($i | inflect!(Command + Negative + *Polite), $pair.with_suffix([concat!($base, "ないでください")]));
        $target.insert($i | inflect!(Hypothetical), $pair.with_suffix([concat!($base, "ば")]));
        $target.insert($i | inflect!(Hypothetical + Negative), $pair.with_suffix([concat!($base, "なければ")]));
        $target.insert($i | inflect!(Conditional), $pair.with_suffix([concat!($base, "たら")]));
        $target.insert($i | inflect!(Conditional + *Polite), $pair.with_suffix([concat!($base, "ましたら")]));
        $target.insert($i | inflect!(Conditional + Negative), $pair.with_suffix([concat!($base, "なかったら")]));
        $target.insert($i | inflect!(Conditional + Negative + *Polite), $pair.with_suffix([concat!($base, "ませんでしたら")]));
        $target.insert($i | inflect!(Passive), $pair.with_suffix([concat!($base, "られる")]));
        $target.insert($i | inflect!(Passive + *Conversation), $pair.with_suffix([concat!($base, "れる")]));
        $target.insert($i | inflect!(Passive + *Polite), $pair.with_suffix([concat!($base, "られます")]));
        $target.insert($i | inflect!(Passive + Negative), $pair.with_suffix([concat!($base, "られない")]));
        $target.insert($i | inflect!(Passive + Negative + *Polite), $pair.with_suffix([concat!($base, "られません")]));
        $target.insert($i | inflect!(Potential), $pair.with_suffix([concat!($base, "られる")]));
        $target.insert($i | inflect!(Potential + *Polite), $pair.with_suffix([concat!($base, "られます")]));
        $target.insert($i | inflect!(Potential + Negative), $pair.with_suffix([concat!($base, "られない")]));
        $target.insert($i | inflect!(Potential + Negative + *Polite), $pair.with_suffix([concat!($base, "られません")]));
        $target.insert($i | inflect!(Volitional), $pair.with_suffix([concat!($base, "よう")]));
        $target.insert($i | inflect!(Volitional + *Polite), $pair.with_suffix([concat!($base, "ましょう")]));
        $target.insert($i | inflect!(Volitional + *Alternate), $pair.with_suffix([concat!($base, "るだろう")]));
        $target.insert($i | inflect!(Volitional + *Alternate + *Polite), $pair.with_suffix([concat!($base, "るでしょう")]));
        $target.insert($i | inflect!(Volitional + Negative), $pair.with_suffix([concat!($base, "ないだろう")]));
        $target.insert($i | inflect!(Volitional + Negative + *Polite), $pair.with_suffix([concat!($base, "ないでしょう")]));
        $target.insert($i | inflect!(Causative), $pair.with_suffix([concat!($base, "させる")]));
        $target.insert($i | inflect!(Tai), $pair.with_suffix([concat!($base, "たい")]));
        $target.insert($i | inflect!(Tai + Negative), $pair.with_suffix([concat!($base, "たくない")]));
        $target.insert($i | inflect!(Tai + Past), $pair.with_suffix([concat!($base, "たかった")]));
        $target.insert($i | inflect!(Tai + Past + Negative), $pair.with_suffix([concat!($base, "たくなかった")]));
    }};
}

// Construct godan conjugations.
#[rustfmt::skip]
macro_rules! godan {
    ($target:expr, $i:expr, $g:expr, $pair:expr, $base:literal) => {{
        $target.insert($i | inflect!(Present + *Polite), $pair.with_suffix([$base, $g.i, "ます"]));
        $target.insert($i | inflect!(Present), $pair.with_suffix([$base, $g.u]));
        $target.insert($i | inflect!(Negative), $pair.with_suffix([$base, $g.a, "ない"]));
        $target.insert($i | inflect!(Negative + *Polite), $pair.with_suffix([$base, $g.i, "ません"]));
        $target.insert($i | inflect!(Past), $pair.with_suffix([$base, $g.past]));
        $target.insert($i | inflect!(Past + *Polite), $pair.with_suffix([$base, $g.i, "ました"]));
        $target.insert($i | inflect!(Past + Negative), $pair.with_suffix([$base, $g.a, "なかった"]));
        $target.insert($i | inflect!(Past + Negative + *Polite), $pair.with_suffix([$base, $g.i, "ませんでした"]));
        $target.insert($i | inflect!(Command), $pair.with_suffix([$base, $g.e]));
        $target.insert($i | inflect!(Command + *Polite), $pair.with_suffix([$base, $g.i, "なさい"]));
        $target.insert($i | inflect!(Command + *Alternate + *Polite), $pair.with_suffix([$base, $g.te, "ください"]));
        $target.insert($i | inflect!(Command + Negative), $pair.with_suffix([$base, $g.u, "な"]));
        $target.insert($i | inflect!(Command + Negative + *Polite), $pair.with_suffix([$base, $g.a, "ないでください"]));
        $target.insert($i | inflect!(Hypothetical), $pair.with_suffix([$base, $g.e, "ば"]));
        $target.insert($i | inflect!(Hypothetical + Negative), $pair.with_suffix([$base, $g.a, "なければ"]));
        $target.insert($i | inflect!(Conditional), $pair.with_suffix([$base, $g.past, "ら"]));
        $target.insert($i | inflect!(Conditional + *Polite), $pair.with_suffix([$base, $g.i, "ましたら"]));
        $target.insert($i | inflect!(Conditional + Negative), $pair.with_suffix([$base, $g.a, "なかったら"]));
        $target.insert($i | inflect!(Conditional + Negative + *Polite), $pair.with_suffix([$base, $g.i, "ませんでしたら"]));
        $target.insert($i | inflect!(Passive), $pair.with_suffix([$base, $g.a, "れる"]));
        $target.insert($i | inflect!(Passive + *Polite), $pair.with_suffix([$base, $g.a, "れます"]));
        $target.insert($i | inflect!(Passive + Negative), $pair.with_suffix([$base, $g.a, "れない"]));
        $target.insert($i | inflect!(Passive + Negative + *Polite), $pair.with_suffix([$base, $g.a, "れません"]));
        $target.insert($i | inflect!(Potential), $pair.with_suffix([$base, $g.e, "る"]));
        $target.insert($i | inflect!(Potential + *Polite), $pair.with_suffix([$base, $g.e, "ます"]));
        $target.insert($i | inflect!(Potential + Negative), $pair.with_suffix([$base, $g.e, "ない"]));
        $target.insert($i | inflect!(Potential + Negative + *Polite), $pair.with_suffix([$base, $g.e, "ません"]));
        $target.insert($i | inflect!(Volitional), $pair.with_suffix([$base, $g.o, "う"]));
        $target.insert($i | inflect!(Volitional + *Polite), $pair.with_suffix([$base, $g.i, "ましょう"]));
        $target.insert($i | inflect!(Volitional + *Alternate), $pair.with_suffix([$base, $g.u, "だろう"]));
        $target.insert($i | inflect!(Volitional + *Alternate + *Polite), $pair.with_suffix([$base, $g.u, "でしょう"]));
        $target.insert($i | inflect!(Volitional + Negative), $pair.with_suffix([$base, $g.a, "ないだろう"]));
        $target.insert($i | inflect!(Volitional + Negative + *Polite), $pair.with_suffix([$base, $g.a, "ないでしょう"]));
        $target.insert($i | inflect!(Causative), $pair.with_suffix([$base, $g.a, "せる"]));
        $target.insert($i | inflect!(Tai), $pair.with_suffix([$base, $g.i, "たい"]));
        $target.insert($i | inflect!(Tai + Negative), $pair.with_suffix([$base, $g.i, "たくない"]));
        $target.insert($i | inflect!(Tai + Past), $pair.with_suffix([$base, $g.i, "たかった"]));
        $target.insert($i | inflect!(Tai + Past + Negative), $pair.with_suffix([$base, $g.i, "たくなかった"]));
    }};
}

/// Try to conjugate the given entry as a verb.
pub fn conjugate<'a>(entry: &Entry<'a>) -> Option<Inflections<'a>> {
    let (Some(kind), [kanji, ..], [reading, ..]) = (as_verb_kind(entry), &entry.kanji_elements[..], &entry.reading_elements[..]) else {
        return None;
    };

    let kanji_text = if kanji.info.contains(KanjiInfo::RareKanji) {
        reading.text
    } else {
        kanji.text
    };

    let mut inflections = match kind {
        VerbKind::Ichidan => {
            let (Some(k), Some(r)) = (kanji_text.strip_suffix('る'), reading.text.strip_suffix('る')) else {
                return None;
            };

            let mut inflections = inflections! {
                k, r,
                Te ("て"),
                Present ("る"),
            };

            let pair = pair!(k, r, "");
            ichidan!(inflections, Inflection::default(), pair, "");
            inflections
        }
        VerbKind::GodanIku => {
            let (Some(k), Some(r)) = (kanji_text.strip_suffix('く'), reading.text.strip_suffix('く')) else {
                return None;
            };

            let g = godan::IKU;

            let mut inflections = inflections! {
                k, r,
                Te (g.te),
            };

            let pair = pair!(k, r, "");
            godan!(inflections, Inflection::default(), g, pair, "");
            inflections
        }
        VerbKind::Godan => {
            let mut k = kanji_text.chars();
            let mut r = reading.text.chars();

            let g = match k.next_back() {
                Some('う') => godan::U,
                Some('つ') => godan::TSU,
                Some('る') => godan::RU,
                Some('く') => godan::KU,
                Some('ぐ') => godan::GU,
                Some('む') => godan::MU,
                Some('ぶ') => godan::BU,
                Some('ぬ') => godan::NU,
                Some('す') => godan::SU,
                _ => return None,
            };

            r.next_back();

            let k = k.as_str();
            let r = r.as_str();

            let mut inflections = inflections! {
                k, r,
                Te (g.te),
            };

            let pair = pair!(k, r, "");
            godan!(inflections, Inflection::default(), g, pair, "");
            inflections
        }
        VerbKind::Suru => {
            let (Some(k), Some(r)) = (kanji_text.strip_suffix("する"), reading.text.strip_suffix("する")) else {
                return None;
            };

            inflections! {
                k, r,
                Te ("して"),
                Present ("する"),
                Present + *Polite ("します"),
                Negative ("しない"),
                Negative + *Polite ("しません"),
                Past ("した"),
                Past + *Polite ("しました"),
                Past + Negative ("しなかった"),
                Past + Negative + *Polite ("しませんでした"),
                Command ("しろ"),
                Command + *Polite ("しなさい"),
                Command + *Alternate + *Polite ("してください"),
                Command + Negative ("するな"),
                Command + Negative + *Polite ("しないでください"),
                Hypothetical ("すれば"),
                Conditional ("したら"),
                Conditional + *Polite ("しましたら"),
                Conditional + Negative ("しなかったら"),
                Conditional + Negative + *Polite ("しませんでしたら"),
                Passive ("される"),
                Potential ("できる"),
                Potential + *Polite ("できます"),
                Potential + Negative ("できない"),
                Potential + Negative + *Polite ("できまあせん"),
                Volitional ("しよう"),
                Volitional + *Polite ("しましょう"),
                Volitional + *Alternate ("するだろう"),
                Volitional + *Alternate + *Polite ("するでしょう"),
                Volitional + Negative ("しないだろう"),
                Volitional + Negative + *Polite ("しないでしょう"),
                Volitional + Past ("したろう"),
                Volitional + Past + *Polite ("しましたろう"),
                Volitional + Past + *Alternate ("しただろう"),
                Volitional + Past + Negative ("しなかっただろう"),
                Volitional + Past + Negative + *Polite ("しなかったでしょう"),
                Causative ("させる"),
                Tai ("したい"),
                Tai + Negative ("したくない"),
                Tai + Past ("したかった"),
                Tai + Past + Negative ("したくなかった"),
            }
        }
        VerbKind::Kuru => {
            let (Some(k), Some(r)) = (kanji_text.strip_suffix("来る"), reading.text.strip_suffix("くる")) else {
                return None;
            };

            inflections! {
                k, r,
                Te ("来", "き", "て"),
                Present ("来", "く", "くる"),
                Present + *Polite ("来", "き", "ます"),
                Negative ("来", "こ", "ない"),
                Negative + *Polite ("来", "き", "ません"),
                Past ("来", "き", "た"),
                Past + *Polite ("来", "き", "ました"),
                Past + Negative ("来", "こ", "なかった"),
                Past + Negative + *Polite ("来", "き", "ませんでした"),
                Command ("来", "こ", "い"),
                Command + *Polite ("来", "き", "なさい"),
                Command + *Alternate + *Polite ("来て", "きて", "ください"),
                Command + Negative ("来", "く", "るな"),
                Command + Negative + *Polite ("来", "こ", "ないでください"),
                Hypothetical ("来", "く", "れば"),
                Conditional ("来", "き", "たら"),
                Conditional + *Polite ("来", "き", "ましたら"),
                Conditional + Negative ("来", "こ", "なかったら"),
                Conditional + Negative + *Polite ("来", "き", "ませんでしたら"),
                Passive ("来", "こ", "られる"),
                Passive + *Polite ("来", "こ", "られます"),
                Passive + Negative ("来", "こ", "られない"),
                Passive + Negative + *Polite ("来", "こ", "られません"),
                Potential ("来", "こ", "られる"),
                Volitional ("来", "こ", "よう"),
                Volitional + *Polite ("来", "き", "ましょう"),
                Volitional + *Alternate ("来", "く", "るだろう"),
                Volitional + *Alternate + *Polite ("来", "く", "るでしょう"),
                Volitional + Negative ("来", "こ", "ないだろう"),
                Volitional + Negative + *Polite ("来", "こ", "ないでしょう"),
                Causative ("来", "こ", "させる"),
                Causative + *Polite ("来", "こ", "させます"),
                Causative + Negative ("来", "こ", "させない"),
                Causative + Negative + *Polite ("来", "こ", "させません"),
                Tai ("来", "き", "たい"),
                Tai + Negative ("来", "き", "たくない"),
                Tai + Past ("来", "き", "たかった"),
                Tai + Past + Negative ("来", "き", "たくなかった"),
            }
        }
    };

    if let Some(pair) = inflections.get(&inflect!(Te)).cloned() {
        inflections.insert(inflect!(Progressive), pair.with_suffix(["いる"]));
        inflections.insert(inflect!(Progressive + *Alternate), pair.with_suffix(["る"]));
        ichidan!(inflections, inflect!(Progressive), pair, "い");
        godan!(inflections, inflect!(Resulting), godan::RU, pair, "あ");
        godan!(inflections, inflect!(Iku), godan::IKU, pair, "い");
        godan!(inflections, inflect!(Shimau), godan::U, pair, "しま");
    }

    Some(Inflections {
        dictionary: Word::new(kanji_text, reading.text),
        inflections,
    })
}

#[derive(Debug, Clone, Copy)]
enum VerbKind {
    /// Ichidan verb.
    Ichidan,
    /// Godan verb.
    Godan,
    /// Special godan verb.
    GodanIku,
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
                PartOfSpeech::VerbIchidan => VerbKind::Ichidan,
                PartOfSpeech::VerbIchidanS => VerbKind::Ichidan,
                PartOfSpeech::VerbGodanAru => VerbKind::Godan,
                PartOfSpeech::VerbGodanB => VerbKind::Godan,
                PartOfSpeech::VerbGodanG => VerbKind::Godan,
                PartOfSpeech::VerbGodanK => VerbKind::Godan,
                PartOfSpeech::VerbGodanKS => VerbKind::GodanIku,
                PartOfSpeech::VerbGodanM => VerbKind::Godan,
                PartOfSpeech::VerbGodanN => VerbKind::Godan,
                PartOfSpeech::VerbGodanR => VerbKind::Godan,
                PartOfSpeech::VerbGodanRI => VerbKind::Godan,
                PartOfSpeech::VerbGodanS => VerbKind::Godan,
                PartOfSpeech::VerbGodanT => VerbKind::Godan,
                PartOfSpeech::VerbGodanU => VerbKind::Godan,
                PartOfSpeech::VerbGodanUS => VerbKind::Godan,
                PartOfSpeech::VerbGodanUru => VerbKind::Godan,
                PartOfSpeech::VerbKuru => VerbKind::Kuru,
                PartOfSpeech::VerbSuru => VerbKind::Suru,
                PartOfSpeech::VerbSuruIncluded => VerbKind::Suru,
                PartOfSpeech::VerbSuruSpecial => VerbKind::Suru,
                _ => continue,
            };

            return Some(kind);
        }
    }

    None
}
