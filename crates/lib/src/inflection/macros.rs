//! Macros to construct conjugations.

use crate::inflection::godan::{self, Godan};
use crate::inflection::Form;

use Form::*;

/// Perform ichidan conjugations.
pub fn ichidan(mut r: impl FnMut(&'static str, &[Form])) {
    r("る", &[]);
    r("ます", &[Polite]);
    r("ない", &[Negative]);
    r("ません", &[Negative, Polite]);
    r("た", &[Past]);
    r("ました", &[Past, Polite]);
    r("なかった", &[Past, Negative]);
    r("ませんでした", &[Past, Negative, Polite]);
    r("ろ", &[Command]);
    r("なさい", &[Command, Polite]);
    r("てください", &[Command, Polite, Kudasai]);
    r("よ", &[Command, Yo]);
    r("るな", &[Command, Negative]);
    r("ないでください", &[Command, Negative, Polite]);
    r("ば", &[Hypothetical]);
    r("なければ", &[Hypothetical, Negative]);
    r("なきゃ", &[Hypothetical, Negative, Kya]);
    r("たら", &[Conditional]);
    r("ましたら", &[Conditional, Polite]);
    r("なかったら", &[Conditional, Negative]);
    r("ませんでしたら", &[Conditional, Negative, Polite]);
    r("れる", &[Passive, Conversation]);
    r("られる", &[Passive]);
    r("られます", &[Passive, Polite]);
    r("られない", &[Passive, Negative]);
    r("られません", &[Passive, Negative, Polite]);
    r("られた", &[Passive, Past]);
    r("られました", &[Passive, Past, Polite]);
    r("られる", &[Potential]);
    r("られます", &[Potential, Polite]);
    r("られない", &[Potential, Negative]);
    r("られません", &[Potential, Negative, Polite]);
    r("られた", &[Potential, Past]);
    r("られました", &[Potential, Past, Polite]);
    r("られなかった", &[Potential, Past, Negative]);
    r("られませんでした", &[Potential, Past, Negative, Polite]);
    r("よう", &[Volitional]);
    r("ましょう", &[Volitional, Polite]);
    r("るだろう", &[Volitional, Darou]);
    r("るでしょう", &[Volitional, Darou, Polite]);
    r("ないだろう", &[Volitional, Negative]);
    r("ないでしょう", &[Volitional, Negative, Polite]);
    r("させる", &[Causative]);
    r("たい", &[Tai]);
    r("たくない", &[Tai, Negative]);
    r("たかった", &[Tai, Past]);
    r("たくなかった", &[Tai, Past, Negative]);
}

pub fn ichidan_te(mut r: impl FnMut(&'static str, &[Form])) {
    r("", &[Stem]);
    r("て", &[Te]);
    ichidan(r);
}

pub(crate) fn godan_lit(g: &'static Godan, mut r: impl FnMut(&'static str, &'static str, &[Form])) {
    r("", g.u, &[]);
    r("", g.past, &[Past]);
    r("", g.past_conditional, &[Conditional]);
    r("", g.e, &[Command]);
    r(g.i, "ます", &[Polite]);
    r(g.a, "ない", &[Negative]);
    r(g.i, "ません", &[Negative, Polite]);
    r(g.i, "ました", &[Past, Polite]);
    r(g.a, "なかった", &[Past, Negative]);
    r(g.i, "ませんでした", &[Past, Negative, Polite]);
    r(g.i, "なさい", &[Command, Polite]);
    r(g.te, "ください", &[Command, Polite, Kudasai]);
    r(g.e, "よ", &[Command, Yo]);
    r(g.u, "な", &[Command, Negative]);
    r(g.a, "ないでください", &[Command, Negative, Polite]);
    r(g.e, "ば", &[Hypothetical]);
    r(g.a, "なければ", &[Hypothetical, Negative]);
    r(g.a, "なきゃ", &[Hypothetical, Negative, Kya]);
    r(g.i, "ましたら", &[Conditional, Polite]);
    r(g.a, "なかったら", &[Conditional, Negative]);
    r(g.i, "ませんでしたら", &[Conditional, Negative, Polite]);
    r(g.a, "れる", &[Passive]);
    r(g.a, "れます", &[Passive, Polite]);
    r(g.a, "れない", &[Passive, Negative]);
    r(g.a, "れません", &[Passive, Negative, Polite]);
    r(g.a, "れた", &[Passive, Past]);
    r(g.a, "れました", &[Passive, Past, Polite]);
    r(g.e, "る", &[Potential]);
    r(g.e, "ます", &[Potential, Polite]);
    r(g.e, "ない", &[Potential, Negative]);
    r(g.e, "ません", &[Potential, Negative, Polite]);
    r(g.e, "た", &[Potential, Past]);
    r(g.e, "ました", &[Potential, Past, Polite]);
    r(g.e, "なかった", &[Potential, Past, Negative]);
    r(g.e, "ませんでした", &[Potential, Past, Negative, Polite]);
    r(g.o, "う", &[Volitional]);
    r(g.i, "ましょう", &[Volitional, Polite]);
    r(g.u, "だろう", &[Volitional, Darou]);
    r(g.u, "でしょう", &[Volitional, Darou, Polite]);
    r(g.a, "ないだろう", &[Volitional, Negative]);
    r(g.a, "ないでしょう", &[Volitional, Negative, Polite]);
    r(g.a, "せる", &[Causative]);
    r(g.i, "たい", &[Tai]);
    r(g.i, "たくない", &[Tai, Negative]);
    r(g.i, "たかった", &[Tai, Past]);
    r(g.i, "たくなかった", &[Tai, Past, Negative]);
}

pub(crate) fn godan_u(r: impl FnMut(&'static str, &'static str, &[Form])) {
    godan_lit(godan::U, r);
}

pub(crate) fn godan_u_base(mut r: impl FnMut(&'static str, &'static str, &[Form])) {
    r("", "い", &[Stem]);
    r("", "って", &[Te]);
    godan_lit(godan::U, r);
}

pub(crate) fn godan_iku(r: impl FnMut(&'static str, &'static str, &[Form])) {
    godan_lit(godan::IKU, r);
}

pub(crate) fn godan_iku_base(mut r: impl FnMut(&'static str, &'static str, &[Form])) {
    r("", "き", &[Stem]);
    r("", "って", &[Te]);
    godan_lit(godan::IKU, r);
}

pub(crate) fn godan_tsu_base(mut r: impl FnMut(&'static str, &'static str, &[Form])) {
    r("", "ち", &[Stem]);
    r("", "って", &[Te]);
    godan_lit(godan::TSU, r);
}

pub(crate) fn godan_ru(r: impl FnMut(&'static str, &'static str, &[Form])) {
    godan_lit(godan::RU, r);
}

pub(crate) fn godan_ru_base(mut r: impl FnMut(&'static str, &'static str, &[Form])) {
    r("", "り", &[Stem]);
    r("", "って", &[Te]);
    godan_lit(godan::RU, r);
}

pub(crate) fn godan_ku(r: impl FnMut(&'static str, &'static str, &[Form])) {
    godan_lit(godan::KU, r);
}

pub(crate) fn godan_ku_base(mut r: impl FnMut(&'static str, &'static str, &[Form])) {
    r("", "き", &[Stem]);
    r("", "いて", &[Te]);
    godan_lit(godan::KU, r);
}

pub(crate) fn godan_gu_base(mut r: impl FnMut(&'static str, &'static str, &[Form])) {
    r("", "ぎ", &[Stem]);
    r("", "いで", &[Te]);
    godan_lit(godan::GU, r);
}

pub(crate) fn godan_mu_base(mut r: impl FnMut(&'static str, &'static str, &[Form])) {
    r("", "み", &[Stem]);
    r("", "んで", &[Te]);
    godan_lit(godan::MU, r);
}

pub(crate) fn godan_bu_base(mut r: impl FnMut(&'static str, &'static str, &[Form])) {
    r("", "び", &[Stem]);
    r("", "んで", &[Te]);
    godan_lit(godan::BU, r);
}

pub(crate) fn godan_nu_base(mut r: impl FnMut(&'static str, &'static str, &[Form])) {
    r("", "に", &[Stem]);
    r("", "んで", &[Te]);
    godan_lit(godan::NU, r);
}

pub(crate) fn godan_su_base(mut r: impl FnMut(&'static str, &'static str, &[Form])) {
    r("", "し", &[Stem]);
    r("", "して", &[Te]);
    godan_lit(godan::SU, r);
}

/// Generate kuru conjugations.
pub(crate) fn kuru(mut r: impl FnMut(&'static str, &'static str, &[Form])) {
    r("く", "る", &[]);
    r("き", "ます", &[Polite]);
    r("こ", "ない", &[Negative]);
    r("き", "ません", &[Negative, Polite]);
    r("き", "た", &[Past]);
    r("き", "ました", &[Past, Polite]);
    r("こ", "なかった", &[Past, Negative]);
    r("き", "ませんでした", &[Past, Negative, Polite]);
    r("こ", "い", &[Command]);
    r("き", "なさい", &[Command, Polite]);
    r("き", "てください", &[Command, Polite, Kudasai]);
    r("く", "るな", &[Command, Negative]);
    r("こ", "ないでください", &[Command, Negative, Polite]);
    r("く", "れば", &[Hypothetical]);
    r("こ", "なければ", &[Hypothetical, Negative]);
    r("こ", "なきゃ", &[Hypothetical, Negative, Kya]);
    r("き", "たら", &[Conditional]);
    r("き", "ましたら", &[Conditional, Polite]);
    r("こ", "なかったら", &[Conditional, Negative]);
    r("き", "ませんでしたら", &[Conditional, Negative, Polite]);
    r("こ", "られる", &[Passive]);
    r("こ", "られます", &[Passive, Polite]);
    r("こ", "られない", &[Passive, Negative]);
    r("こ", "られません", &[Passive, Negative, Polite]);
    r("こ", "られた", &[Passive, Past]);
    r("こ", "られました", &[Passive, Past, Polite]);
    r("こ", "られる", &[Potential]);
    r("こ", "よう", &[Volitional]);
    r("き", "ましょう", &[Volitional, Polite]);
    r("く", "るだろう", &[Volitional, Darou]);
    r("く", "るでしょう", &[Volitional, Darou, Polite]);
    r("こ", "ないだろう", &[Volitional, Negative]);
    r("こ", "ないでしょう", &[Volitional, Negative, Polite]);
    r("こ", "させる", &[Causative]);
    r("こ", "させます", &[Causative, Polite]);
    r("こ", "させない", &[Causative, Negative]);
    r("こ", "させません", &[Causative, Negative, Polite]);
    r("き", "たい", &[Tai]);
    r("き", "たくない", &[Tai, Negative]);
    r("き", "たかった", &[Tai, Past]);
    r("き", "たくなかった", &[Tai, Past, Negative]);
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
    r("し", "ます", &[Polite]);
    r("し", "ない", &[Negative]);
    r("し", "ません", &[Negative, Polite]);
    r("し", "た", &[Past]);
    r("し", "ました", &[Past, Polite]);
    r("し", "なかった", &[Past, Negative]);
    r("し", "ませんでした", &[Past, Negative, Polite]);
    r("し", "ろ", &[Command]);
    r("し", "なさい", &[Command, Polite]);
    r("し", "てください", &[Command, Polite, Kudasai]);
    r("し", "よ", &[Command, Yo]);
    r("す", "るな", &[Command, Negative]);
    r("し", "ないでください", &[Command, Negative, Polite]);
    r("す", "れば", &[Hypothetical]);
    r("し", "なければ", &[Hypothetical, Negative]);
    r("し", "なきゃ", &[Hypothetical, Negative, Kya]);
    r("し", "たら", &[Conditional]);
    r("し", "ましたら", &[Conditional, Polite]);
    r("し", "なかったら", &[Conditional, Negative]);
    r("し", "ませんでしたら", &[Conditional, Negative, Polite]);
    r("さ", "れる", &[Passive]);
    r("さ", "れます", &[Passive, Polite]);
    r("さ", "れない", &[Passive, Negative]);
    r("さ", "れません", &[Passive, Negative, Polite]);
    r("さ", "れた", &[Passive, Past]);
    r("さ", "れました", &[Passive, Past, Polite]);
    r("で", "きる", &[Potential]);
    r("で", "きます", &[Potential, Polite]);
    r("で", "きない", &[Potential, Negative]);
    r("で", "きません", &[Potential, Negative, Polite]);
    r("で", "きた", &[Potential, Past]);
    r("で", "きました", &[Potential, Past, Polite]);
    r("で", "きなかった", &[Potential, Past, Negative]);
    r("で", "きませんでした", &[Potential, Past, Negative, Polite]);
    r("し", "よう", &[Volitional]);
    r("し", "ましょう", &[Volitional, Polite]);
    r("す", "るだろう", &[Volitional, Darou]);
    r("す", "るでしょう", &[Volitional, Darou, Polite]);
    r("し", "ないだろう", &[Volitional, Negative]);
    r("し", "ないでしょう", &[Volitional, Negative, Polite]);
    r("し", "たろう", &[Volitional, Past]);
    r("し", "ましたろう", &[Volitional, Past, Polite]);
    r("し", "ただろう", &[Volitional, Past, Darou]);
    r("し", "なかっただろう", &[Volitional, Past, Negative]);
    r("し", "なかったでしょう", &[Volitional, Past, Negative, Polite]);
    r("さ", "せる", &[Causative]);
    r("し", "たい", &[Tai]);
    r("し", "たくない", &[Tai, Negative]);
    r("し", "たかった", &[Tai, Past]);
    r("し", "たくなかった", &[Tai, Past, Negative]);
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
    r("いです", &[Polite]);
    r("かった", &[Past]);
    r("かったです", &[Past, Polite]);
    r("くない", &[Negative]);
    r("くないです", &[Negative, Polite]);
    r("なかった", &[Past, Negative]);
    r("なかったです", &[Past, Negative, Polite]);
    r("いよう", &[Volitional]);
}

pub(crate) fn adjective_ii(mut r: impl FnMut(&'static str, &'static str, &[Form])) {
    r("い", "い", &[]);
    r("い", "いです", &[Polite]);
    r("よ", "かった", &[Past]);
    r("よ", "かったです", &[Past, Polite]);
    r("よ", "くない", &[Negative]);
    r("よ", "くないです", &[Negative, Polite]);
    r("よ", "なかった", &[Past, Negative]);
    r("よ", "なかったです", &[Past, Negative, Polite]);
    r("い", "いよう", &[Volitional]);
}

pub(crate) fn adjective_na(mut r: impl FnMut(&'static str, &[Form])) {
    r("だ", &[]);
    r("です", &[Polite]);
    r("だった", &[Past]);
    r("でした", &[Past, Polite]);
    r("ではない", &[Negative]);
    r("ではありません", &[Negative, Polite]);
    r("ではなかった", &[Past, Negative]);
    r("ではありませんでした", &[Past, Negative, Polite]);
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
