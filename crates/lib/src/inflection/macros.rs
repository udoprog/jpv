//! Macros to construct conjugations.

// Construct ichidan conjugations.
#[rustfmt::skip]
#[cfg(not(fake))]
macro_rules! ichidan {
    ($macro:path) => {
        $macro!("る");
        $macro!("ます", Polite);
        $macro!("ない", Negative);
        $macro!("ません", Negative, Polite);
        $macro!("た", Past);
        $macro!("ました", Past, Polite);
        $macro!("なかった", Past, Negative);
        $macro!("ませんでした", Past, Negative, Polite);
        $macro!("ろ", Command);
        $macro!("なさい", Command, Polite);
        $macro!("てください", Command, Polite, Kudasai);
        $macro!("よ", Command, Yo);
        $macro!("るな", Command, Negative);
        $macro!("ないでください", Command, Negative, Polite);
        $macro!("ば", Hypothetical);
        $macro!("なければ", Hypothetical, Negative);
        $macro!("なきゃ", Hypothetical, Negative, Kya);
        $macro!("たら", Conditional);
        $macro!("ましたら", Conditional, Polite);
        $macro!("なかったら", Conditional, Negative);
        $macro!("ませんでしたら", Conditional, Negative, Polite);
        $macro!("れる", Passive, Conversation);
        $macro!("られる", Passive);
        $macro!("られます", Passive, Polite);
        $macro!("られない", Passive, Negative);
        $macro!("られません", Passive, Negative, Polite);
        $macro!("られた", Passive, Past);
        $macro!("られました", Passive, Past, Polite);
        $macro!("られる", Potential);
        $macro!("られます", Potential, Polite);
        $macro!("られない", Potential, Negative);
        $macro!("られません", Potential, Negative, Polite);
        $macro!("られた", Potential, Past);
        $macro!("られました", Potential, Past, Polite);
        $macro!("られなかった", Potential, Past, Negative);
        $macro!("られませんでした", Potential, Past, Negative, Polite);
        $macro!("よう", Volitional);
        $macro!("ましょう", Volitional, Polite);
        $macro!("るだろう", Volitional, Darou);
        $macro!("るでしょう", Volitional, Darou, Polite);
        $macro!("ないだろう", Volitional, Negative);
        $macro!("ないでしょう", Volitional, Negative, Polite);
        $macro!("させる", Causative);
        $macro!("たい", Tai);
        $macro!("たくない", Tai, Negative);
        $macro!("たかった", Tai, Past);
        $macro!("たくなかった", Tai, Past, Negative);
    };

    ($macro:path, te) => {
        ichidan!($macro);
        $macro!("", Stem);
        $macro!("て", Te);
    };
}

#[cfg(fake)]
macro_rules! ichidan {
    ($macro:path) => {};
    ($macro:path, te) => {};
}

// Construct godan conjugations.
#[rustfmt::skip]
#[cfg(not(fake))]
macro_rules! godan_lit {
    ($macro:path, {$a:literal, $i:literal, $u:literal, $e:literal, $o:literal, $te:literal, $past:literal}) => {{
        $macro!("", $u);
        $macro!("", $past, Past);
        $macro!("", concat!($past, "ら"), Conditional);
        $macro!("", $e, Command);
        $macro!($i, "ます", Polite);
        $macro!($a, "ない", Negative);
        $macro!($i, "ません", Negative, Polite);
        $macro!($i, "ました", Past, Polite);
        $macro!($a, "なかった", Past, Negative);
        $macro!($i, "ませんでした", Past, Negative, Polite);
        $macro!($i, "なさい", Command, Polite);
        $macro!($te, "ください", Command, Polite, Kudasai);
        $macro!($e, "よ", Command, Yo);
        $macro!($u, "な", Command, Negative);
        $macro!($a, "ないでください", Command, Negative, Polite);
        $macro!($e, "ば", Hypothetical);
        $macro!($a, "なければ", Hypothetical, Negative);
        $macro!($a, "なきゃ", Hypothetical, Negative, Kya);
        $macro!($i, "ましたら", Conditional, Polite);
        $macro!($a, "なかったら", Conditional, Negative);
        $macro!($i, "ませんでしたら", Conditional, Negative, Polite);
        $macro!($a, "れる", Passive);
        $macro!($a, "れます", Passive, Polite);
        $macro!($a, "れない", Passive, Negative);
        $macro!($a, "れません", Passive, Negative, Polite);
        $macro!($a, "れた", Passive, Past);
        $macro!($a, "れました", Passive, Past, Polite);
        $macro!($e, "る", Potential);
        $macro!($e, "ます", Potential, Polite);
        $macro!($e, "ない", Potential, Negative);
        $macro!($e, "ません", Potential, Negative, Polite);
        $macro!($e, "た", Potential, Past);
        $macro!($e, "ました", Potential, Past, Polite);
        $macro!($e, "なかった", Potential, Past, Negative);
        $macro!($e, "ませんでした", Potential, Past, Negative, Polite);
        $macro!($o, "う", Volitional);
        $macro!($i, "ましょう", Volitional, Polite);
        $macro!($u, "だろう", Volitional, Darou);
        $macro!($u, "でしょう", Volitional, Darou, Polite);
        $macro!($a, "ないだろう", Volitional, Negative);
        $macro!($a, "ないでしょう", Volitional, Negative, Polite);
        $macro!($a, "せる", Causative);
        $macro!($i, "たい", Tai);
        $macro!($i, "たくない", Tai, Negative);
        $macro!($i, "たかった", Tai, Past);
        $macro!($i, "たくなかった", Tai, Past, Negative);
    }};
}

