//! The web UI's translations: jpv's own catalog and sockel's, in one
//! [`Phrasebook`].
//!
//! Every word the UI shows is a message in `locales/*.ftl`, looked up by id
//! through the [`Phrases`] handle the root provides. Dictionary content
//! (glosses) is shown as JMdict provides it.

use phrasebook::{koine03, Args, Catalog, Lang, Phrasebook, Phrases, Source};
use yew::prelude::*;

/// Japanese.
pub(crate) const JA: Lang = Lang::new("ja");

/// jpv's own messages.
pub(crate) static L10N: Catalog = Catalog::new(
    "jpv",
    &[
        Source::new(Lang::EN_US, include_str!("../locales/en-US.ftl")),
        Source::new(JA, include_str!("../locales/ja.ftl")),
    ],
);

/// The languages the UI ships, each named in itself, in picker order.
const LANGUAGES: &[(Lang, &str)] = &[(Lang::EN_US, "English"), (JA, "日本語")];

pub(crate) fn phrasebook() -> Phrasebook {
    Phrasebook::builder()
        .languages(LANGUAGES.iter().copied())
        .catalog(&L10N)
        .catalog(&sockel::phrasebookcatalog01::L10N)
        .build()
}

/// What changes the UI's language, provided by the root next to
/// [`Phrases`].
#[derive(Clone, PartialEq)]
pub(crate) struct SetLang(pub(crate) Callback<Lang>);

/// The gloss languages JMdict has, as its ISO 639-2 codes, each with the
/// primary subtag of the UI language that defaults to it, in picker order.
pub(crate) const GLOSS_LANGUAGES: &[(&str, &str)] = &[
    ("eng", "en"),
    ("dut", "nl"),
    ("fre", "fr"),
    ("ger", "de"),
    ("hun", "hu"),
    ("rus", "ru"),
    ("slv", "sl"),
    ("spa", "es"),
    ("swe", "sv"),
];

/// English, the gloss language every entry has.
pub(crate) const GLOSS_DEFAULT: &str = "eng";

const GLOSS_KEY: &str = "jpv-gloss-lang";

/// The gloss language that goes with a UI language, English when JMdict has
/// none in it.
pub(crate) fn gloss_for(lang: Lang) -> &'static str {
    GLOSS_LANGUAGES
        .iter()
        .find(|(_, ui)| *ui == lang.language())
        .map_or(GLOSS_DEFAULT, |(code, _)| *code)
}

/// The gloss language chosen in this browser, if any.
pub(crate) fn stored_gloss() -> Option<&'static str> {
    let value = web_sys::window()
        .and_then(|w| w.local_storage().ok().flatten())
        .and_then(|s| s.get_item(GLOSS_KEY).ok().flatten())?;

    GLOSS_LANGUAGES
        .iter()
        .find(|(code, _)| *code == value)
        .map(|(code, _)| *code)
}

/// Remembers the gloss language in this browser.
pub(crate) fn store_gloss(code: &str) {
    let Some(storage) = web_sys::window().and_then(|w| w.local_storage().ok().flatten()) else {
        return;
    };

    if let Err(error) = storage.set_item(GLOSS_KEY, code) {
        log::warn!("Failed to store gloss language: {error:?}");
    }
}

/// The gloss language glosses are shown in, and what changes it, provided by
/// the root next to [`Phrases`].
#[derive(Clone, PartialEq)]
pub(crate) struct GlossLang {
    pub(crate) code: &'static str,
    pub(crate) set: Callback<&'static str>,
}

/// A language as the picker shows it: its name in itself, and koine's
/// English name of it when that differs.
pub(crate) fn language_label(phrases: &Phrases, lang: Lang, native: &str) -> String {
    match koine03::name(lang) {
        Some(english) if english != native => phrases.msg(
            "language-option",
            &phrasebook::args!(native = native, english = english),
        ),
        _ => native.to_owned(),
    }
}

