//! Macros to construct conjugations.

// Construct ichidan conjugations.
#[rustfmt::skip]
macro_rules! ichidan {
    ($out:ident) => {{
        $out!("る");
        $out!("ます", Polite);
        $out!("ない", Negative);
        $out!("ません", Negative, Polite);
        $out!("た", Past);
        $out!("ました", Past, Polite);
        $out!("なかった", Past, Negative);
        $out!("ませんでした", Past, Negative, Polite);
        $out!("ろ", Command);
        $out!("なさい", Command, Polite);
        $out!("てください", Command, Polite, Kudasai);
        $out!("よ", Command, Yo);
        $out!("るな", Command, Negative);
        $out!("ないでください", Command, Negative, Polite);
        $out!("ば", Hypothetical);
        $out!("なければ", Hypothetical, Negative);
        $out!("たら", Conditional);
        $out!("ましたら", Conditional, Polite);
        $out!("なかったら", Conditional, Negative);
        $out!("ませんでしたら", Conditional, Negative, Polite);
        $out!("れる", Passive, Conversation);
        $out!("られる", Passive);
        $out!("られます", Passive, Polite);
        $out!("られない", Passive, Negative);
        $out!("られません", Passive, Negative, Polite);
        $out!("られた", Passive, Past);
        $out!("られました", Passive, Past, Polite);
        $out!("られる", Potential);
        $out!("られます", Potential, Polite);
        $out!("られない", Potential, Negative);
        $out!("られません", Potential, Negative, Polite);
        $out!("られた", Potential, Past);
        $out!("られました", Potential, Past, Polite);
        $out!("られなかった", Potential, Past, Negative);
        $out!("られませんでした", Potential, Past, Negative, Polite);
        $out!("よう", Volitional);
        $out!("ましょう", Volitional, Polite);
        $out!("るだろう", Volitional, Darou);
        $out!("るでしょう", Volitional, Darou, Polite);
        $out!("ないだろう", Volitional, Negative);
        $out!("ないでしょう", Volitional, Negative, Polite);
        $out!("させる", Causative);
        $out!("たい", Tai);
        $out!("たくない", Tai, Negative);
        $out!("たかった", Tai, Past);
        $out!("たくなかった", Tai, Past, Negative);
    }};
}

// Construct godan conjugations.
#[rustfmt::skip]
macro_rules! godan {
    ($out:ident, $g:expr$(, $base:literal)?) => {{
        $out!([$($base, )? $g.u]);
        $out!([$($base, )? $g.i, "ます"], Polite);
        $out!([$($base, )? $g.a, "ない"], Negative);
        $out!([$($base, )? $g.i, "ません"], Negative, Polite);
        $out!([$($base, )? $g.past], Past);
        $out!([$($base, )? $g.i, "ました"], Past, Polite);
        $out!([$($base, )? $g.a, "なかった"], Past, Negative);
        $out!([$($base, )? $g.i, "ませんでした"], Past, Negative, Polite);
        $out!([$($base, )? $g.e], Command);
        $out!([$($base, )? $g.i, "なさい"], Command, Polite);
        $out!([$($base, )? $g.te, "ください"], Command, Polite, Kudasai);
        $out!([$($base, )? $g.e, "よ"], Command, Yo);
        $out!([$($base, )? $g.u, "な"], Command, Negative);
        $out!([$($base, )? $g.a, "ないでください"], Command, Negative, Polite);
        $out!([$($base, )? $g.e, "ば"], Hypothetical);
        $out!([$($base, )? $g.a, "なければ"], Hypothetical, Negative);
        $out!([$($base, )? $g.past, "ら"], Conditional);
        $out!([$($base, )? $g.i, "ましたら"], Conditional, Polite);
        $out!([$($base, )? $g.a, "なかったら"], Conditional, Negative);
        $out!([$($base, )? $g.i, "ませんでしたら"], Conditional, Negative, Polite);
        $out!([$($base, )? $g.a, "れる"], Passive);
        $out!([$($base, )? $g.a, "れます"], Passive, Polite);
        $out!([$($base, )? $g.a, "れない"], Passive, Negative);
        $out!([$($base, )? $g.a, "れません"], Passive, Negative, Polite);
        $out!([$($base, )? $g.a, "れた"], Passive, Past);
        $out!([$($base, )? $g.a, "れました"], Passive, Past, Polite);
        $out!([$($base, )? $g.e, "る"], Potential);
        $out!([$($base, )? $g.e, "ます"], Potential, Polite);
        $out!([$($base, )? $g.e, "ない"], Potential, Negative);
        $out!([$($base, )? $g.e, "ません"], Potential, Negative, Polite);
        $out!([$($base, )? $g.e, "た"], Potential, Past);
        $out!([$($base, )? $g.e, "ました"], Potential, Past, Polite);
        $out!([$($base, )? $g.e, "なかった"], Potential, Past, Negative);
        $out!([$($base, )? $g.e, "ませんでした"], Potential, Past, Negative, Polite);
        $out!([$($base, )? $g.o, "う"], Volitional);
        $out!([$($base, )? $g.i, "ましょう"], Volitional, Polite);
        $out!([$($base, )? $g.u, "だろう"], Volitional, Darou);
        $out!([$($base, )? $g.u, "でしょう"], Volitional, Darou, Polite);
        $out!([$($base, )? $g.a, "ないだろう"], Volitional, Negative);
        $out!([$($base, )? $g.a, "ないでしょう"], Volitional, Negative, Polite);
        $out!([$($base, )? $g.a, "せる"], Causative);
        $out!([$($base, )? $g.i, "たい"], Tai);
        $out!([$($base, )? $g.i, "たくない"], Tai, Negative);
        $out!([$($base, )? $g.i, "たかった"], Tai, Past);
        $out!([$($base, )? $g.i, "たくなかった"], Tai, Past, Negative);
    }};
}