#[cfg(not(fake))]
macro_rules! godan_u {
    ($macro:path) => {
        godan_lit!($macro, {"わ", "い", "う", "え", "お", "って", "った"})
    };

    ($macro:path, te) => {
        godan_u!($macro);
        $macro!("", "い", Stem);
        $macro!("", "って", Te);
    };
}

#[cfg(fake)]
macro_rules! godan_u {
    ($macro:path) => {};
    ($macro:path, te) => {};
}

#[cfg(not(fake))]
macro_rules! godan_tsu {
    ($macro:path) => {
        godan_lit!($macro, {"た", "ち", "つ", "て", "と", "って", "った"})
    };

    ($macro:path, te) => {
        godan_tsu!($macro);
        $macro!("", "ち", Stem);
        $macro!("", "って", Te);
    };
}

#[cfg(fake)]
macro_rules! godan_tsu {
    ($macro:path) => {};
    ($macro:path, te) => {};
}

#[cfg(not(fake))]
macro_rules! godan_ru {
    ($macro:path) => {
        godan_lit!($macro, {"ら", "り", "る", "れ", "ろ", "って", "った"});
    };

    ($macro:path, te) => {
        godan_ru!($macro);
        $macro!("", "り", Stem);
        $macro!("", "って", Te);
    };
}

#[cfg(fake)]
macro_rules! godan_ru {
    ($macro:path) => {};
    ($macro:path, te) => {};
}

#[cfg(not(fake))]
macro_rules! godan_ku {
    ($macro:path) => {
        godan_lit!($macro, {"か", "き", "く", "け", "こ", "いて", "いた"});
    };

    ($macro:path, te) => {
        godan_ku!($macro);
        $macro!("", "き", Stem);
        $macro!("", "いて", Te);
    };
}

#[cfg(fake)]
macro_rules! godan_ku {
    ($macro:path) => {};
    ($macro:path, te) => {};
}

#[cfg(not(fake))]
macro_rules! godan_gu {
    ($macro:path) => {
        godan_lit!($macro, {"が", "ぎ", "ぐ", "げ", "ご", "いで", "いだ"});
    };

    ($macro:path, te) => {
        godan_gu!($macro);
        $macro!("", "ぎ", Stem);
        $macro!("", "いで", Te);
    };
}

#[cfg(fake)]
macro_rules! godan_gu {
    ($macro:path) => {};
    ($macro:path, te) => {};
}

#[cfg(not(fake))]
macro_rules! godan_mu {
    ($macro:path) => {
        godan_lit!($macro, {"ま", "み", "む", "め", "も", "んで", "んだ"});
    };

    ($macro:path, te) => {
        godan_mu!($macro);
        $macro!("", "み", Stem);
        $macro!("", "んで", Te);
    };
}

#[cfg(fake)]
macro_rules! godan_mu {
    ($macro:path) => {};
    ($macro:path, te) => {};
}

#[cfg(not(fake))]
macro_rules! godan_bu {
    ($macro:path) => {
        godan_lit!($macro, {"ば", "び", "ぶ", "べ", "ぼ", "んで", "んだ"});
    };

    ($macro:path, te) => {
        godan_bu!($macro);
        $macro!("", "び", Stem);
        $macro!("", "んで", Te);
    };
}

#[cfg(fake)]
macro_rules! godan_bu {
    ($macro:path) => {};
    ($macro:path, te) => {};
}

#[cfg(not(fake))]
macro_rules! godan_nu {
    ($macro:path) => {
        godan_lit!($macro, {"な", "に", "ぬ", "ね", "の", "んで", "んだ"});
    };

    ($macro:path, te) => {
        godan_nu!($macro);
        $macro!("", "に", Stem);
        $macro!("", "んで", Te);
    };
}

