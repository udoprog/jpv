//! Macros to construct conjugations.

// Construct ichidan conjugations.
#[rustfmt::skip]
macro_rules! ichidan {
    ($out:ident) => {{
        $out!("る", Present);
        $out!("ます", Present + *Polite);
        $out!("ない", Negative);
        $out!("ません", Negative + *Polite);
        $out!("た", Past);
        $out!("ました", Past + *Polite);
        $out!("なかった", Past + Negative);
        $out!("ませんでした", Past + Negative + *Polite);
        $out!("ろ", Command);
        $out!("なさい", Command + *Polite);
        $out!("よ", Command + *Alternate);
        $out!("てください", Command + *Alternate + *Polite);
        $out!("るな", Command + Negative);
        $out!("ないでください", Command + Negative + *Polite);
        $out!("ば", Hypothetical);
        $out!("なければ", Hypothetical + Negative);
        $out!("たら", Conditional);
        $out!("ましたら", Conditional + *Polite);
        $out!("なかったら", Conditional + Negative);
        $out!("ませんでしたら", Conditional + Negative + *Polite);
        $out!("られる", Passive);
        $out!("れる", Passive + *Conversation);
        $out!("られます", Passive + *Polite);
        $out!("られない", Passive + Negative);
        $out!("られません", Passive + Negative + *Polite);
        $out!("られる", Potential);
        $out!("られます", Potential + *Polite);
        $out!("られない", Potential + Negative);
        $out!("られません", Potential + Negative + *Polite);
        $out!("よう", Volitional);
        $out!("ましょう", Volitional + *Polite);
        $out!("るだろう", Volitional + *Alternate);
        $out!("るでしょう", Volitional + *Alternate + *Polite);
        $out!("ないだろう", Volitional + Negative);
        $out!("ないでしょう", Volitional + Negative + *Polite);
        $out!("させる", Causative);
        $out!("たい", Tai);
        $out!("たくない", Tai + Negative);
        $out!("たかった", Tai + Past);
        $out!("たくなかった", Tai + Past + Negative);
    }};
}

// Construct godan conjugations.
#[rustfmt::skip]
macro_rules! godan {
    ($out:ident, $g:expr$(, $base:literal)?) => {{
        $out!([$($base, )? $g.i, "ます"], Present + *Polite);
        $out!([$($base, )? $g.u], Present);
        $out!([$($base, )? $g.a, "ない"], Negative);
        $out!([$($base, )? $g.i, "ません"], Negative + *Polite);
        $out!([$($base, )? $g.past], Past);
        $out!([$($base, )? $g.i, "ました"], Past + *Polite);
        $out!([$($base, )? $g.a, "なかった"], Past + Negative);
        $out!([$($base, )? $g.i, "ませんでした"], Past + Negative + *Polite);
        $out!([$($base, )? $g.e], Command);
        $out!([$($base, )? $g.i, "なさい"], Command + *Polite);
        $out!([$($base, )? $g.te, "ください"], Command + *Alternate + *Polite);
        $out!([$($base, )? $g.u, "な"], Command + Negative);
        $out!([$($base, )? $g.a, "ないでください"], Command + Negative + *Polite);
        $out!([$($base, )? $g.e, "ば"], Hypothetical);
        $out!([$($base, )? $g.a, "なければ"], Hypothetical + Negative);
        $out!([$($base, )? $g.past, "ら"], Conditional);
        $out!([$($base, )? $g.i, "ましたら"], Conditional + *Polite);
        $out!([$($base, )? $g.a, "なかったら"], Conditional + Negative);
        $out!([$($base, )? $g.i, "ませんでしたら"], Conditional + Negative + *Polite);
        $out!([$($base, )? $g.a, "れる"], Passive);
        $out!([$($base, )? $g.a, "れます"], Passive + *Polite);
        $out!([$($base, )? $g.a, "れない"], Passive + Negative);
        $out!([$($base, )? $g.a, "れません"], Passive + Negative + *Polite);
        $out!([$($base, )? $g.e, "る"], Potential);
        $out!([$($base, )? $g.e, "ます"], Potential + *Polite);
        $out!([$($base, )? $g.e, "ない"], Potential + Negative);
        $out!([$($base, )? $g.e, "ません"], Potential + Negative + *Polite);
        $out!([$($base, )? $g.o, "う"], Volitional);
        $out!([$($base, )? $g.i, "ましょう"], Volitional + *Polite);
        $out!([$($base, )? $g.u, "だろう"], Volitional + *Alternate);
        $out!([$($base, )? $g.u, "でしょう"], Volitional + *Alternate + *Polite);
        $out!([$($base, )? $g.a, "ないだろう"], Volitional + Negative);
        $out!([$($base, )? $g.a, "ないでしょう"], Volitional + Negative + *Polite);
        $out!([$($base, )? $g.a, "せる"], Causative);
        $out!([$($base, )? $g.i, "たい"], Tai);
        $out!([$($base, )? $g.i, "たくない"], Tai + Negative);
        $out!([$($base, )? $g.i, "たかった"], Tai + Past);
        $out!([$($base, )? $g.i, "たくなかった"], Tai + Past + Negative);
    }};
}

