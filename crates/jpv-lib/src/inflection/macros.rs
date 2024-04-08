//! Macros to construct conjugations.

use crate::inflection::godan::Godan;
use crate::inflection::Form;

use Form::*;

/// Perform ichidan conjugations.
pub fn ichidan(mut r: impl FnMut(&'static str, &[Form])) {
    r("る", &[]);
    r("ます", &[Honorific]);
    r("ない", &[Negative]);
    r("ません", &[Negative, Honorific]);
    r("た", &[Past]);
    r("ました", &[Past, Honorific]);
    r("なかった", &[Past, Negative]);
    r("ませんでした", &[Past, Negative, Honorific]);
    r("ろ", &[Command]);
    r("なさい", &[Command, Honorific]);
    r("てください", &[Command, Honorific, CommandTeKudasai]);
    r("よ", &[Command, CommandYo]);
    r("るな", &[Command, Negative]);
    r("ないでください", &[Command, Negative, Honorific]);
    r("りゃ", &[Hypothetical, Conversation]);
    r("なけりゃ", &[Hypothetical, Conversation, Negative]);
    r("ば", &[Hypothetical]);
    r("なければ", &[Hypothetical, Negative]);
    r("なきゃ", &[Hypothetical, Negative, Kya]);
    r("たら", &[Conditional]);
    r("ましたら", &[Conditional, Honorific]);
    r("なかったら", &[Conditional, Negative]);
    r("ませんでしたら", &[Conditional, Negative, Honorific]);
    r("れる", &[Passive, Conversation]);
    r("られる", &[Passive]);
    r("られます", &[Passive, Honorific]);
    r("られない", &[Passive, Negative]);
    r("られません", &[Passive, Negative, Honorific]);
    r("られた", &[Passive, Past]);
    r("られました", &[Passive, Past, Honorific]);
    r("られる", &[Potential]);
    r("られます", &[Potential, Honorific]);
    r("られない", &[Potential, Negative]);
    r("られません", &[Potential, Negative, Honorific]);
    r("られた", &[Potential, Past]);
    r("られました", &[Potential, Past, Honorific]);
    r("られなかった", &[Potential, Past, Negative]);
    r("られませんでした", &[Potential, Past, Negative, Honorific]);
    r("よう", &[Volitional]);
    r("ましょう", &[Volitional, Honorific]);
    r("るだろう", &[Volitional, Darou]);
    r("るでしょう", &[Volitional, Darou, Honorific]);
    r("ないだろう", &[Volitional, Negative]);
    r("ないでしょう", &[Volitional, Negative, Honorific]);
    r("させる", &[Causative]);
    r("ながら", &[Simultaneous]);
    r("そう", &[LooksLike]);
}

pub(crate) fn ichidan_te(mut r: impl FnMut(&'static str, &[Form])) {
    r("", &[Stem]);
    r("て", &[Te]);
    ichidan(r);
}

pub(crate) fn godan(g: &'static Godan, mut r: impl FnMut(&'static str, &'static str, &[Form])) {
    r("", g.u, &[]);
    r("", g.past, &[Past]);
    r("", g.tara, &[Conditional]);
    r("", g.e, &[Command]);
    r(g.i, "ます", &[Honorific]);
    r(g.a, "ない", &[Negative]);
    r(g.i, "ません", &[Negative, Honorific]);
    r(g.i, "ました", &[Past, Honorific]);
    r(g.a, "なかった", &[Past, Negative]);
    r(g.i, "ませんでした", &[Past, Negative, Honorific]);
    r(g.i, "なさい", &[Command, Honorific]);
    r(g.te, "ください", &[Command, Honorific, CommandTeKudasai]);
    r(g.e, "よ", &[Command, CommandYo]);
    r(g.u, "な", &[Command, Negative]);
    r(g.a, "ないでください", &[Command, Negative, Honorific]);

    if let Some(kya) = g.kya {
        r("", kya, &[Hypothetical, Conversation]);
    }

    if let Some(nake_kya) = g.nake_kya {
        r("", nake_kya, &[Hypothetical, Negative, Conversation]);
    }

    r(g.e, "ば", &[Hypothetical]);
    r(g.a, "なければ", &[Hypothetical, Negative]);
    r(g.a, "なきゃ", &[Hypothetical, Negative, Kya]);
    r(g.i, "ましたら", &[Conditional, Honorific]);
    r(g.a, "なかったら", &[Conditional, Negative]);
    r(g.i, "ませんでしたら", &[Conditional, Negative, Honorific]);
    r(g.a, "れる", &[Passive]);
    r(g.a, "れます", &[Passive, Honorific]);
    r(g.a, "れない", &[Passive, Negative]);
    r(g.a, "れません", &[Passive, Negative, Honorific]);
    r(g.a, "れた", &[Passive, Past]);
    r(g.a, "れました", &[Passive, Past, Honorific]);
    r(g.e, "る", &[Potential]);
    r(g.e, "ます", &[Potential, Honorific]);
    r(g.e, "ない", &[Potential, Negative]);
    r(g.e, "ません", &[Potential, Negative, Honorific]);
    r(g.e, "た", &[Potential, Past]);
    r(g.e, "ました", &[Potential, Past, Honorific]);
    r(g.e, "なかった", &[Potential, Past, Negative]);
    r(g.e, "ませんでした", &[Potential, Past, Negative, Honorific]);
    r(g.o, "う", &[Volitional]);
    r(g.i, "ましょう", &[Volitional, Honorific]);
    r(g.u, "だろう", &[Volitional, Darou]);
    r(g.u, "でしょう", &[Volitional, Darou, Honorific]);
    r(g.a, "ないだろう", &[Volitional, Negative]);
    r(g.a, "ないでしょう", &[Volitional, Negative, Honorific]);
    r(g.a, "せる", &[Causative]);
    r(g.i, "ながら", &[Simultaneous]);
    r(g.i, "そう", &[LooksLike]);
}

pub(crate) fn godan_base(
    g: &'static Godan,
    mut r: impl FnMut(&'static str, &'static str, &[Form]),
) {
    r("", g.i, &[Stem]);
    r("", g.te, &[Te]);
    godan(g, r);
}

/// Generate kuru conjugations.
pub(crate) fn kuru(mut r: impl FnMut(&'static str, &'static str, &[Form])) {
    r("く", "る", &[]);
    r("き", "ます", &[Honorific]);
    r("こ", "ない", &[Negative]);
    r("き", "ません", &[Negative, Honorific]);
    r("き", "た", &[Past]);
    r("き", "ました", &[Past, Honorific]);
    r("こ", "なかった", &[Past, Negative]);
    r("き", "ませんでした", &[Past, Negative, Honorific]);
    r("こ", "い", &[Command]);
    r("き", "なさい", &[Command, Honorific]);
    r("き", "てください", &[Command, Honorific, CommandTeKudasai]);
    r("く", "るな", &[Command, Negative]);
    r("こ", "ないでください", &[Command, Negative, Honorific]);
    r("く", "りゃ", &[Hypothetical, Conversation]);
    r("こ", "なけりゃ", &[Hypothetical, Conversation, Negative]);
    r("く", "れば", &[Hypothetical]);
    r("こ", "なければ", &[Hypothetical, Negative]);
    r("こ", "なきゃ", &[Hypothetical, Negative, Kya]);
    r("き", "たら", &[Conditional]);
    r("き", "ましたら", &[Conditional, Honorific]);
    r("こ", "なかったら", &[Conditional, Negative]);
    r("き", "ませんでしたら", &[Conditional, Negative, Honorific]);
    r("こ", "られる", &[Passive]);
    r("こ", "られます", &[Passive, Honorific]);
    r("こ", "られない", &[Passive, Negative]);
    r("こ", "られません", &[Passive, Negative, Honorific]);
    r("こ", "られた", &[Passive, Past]);
    r("こ", "られました", &[Passive, Past, Honorific]);
    r("こ", "られる", &[Potential]);
    r("こ", "よう", &[Volitional]);
    r("き", "ましょう", &[Volitional, Honorific]);
    r("く", "るだろう", &[Volitional, Darou]);
    r("く", "るでしょう", &[Volitional, Darou, Honorific]);
    r("こ", "ないだろう", &[Volitional, Negative]);
    r("こ", "ないでしょう", &[Volitional, Negative, Honorific]);
    r("こ", "させる", &[Causative]);
    r("こ", "させます", &[Causative, Honorific]);
    r("こ", "させない", &[Causative, Negative]);
    r("こ", "させません", &[Causative, Negative, Honorific]);
    r("き", "ながら", &[Simultaneous]);
    r("き", "そう", &[LooksLike]);
}

pub(crate) fn kuru_base(mut r: impl FnMut(&'static str, &'static str, &[Form])) {
    r("き", "", &[Stem]);
    r("き", "て", &[Te]);
    kuru(r);
}

/// Conjugations for a suru verb.
#[rustfmt::skip]
pub(crate) fn suru(mut r: impl FnMut(&'static str, &'static str, &[Form])) {
    r("す", "る", &[]);
    r("し", "ます", &[Honorific]);
    r("し", "ない", &[Negative]);
    r("し", "ません", &[Negative, Honorific]);
    r("し", "た", &[Past]);
    r("し", "ました", &[Past, Honorific]);
    r("し", "なかった", &[Past, Negative]);
    r("し", "ませんでした", &[Past, Negative, Honorific]);
    r("し", "ろ", &[Command]);
    r("し", "なさい", &[Command, Honorific]);
    r("し", "てください", &[Command, Honorific, CommandTeKudasai]);
    r("し", "よ", &[Command, CommandYo]);
    r("す", "るな", &[Command, Negative]);
    r("し", "ないでください", &[Command, Negative, Honorific]);
    r("す", "りゃ", &[Hypothetical, Conversation]);
    r("し", "なけりゃ", &[Hypothetical, Conversation, Negative]);
    r("す", "れば", &[Hypothetical]);
    r("し", "なければ", &[Hypothetical, Negative]);
    r("し", "なきゃ", &[Hypothetical, Negative, Kya]);
    r("し", "たら", &[Conditional]);
    r("し", "ましたら", &[Conditional, Honorific]);
    r("し", "なかったら", &[Conditional, Negative]);
    r("し", "ませんでしたら", &[Conditional, Negative, Honorific]);
    r("さ", "れる", &[Passive]);
    r("さ", "れます", &[Passive, Honorific]);
    r("さ", "れない", &[Passive, Negative]);
    r("さ", "れません", &[Passive, Negative, Honorific]);
    r("さ", "れた", &[Passive, Past]);
    r("さ", "れました", &[Passive, Past, Honorific]);
    r("で", "きる", &[Potential]);
    r("で", "きます", &[Potential, Honorific]);
    r("で", "きない", &[Potential, Negative]);
    r("で", "きません", &[Potential, Negative, Honorific]);
    r("で", "きた", &[Potential, Past]);
    r("で", "きました", &[Potential, Past, Honorific]);
    r("で", "きなかった", &[Potential, Past, Negative]);
    r("で", "きませんでした", &[Potential, Past, Negative, Honorific]);
    r("し", "よう", &[Volitional]);
    r("し", "ましょう", &[Volitional, Honorific]);
    r("す", "るだろう", &[Volitional, Darou]);
    r("す", "るでしょう", &[Volitional, Darou, Honorific]);
    r("し", "ないだろう", &[Volitional, Negative]);
    r("し", "ないでしょう", &[Volitional, Negative, Honorific]);
    r("し", "たろう", &[Volitional, Past]);
    r("し", "ましたろう", &[Volitional, Past, Honorific]);
    r("し", "ただろう", &[Volitional, Past, Darou]);
    r("し", "なかっただろう", &[Volitional, Past, Negative]);
    r("し", "なかったでしょう", &[Volitional, Past, Negative, Honorific]);
    r("さ", "せる", &[Causative]);
    r("し", "ながら", &[Simultaneous]);
    r("し", "そう", &[LooksLike]);
}

pub(crate) fn suru_base(mut r: impl FnMut(&'static str, &'static str, &[Form])) {
    r("し", "", &[Stem]);
    r("し", "て", &[Te]);
    suru(r);
}

#[cfg(fake)]
macro_rules! suru {
    ($macro:path) => {};
    ($macro:path, te) => {};
}

pub(crate) fn adjective_i(mut r: impl FnMut(&'static str, &[Form])) {
    r("い", &[]);
    r("いです", &[Honorific]);
    r("かった", &[Past]);
    r("かったです", &[Past, Honorific]);
    r("くない", &[Negative]);
    r("くないです", &[Negative, Honorific]);
    r("なかった", &[Past, Negative]);
    r("なかったです", &[Past, Negative, Honorific]);
    r("いよう", &[Volitional]);
}

pub(crate) fn adjective_ii(mut r: impl FnMut(&'static str, &'static str, &[Form])) {
    r("い", "い", &[]);
    r("い", "いです", &[Honorific]);
    r("よ", "かった", &[Past]);
    r("よ", "かったです", &[Past, Honorific]);
    r("よ", "くない", &[Negative]);
    r("よ", "くないです", &[Negative, Honorific]);
    r("よ", "なかった", &[Past, Negative]);
    r("よ", "なかったです", &[Past, Negative, Honorific]);
    r("い", "いよう", &[Volitional]);
}

pub(crate) fn adjective_na(mut r: impl FnMut(&'static str, &[Form])) {
    r("だ", &[]);
    r("です", &[Honorific]);
    r("だった", &[Past]);
    r("でした", &[Past, Honorific]);
    r("ではない", &[Negative]);
    r("ではありません", &[Negative, Honorific]);
    r("ではなかった", &[Past, Negative]);
    r("ではありませんでした", &[Past, Negative, Honorific]);
}

/// Helper to construct a particular [`Inflection`].
///
/// [`Inflection`]: crate::Inflection
///
/// # Examples
///
/// ```rust
/// jpv_lib::inflect!(Past);
/// jpv_lib::inflect!(Past, Honorific);
/// jpv_lib::inflect!(Past, Short);
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