#[cfg(fake)]
macro_rules! godan_nu {
    ($macro:path) => {};
    ($macro:path, te) => {};
}

#[cfg(not(fake))]
macro_rules! godan_su {
    ($macro:path) => {
        godan_lit!($macro, {"さ", "し", "す", "せ", "そ", "して", "した"});
    };

    ($macro:path, te) => {
        godan_su!($macro);
        $macro!("", "し", Stem);
        $macro!("", "して", Te);
    };
}

#[cfg(fake)]
macro_rules! godan_su {
    ($macro:path) => {};
    ($macro:path, te) => {};
}

#[cfg(not(fake))]
macro_rules! godan_iku {
    ($macro:path) => {
        godan_lit!($macro, {"か", "き", "く", "け", "こ", "って", "った"});
    };

    ($macro:path, te) => {
        godan_iku!($macro);
        $macro!("", "き", Stem);
        $macro!("", "って", Te);
    };
}

#[cfg(fake)]
macro_rules! godan_iku {
    ($macro:path) => {};
    ($macro:path, te) => {};
}

/// Generate kuru conjugations.
#[cfg(not(fake))]
macro_rules! kuru {
    ($macro:path) => {
        $macro!("く", "る");
        $macro!("き", "ます", Polite);
        $macro!("こ", "ない", Negative);
        $macro!("き", "ません", Negative, Polite);
        $macro!("き", "た", Past);
        $macro!("き", "ました", Past, Polite);
        $macro!("こ", "なかった", Past, Negative);
        $macro!("き", "ませんでした", Past, Negative, Polite);
        $macro!("こ", "い", Command);
        $macro!("き", "なさい", Command, Polite);
        $macro!("き", "てください", Command, Polite, Kudasai);
        $macro!("く", "るな", Command, Negative);
        $macro!("こ", "ないでください", Command, Negative, Polite);
        $macro!("く", "れば", Hypothetical);
        $macro!("こ", "なければ", Hypothetical, Negative);
        $macro!("こ", "なきゃ", Hypothetical, Negative, Kya);
        $macro!("き", "たら", Conditional);
        $macro!("き", "ましたら", Conditional, Polite);
        $macro!("こ", "なかったら", Conditional, Negative);
        $macro!("き", "ませんでしたら", Conditional, Negative, Polite);
        $macro!("こ", "られる", Passive);
        $macro!("こ", "られます", Passive, Polite);
        $macro!("こ", "られない", Passive, Negative);
        $macro!("こ", "られません", Passive, Negative, Polite);
        $macro!("こ", "られた", Passive, Past);
        $macro!("こ", "られました", Passive, Past, Polite);
        $macro!("こ", "られる", Potential);
        $macro!("こ", "よう", Volitional);
        $macro!("き", "ましょう", Volitional, Polite);
        $macro!("く", "るだろう", Volitional, Darou);
        $macro!("く", "るでしょう", Volitional, Darou, Polite);
        $macro!("こ", "ないだろう", Volitional, Negative);
        $macro!("こ", "ないでしょう", Volitional, Negative, Polite);
        $macro!("こ", "させる", Causative);
        $macro!("こ", "させます", Causative, Polite);
        $macro!("こ", "させない", Causative, Negative);
        $macro!("こ", "させません", Causative, Negative, Polite);
        $macro!("き", "たい", Tai);
        $macro!("き", "たくない", Tai, Negative);
        $macro!("き", "たかった", Tai, Past);
        $macro!("き", "たくなかった", Tai, Past, Negative);
    };

    ($macro:path, te) => {
        kuru!($macro);
        $macro!("き", "", Stem);
        $macro!("き", "て", Te);
    };
}

#[cfg(fake)]
macro_rules! kuru {
    ($macro:path) => {};
    ($macro:path, te) => {};
}

