use yew::prelude::*;

const KEY: &str = "jpv-theme";
const SYSTEM: &str = "system";
const LIGHT: &str = "light";
const DARK: &str = "dark";

fn stored() -> AttrValue {
    let value = web_sys::window()
        .and_then(|w| w.local_storage().ok().flatten())
        .and_then(|s| s.get_item(KEY).ok().flatten());

    match value.as_deref() {
        Some(LIGHT) => AttrValue::Static(LIGHT),
        Some(DARK) => AttrValue::Static(DARK),
        _ => AttrValue::Static(SYSTEM),
    }
}

/// Sets `data-theme` on the document, which the stylesheet's palettes read.
fn apply(theme: &str) {
    let Some(root) = web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.document_element())
    else {
        return;
    };

    let result = match theme {
        LIGHT | DARK => root.set_attribute("data-theme", theme),
        _ => root.remove_attribute("data-theme"),
    };

    if let Err(error) = result {
        log::warn!("Failed to apply theme: {error:?}");
    }
}

fn store(theme: &str) {
    let Some(storage) = web_sys::window().and_then(|w| w.local_storage().ok().flatten()) else {
        return;
    };

    let result = match theme {
        LIGHT | DARK => storage.set_item(KEY, theme),
        _ => storage.remove_item(KEY),
    };

    if let Err(error) = result {
        log::warn!("Failed to store theme: {error:?}");
    }
}

/// Applies the stored theme, before the first render.
pub(crate) fn init() {
    apply(&stored());
}

#[function_component]
pub(crate) fn ThemeSwitch() -> Html {
    let theme = use_state(stored);

    let on_change = {
        let theme = theme.clone();

        Callback::from(move |next: AttrValue| {
            apply(&next);
            store(&next);
            theme.set(next);
        })
    };

    let options = vec![
        sockel::ChoiceOption::new(SYSTEM, "System"),
        sockel::ChoiceOption::new(LIGHT, "Light"),
        sockel::ChoiceOption::new(DARK, "Dark"),
    ];

    html! {
        <sockel::Cluster>
            <sockel::Choice label="Theme" {options} value={Some((*theme).clone())} {on_change} />
        </sockel::Cluster>
    }
}