/// Generate kuru conjugations.
macro_rules! kuru {
    ($out:ident) => {
        $out!("く", "くる", Present);
        $out!("き", "ます", Present + *Polite);
        $out!("こ", "ない", Negative);
        $out!("き", "ません", Negative + *Polite);
        $out!("き", "た", Past);
        $out!("き", "ました", Past + *Polite);
        $out!("こ", "なかった", Past + Negative);
        $out!("き", "ませんでした", Past + Negative + *Polite);
        $out!("こ", "い", Command);
        $out!("き", "なさい", Command + *Polite);
        $out!("き", "てください", Command + *Alternate + *Polite);
        $out!("く", "るな", Command + Negative);
        $out!("こ", "ないでください", Command + Negative + *Polite);
        $out!("く", "れば", Hypothetical);
        $out!("き", "たら", Conditional);
        $out!("き", "ましたら", Conditional + *Polite);
        $out!("こ", "なかったら", Conditional + Negative);
        $out!("き", "ませんでしたら", Conditional + Negative + *Polite);
        $out!("こ", "られる", Passive);
        $out!("こ", "られます", Passive + *Polite);
        $out!("こ", "られない", Passive + Negative);
        $out!("こ", "られません", Passive + Negative + *Polite);
        $out!("こ", "られる", Potential);
        $out!("こ", "よう", Volitional);
        $out!("き", "ましょう", Volitional + *Polite);
        $out!("く", "るだろう", Volitional + *Alternate);
        $out!("く", "るでしょう", Volitional + *Alternate + *Polite);
        $out!("こ", "ないだろう", Volitional + Negative);
        $out!("こ", "ないでしょう", Volitional + Negative + *Polite);
        $out!("こ", "させる", Causative);
        $out!("こ", "させます", Causative + *Polite);
        $out!("こ", "させない", Causative + Negative);
        $out!("こ", "させません", Causative + Negative + *Polite);
        $out!("き", "たい", Tai);
        $out!("き", "たくない", Tai + Negative);
        $out!("き", "たかった", Tai + Past);
        $out!("き", "たくなかった", Tai + Past + Negative);
    };
}

/// Conjugations for a suru verb.
macro_rules! suru {
    ($out:ident) => {
        $out!("する", Present);
        $out!("します", Present + *Polite);
        $out!("しない", Negative);
        $out!("しません", Negative + *Polite);
        $out!("した", Past);
        $out!("しました", Past + *Polite);
        $out!("しなかった", Past + Negative);
        $out!("しませんでした", Past + Negative + *Polite);
        $out!("しろ", Command);
        $out!("しなさい", Command + *Polite);
        $out!("してください", Command + *Alternate + *Polite);
        $out!("するな", Command + Negative);
        $out!("しないでください", Command + Negative + *Polite);
        $out!("すれば", Hypothetical);
        $out!("したら", Conditional);
        $out!("しましたら", Conditional + *Polite);
        $out!("しなかったら", Conditional + Negative);
        $out!("しませんでしたら", Conditional + Negative + *Polite);
        $out!("される", Passive);
        $out!("できる", Potential);
        $out!("できます", Potential + *Polite);
        $out!("できない", Potential + Negative);
        $out!("できまあせん", Potential + Negative + *Polite);
        $out!("しよう", Volitional);
        $out!("しましょう", Volitional + *Polite);
        $out!("するだろう", Volitional + *Alternate);
        $out!("するでしょう", Volitional + *Alternate + *Polite);
        $out!("しないだろう", Volitional + Negative);
        $out!("しないでしょう", Volitional + Negative + *Polite);
        $out!("したろう", Volitional + Past);
        $out!("しましたろう", Volitional + Past + *Polite);
        $out!("しただろう", Volitional + Past + *Alternate);
        $out!("しなかっただろう", Volitional + Past + Negative);
        $out!("しなかったでしょう", Volitional + Past + Negative + *Polite);
        $out!("させる", Causative);
        $out!("したい", Tai);
        $out!("したくない", Tai + Negative);
        $out!("したかった", Tai + Past);
        $out!("したくなかった", Tai + Past + Negative);
    };
}