/// Conjugations for a suru verb.
#[cfg(not(fake))]
macro_rules! suru {
    ($macro:path) => {
        $macro!("す", "る");
        $macro!("し", "ます", Polite);
        $macro!("し", "ない", Negative);
        $macro!("し", "ません", Negative, Polite);
        $macro!("し", "た", Past);
        $macro!("し", "ました", Past, Polite);
        $macro!("し", "なかった", Past, Negative);
        $macro!("し", "ませんでした", Past, Negative, Polite);
        $macro!("し", "ろ", Command);
        $macro!("し", "なさい", Command, Polite);
        $macro!("し", "てください", Command, Polite, Kudasai);
        $macro!("し", "よ", Command, Yo);
        $macro!("す", "るな", Command, Negative);
        $macro!("し", "ないでください", Command, Negative, Polite);
        $macro!("す", "れば", Hypothetical);
        $macro!("し", "なければ", Hypothetical, Negative);
        $macro!("し", "なきゃ", Hypothetical, Negative, Kya);
        $macro!("し", "たら", Conditional);
        $macro!("し", "ましたら", Conditional, Polite);
        $macro!("し", "なかったら", Conditional, Negative);
        $macro!("し", "ませんでしたら", Conditional, Negative, Polite);
        $macro!("さ", "れる", Passive);
        $macro!("さ", "れます", Passive, Polite);
        $macro!("さ", "れない", Passive, Negative);
        $macro!("さ", "れません", Passive, Negative, Polite);
        $macro!("さ", "れた", Passive, Past);
        $macro!("さ", "れました", Passive, Past, Polite);
        $macro!("で", "きる", Potential);
        $macro!("で", "きます", Potential, Polite);
        $macro!("で", "きない", Potential, Negative);
        $macro!("で", "きません", Potential, Negative, Polite);
        $macro!("で", "きた", Potential, Past);
        $macro!("で", "きました", Potential, Past, Polite);
        $macro!("で", "きなかった", Potential, Past, Negative);
        $macro!("で", "きませんでした", Potential, Past, Negative, Polite);
        $macro!("し", "よう", Volitional);
        $macro!("し", "ましょう", Volitional, Polite);
        $macro!("す", "るだろう", Volitional, Darou);
        $macro!("す", "るでしょう", Volitional, Darou, Polite);
        $macro!("し", "ないだろう", Volitional, Negative);
        $macro!("し", "ないでしょう", Volitional, Negative, Polite);
        $macro!("し", "たろう", Volitional, Past);
        $macro!("し", "ましたろう", Volitional, Past, Polite);
        $macro!("し", "ただろう", Volitional, Past, Darou);
        $macro!("し", "なかっただろう", Volitional, Past, Negative);
        $macro!("し", "なかったでしょう", Volitional, Past, Negative, Polite);
        $macro!("さ", "せる", Causative);
        $macro!("し", "たい", Tai);
        $macro!("し", "たくない", Tai, Negative);
        $macro!("し", "たかった", Tai, Past);
        $macro!("し", "たくなかった", Tai, Past, Negative);
    };

    ($macro:path, te) => {
        suru!($macro);
        $macro!("し", "", Stem);
        $macro!("し", "て", Te);
    };
}

#[cfg(fake)]
macro_rules! suru {
    ($macro:path) => {};
    ($macro:path, te) => {};
}

#[cfg(not(fake))]
macro_rules! adjective_i {
    ($macro:path) => {
        $macro!("い");
        $macro!("いです", Polite);
        $macro!("かった", Past);
        $macro!("かったです", Past, Polite);
        $macro!("くない", Negative);
        $macro!("くないです", Negative, Polite);
        $macro!("なかった", Past, Negative);
        $macro!("なかったです", Past, Negative, Polite);
    };
}

#[cfg(fake)]
macro_rules! adjective_i {
    ($macro:path) => {};
}

#[cfg(not(fake))]
macro_rules! adjective_ii {
    ($macro:path) => {
        $macro!("い", "い");
        $macro!("い", "いです", Polite);
        $macro!("よ", "かった", Past);
        $macro!("よ", "かったです", Past, Polite);
        $macro!("よ", "くない", Negative);
        $macro!("よ", "くないです", Negative, Polite);
        $macro!("よ", "なかった", Past, Negative);
        $macro!("よ", "なかったです", Past, Negative, Polite);
    };
}

#[cfg(fake)]
macro_rules! adjective_ii {
    ($macro:path) => {};
}

#[cfg(not(fake))]
macro_rules! adjective_na {
    ($macro:path) => {
        $macro!("だ");
        $macro!("です", Polite);
        $macro!("だった", Past);
        $macro!("でした", Past, Polite);
        $macro!("ではない", Negative);
        $macro!("ではありません", Negative, Polite);
        $macro!("ではなかった", Past, Negative);
        $macro!("ではありませんでした", Past, Negative, Polite);
    };
}

#[cfg(fake)]
macro_rules! adjective_na {
    ($macro:path) => {};
}

/// Helper to construct a particular [`Inflection`].
///
/// # Examples
///
/// ```rust
/// lib::inflect!(Past);
/// lib::inflect!(Past, Polite);
/// lib::inflect!(Past, Short);
/// ```
#[macro_export]
macro_rules! inflect {
    ($($form:ident),* $(,)?) => {{
        #[allow(unused_mut)]
        let mut form = $crate::macro_support::fixed_map::Set::new();
        $(form.insert($crate::Form::$form);)*
        $crate::Inflection::new(form)
    }}
}