/// A message whose arguments are elements, such as links, placed where the
/// translation puts them. Like [`phrasebook::yew023::html_around`] for any
/// number of arguments; an element the translation leaves out is placed
/// after its text, so nothing is lost.
pub(crate) fn rich<const N: usize>(phrases: &Phrases, id: &str, parts: [(&str, Html); N]) -> Html {
    // Private use characters, which no message contains, one per part.
    const BASE: u32 = 0xE000;
    const ISOLATES: [char; 2] = ['\u{2068}', '\u{2069}'];

    let marks = (0..N as u32)
        .map(|n| {
            BASE.checked_add(n)
                .and_then(char::from_u32)
                .unwrap_or_default()
                .to_string()
        })
        .collect::<Vec<_>>();

    let mut args = Args::new();

    for ((name, _), mark) in parts.iter().zip(&marks) {
        args.set(name, mark.as_str());
    }

    let text = phrases.msg(id, &args);
    let mut parts = parts.map(|(_, html)| Some(html));
    let mut nodes = Vec::new();
    let mut buf = String::new();

    for c in text.chars() {
        if ISOLATES.contains(&c) {
            continue;
        }

        let index = (c as u32).wrapping_sub(BASE) as usize;

        if let Some(part) = parts.get_mut(index) {
            if !buf.is_empty() {
                nodes.push(Html::from(std::mem::take(&mut buf)));
            }

            nodes.extend(part.take());
            continue;
        }

        buf.push(c);
    }

    if !buf.is_empty() {
        nodes.push(Html::from(buf));
    }

    nodes.extend(parts.into_iter().flatten());
    nodes.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;

    use phrasebook::{check, koine03};

    use super::*;

    /// The ids the code spells out as the first argument of `text`, `msg`
    /// and `msg_around` calls on a handle, and the second of [`rich`].
    fn asked_ids() -> Vec<String> {
        fn walk(dir: &Path, out: &mut Vec<String>) {
            for entry in fs::read_dir(dir).unwrap() {
                let path = entry.unwrap().path();

                if path.is_dir() {
                    walk(&path, out);
                    continue;
                }

                if path.extension().is_none_or(|e| e != "rs") {
                    continue;
                }

                let source = fs::read_to_string(&path).unwrap();

                // Spelled apart so this scan does not find itself.
                for (needle, skip_first) in [
                    (concat!(".", "text("), false),
                    (concat!(".", "msg("), false),
                    (concat!(".", "msg_around("), false),
                    (concat!("l10n::", "rich("), true),
                ] {
                    for (at, _) in source.match_indices(needle) {
                        let mut rest = &source[at + needle.len()..];

                        if skip_first {
                            rest = &rest[rest.find(',').unwrap() + 1..];
                        }

                        let rest = rest.trim_start();

                        if let Some(rest) = rest.strip_prefix('"') {
                            let end = rest.find('"').unwrap();
                            out.push(rest[..end].to_owned());
                        }
                    }
                }
            }
        }

        let mut out = Vec::new();
        walk(&Path::new(env!("CARGO_MANIFEST_DIR")).join("src"), &mut out);
        out
    }

    #[test]
    fn catalog() {
        check::all(&L10N, &[&sockel::phrasebookcatalog01::L10N]).unwrap();
        assert!(phrasebook().has_catalog(&sockel::phrasebookcatalog01::L10N));
    }

    #[test]
    fn coverage() {
        let asked = asked_ids();
        check::coverage(
            &L10N,
            Lang::EN_US,
            asked.iter().map(String::as_str),
            &[&sockel::phrasebookcatalog01::L10N],
        )
        .unwrap();
    }

    #[test]
    fn languages_known_to_koine() {
        for &(lang, _) in LANGUAGES {
            assert!(koine03::is_known(lang), "{lang} is not known to koine");
            assert!(koine03::name(lang).is_some());
        }
    }

    #[test]
    fn gloss_defaults_from_ui_language() {
        assert_eq!(gloss_for(Lang::EN_US), "eng");
        assert_eq!(gloss_for(JA), "eng");
        assert_eq!(gloss_for(Lang::new("de-AT")), "ger");
        assert_eq!(gloss_for(Lang::new("sv")), "swe");
    }
}
