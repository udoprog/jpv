use std::rc::Rc;

use phrasebook::yew023::use_phrases;
use phrasebook::{Lang, Phrases};
use yew::prelude::*;

use crate::l10n::{self, GlossLang, SetLang};

/// Picks the UI's language, remembered in the browser.
#[function_component]
pub(crate) fn LanguagePicker() -> Html {
    let phrases = use_phrases();
    let set_lang = use_context::<SetLang>();

    let options = phrases
        .languages()
        .iter()
        .map(|language| {
            sockel::SelectOption::new(
                language.lang(),
                l10n::language_label(&phrases, language.lang(), language.name()),
            )
        })
        .collect::<Vec<_>>();

    let on_change = match set_lang {
        Some(SetLang(callback)) => callback,
        None => Callback::noop(),
    };

    html! {
        <sockel::Field id="language" label={html!({phrases.text("language")})}>
            <sockel::Select<Lang> id="language" {options} value={Some(phrases.lang())} {on_change} />
        </sockel::Field>
    }
}

/// Picks the language glosses are shown in, remembered in the browser.
/// Entries without glosses in it show their English ones.
#[function_component]
pub(crate) fn GlossLanguagePicker() -> Html {
    let phrases = use_phrases();
    let Some(gloss) = use_context::<GlossLang>() else {
        return Html::default();
    };

    let options = l10n::GLOSS_LANGUAGES
        .iter()
        .map(|&(code, _)| sockel::SelectOption::new(code, gloss_label(&phrases, code)))
        .collect::<Vec<_>>();

    html! {
        <sockel::Field id="gloss-language" label={html!({phrases.text("gloss-language")})} hint={html!({phrases.text("gloss-language-hint")})}>
            <sockel::Select<&'static str> id="gloss-language" {options} value={Some(gloss.code)} on_change={gloss.set} />
        </sockel::Field>
    }
}

/// A gloss language's name in the UI's language.
fn gloss_label(phrases: &Phrases, code: &str) -> Rc<str> {
    match code {
        "dut" => phrases.text("gloss-language-dut"),
        "fre" => phrases.text("gloss-language-fre"),
        "ger" => phrases.text("gloss-language-ger"),
        "hun" => phrases.text("gloss-language-hun"),
        "rus" => phrases.text("gloss-language-rus"),
        "slv" => phrases.text("gloss-language-slv"),
        "spa" => phrases.text("gloss-language-spa"),
        "swe" => phrases.text("gloss-language-swe"),
        _ => phrases.text("gloss-language-eng"),
    }
}
