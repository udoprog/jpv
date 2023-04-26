//! Module which performs verb inflection, based on a words class.

use crate::elements::Entry;
use crate::entities::KanjiInfo;
use crate::inflection::Inflections;
use crate::kana::Word;
use crate::PartOfSpeech;

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

    match kind {
        VerbKind::Ichidan => {
            let (Some(k), Some(r)) = (kanji_text.strip_suffix('る'), reading.text.strip_suffix('る')) else {
                return None;
            };

            let inflections = inflections! {
                k, r,
                Te ("て"),
                Present ("る"),
                Present + *Polite ("ます"),
                Negative ("ない"),
                Negative + *Polite ("ません"),
                Past ("た"),
                Past + *Polite ("ました"),
                Past + Negative ("なかった"),
                Past + Negative + *Polite ("ませんでした"),
                Command ("ろ"),
                Command + *Polite ("なさい"),
                Command + *Alternate ("よ"),
                Command + *Alternate + *Polite ("てください"),
                Command + Negative ("るな"),
                Command + Negative + *Polite ("ないでください"),
                Hypothetical ("ば"),
                Hypothetical + Negative ("なければ"),
                Conditional ("たら"),
                Conditional + *Polite ("ましたら"),
                Conditional + Negative ("なかったら"),
                Conditional + Negative + *Polite ("ませんでしたら"),
                Passive ("られる"),
                Passive + *Conversation ("れる"),
                Passive + *Polite ("られます"),
                Passive + Negative ("られない"),
                Passive + Negative + *Polite ("られません"),
                Potential ("られる"),
                Potential + *Polite ("られます"),
                Potential + Negative ("られない"),
                Potential + Negative + *Polite ("られません"),
                Volitional ("よう"),
                Volitional + *Polite ("ましょう"),
                Volitional + *Alternate ("るだろう"),
                Volitional + *Alternate + *Polite ("るでしょう"),
                Volitional + Negative ("ないだろう"),
                Volitional + Negative + *Polite ("ないでしょう"),
                Causative ("させる"),
                Tai ("たい"),
                Tai + Negative ("たくない"),
                Tai + Past ("たかった"),
                Tai + Past + Negative ("たくなかった"),
            };

            Some(Inflections {
                dictionary: Word::new(kanji_text, reading.text),
                inflections,
            })
        }
        VerbKind::Godan | VerbKind::GodanSpecial => {
            let mut k = kanji_text.chars();
            let mut r = reading.text.chars();

            let ([a, i, u, e, o], te, past) = match k.next_back() {
                Some('う') => (["わ", "い", "う", "え", "お"], "って", "った"),
                Some('つ') => (["た", "ち", "つ", "て", "と"], "って", "った"),
                Some('る') => (["ら", "り", "る", "れ", "ろ"], "って", "った"),
                Some('く') => (["か", "き", "く", "け", "こ"], "いて", "いた"),
                Some('ぐ') => (["が", "ぎ", "ぐ", "げ", "ご"], "いで", "いだ"),
                Some('む') => (["ま", "み", "む", "め", "も"], "んで", "んだ"),
                Some('ぶ') => (["ば", "び", "ぶ", "べ", "ぼ"], "んで", "んだ"),
                Some('ぬ') => (["な", "に", "ぬ", "ね", "の"], "んで", "んだ"),
                Some('す') => (["さ", "し", "す", "せ", "そ"], "して", "した"),
                _ => return None,
            };

            // Special te-inflection.
            let (te, past) = match kind {
                VerbKind::GodanSpecial => ("って", "った"),
                _ => (te, past),
            };

            r.next_back();

            let k = k.as_str();
            let r = r.as_str();

            let inflections = inflections! {
                k, r,
                Te (te),
                Present (u),
                Present + *Polite (i, "ます"),
                Negative (a, "ない"),
                Negative + *Polite (i, "ません"),
                Past (past),
                Past + *Polite (i, "ました"),
                Past + Negative (a, "なかった"),
                Past + Negative + *Polite (i, "ませんでした"),
                Command (e),
                Command + *Polite (i, "なさい"),
                Command + *Alternate + *Polite (te, "ください"),
                Command + Negative (u, "な"),
                Command + Negative + *Polite (a, "ないでください"),
                Hypothetical (e, "ば"),
                Hypothetical + Negative (a, "なければ"),
                Conditional (past, "ら"),
                Conditional + *Polite (i, "ましたら"),
                Conditional + Negative (a, "なかったら"),
                Conditional + Negative + *Polite (i, "ませんでしたら"),
                Passive (a, "れる"),
                Passive + *Polite (a, "れます"),
                Passive + Negative (a, "れない"),
                Passive + Negative + *Polite (a, "れません"),
                Potential (e, "る"),
                Potential + *Polite (e, "ます"),
                Potential + Negative (e, "ない"),
                Potential + Negative + *Polite (e, "ません"),
                Volitional (o, "う"),
                Volitional + *Polite (i, "ましょう"),
                Volitional + *Alternate (u, "だろう"),
                Volitional + *Alternate + *Polite (u, "でしょう"),
                Volitional + Negative (a, "ないだろう"),
                Volitional + Negative + *Polite (a, "ないでしょう"),
                Causative (a, "せる"),
                Tai (i, "たい"),
                Tai + Negative (i, "たくない"),
                Tai + Past (i, "たかった"),
                Tai + Past + Negative (i, "たくなかった"),
            };

            Some(Inflections {
                dictionary: Word::new(kanji_text, reading.text),
                inflections,
            })
        }
        VerbKind::Suru => {
            let (Some(k), Some(r)) = (kanji_text.strip_suffix("する"), reading.text.strip_suffix("する")) else {
                return None;
            };

            let inflections = inflections! {
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
            };

            Some(Inflections {
                dictionary: Word::new(kanji_text, reading.text),
                inflections,
            })
        }
        VerbKind::Kuru => {
            let (Some(k), Some(r)) = (kanji_text.strip_suffix("来る"), reading.text.strip_suffix("くる")) else {
                return None;
            };

            let inflections = inflections! {
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
            };

            Some(Inflections {
                dictionary: Word::new(kanji_text, reading.text),
                inflections,
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
                PartOfSpeech::VerbIchidan => VerbKind::Ichidan,
                PartOfSpeech::VerbIchidanS => VerbKind::Ichidan,
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
