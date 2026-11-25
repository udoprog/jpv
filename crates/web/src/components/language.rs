use phrasebook::yew023::use_phrases;
use phrasebook::Lang;
use yew::prelude::*;

use crate::l10n::{self, SetLang};

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
