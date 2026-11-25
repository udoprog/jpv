//! The web UI's translations: jpv's own catalog and sockel's, in one
//! [`Phrasebook`].
//!
//! Every word the UI shows is a message in `locales/*.ftl`, looked up by id
//! through the [`Phrases`] handle the root provides. Dictionary content
//! (glosses) is shown as JMdict provides it.

use phrasebook::{koine03, Catalog, Lang, Phrasebook, Phrases, Source};
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

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;

    use phrasebook::{check, koine03};

    use super::*;

    /// The ids the code spells out as the first argument of `text` and
    /// `msg` calls on a handle.
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
                for needle in [concat!(".", "text("), concat!(".", "msg(")] {
                    for (at, _) in source.match_indices(needle) {
                        let rest = source[at + needle.len()..].trim_start();

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
}
