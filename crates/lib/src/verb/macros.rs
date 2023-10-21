//! Macros to construct conjugations.

// Construct ichidan conjugations.
#[rustfmt::skip]
macro_rules! ichidan {
    ($out:ident) => {{
        $out!("る");
        $out!("", Stem);
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
        $out!("なきゃ", Hypothetical, Negative, Kya);
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
        $out!([$($base, )? $g.i], Stem);
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
        $out!([$($base, )? $g.a, "なきゃ"], Hypothetical, Negative, Kya);
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
        $out!("き", "て", Te);
        $out!("き", "", Stem);
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
        $out!("こ", "なきゃ", Hypothetical, Negative, Kya);
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
        $out!("す", "る");
        $out!("し", "て", Te);
        $out!("し", "", Stem);
        $out!("し", "ます", Polite);
        $out!("し", "ない", Negative);
        $out!("し", "ません", Negative, Polite);
        $out!("し", "た", Past);
        $out!("し", "ました", Past, Polite);
        $out!("し", "なかった", Past, Negative);
        $out!("し", "ませんでした", Past, Negative, Polite);
        $out!("し", "ろ", Command);
        $out!("し", "なさい", Command, Polite);
        $out!("し", "てください", Command, Polite, Kudasai);
        $out!("し", "よ", Command, Yo);
        $out!("す", "るな", Command, Negative);
        $out!("し", "ないでください", Command, Negative, Polite);
        $out!("す", "れば", Hypothetical);
        $out!("し", "なければ", Hypothetical, Negative);
        $out!("し", "なきゃ", Hypothetical, Negative, Kya);
        $out!("し", "たら", Conditional);
        $out!("し", "ましたら", Conditional, Polite);
        $out!("し", "なかったら", Conditional, Negative);
        $out!("し", "ませんでしたら", Conditional, Negative, Polite);
        $out!("さ", "れる", Passive);
        $out!("さ", "れます", Passive, Polite);
        $out!("さ", "れない", Passive, Negative);
        $out!("さ", "れません", Passive, Negative, Polite);
        $out!("さ", "れた", Passive, Past);
        $out!("さ", "れました", Passive, Past, Polite);
        $out!("で", "きる", Potential);
        $out!("で", "きます", Potential, Polite);
        $out!("で", "きない", Potential, Negative);
        $out!("で", "きません", Potential, Negative, Polite);
        $out!("で", "きた", Potential, Past);
        $out!("で", "きました", Potential, Past, Polite);
        $out!("で", "きなかった", Potential, Past, Negative);
        $out!("で", "きませんでした", Potential, Past, Negative, Polite);
        $out!("し", "よう", Volitional);
        $out!("し", "ましょう", Volitional, Polite);
        $out!("す", "るだろう", Volitional, Darou);
        $out!("す", "るでしょう", Volitional, Darou, Polite);
        $out!("し", "ないだろう", Volitional, Negative);
        $out!("し", "ないでしょう", Volitional, Negative, Polite);
        $out!("し", "たろう", Volitional, Past);
        $out!("し", "ましたろう", Volitional, Past, Polite);
        $out!("し", "ただろう", Volitional, Past, Darou);
        $out!("し", "なかっただろう", Volitional, Past, Negative);
        $out!("し", "なかったでしょう", Volitional, Past, Negative, Polite);
        $out!("さ", "せる", Causative);
        $out!("し", "たい", Tai);
        $out!("し", "たくない", Tai, Negative);
        $out!("し", "たかった", Tai, Past);
        $out!("し", "たくなかった", Tai, Past, Negative);
    };
}
