//! jpv's action icons, added to sockel's [`sockel::IconSet`] by name and
//! drawn like sockel's own glyphs: a 16 unit grid, stroked in
//! `currentColor`.

use yew::prelude::*;

pub(crate) const SETTINGS: &str = "settings";
pub(crate) const WINDOW: &str = "window";
pub(crate) const CLIPBOARD: &str = "clipboard";
pub(crate) const BACK: &str = "back";
pub(crate) const EDIT: &str = "edit";
pub(crate) const DOWNLOAD: &str = "download";
pub(crate) const RESET: &str = "reset";
pub(crate) const DATA: &str = "data";
pub(crate) const HELP: &str = "help";
pub(crate) const SAVE: &str = "save";
pub(crate) const MODE: &str = "mode";

fn svg(paths: Html) -> Html {
    html! {
        <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5"
            stroke-linecap="round" stroke-linejoin="round" aria-hidden="true" focusable="false">
            { paths }
        </svg>
    }
}

/// The icon set the root provides.
pub(crate) fn set() -> sockel::IconSet {
    sockel::IconSet::default()
        .with_named(
            SETTINGS,
            svg(html! {
                <>
                    <circle cx="8" cy="8" r="2" />
                    <path d="M8 1.5v2M8 12.5v2M1.5 8h2M12.5 8h2M3.4 3.4l1.4 1.4M11.2 11.2l1.4 1.4M3.4 12.6l1.4-1.4M11.2 4.8l1.4-1.4" />
                    <circle cx="8" cy="8" r="4.5" />
                </>
            }),
        )
        .with_named(
            WINDOW,
            svg(html! { <><path d="M9 2.5h4.5V7M13.5 2.5L8 8" /><path d="M11.5 9.5v3a1 1 0 0 1-1 1h-7a1 1 0 0 1-1-1v-7a1 1 0 0 1 1-1h3" /></> }),
        )
        .with_named(
            CLIPBOARD,
            svg(html! { <><rect x="3" y="3" width="10" height="11" rx="1.5" /><path d="M6 3V2h4v1" /><path d="M5.5 7h5M5.5 10h3" /></> }),
        )
        .with_named(BACK, svg(html! { <path d="M13 8H3M7 4L3 8l4 4" /> }))
        .with_named(
            EDIT,
            svg(html! { <path d="M10.5 2.5l3 3-8 8H2.5v-3z M9 4l3 3" /> }),
        )
        .with_named(
            DOWNLOAD,
            svg(html! { <path d="M8 2v8M4.5 6.5L8 10l3.5-3.5M2.5 13.5h11" /> }),
        )
        .with_named(
            RESET,
            svg(html! { <><path d="M3 8a5 5 0 1 0 1.5-3.5" /><path d="M3 2.5v3h3" /></> }),
        )
        .with_named(
            DATA,
            svg(html! { <path d="M5.5 3C4 3 4 4 4 5.5S3.5 8 2.5 8c1 0 1.5 1 1.5 2.5S4 13 5.5 13M10.5 3C12 3 12 4 12 5.5s.5 2.5 1.5 2.5c-1 0-1.5 1-1.5 2.5S12 13 10.5 13" /> }),
        )
        .with_named(
            HELP,
            svg(html! { <><circle cx="8" cy="8" r="6" /><path d="M6.2 6.2a1.9 1.9 0 1 1 2.6 1.8c-.5.2-.8.6-.8 1.1v.4" /><circle cx="8" cy="11.5" r=".6" fill="currentColor" stroke="none" /></> }),
        )
        .with_named(
            SAVE,
            svg(html! { <><path d="M3 2.5h8l2.5 2.5v8.5h-11v-11z" /><path d="M5 2.5v3h5v-3M5 13.5v-4h6v4" /></> }),
        )
        .with_named(
            MODE,
            svg(html! { <><path d="M2.5 4h11M2.5 8h11M2.5 12h11" /><circle cx="5" cy="4" r="1.2" fill="currentColor" /><circle cx="10.5" cy="8" r="1.2" fill="currentColor" /><circle cx="7" cy="12" r="1.2" fill="currentColor" /></> }),
        )
}

/// A named jpv icon.
pub(crate) fn named(name: &'static str) -> Html {
    html!(<sockel::Icon name={AttrValue::Static(name)} />)
}

/// One of sockel's own glyphs.
pub(crate) fn glyph(glyph: sockel::Glyph) -> Html {
    html!(<sockel::Icon {glyph} />)
}
