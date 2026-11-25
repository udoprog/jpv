//! A dialog for choosing a file on the service's machine, to open or to save
//! to, since a browser cannot hand the service a path of its own.
//!
//! It knows nothing of how directories are listed: the owner answers each
//! [`ListRequest`] through `list`, so the dialog can move into sockel as is.

use std::rc::Rc;

use phrasebook::yew023::use_phrases;
use yew::prelude::*;

use crate::class;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum FileBrowserMode {
    /// Choose an existing file.
    #[default]
    Open,
    /// Choose a path to save to, which may not exist yet.
    #[allow(dead_code, reason = "nothing in jpv saves to a chosen file yet")]
    Save,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum EntryKind {
    Directory,
    File,
    Other,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ListingEntry {
    pub(crate) name: String,
    pub(crate) kind: EntryKind,
    pub(crate) size: Option<u64>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PlaceKind {
    Home,
    Documents,
    Downloads,
    Root,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Place {
    pub(crate) kind: PlaceKind,
    pub(crate) path: String,
}

/// One listed directory.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Listing {
    /// Its absolute path.
    pub(crate) path: String,
    pub(crate) parent: Option<String>,
    /// What joins a directory and a name on the listing machine.
    pub(crate) separator: String,
    pub(crate) entries: Vec<ListingEntry>,
    /// The name in it the requested path named, when that was not a
    /// directory.
    pub(crate) file: Option<String>,
    pub(crate) places: Vec<Place>,
}

impl Listing {
    fn join(&self, name: &str) -> String {
        if self.path.ends_with(self.separator.as_str()) {
            format!("{}{name}", self.path)
        } else {
            format!("{}{}{name}", self.path, self.separator)
        }
    }

    fn find(&self, name: &str) -> Option<&ListingEntry> {
        self.entries.iter().find(|e| e.name == name)
    }
}

/// Asks the owner to list `path`: a directory, or a file to list the
/// directory of; empty for the user's home. With `nearest`, a missing path
/// lists its closest existing ancestor instead of failing. A newer request
/// replaces an older one, whose answer is then ignored.
pub(crate) struct ListRequest {
    pub(crate) path: String,
    pub(crate) nearest: bool,
    pub(crate) reply: Callback<Result<Listing, String>>,
}

#[derive(Properties, PartialEq)]
pub(crate) struct Props {
    pub(crate) open: bool,
    #[prop_or_default]
    pub(crate) mode: FileBrowserMode,
    pub(crate) title: AttrValue,
    /// Where it opens: a file or a directory, empty for the user's home.
    #[prop_or_default]
    pub(crate) path: AttrValue,
    /// The name suffixes a chosen file must have (`.gz`); any when empty.
    #[prop_or_default]
    pub(crate) extensions: Vec<AttrValue>,
    pub(crate) list: Callback<ListRequest>,
    /// The chosen path.
    pub(crate) on_choose: Callback<String>,
    pub(crate) on_dismiss: Callback<()>,
}

#[derive(Clone, Default, PartialEq)]
struct State {
    listing: Option<Rc<Listing>>,
    error: Option<String>,
    loading: bool,
    /// The editable location, the listed path unless being typed.
    location: String,
    /// The chosen name in the listed directory.
    name: String,
}

enum Action {
    Reset,
    Loading,
    Listed(FileBrowserMode, Listing),
    Failed(String),
    Location(String),
    Name(String),
}

impl Reducible for State {
    type Action = Action;

    fn reduce(self: Rc<Self>, action: Action) -> Rc<Self> {
        let mut state = (*self).clone();

        match action {
            Action::Reset => return Rc::new(State::default()),
            Action::Loading => {
                state.loading = true;
                state.error = None;
            }
            Action::Listed(mode, listing) => {
                // Saving keeps a typed name while browsing to where it goes.
                if mode == FileBrowserMode::Open || listing.file.is_some() {
                    state.name = listing.file.clone().unwrap_or_default();
                }

                state.location = listing.path.clone();
                state.listing = Some(Rc::new(listing));
                state.loading = false;
                state.error = None;
            }
            Action::Failed(error) => {
                state.loading = false;
                state.error = Some(error);
            }
            Action::Location(location) => state.location = location,
            Action::Name(name) => state.name = name,
        }

        Rc::new(state)
    }
}

#[function_component]
pub(crate) fn FileBrowser(props: &Props) -> Html {
    let phrases = use_phrases();
    let state = use_reducer(State::default);
    let latest = use_mut_ref(|| 0u32);
    let show_hidden = use_state_eq(|| false);
    let mode = props.mode;

    let navigate = {
        let dispatcher = state.dispatcher();
        let latest = latest.clone();
        let list = props.list.clone();

        Callback::from(move |(path, nearest): (String, bool)| {
            let serial = {
                let mut latest = latest.borrow_mut();
                *latest = latest.wrapping_add(1);
                *latest
            };

            dispatcher.dispatch(Action::Loading);

            let dispatcher = dispatcher.clone();
            let latest = latest.clone();

            let reply = Callback::from(move |result: Result<Listing, String>| {
                if *latest.borrow() != serial {
                    return;
                }

                dispatcher.dispatch(match result {
                    Ok(listing) => Action::Listed(mode, listing),
                    Err(error) => Action::Failed(error),
                });
            });

            list.emit(ListRequest {
                path,
                nearest,
                reply,
            });
        })
    };

    {
        let navigate = navigate.clone();
        let dispatcher = state.dispatcher();
        let latest = latest.clone();
        let path = props.path.to_string();

        use_effect_with(props.open, move |&open| {
            if open {
                dispatcher.dispatch(Action::Reset);
                navigate.emit((path, true));
            } else {
                let mut latest = latest.borrow_mut();
                *latest = latest.wrapping_add(1);
            }
        });
    }

    let listing = state.listing.clone();

    let accepts = {
        let extensions = props.extensions.clone();
        move |name: &str| extensions.is_empty() || extensions.iter().any(|e| name.ends_with(&**e))
    };

    let set_name = {
        let dispatcher = state.dispatcher();
        Callback::from(move |name: String| dispatcher.dispatch(Action::Name(name)))
    };

    let chosen = listing.as_ref().and_then(|listing| {
        let name = state.name.trim();

        if name.is_empty() || !accepts(name) {
            return None;
        }

        match (mode, listing.find(name).map(|e| e.kind)) {
            (FileBrowserMode::Open, Some(EntryKind::File)) => Some(listing.join(name)),
            (FileBrowserMode::Save, None | Some(EntryKind::File)) => Some(listing.join(name)),
            _ => None,
        }
    });

    let replaces = mode == FileBrowserMode::Save
        && listing.as_ref().is_some_and(|listing| {
            listing
                .find(state.name.trim())
                .is_some_and(|e| e.kind == EntryKind::File)
        });

    let current = state.name.trim();
    let mut rows = Vec::new();

    if let Some(listing) = &listing {
        let visible = listing
            .entries
            .iter()
            .filter(|e| *show_hidden || !e.name.starts_with('.'));

        for entry in visible {
            let path = listing.join(&entry.name);
            let is_dir = entry.kind == EntryKind::Directory;

            let onclick = {
                let navigate = navigate.clone();
                let set_name = set_name.clone();
                let name = entry.name.clone();

                Callback::from(move |_: MouseEvent| {
                    if is_dir {
                        navigate.emit((path.clone(), false));
                    } else {
                        set_name.emit(name.clone());
                    }
                })
            };

            let disabled = match entry.kind {
                EntryKind::Directory => false,
                EntryKind::File => !accepts(&entry.name),
                EntryKind::Other => true,
            };

            let detail = entry.size.map(|size| html!({ format_size(size) }));

            rows.push(html! {
                <sockel::PickRow key={entry.name.clone()}
                    name={html!(<span class={class::FILE_BROWSER_NAME}>{entry.name.clone()}</span>)} {detail}
                    mark={if is_dir { folder_icon() } else { file_icon() }}
                    current={!is_dir && entry.name == current}
                    {disabled} {onclick} />
            });
        }
    }

    let rows = rows.into_iter().collect::<Html>();

    let note = if state.listing.is_none() && state.loading {
        Some(phrases.text("loading"))
    } else if listing.as_ref().is_some_and(|l| {
        !l.entries
            .iter()
            .any(|e| *show_hidden || !e.name.starts_with('.'))
    }) {
        Some(phrases.text("file-browser-empty"))
    } else {
        None
    };

    let places = listing
        .as_ref()
        .map(|listing| {
            listing
                .places
                .iter()
                .map(|place| {
                    let onclick = {
                        let navigate = navigate.clone();
                        let path = place.path.clone();
                        Callback::from(move |_: MouseEvent| navigate.emit((path.clone(), false)))
                    };

                    let label = match place.kind {
                        PlaceKind::Home => phrases.text("file-browser-home"),
                        PlaceKind::Documents => phrases.text("file-browser-documents"),
                        PlaceKind::Downloads => phrases.text("file-browser-downloads"),
                        PlaceKind::Root => place.path.clone().into(),
                    };

                    html! {
                        <sockel::Button size={sockel::ButtonSize::Small} variant={sockel::ButtonVariant::Quiet}
                            selected={listing.path == place.path} tip={place.path.clone()} {onclick}>
                            {label}
                        </sockel::Button>
                    }
                })
                .collect::<Html>()
        })
        .unwrap_or_default();

    let up = {
        let parent = listing.as_ref().and_then(|l| l.parent.clone());
        let disabled = parent.is_none();
        let navigate = navigate.clone();
        let onclick = Callback::from(move |_: MouseEvent| {
            if let Some(parent) = &parent {
                navigate.emit((parent.clone(), false));
            }
        });

        html! {
            <sockel::Button size={sockel::ButtonSize::Small} variant={sockel::ButtonVariant::Quiet}
                icon={html!(<sockel::Icon glyph={sockel::Glyph::ChevronUp} />)}
                label={phrases.text("file-browser-up")} tip={phrases.text("file-browser-up")}
                {disabled} {onclick} />
        }
    };

    let hidden = {
        let show_hidden = show_hidden.clone();
        let glyph = if *show_hidden {
            sockel::Glyph::Eye
        } else {
            sockel::Glyph::EyeOff
        };

        html! {
            <sockel::Button size={sockel::ButtonSize::Small} variant={sockel::ButtonVariant::Quiet}
                icon={html!(<sockel::Icon {glyph} />)} pressed={Some(*show_hidden)}
                label={phrases.text("file-browser-hidden")} tip={phrases.text("file-browser-hidden")}
                onclick={Callback::from(move |_: MouseEvent| show_hidden.set(!*show_hidden))} />
        }
    };

    let on_location = {
        let dispatcher = state.dispatcher();
        Callback::from(move |location: String| dispatcher.dispatch(Action::Location(location)))
    };

    let on_location_key = {
        let navigate = navigate.clone();
        let location = state.location.clone();

        Callback::from(move |e: KeyboardEvent| {
            if e.key() == "Enter" {
                e.prevent_default();
                navigate.emit((location.clone(), false));
            }
        })
    };

    let error = state.error.clone().map(|error| {
        html! {
            <sockel::Notice tone={sockel::Tone::Danger}>{error}</sockel::Notice>
        }
    });

    let name_field = (mode == FileBrowserMode::Save).then(|| {
        let hint = replaces.then(|| html!({ phrases.text("file-browser-replaces") }));
        let describedby = sockel::field_describedby("file-browser-name", hint.is_some(), false);

        html! {
            <sockel::Field id="file-browser-name" label={html!({ phrases.text("file-browser-name") })} {hint}>
                <sockel::TextInput id="file-browser-name" value={state.name.clone()}
                    on_input={set_name.clone()} {describedby} />
            </sockel::Field>
        }
    });

    let on_dismiss = props.on_dismiss.reform(|_| ());

    let choose = {
        let on_choose = props.on_choose.clone();
        let chosen = chosen.clone();

        Callback::from(move |_: MouseEvent| {
            if let Some(path) = &chosen {
                on_choose.emit(path.clone());
            }
        })
    };

    let choose_label = match mode {
        FileBrowserMode::Open => phrases.text("file-browser-open"),
        FileBrowserMode::Save => phrases.text("save"),
    };

    let actions = html! {
        <>
            <sockel::Button onclick={props.on_dismiss.reform(|_| ())}>{phrases.text("cancel")}</sockel::Button>
            <sockel::Button variant={sockel::ButtonVariant::Primary} disabled={chosen.is_none()}
                title={chosen.clone()} onclick={choose}>{choose_label}</sockel::Button>
        </>
    };

    let head_actions = html! { <>{up}{hidden}</> };

    html! {
        <sockel::Dialog open={props.open} {on_dismiss} title={props.title.clone()}
            size={sockel::DialogSize::WideTall} {head_actions} {actions} testid="file-browser">
            <sockel::Stack>
                <sockel::Cluster gap={sockel::Gap::Tight}>{places}</sockel::Cluster>
                <sockel::TextInput value={state.location.clone()} on_input={on_location}
                    on_keydown={on_location_key} label={phrases.text("file-browser-location")}
                    enterkeyhint="go" spellcheck={false} busy={state.loading} />
                {error}
                if let Some(note) = note {
                    <p class={class::FILE_BROWSER_NOTE}>{note}</p>
                }
                <sockel::RowList tag="div" divided={true} boxed={true}>{rows}</sockel::RowList>
                {name_field}
            </sockel::Stack>
        </sockel::Dialog>
    }
}

fn format_size(size: u64) -> String {
    const UNITS: [&str; 4] = ["KB", "MB", "GB", "TB"];

    if size < 1024 {
        return format!("{size} B");
    }

    let mut value = size as f64 / 1024.0;
    let mut unit = "KB";

    for next in UNITS.iter().skip(1) {
        if value < 1024.0 {
            break;
        }

        value /= 1024.0;
        unit = next;
    }

    format!("{value:.1} {unit}")
}

fn folder_icon() -> Html {
    glyph(html! {
        <path d="M1.5 4.5a1 1 0 0 1 1-1h3.5l1.5 1.5h6a1 1 0 0 1 1 1v6.5a1 1 0 0 1-1 1h-11a1 1 0 0 1-1-1z" />
    })
}

fn file_icon() -> Html {
    glyph(html! {
        <><path d="M4 1.5h5l3.5 3.5v9a.5.5 0 0 1-.5.5H4a.5.5 0 0 1-.5-.5V2a.5.5 0 0 1 .5-.5z" /><path d="M9 1.5V5h3.5" /></>
    })
}

fn glyph(paths: Html) -> Html {
    html! {
        <span class={sockel::class::GLYPH} aria-hidden="true">
            <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5"
                stroke-linecap="round" stroke-linejoin="round" focusable="false">
                {paths}
            </svg>
        </span>
    }
}

#[cfg(test)]
mod tests {
    use super::format_size;

    #[test]
    fn sizes() {
        assert_eq!(format_size(512), "512 B");
        assert_eq!(format_size(1536), "1.5 KB");
        assert_eq!(format_size(3 * 1024 * 1024), "3.0 MB");
    }
}