/// Generate kuru conjugations.
macro_rules! kuru {
    ($out:ident) => {
        $out!("く", "る");
        $out!("き", "ます", Polite);
        $out!("こ", "ない", Negative);
        $out!("き", "ません", Negative, Polite);
        $out!("き", "た", Past);
        $out!("き", "ました", Past, Polite);
        $out!("こ", "なかった", Past, Negative);
        $out!("き", "ませんでした", Past, Negative, Polite);
        $out!("こ", "い", Command);
        $out!("き", "なさい", Command, Polite);
        $out!("き", "てください", Command, Polite, Kudasai);
        $out!("く", "るな", Command, Negative);
        $out!("こ", "ないでください", Command, Negative, Polite);
        $out!("く", "れば", Hypothetical);
        $out!("こ", "なければ", Hypothetical, Negative);
        $out!("き", "たら", Conditional);
        $out!("き", "ましたら", Conditional, Polite);
        $out!("こ", "なかったら", Conditional, Negative);
        $out!("き", "ませんでしたら", Conditional, Negative, Polite);
        $out!("こ", "られる", Passive);
        $out!("こ", "られます", Passive, Polite);
        $out!("こ", "られない", Passive, Negative);
        $out!("こ", "られません", Passive, Negative, Polite);
        $out!("こ", "られた", Passive, Past);
        $out!("こ", "られました", Passive, Past, Polite);
        $out!("こ", "られる", Potential);
        $out!("こ", "よう", Volitional);
        $out!("き", "ましょう", Volitional, Polite);
        $out!("く", "るだろう", Volitional, Darou);
        $out!("く", "るでしょう", Volitional, Darou, Polite);
        $out!("こ", "ないだろう", Volitional, Negative);
        $out!("こ", "ないでしょう", Volitional, Negative, Polite);
        $out!("こ", "させる", Causative);
        $out!("こ", "させます", Causative, Polite);
        $out!("こ", "させない", Causative, Negative);
        $out!("こ", "させません", Causative, Negative, Polite);
        $out!("き", "たい", Tai);
        $out!("き", "たくない", Tai, Negative);
        $out!("き", "たかった", Tai, Past);
        $out!("き", "たくなかった", Tai, Past, Negative);
    };
}

/// Conjugations for a suru verb.
macro_rules! suru {
    ($out:ident) => {
        $out!("する");
        $out!("します", Polite);
        $out!("しない", Negative);
        $out!("しません", Negative, Polite);
        $out!("した", Past);
        $out!("しました", Past, Polite);
        $out!("しなかった", Past, Negative);
        $out!("しませんでした", Past, Negative, Polite);
        $out!("しろ", Command);
        $out!("しなさい", Command, Polite);
        $out!("してください", Command, Polite, Kudasai);
        $out!("しよ", Command, Yo);
        $out!("するな", Command, Negative);
        $out!("しないでください", Command, Negative, Polite);
        $out!("すれば", Hypothetical);
        $out!("しなければ", Hypothetical, Negative);
        $out!("したら", Conditional);
        $out!("しましたら", Conditional, Polite);
        $out!("しなかったら", Conditional, Negative);
        $out!("しませんでしたら", Conditional, Negative, Polite);
        $out!("される", Passive);
        $out!("されます", Passive, Polite);
        $out!("されない", Passive, Negative);
        $out!("されません", Passive, Negative, Polite);
        $out!("された", Passive, Past);
        $out!("されました", Passive, Past, Polite);
        $out!("できる", Potential);
        $out!("できます", Potential, Polite);
        $out!("できない", Potential, Negative);
        $out!("できません", Potential, Negative, Polite);
        $out!("できた", Potential, Past);
        $out!("できました", Potential, Past, Polite);
        $out!("できなかった", Potential, Past, Negative);
        $out!("できませんでした", Potential, Past, Negative, Polite);
        $out!("しよう", Volitional);
        $out!("しましょう", Volitional, Polite);
        $out!("するだろう", Volitional, Darou);
        $out!("するでしょう", Volitional, Darou, Polite);
        $out!("しないだろう", Volitional, Negative);
        $out!("しないでしょう", Volitional, Negative, Polite);
        $out!("したろう", Volitional, Past);
        $out!("しましたろう", Volitional, Past, Polite);
        $out!("しただろう", Volitional, Past, Darou);
        $out!("しなかっただろう", Volitional, Past, Negative);
        $out!("しなかったでしょう", Volitional, Past, Negative, Polite);
        $out!("させる", Causative);
        $out!("したい", Tai);
        $out!("したくない", Tai, Negative);
        $out!("したかった", Tai, Past);
        $out!("したくなかった", Tai, Past, Negative);
    };
}
